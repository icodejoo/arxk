use super::super::*;

/// 分支后缀最多占可用宽度的比例；超过就不再为它预留，改为整体截断。
const BRANCH_SUFFIX_MAX_SHARE: f32 = 0.6;
/// 省略号文本。
const ELLIPSIS: &str = "...";

impl TerminalView {
    pub(crate) fn truncate_tab_title(title: &str) -> String {
        // Keep titles single-line so shell-provided newlines do not break tab layout.
        let normalized = title.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.chars().count() > MAX_TAB_TITLE_CHARS {
            return normalized.chars().take(MAX_TAB_TITLE_CHARS).collect();
        }
        normalized
    }

    /// Shorten a shell-sourced tab title to its working-directory basename.
    /// Strips a leading `user@host:` prefix and returns the last path segment.
    /// Non-path titles (running commands, custom labels) pass through unchanged.
    pub(crate) fn shorten_shell_tab_title(title: &str) -> String {
        let path_part = match title.split_once(':') {
            Some((prefix, suffix))
                if prefix.contains('@') && (suffix.starts_with('/') || suffix.starts_with('~')) =>
            {
                suffix
            }
            _ => title,
        };

        if !(path_part.starts_with('/') || path_part.starts_with('~')) {
            return title.to_string();
        }

        let trimmed = path_part.trim_end_matches(['/', '\\']);
        let basename = trimmed.rsplit(['/', '\\']).next().unwrap_or("");
        if basename.is_empty() {
            path_part.to_string()
        } else {
            basename.to_string()
        }
    }

    /// 是否像路径（含 `/` 或 `\`）。
    fn is_path_like_tab_title(title: &str) -> bool {
        title.contains('/') || title.contains('\\')
    }

    /// 路径中间省略：共保留 `preserved_chars` 个字符，优先保住最后一层目录（`basename_len`）。
    fn squeezed_path_tab_label_for_preserved_chars(
        chars: &[char],
        basename_len: usize,
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }

        if preserved_chars == 0 {
            return ELLIPSIS.to_string();
        }

        let (head_chars, tail_chars) = if preserved_chars == 1 {
            (0, 1)
        } else {
            let max_tail_chars = preserved_chars - 1;
            let min_tail_chars = preserved_chars / 2;
            let preferred_tail_chars = (basename_len + 1).min(max_tail_chars);
            let tail_chars = preferred_tail_chars.max(min_tail_chars).min(max_tail_chars);
            (preserved_chars.saturating_sub(tail_chars), tail_chars)
        };

        let mut formatted = String::with_capacity(head_chars + 3 + tail_chars);
        for ch in chars.iter().take(head_chars) {
            formatted.push(*ch);
        }
        formatted.push_str(ELLIPSIS);
        for ch in chars
            .iter()
            .skip(chars.len().saturating_sub(tail_chars))
            .take(tail_chars)
        {
            formatted.push(*ch);
        }

        formatted
    }

    /// 尾部截断：保留开头 `preserved_chars` 个字符，后面接 `...`。
    fn end_truncated_tab_label_for_preserved_chars(
        chars: &[char],
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }
        let mut formatted: String = chars.iter().take(preserved_chars).collect();
        formatted.push_str(ELLIPSIS);
        formatted
    }

    /// 中间省略：共保留 `preserved_chars` 个字符，头部多一个（奇数时），中间接 `...`。
    fn middle_squeezed_tab_label_for_preserved_chars(
        chars: &[char],
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }
        let head_chars = preserved_chars.div_ceil(2);
        let tail_chars = preserved_chars - head_chars;
        let mut formatted: String = chars.iter().take(head_chars).collect();
        formatted.push_str(ELLIPSIS);
        formatted.extend(chars.iter().skip(chars.len().saturating_sub(tail_chars)));
        formatted
    }

    /// 放不下任何字符时，挑一个放得下的省略号（`...`、`..`、`.`），都放不下返回空串。
    fn fitting_dots_for_width(
        available_text_px: f32,
        measure: &mut dyn FnMut(&str) -> f32,
    ) -> String {
        for dots in [ELLIPSIS, "..", "."] {
            if measure(dots) <= available_text_px {
                return dots.to_string();
            }
        }
        String::new()
    }

    /// 二分查找能放下的最大保留字符数，用 `build` 生成候选；一个字符都放不下时退化成省略号。
    fn search_fit(
        chars: &[char],
        available_text_px: f32,
        measure: &mut dyn FnMut(&str) -> f32,
        build: &dyn Fn(usize) -> String,
    ) -> String {
        let (mut low, mut high) = (0usize, chars.len());
        while low < high {
            let mid = (low + high).div_ceil(2);
            if measure(&build(mid)) <= available_text_px {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        let fitted = build(low);
        if measure(&fitted) <= available_text_px {
            fitted
        } else {
            Self::fitting_dots_for_width(available_text_px, measure)
        }
    }

    /// 按可用宽度压缩标签文字：路径保留开头和最后一层目录，普通标题保留两侧，中间用 `...`。
    ///
    /// - `title`：待显示的文字；末尾的 `🔱分支` 后缀会整体保留（分支名里的 `/` 不当路径处理）。
    /// - `available_text_px`：可用宽度（像素）。
    /// - `measure`：量字函数。
    ///
    /// 返回放得下的文字；一点都放不下返回空串。
    pub(crate) fn format_tab_label_for_render_measured(
        title: &str,
        available_text_px: f32,
        measure: &mut dyn FnMut(&str) -> f32,
    ) -> String {
        let available_text_px = if available_text_px.is_finite() {
            available_text_px.max(0.0)
        } else {
            0.0
        };
        if title.is_empty() || available_text_px <= f32::EPSILON {
            return String::new();
        }
        if measure(title) <= available_text_px {
            return title.to_string();
        }

        let chars: Vec<char> = title.chars().collect();

        // `{标题}🔱{分支}`：分支名可能含 `/`（feature/x），不能让它参与路径压缩，
        // 否则仓库目录名和分隔符会被当成路径碎片丢掉。先给后缀留足宽度，只压缩前面的标题。
        if let Some((base, branch)) = title.rsplit_once(super::git::BRANCH_SEPARATOR)
            && !base.is_empty()
            && !branch.is_empty()
        {
            let suffix = format!("{}{branch}", super::git::BRANCH_SEPARATOR);
            let suffix_px = measure(&suffix);
            if suffix_px < available_text_px * BRANCH_SUFFIX_MAX_SHARE {
                let fitted_base = Self::format_tab_label_for_render_measured(
                    base,
                    available_text_px - suffix_px,
                    measure,
                );
                if !fitted_base.is_empty() {
                    return fitted_base + &suffix;
                }
            }
            // 后缀太长或标题一点也放不下：整体按普通文字尾部截断，不做路径压缩。
            return Self::search_fit(&chars, available_text_px, measure, &|n| {
                Self::end_truncated_tab_label_for_preserved_chars(&chars, n)
            });
        }

        if Self::is_path_like_tab_title(title) {
            let basename_len = chars
                .iter()
                .rposition(|ch| *ch == '/' || *ch == '\\')
                .map_or(chars.len(), |index| chars.len() - index - 1);
            Self::search_fit(&chars, available_text_px, measure, &|n| {
                Self::squeezed_path_tab_label_for_preserved_chars(&chars, basename_len, n)
            })
        } else {
            Self::search_fit(&chars, available_text_px, measure, &|n| {
                Self::middle_squeezed_tab_label_for_preserved_chars(&chars, n)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_text_width(text: &str) -> f32 {
        text.chars()
            .map(|ch| match ch {
                '/' | '\\' => 5.0,
                '.' => 3.5,
                'i' | 'l' | '1' => 4.5,
                'W' | 'M' => 9.0,
                _ => 7.0,
            })
            .sum()
    }

    #[test]
    fn measured_tab_title_fit_keeps_exact_fit_path_untruncated() {
        let title = "~/Desktop";
        let width = synthetic_text_width(title);

        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                width,
                &mut synthetic_text_width
            ),
            title
        );
    }

    #[test]
    fn measured_tab_title_fit_middle_squeezes_path_titles() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs";
        let available = synthetic_text_width("~/Desktop/.../docs");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            &mut synthetic_text_width,
        );

        assert!(formatted.contains("..."));
        assert!(formatted.starts_with("~/"));
        assert!(formatted.ends_with("/docs"));
        assert!(synthetic_text_width(&formatted) <= available);
    }

    #[test]
    fn measured_tab_title_fit_returns_dots_for_tiny_widths() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs";
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width("..."),
                &mut synthetic_text_width,
            ),
            "..."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width(".."),
                &mut synthetic_text_width,
            ),
            ".."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                0.0,
                &mut synthetic_text_width
            ),
            ""
        );
    }

    /// CJK 按 2 列、其余 1 列的宽度估算，与窗格标题的量字方式一致。
    fn column_width(text: &str) -> f32 {
        unicode_width::UnicodeWidthStr::width(text) as f32
    }

    #[test]
    fn measured_tab_title_fit_middle_squeezes_non_path_titles() {
        let title = "cargo test --workspace --all-features";
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            column_width("cargo ...atures"),
            &mut column_width,
        );
        assert_eq!(formatted, "cargo ...atures");
    }

    #[test]
    fn measured_tab_title_fit_middle_squeezes_cjk_titles_by_columns() {
        // 总宽 20 列放进 11 列：保留 4 个全角字符（8 列）加 3 列省略号。
        let title = "修复登录页面偶发闪屏问题";
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            column_width("修复...问题"),
            &mut column_width,
        );
        assert_eq!(formatted, "修复...问题");
        let mixed = TerminalView::format_tab_label_for_render_measured(
            "Bingo主站H5首页改版需求",
            12.0,
            &mut column_width,
        );
        assert!(mixed.starts_with("Bi") && mixed.contains("...") && mixed.ends_with("需求"));
        assert!(column_width(&mixed) <= 12.0, "{mixed}");
    }

    #[test]
    fn measured_tab_title_fit_non_path_dots_for_tiny_widths() {
        let title = "plain title that is long";
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(title, 3.0, &mut column_width),
            "..."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(title, 2.0, &mut column_width),
            ".."
        );
    }

    #[test]
    fn measured_tab_title_fit_keeps_branch_suffix_with_middle_squeezed_plain_title() {
        let title = "修复登录页面偶发闪屏问题🔱main";
        let available = column_width("修复...问题🔱main");
        let formatted =
            TerminalView::format_tab_label_for_render_measured(title, available, &mut column_width);
        assert_eq!(formatted, "修复...问题🔱main");
    }

    #[test]
    fn measured_tab_title_fit_keeps_branch_suffix_with_slash() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs🔱feature/x";
        let available = synthetic_text_width("~/Desktop/.../docs🔱feature/x");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            &mut synthetic_text_width,
        );

        assert!(formatted.ends_with("docs🔱feature/x"), "{formatted}");
        assert!(formatted.contains("..."));
        assert!(synthetic_text_width(&formatted) <= available);
    }

    #[test]
    fn measured_tab_title_fit_truncates_plainly_when_branch_suffix_is_too_long() {
        let title = "repo🔱feature/a-very-long-branch-name-that-cannot-fit";
        let available = synthetic_text_width("repo🔱feature...");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            &mut synthetic_text_width,
        );

        assert!(formatted.starts_with("repo🔱"), "{formatted}");
        assert!(formatted.ends_with("..."));
        assert!(synthetic_text_width(&formatted) <= available);
    }

    #[test]
    fn measured_tab_title_fit_never_overflows_available_width() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs/test2/test4/test4";
        let available = synthetic_text_width("~/Desktop/.../test4");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            &mut synthetic_text_width,
        );

        assert!(synthetic_text_width(&formatted) <= available);
    }
}
