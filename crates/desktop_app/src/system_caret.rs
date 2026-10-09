//! 系统光标（Win32 Caret）同步：让语音输入等工具判定当前焦点可输入文字。

use gpui_kit::{Bounds, FocusHandle, Pixels, Window};

#[cfg(target_os = "windows")]
use std::cell::{Cell, RefCell};

/// 系统光标记录。Win32 光标每线程只有一个，所以记录归属窗口，换窗口时重建。
#[derive(Clone, PartialEq, Default, Debug)]
pub struct CaretState {
    /// 当前持有光标的窗口标识
    pub owner: Option<isize>,
    /// 系统光标是否已创建
    pub active: bool,
    /// 上次同步的逻辑像素矩形
    pub logical_bounds: Option<Bounds<Pixels>>,
}

/// 一次同步需要执行的系统光标动作（坐标为客户区物理像素）。
#[derive(Clone, PartialEq, Debug)]
pub enum CaretAction {
    /// 无需操作
    None,
    /// 销毁光标
    Destroy,
    /// 仅移动光标
    Move {
        /// 横坐标
        x: i32,
        /// 纵坐标
        y: i32,
    },
    /// 重建光标并移动
    RecreateAndMove {
        /// 光标宽度
        width: i32,
        /// 光标高度
        height: i32,
        /// 横坐标
        x: i32,
        /// 纵坐标
        y: i32,
    },
}

/// 单色位图所需字节数：每行按 WORD（16 位）对齐。
fn mono_bitmap_len(width: i32, height: i32) -> usize {
    let row_bytes = (width.max(1) as usize).div_ceil(16) * 2;
    row_bytes * height.max(1) as usize
}

/// 逻辑像素乘缩放后取整，得到物理像素。
fn to_physical(value: Pixels, scale: f32) -> i32 {
    (f32::from(value) * scale).round() as i32
}

impl CaretState {
    /// 按窗口归属算出下一步动作：别的窗口持有光标时，`None` 不动它，`Some` 则接管并重建。
    pub fn update_for(
        &mut self,
        window_key: isize,
        new_bounds: Option<Bounds<Pixels>>,
        scale: f32,
    ) -> CaretAction {
        if self.owner != Some(window_key) {
            if new_bounds.is_none() {
                return CaretAction::None;
            }
            *self = Self {
                owner: Some(window_key),
                ..Self::default()
            };
        }
        let action = self.update(new_bounds, scale);
        if action == CaretAction::Destroy {
            self.owner = None;
        }
        action
    }

    /// 依据新的光标矩形与缩放比例算出下一步动作，并更新自身状态（含去重）。
    pub fn update(&mut self, new_bounds: Option<Bounds<Pixels>>, scale: f32) -> CaretAction {
        if self.logical_bounds == new_bounds {
            return CaretAction::None;
        }
        let old_bounds = self.logical_bounds;
        self.logical_bounds = new_bounds;

        let Some(bounds) = new_bounds else {
            return if std::mem::take(&mut self.active) {
                CaretAction::Destroy
            } else {
                CaretAction::None
            };
        };
        let height = to_physical(bounds.size.height, scale);
        let width = to_physical(bounds.size.width, scale).max(1);
        let x = to_physical(bounds.origin.x, scale);
        let y = to_physical(bounds.origin.y, scale);
        let height_changed = old_bounds.is_none_or(|b| to_physical(b.size.height, scale) != height);
        let needs_create = !self.active || height_changed;
        self.active = true;
        if needs_create {
            CaretAction::RecreateAndMove {
                width,
                height,
                x,
                y,
            }
        } else {
            CaretAction::Move { x, y }
        }
    }
}

#[cfg(target_os = "windows")]
thread_local! {
    /// 本线程的系统光标状态
    static CARET_STATE: RefCell<CaretState> = RefCell::new(CaretState::default());
    /// 当前持有的全黑位图句柄（DestroyCaret 后释放）
    static CARET_BITMAP: Cell<isize> = const { Cell::new(0) };
}

/// 释放持有的光标位图。
#[cfg(target_os = "windows")]
fn free_bitmap() {
    use windows::Win32::Graphics::Gdi::{DeleteObject, HBITMAP};
    let raw = CARET_BITMAP.with(|b| b.replace(0));
    if raw != 0 {
        unsafe {
            let _ = DeleteObject(HBITMAP(raw as _).into());
        }
    }
}

/// 同步系统光标。
///
/// 语音输入软件依赖系统光标判断焦点是否可输入。每帧都可调用，内部按上次状态去重。
///
/// - `window`: 当前窗口。
/// - `caret`: 窗口逻辑像素下的光标矩形；`None` 销毁系统光标。
pub fn sync_system_caret(window: &Window, caret: Option<Bounds<Pixels>>) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Gdi::CreateBitmap;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateCaret, DestroyCaret, SetCaretPos, ShowCaret,
        };

        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return;
        };
        let hwnd = HWND(win32.hwnd.get() as _);
        let scale = window.scale_factor();

        let action =
            CARET_STATE.with(|state| state.borrow_mut().update_for(hwnd.0 as isize, caret, scale));
        unsafe {
            match action {
                CaretAction::None => {}
                CaretAction::Destroy => {
                    let _ = DestroyCaret();
                    free_bitmap();
                }
                CaretAction::Move { x, y } => {
                    let _ = SetCaretPos(x, y);
                }
                CaretAction::RecreateAndMove {
                    width,
                    height,
                    x,
                    y,
                } => {
                    let _ = DestroyCaret();
                    free_bitmap();
                    // 全黑位图：与背景异或后画面不变，不会出现可见光标块；
                    // 光标尺寸取位图尺寸，所以按真实宽高建图
                    let zeros = vec![0u8; mono_bitmap_len(width, height)];
                    let bmp = CreateBitmap(width, height, 1, 1, Some(zeros.as_ptr() as *const _));
                    CARET_BITMAP.with(|b| b.set(bmp.0 as isize));
                    let _ = CreateCaret(hwnd, Some(bmp), width, height);
                    let _ = SetCaretPos(x, y);
                    let _ = ShowCaret(Some(hwnd));
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, caret);
    }
}

/// 输入控件已聚焦且窗口激活时上报光标；否则不动（失焦处自行调 `sync_system_caret(window, None)`）。
///
/// - `window`: 当前窗口。
/// - `focus_handle`: 该输入控件的焦点句柄。
/// - `caret`: 窗口逻辑像素下的光标矩形，`None` 表示暂无位置。
pub fn report_caret(window: &Window, focus_handle: &FocusHandle, caret: Option<Bounds<Pixels>>) {
    if caret.is_some() && focus_handle.is_focused(window) && window.is_window_active() {
        sync_system_caret(window, caret);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{point, px, size};

    fn rect(x: f32, y: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(2.0), px(h)))
    }

    #[test]
    fn caret_state_dedups_moves_and_recreates() {
        let mut state = CaretState::default();
        assert_eq!(state.update(None, 2.0), CaretAction::None);
        assert_eq!(
            state.update(Some(rect(10.0, 20.0, 15.0)), 2.0),
            CaretAction::RecreateAndMove {
                width: 4,
                height: 30,
                x: 20,
                y: 40
            }
        );
        assert_eq!(
            state.update(Some(rect(10.0, 20.0, 15.0)), 2.0),
            CaretAction::None
        );
        assert_eq!(
            state.update(Some(rect(15.0, 20.0, 15.0)), 2.0),
            CaretAction::Move { x: 30, y: 40 }
        );
        assert!(matches!(
            state.update(Some(rect(15.0, 20.0, 16.0)), 2.0),
            CaretAction::RecreateAndMove { height: 32, .. }
        ));
        assert_eq!(state.update(None, 2.0), CaretAction::Destroy);
        assert_eq!(state.update(None, 2.0), CaretAction::None);
    }

    #[test]
    fn caret_state_recreates_when_window_changes() {
        let mut state = CaretState::default();
        let caret = Some(rect(10.0, 20.0, 15.0));
        assert!(matches!(
            state.update_for(1, caret, 1.0),
            CaretAction::RecreateAndMove { .. }
        ));
        assert!(matches!(
            state.update_for(2, caret, 1.0),
            CaretAction::RecreateAndMove { .. }
        ));
        // 窗口 1 失焦时不能销毁窗口 2 的光标
        assert_eq!(state.update_for(1, None, 1.0), CaretAction::None);
        // 回到窗口 1，同一位置也要重建
        assert!(matches!(
            state.update_for(1, caret, 1.0),
            CaretAction::RecreateAndMove { .. }
        ));
        assert_eq!(state.update_for(1, None, 1.0), CaretAction::Destroy);
        assert_eq!(state.owner, None);
    }

    #[test]
    fn mono_bitmap_rows_are_word_aligned() {
        assert_eq!(mono_bitmap_len(1, 1), 2);
        assert_eq!(mono_bitmap_len(16, 3), 6);
        assert_eq!(mono_bitmap_len(17, 3), 12);
    }
}
