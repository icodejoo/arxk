use super::super::*;

/// 分支后缀最多占可用宽度的比例；超过就不再为它预留，改为整体截断。
const BRANCH_SUFFIX_MAX_SHARE: f32 = 0.6;

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

    fn is_path_like_tab_title(title: &str) -> bool {
        title.contains('/') || title.contains('\\')
    }

    fn squeezed_path_tab_label_for_preserved_chars(
        chars: &[char],
        basename_len: usize,
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }

        if preserved_chars == 0 {
            return "...".to_string();
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
        formatted.push_str("...");
        for ch in chars
            .iter()
            .skip(chars.len().saturating_sub(tail_chars))
            .take(tail_chars)
        {
            formatted.push(*ch);
        }

        formatted
    }

    fn end_truncated_tab_label_for_preserved_chars(
        chars: &[char],
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }

        if preserved_chars == 0 {
            return "...".to_string();
        }

        let mut formatted = String::with_capacity(preserved_chars + 3);
        for ch in chars.iter().take(preserved_chars) {
            formatted.push(*ch);
        }
        formatted.push_str("...");
        formatted
    }

    fn fitting_dots_for_width<F>(available_text_px: f32, measure_text_px: &mut F) -> String
    where
        F: FnMut(&str) -> f32,
    {
        if available_text_px <= f32::EPSILON {
            return String::new();
        }

        for dots in ["...", "..", "."] {
            if measure_text_px(dots) <= available_text_px {
                return dots.to_string();
            }
        }

        String::new()
    }

    /// 按可用宽度压缩标签文字：路径保留开头和最后一层目录，其余尾部截断。
    ///
    /// - `title`：待显示的文字；末尾的 `🔱分支` 后缀会整体保留（分支名里的 `/` 不当路径处理）。
    /// - `available_text_px`：可用宽度（像素）。
    /// - `measure_text_px`：量字函数。
    ///
    /// 返回放得下的文字；一点都放不下返回空串。
    pub(crate) fn format_tab_label_for_render_measured<F>(
        title: &str,
        available_text_px: f32,
        mut measure_text_px: F,
    ) -> String
    where
        F: FnMut(&str) -> f32,
    {
        Self::fit_label(title, available_text_px, &mut measure_text_px)
    }

    /// `format_tab_label_for_render_measured` 的实现（用 `dyn` 以便处理分支后缀时递归）。
    fn fit_label(
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

        // `{标题}::{分支}`：分支名可能含 `/`（feature/x），不能让它参与路径压缩，
        // 否则仓库目录名和 `::` 会被当成路径碎片丢掉。先给后缀留足宽度，只压缩前面的标题。
        let mut plain_only = false;
        if let Some((base, branch)) = title.rsplit_once(super::git::BRANCH_SEPARATOR)
            && !base.is_empty()
            && !branch.is_empty()
        {
            let suffix = format!("{}{branch}", super::git::BRANCH_SEPARATOR);
            let suffix_px = measure(&suffix);
            if suffix_px < available_text_px * BRANCH_SUFFIX_MAX_SHARE {
                let fitted_base = Self::fit_label(base, available_text_px - suffix_px, measure);
                if !fitted_base.is_empty() {
                    return fitted_base + &suffix;
                }
            }
            // 后缀太长或标题一点也放不下：整体按普通文字截断，不做路径压缩。
            plain_only = true;
        }

        let mut measure_text_px = |text: &str| measure(text);
        if plain_only || !Self::is_path_like_tab_title(title) {
            let chars: Vec<char> = title.chars().collect();
            if chars.is_empty() {
                return String::new();
            }

            let mut low = 0usize;
            let mut high = chars.len();
            while low < high {
                let mid = (low + high).div_ceil(2);
                let candidate = Self::end_truncated_tab_label_for_preserved_chars(&chars, mid);
                if measure_text_px(candidate.as_str()) <= available_text_px {
                    low = mid;
                } else {
                    high = mid.saturating_sub(1);
                }
            }

            let fitted = Self::end_truncated_tab_label_for_preserved_chars(&chars, low);
            if measure_text_px(fitted.as_str()) <= available_text_px {
                return fitted;
            }

            return Self::fitting_dots_for_width(available_text_px, &mut measure_text_px);
        }

        let chars: Vec<char> = title.chars().collect();
        if chars.is_empty() {
            return String::new();
        }
        let basename_len = chars
            .iter()
            .rposition(|ch| *ch == '/' || *ch == '\\')
            .map_or(chars.len(), |index| chars.len().saturating_sub(index + 1));
        let candidate_for = |preserved_chars: usize| {
            Self::squeezed_path_tab_label_for_preserved_chars(&chars, basename_len, preserved_chars)
        };

        let mut low = 0usize;
        let mut high = chars.len();
        while low < high {
            let mid = (low + high).div_ceil(2);
            let candidate = candidate_for(mid);
            if measure_text_px(candidate.as_str()) <= available_text_px {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }

        let fitted = candidate_for(low);
        if measure_text_px(fitted.as_str()) <= available_text_px {
            fitted
        } else {
            Self::fitting_dots_for_width(available_text_px, &mut measure_text_px)
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
            TerminalView::format_tab_label_for_render_measured(title, width, synthetic_text_width),
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
            synthetic_text_width,
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
                synthetic_text_width,
            ),
            "..."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width(".."),
                synthetic_text_width,
            ),
            ".."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(title, 0.0, synthetic_text_width),
            ""
        );
    }

    #[test]
    fn measured_tab_title_fit_end_truncates_non_path_titles() {
        let title = "cargo test --workspace --all-features";
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width("cargo test..."),
                synthetic_text_width,
            ),
            "cargo test..."
        );
    }

    #[test]
    fn measured_tab_title_fit_keeps_branch_suffix_with_slash() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs🔱feature/x";
        let available = synthetic_text_width("~/Desktop/.../docs🔱feature/x");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            synthetic_text_width,
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
            synthetic_text_width,
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
            synthetic_text_width,
        );

        assert!(synthetic_text_width(&formatted) <= available);
    }
}
