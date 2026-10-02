//! Windows 自绘标题栏：顶栏右侧的“可拖动空白 + 最小化/最大化/关闭”。
//!
//! 窗口用 `appears_transparent` 去掉了系统标题栏，所以拖动和三个按钮都要自己画。
//! 点击由系统按 `WindowControlArea` 命中测试处理，这里不写点击事件。
//! GPUI 取“鼠标下第一个登记的控制区域”，因此拖动区与按钮必须并排、不能重叠。

use super::super::*;
use super::layout::WINDOWS_CAPTION_BUTTONS_RESERVED_WIDTH;

/// 图标字体（Win10/11 自带）。
const CAPTION_ICON_FONT: &str = "Segoe MDL2 Assets";
/// 图标字号。
const CAPTION_ICON_SIZE: f32 = 10.0;
/// 最小化图标字形。
const GLYPH_MINIMIZE: &str = "\u{E921}";
/// 最大化图标字形。
const GLYPH_MAXIMIZE: &str = "\u{E922}";
/// 还原图标字形。
const GLYPH_RESTORE: &str = "\u{E923}";
/// 关闭图标字形。
const GLYPH_CLOSE: &str = "\u{E8BB}";
/// 普通按钮悬停时的底色透明度。
const CAPTION_HOVER_ALPHA: f32 = 0.10;

/// 窗口控制按钮的种类。
#[derive(Clone, Copy)]
enum CaptionButton {
    Minimize,
    Maximize,
    Close,
}

impl TerminalView {
    /// 渲染顶栏右侧的拖动区和窗口控制按钮。
    ///
    /// `width` 为 `Some` 时是固定宽度的留白带（标签栏右侧）；为 `None` 时占满剩余空间
    /// （标签栏被隐藏时整条顶栏的右半部分）。`foreground` 是图标颜色。
    pub(crate) fn render_windows_caption_lane(
        width: Option<f32>,
        window: &Window,
        foreground: gpui_kit::Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let maximized = window.is_maximized();
        let button_width = WINDOWS_CAPTION_BUTTONS_RESERVED_WIDTH / 3.0;
        let lane = div().id("tabbar-right-inset").relative().h_full().flex();
        let lane = match width {
            Some(width) => lane.flex_none().w(px(width)),
            None => lane.flex_1(),
        };
        lane.on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
            this.on_action_rail_mouse_move(event, window, cx);
        }))
        .child(
            div()
                .id("titlebar-drag-area")
                .flex_1()
                .h_full()
                .window_control_area(gpui_kit::WindowControlArea::Drag),
        )
        .children(
            [
                CaptionButton::Minimize,
                CaptionButton::Maximize,
                CaptionButton::Close,
            ]
            .map(|kind| Self::render_caption_button(kind, button_width, maximized, foreground)),
        )
        .into_any_element()
    }

    /// 渲染单个窗口控制按钮；最大化按钮在已最大化时显示“还原”图标。
    fn render_caption_button(
        kind: CaptionButton,
        width: f32,
        maximized: bool,
        foreground: gpui_kit::Rgba,
    ) -> AnyElement {
        let (id, glyph, area, is_close) = match kind {
            CaptionButton::Minimize => (
                "caption-minimize",
                GLYPH_MINIMIZE,
                gpui_kit::WindowControlArea::Min,
                false,
            ),
            CaptionButton::Maximize => (
                "caption-maximize",
                if maximized {
                    GLYPH_RESTORE
                } else {
                    GLYPH_MAXIMIZE
                },
                gpui_kit::WindowControlArea::Max,
                false,
            ),
            CaptionButton::Close => (
                "caption-close",
                GLYPH_CLOSE,
                gpui_kit::WindowControlArea::Close,
                true,
            ),
        };
        let mut hover_bg = foreground;
        hover_bg.a = CAPTION_HOVER_ALPHA;
        // 关闭按钮悬停用系统惯用的红底白字。
        let close_bg = gpui_kit::Rgba {
            r: 0.91,
            g: 0.07,
            b: 0.14,
            a: 1.0,
        };
        let close_fg: gpui_kit::Hsla = gpui_kit::Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
        .into();
        let icon_fg: gpui_kit::Hsla = foreground.into();

        div()
            .id(id)
            .flex()
            .flex_none()
            .w(px(width))
            .h_full()
            .items_center()
            .justify_center()
            .font_family(SharedString::from(CAPTION_ICON_FONT))
            .text_size(px(CAPTION_ICON_SIZE))
            .text_color(icon_fg)
            .window_control_area(area)
            .hover(move |style| {
                if is_close {
                    style.bg(close_bg).text_color(close_fg)
                } else {
                    style.bg(hover_bg)
                }
            })
            .child(glyph)
            .into_any_element()
    }
}
