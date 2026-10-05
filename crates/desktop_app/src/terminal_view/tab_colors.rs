//! 标签背景色：在标签右键菜单里给单个标签选一个预设色，重启后不保留。
//!
//! 颜色按 `TabId` 存在视图的侧表里，不改 `TerminalTab` 结构；标签关闭后残留的条目只有几个字节，
//! 且 `TabId` 单调递增不会复用，所以不额外清理。

#![cfg_attr(target_os = "macos", allow(dead_code))]

use super::*;

/// 标签背景色预设。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TabColor {
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
}

impl TabColor {
    /// 菜单里按顺序显示的全部预设色。
    pub(crate) const ALL: [TabColor; 8] = [
        TabColor::Red,
        TabColor::Orange,
        TabColor::Yellow,
        TabColor::Green,
        TabColor::Teal,
        TabColor::Blue,
        TabColor::Purple,
        TabColor::Pink,
    ];

    /// 颜色的 RGB 分量（0.0–1.0）。
    pub(crate) const fn rgb(self) -> (f32, f32, f32) {
        match self {
            TabColor::Red => (0.94, 0.33, 0.31),
            TabColor::Orange => (0.97, 0.60, 0.20),
            TabColor::Yellow => (0.96, 0.80, 0.25),
            TabColor::Green => (0.40, 0.75, 0.45),
            TabColor::Teal => (0.25, 0.75, 0.75),
            TabColor::Blue => (0.35, 0.55, 0.95),
            TabColor::Purple => (0.65, 0.50, 0.90),
            TabColor::Pink => (0.93, 0.45, 0.70),
        }
    }

    /// 带指定透明度的颜色。
    pub(crate) fn with_alpha(self, alpha: f32) -> gpui_kit::Rgba {
        let (r, g, b) = self.rgb();
        gpui_kit::Rgba { r, g, b, a: alpha }
    }
}

/// 活动标签着色后的背景透明度。
pub(crate) const TAB_TINT_ACTIVE_ALPHA: f32 = 0.55;
/// 非活动标签着色后的背景透明度。
pub(crate) const TAB_TINT_IDLE_ALPHA: f32 = 0.28;
/// 鼠标悬停时额外增加的透明度。
pub(crate) const TAB_TINT_HOVER_BOOST: f32 = 0.15;
/// 右键菜单里“Tab Color”一项（标签行 + 色块行）的总高度。
pub(crate) const TAB_COLOR_MENU_HEIGHT: f32 = 58.0;
/// 色块直径。
pub(crate) const TAB_COLOR_SWATCH_SIZE: f32 = 12.0;
/// 色块间距。
pub(crate) const TAB_COLOR_SWATCH_GAP: f32 = 5.0;

impl TerminalView {
    /// 设置或清除某个标签的背景色（`None` 恢复默认），返回是否有变化。
    pub(crate) fn set_tab_color_by_id(
        &mut self,
        tab_id: TabId,
        color: Option<TabColor>,
        cx: &mut Context<Self>,
    ) -> bool {
        let changed = match color {
            Some(color) => self.tab_colors.insert(tab_id, color) != Some(color),
            None => self.tab_colors.remove(&tab_id).is_some(),
        };
        if changed {
            cx.notify();
        }
        changed
    }

    /// 某个标签（按序号）当前的自定义背景色，没设置返回 `None`。
    pub(crate) fn tab_color_at(&self, index: usize) -> Option<TabColor> {
        let tab = self.session.tabs.get(index)?;
        self.tab_colors.get(&tab.id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_distinct_and_in_range() {
        let mut seen = std::collections::HashSet::new();
        for color in TabColor::ALL {
            let (r, g, b) = color.rgb();
            assert!(
                [r, g, b]
                    .iter()
                    .all(|channel| (0.0..=1.0).contains(channel))
            );
            assert!(seen.insert(color), "重复预设: {color:?}");
        }
        assert_eq!(seen.len(), 8);
    }

    #[test]
    fn with_alpha_keeps_rgb_and_sets_alpha() {
        let color = TabColor::Blue.with_alpha(0.4);
        let (r, g, b) = TabColor::Blue.rgb();
        assert_eq!((color.r, color.g, color.b, color.a), (r, g, b, 0.4));
    }
}
