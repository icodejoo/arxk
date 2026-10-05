use super::super::*;
use crate::terminal_ui::TmuxPaneState;
use std::path::Path;

/// 窗格标签的两段文字：左段用标题样式，右段（可选）用路径样式。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PaneLabelTexts {
    /// 左段：自定义标题；没有自定义标题时就是路径。
    pub(crate) left: String,
    /// 右段：路径；仅在有自定义标题且目录已知时存在。
    pub(crate) right: Option<String>,
}

impl TerminalView {
    pub(crate) fn fallback_title(&self) -> &str {
        let fallback = self.tab_title.fallback.trim();
        if fallback.is_empty() {
            DEFAULT_TAB_TITLE
        } else {
            fallback
        }
    }

    pub(crate) fn resolve_template(
        template: &str,
        cwd: Option<&str>,
        command: Option<&str>,
    ) -> String {
        const CWD_TOKEN: &str = "{cwd}";
        const COMMAND_TOKEN: &str = "{command}";

        let cwd = cwd.unwrap_or("");
        let command = command.unwrap_or("");
        let mut remaining = template;
        let mut resolved = String::with_capacity(template.len());

        while let Some(open_brace_idx) = remaining.find('{') {
            resolved.push_str(&remaining[..open_brace_idx]);
            remaining = &remaining[open_brace_idx..];
            if let Some(tail) = remaining.strip_prefix(CWD_TOKEN) {
                resolved.push_str(cwd);
                remaining = tail;
                continue;
            }
            if let Some(tail) = remaining.strip_prefix(COMMAND_TOKEN) {
                resolved.push_str(command);
                remaining = tail;
                continue;
            }

            resolved.push('{');
            remaining = &remaining['{'.len_utf8()..];
        }

        resolved.push_str(remaining);
        resolved
    }

    pub(crate) fn is_shell_command(command: &str) -> bool {
        let command = command.trim();
        if command.is_empty() {
            return false;
        }
        let Some(first_token) = command.split_whitespace().next() else {
            return false;
        };
        let token = Path::new(first_token)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(first_token)
            .trim_start_matches('-');
        if token.is_empty() {
            return false;
        }
        let mut normalized = token.to_ascii_lowercase();
        if let Some(stem) = normalized.strip_suffix(".exe") {
            normalized = stem.to_string();
        }

        matches!(
            normalized.as_str(),
            "bash"
                | "zsh"
                | "fish"
                | "sh"
                | "dash"
                | "ksh"
                | "tcsh"
                | "csh"
                | "nu"
                | "pwsh"
                | "powershell"
                | "cmd"
        )
    }

    fn display_cwd_for_tab_title(cwd: Option<&str>) -> Option<String> {
        let cwd = cwd.map(str::trim).filter(|cwd| !cwd.is_empty())?;
        Some(Self::display_working_directory_for_prompt(Path::new(cwd)))
    }

    fn resolve_prompt_title(template: &str, cwd: Option<&str>) -> String {
        let cwd = Self::display_cwd_for_tab_title(cwd);
        Self::resolve_template(template, cwd.as_deref(), None)
    }

    pub(crate) fn derive_tmux_shell_title(
        tab_title: &TabTitleConfig,
        pane: &TmuxPaneState,
    ) -> Option<String> {
        let cwd = pane.current_path.trim();
        let command = pane.current_command.trim();
        let resolved = if Self::is_shell_command(command) {
            Self::resolve_prompt_title(&tab_title.prompt_format, (!cwd.is_empty()).then_some(cwd))
        } else {
            Self::resolve_template(
                &tab_title.command_format,
                None,
                (!command.is_empty()).then_some(command),
            )
        };

        let resolved = resolved.trim();
        if resolved.is_empty() {
            return None;
        }

        Some(Self::truncate_tab_title(resolved))
    }

    pub(crate) fn should_seed_predicted_prompt_title(tab_title: &TabTitleConfig) -> bool {
        tab_title.priority.contains(&TabTitleSource::Explicit)
    }

    pub(crate) fn predicted_prompt_seed_title(
        tab_title: &TabTitleConfig,
        cwd: Option<&str>,
    ) -> Option<String> {
        if !Self::should_seed_predicted_prompt_title(tab_title) {
            return None;
        }

        let resolved = Self::resolve_prompt_title(&tab_title.prompt_format, cwd);
        let resolved = resolved.trim();
        if resolved.is_empty() {
            return None;
        }

        Some(Self::truncate_tab_title(resolved))
    }

    fn smart_mode_shell_fallback_enabled(tab_title: &TabTitleConfig) -> bool {
        tab_title.mode == termy_core::config_core::TabTitleMode::Smart
            && !tab_title.shell_integration
            && tab_title.priority.contains(&TabTitleSource::Explicit)
            && tab_title.priority.contains(&TabTitleSource::Shell)
    }

    /// Returns the title candidate for a single source in the priority list.
    ///
    /// The caller walks sources in priority order and uses the first non-empty
    /// candidate.  The `Explicit` source has two special deference modes that
    /// both yield to `shell_title` when available:
    ///
    /// - **Prediction**: the explicit title was pre-seeded at tab creation from
    ///   the cwd and has not yet been confirmed by a shell-integration event.
    /// - **Smart-mode shell fallback**: shell integration is disabled in smart
    ///   mode, so the shell's own title (set via terminal escape sequences) is
    ///   preferred once available.
    ///
    /// In both cases the explicit title is kept as a fallback so the tab is
    /// never blank.
    fn title_source_candidate<'a>(
        source: TabTitleSource,
        manual_title: Option<&'a str>,
        explicit_title: Option<&'a str>,
        explicit_title_is_prediction: bool,
        prediction_allows_shell: bool,
        shell_title: Option<&'a str>,
        fallback_title: &'a str,
        smart_mode_shell_fallback: bool,
    ) -> Option<&'a str> {
        match source {
            TabTitleSource::Manual => manual_title,
            // The explicit title is speculative—prefer a live shell title.
            TabTitleSource::Explicit if explicit_title_is_prediction && prediction_allows_shell => {
                shell_title.or(explicit_title)
            }
            TabTitleSource::Explicit if explicit_title_is_prediction => explicit_title,
            TabTitleSource::Explicit if smart_mode_shell_fallback => {
                // Smart mode seeds an explicit title before the shell emits a title.
                // When shell integration is disabled, prefer live shell titles once
                // available while keeping explicit as a fallback.
                shell_title.or(explicit_title)
            }
            TabTitleSource::Explicit => explicit_title,
            TabTitleSource::Shell => shell_title,
            TabTitleSource::Fallback => Some(fallback_title),
        }
    }

    fn parse_explicit_title(&self, title: &str) -> Option<ExplicitTitlePayload> {
        let prefix = self.tab_title.explicit_prefix.trim();
        if prefix.is_empty() {
            return None;
        }

        let payload = title.strip_prefix(prefix)?.trim();
        if payload.is_empty() {
            return None;
        }

        if let Some(prompt) = payload.strip_prefix("prompt:") {
            let prompt = prompt.trim();
            if prompt.is_empty() {
                return None;
            }
            return Some(ExplicitTitlePayload::Prompt {
                title: Self::resolve_prompt_title(&self.tab_title.prompt_format, Some(prompt)),
                cwd: prompt.to_string(),
            });
        }

        if let Some(command) = payload.strip_prefix("command:") {
            let command = command.trim();
            if command.is_empty() {
                return None;
            }
            return Some(ExplicitTitlePayload::Command {
                title: Self::resolve_template(&self.tab_title.command_format, None, Some(command)),
                command: command.to_string(),
            });
        }

        let explicit = payload.strip_prefix("title:").unwrap_or(payload).trim();
        if explicit.is_empty() {
            return None;
        }

        Some(ExplicitTitlePayload::Title(explicit.to_string()))
    }

    /// 把终端上报的原始标题转成窗格标签文字。
    /// 带内部前缀（`termy:tab:`）的载荷会按标签标题同样的规则解析，其余原样清洗。
    /// 清洗后为空返回 `None`。
    pub(crate) fn pane_display_title(&self, raw: &str) -> Option<String> {
        let raw = raw.trim();
        let text = match self.parse_explicit_title(raw) {
            Some(
                ExplicitTitlePayload::Prompt { title, .. }
                | ExplicitTitlePayload::Command { title, .. }
                | ExplicitTitlePayload::Title(title),
            ) => title,
            None => raw.to_string(),
        };
        let text = Self::truncate_tab_title(text.trim());
        (!text.is_empty()).then_some(text)
    }

    /// 记录某个窗格的标题，返回显示内容是否发生变化（决定是否需要重绘）。
    pub(crate) fn record_pane_title(&mut self, pane_id: &str, raw: &str) -> bool {
        let Some(title) = self.pane_display_title(raw) else {
            return false;
        };
        if self.pane_titles.get(pane_id) == Some(&title) {
            return false;
        }
        self.pane_titles.insert(pane_id.to_string(), title);
        true
    }

    /// 记录某个窗格的当前目录（来自 shell 上报的 OSC 7 / OSC 9;9），
    /// 返回显示内容是否发生变化。
    pub(crate) fn record_pane_cwd(&mut self, pane_id: &str, raw: &str) -> bool {
        let raw = raw.trim().trim_matches('"');
        // Windows 上 Git Bash / MSYS2 / Cygwin 报的是 `/c/...`，转回盘符路径才能显示与恢复。
        #[cfg(target_os = "windows")]
        let raw = Self::normalize_msys_cwd(raw);
        let cwd = Self::truncate_tab_title(&raw);
        if cwd.is_empty() || self.pane_cwds.get(pane_id) == Some(&cwd) {
            return false;
        }
        self.pane_cwds.insert(pane_id.to_string(), cwd);
        true
    }

    /// 解析窗格标签使用的标题：手动名优先于终端上报标题。
    ///
    /// - `manual`：手动名表（窗格 id -> 名字）。
    /// - `reported`：终端上报标题表（窗格 id -> 标题）。
    /// - `pane_id`：目标窗格 id。
    ///
    /// 返回应显示的标题，两者都没有时为 `None`。
    pub(crate) fn resolve_pane_title<'a>(
        manual: &'a HashMap<String, String>,
        reported: &'a HashMap<String, String>,
        pane_id: &str,
    ) -> Option<&'a str> {
        manual
            .get(pane_id)
            .or_else(|| reported.get(pane_id))
            .map(String::as_str)
    }

    /// 把用户输入写入手动名表：截断后为空则清除，返回表是否发生变化。
    ///
    /// - `manual`：手动名表（窗格 id -> 名字）。
    /// - `pane_id`：目标窗格 id。
    /// - `raw`：用户输入的原始文本。
    pub(crate) fn apply_manual_pane_title(
        manual: &mut HashMap<String, String>,
        pane_id: &str,
        raw: &str,
    ) -> bool {
        let title = Self::truncate_tab_title(raw.trim());
        if title.is_empty() {
            return manual.remove(pane_id).is_some();
        }
        if manual.get(pane_id) == Some(&title) {
            return false;
        }
        manual.insert(pane_id.to_string(), title);
        true
    }

    /// 设置（或清除，传空串）某个窗格的手动名，返回是否发生变化。
    pub(crate) fn set_pane_manual_title(&mut self, pane_id: &str, raw: &str) -> bool {
        Self::apply_manual_pane_title(&mut self.pane_manual_titles, pane_id, raw)
    }

    /// 窗格被关闭时，清理这些窗格的手动名。
    pub(crate) fn forget_pane_manual_titles(&mut self, pane_ids: &[String]) {
        for pane_id in pane_ids {
            self.pane_manual_titles.remove(pane_id.as_str());
        }
    }

    /// 把 Git Bash / MSYS2 / Cygwin 上报的 `/c/Users/x`、`/cygdrive/c/Users/x`
    /// 转成 `C:\Users\x`；不是这种形式的路径原样返回。
    ///
    /// - `raw`：shell 上报的路径文本。
    pub(crate) fn normalize_msys_cwd(raw: &str) -> String {
        let rest = raw.strip_prefix("/cygdrive").unwrap_or(raw);
        let Some(after) = rest.strip_prefix('/') else {
            return raw.to_string();
        };
        let mut chars = after.chars();
        let Some(drive) = chars.next().filter(char::is_ascii_alphabetic) else {
            return raw.to_string();
        };
        let tail = chars.as_str();
        if tail.is_empty() {
            return format!("{}:\\", drive.to_ascii_uppercase());
        }
        if !tail.starts_with('/') {
            return raw.to_string();
        }
        format!("{}:{}", drive.to_ascii_uppercase(), tail.replace('/', "\\"))
    }

    /// 比较两段文字是否指向同一路径：忽略首尾空白、末尾分隔符、`/` 与 `\` 的差别和大小写。
    fn same_path_text(a: &str, b: &str) -> bool {
        let normalize = |text: &str| {
            text.trim()
                .trim_end_matches(['/', '\\'])
                .replace('\\', "/")
                .to_lowercase()
        };
        normalize(a) == normalize(b)
    }

    /// 计算窗格标签的两段文字。
    ///
    /// - 标题与当前目录不同 → 有自定义标题：左标题、右路径（目录未知时只有左标题）；
    /// - 否则（没有标题，或标题只是路径本身）→ 路径就是标题：只有左段。
    ///
    /// 标题和目录都没有时返回 `None`。
    pub(crate) fn pane_label_texts(
        title: Option<&str>,
        cwd: Option<&str>,
    ) -> Option<PaneLabelTexts> {
        let title = title.map(str::trim).filter(|text| !text.is_empty());
        let cwd = cwd.map(str::trim).filter(|text| !text.is_empty());
        match (title, cwd) {
            (Some(title), Some(cwd)) if !Self::same_path_text(title, cwd) => Some(PaneLabelTexts {
                left: title.to_string(),
                right: Some(cwd.to_string()),
            }),
            (title, cwd) => cwd.or(title).map(|text| PaneLabelTexts {
                left: text.to_string(),
                right: None,
            }),
        }
    }

    pub(crate) fn resolved_tab_title(&self, index: usize) -> String {
        let tab = &self.session.tabs[index];
        let fallback_title = self.fallback_title();
        let smart_mode_shell_fallback = Self::smart_mode_shell_fallback_enabled(&self.tab_title);
        let prediction_allows_shell = self.tab_title.priority.contains(&TabTitleSource::Shell);

        for source in &self.tab_title.priority {
            let candidate = Self::title_source_candidate(
                *source,
                tab.manual_title.as_deref(),
                tab.explicit_title.as_deref(),
                tab.explicit_title_is_prediction,
                prediction_allows_shell,
                tab.shell_title.as_deref(),
                fallback_title,
                smart_mode_shell_fallback,
            );

            if let Some(candidate) = candidate.map(str::trim).filter(|value| !value.is_empty()) {
                return Self::truncate_tab_title(candidate);
            }
        }

        Self::truncate_tab_title(fallback_title)
    }

    pub(crate) fn refresh_tab_title(&mut self, index: usize) -> bool {
        if index >= self.session.tabs.len() {
            return false;
        }

        let next = self.resolved_tab_title(index);
        if self.session.tabs[index].title == next {
            return false;
        }

        let previous = std::mem::replace(&mut self.session.tabs[index].title, next);
        let current_title = self.session.tabs[index].title.clone();
        self.invalidate_tab_title_width_cache_for_title(previous.as_str());
        self.invalidate_tab_title_width_cache_for_title(current_title.as_str());

        // Keep title-width behavior uniform across manual, shell, explicit, and fallback sources.
        self.session.tabs[index].sticky_title_width = 0.0;
        self.session.tabs[index].title_text_width = 0.0;
        self.mark_tab_strip_layout_dirty();
        true
    }

    pub(crate) fn apply_terminal_title(
        &mut self,
        index: usize,
        title: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        let title = title.trim();
        if title.is_empty() || index >= self.session.tabs.len() {
            return false;
        }

        if let Some(explicit_payload) = self.parse_explicit_title(title) {
            return match explicit_payload {
                ExplicitTitlePayload::Prompt { title, cwd } => {
                    self.session.tabs[index].last_prompt_cwd = Some(cwd);
                    self.session.tabs[index].running_process = false;
                    self.session.tabs[index].current_command = None;
                    self.cancel_pending_command_title(index);
                    self.set_explicit_title(index, title)
                }
                ExplicitTitlePayload::Title(prompt_title) => {
                    self.session.tabs[index].current_command = None;
                    self.cancel_pending_command_title(index);
                    self.set_explicit_title(index, prompt_title)
                }
                ExplicitTitlePayload::Command { title, command } => {
                    self.session.tabs[index].running_process = true;
                    self.session.tabs[index].current_command = Some(command);
                    let tab_id = self.session.tabs[index].id;
                    self.schedule_delayed_command_title(tab_id, title, COMMAND_TITLE_DELAY_MS, cx);
                    false
                }
            };
        }

        let shell_title = Self::truncate_tab_title(title);
        if self.session.tabs[index].shell_title.as_deref() == Some(shell_title.as_str()) {
            return false;
        }

        self.session.tabs[index].shell_title = Some(shell_title);
        self.refresh_tab_title(index)
    }

    pub(crate) fn clear_terminal_titles(&mut self, index: usize) -> bool {
        if index >= self.session.tabs.len() {
            return false;
        }

        self.cancel_pending_command_title(index);
        let tab = &mut self.session.tabs[index];
        tab.running_process = false;
        tab.current_command = None;
        let had_shell = tab.shell_title.take().is_some();
        let had_explicit = tab.explicit_title.take().is_some();
        tab.explicit_title_is_prediction = false;
        if !had_shell && !had_explicit {
            return false;
        }

        self.refresh_tab_title(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TabTitleConfig, TabTitleSource};

    fn expected_home_relative_path(parts: &[&str]) -> String {
        if parts.is_empty() {
            return "~".to_string();
        }

        let separator = std::path::MAIN_SEPARATOR.to_string();
        format!("~{}{}", std::path::MAIN_SEPARATOR, parts.join(&separator))
    }

    fn pane_with(path: &str, command: &str) -> TmuxPaneState {
        TmuxPaneState {
            id: "%1".to_string(),
            window_id: "@1".to_string(),
            session_id: "$1".to_string(),
            is_active: true,
            left: 0,
            top: 0,
            width: 80,
            height: 24,
            cursor_x: 0,
            cursor_y: 0,
            mouse_mode: TmuxPaneMouseMode::default(),
            current_path: path.to_string(),
            current_command: command.to_string(),
        }
    }

    #[test]
    fn predicted_prompt_seed_title_uses_cwd_template_when_explicit_is_enabled() {
        let config = TabTitleConfig::default();
        let title = TerminalView::predicted_prompt_seed_title(&config, Some("~/projects/termy"));
        assert_eq!(title.as_deref(), Some("~/projects/termy"));
    }

    #[test]
    fn predicted_prompt_seed_title_formats_absolute_home_path_relative() {
        let config = TabTitleConfig::default();
        let home = TerminalView::user_home_dir().expect("home dir");
        let cwd = home.join("projects").join("termy");
        let expected = expected_home_relative_path(&["projects", "termy"]);
        let title = TerminalView::predicted_prompt_seed_title(
            &config,
            Some(cwd.to_string_lossy().as_ref()),
        );

        assert_eq!(title.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn predicted_prompt_seed_title_skips_static_only_priority() {
        let config = TabTitleConfig {
            priority: vec![TabTitleSource::Manual, TabTitleSource::Fallback],
            ..Default::default()
        };

        let title = TerminalView::predicted_prompt_seed_title(&config, Some("~/projects/termy"));
        assert!(title.is_none());
    }

    #[test]
    fn predicted_prompt_seed_title_ignores_empty_resolved_output() {
        let config = TabTitleConfig {
            prompt_format: "{cwd}".to_string(),
            ..Default::default()
        };

        let title = TerminalView::predicted_prompt_seed_title(&config, None);
        assert!(title.is_none());
    }

    #[test]
    fn smart_mode_shell_fallback_enabled_when_shell_integration_is_off() {
        let config = TabTitleConfig {
            shell_integration: false,
            ..Default::default()
        };
        assert!(TerminalView::smart_mode_shell_fallback_enabled(&config));
    }

    #[test]
    fn smart_mode_shell_fallback_disabled_when_shell_integration_is_on() {
        let config = TabTitleConfig::default();
        assert!(!TerminalView::smart_mode_shell_fallback_enabled(&config));
    }

    #[test]
    fn smart_mode_shell_fallback_disabled_for_non_smart_mode() {
        let config = TabTitleConfig {
            mode: termy_core::config_core::TabTitleMode::Shell,
            shell_integration: false,
            ..Default::default()
        };
        assert!(!TerminalView::smart_mode_shell_fallback_enabled(&config));
    }

    #[test]
    fn title_source_candidate_prefers_shell_when_smart_shell_fallback_is_enabled() {
        let candidate = TerminalView::title_source_candidate(
            TabTitleSource::Explicit,
            None,
            Some("explicit"),
            false,
            false,
            Some("shell"),
            "fallback",
            true,
        );
        assert_eq!(candidate, Some("shell"));
    }

    #[test]
    fn title_source_candidate_uses_explicit_when_shell_is_unavailable() {
        let candidate = TerminalView::title_source_candidate(
            TabTitleSource::Explicit,
            None,
            Some("explicit"),
            false,
            false,
            None,
            "fallback",
            true,
        );
        assert_eq!(candidate, Some("explicit"));
    }

    #[test]
    fn title_source_candidate_prefers_shell_over_predicted_explicit_title() {
        let candidate = TerminalView::title_source_candidate(
            TabTitleSource::Explicit,
            None,
            Some("predicted"),
            true,
            true,
            Some("shell"),
            "fallback",
            false,
        );
        assert_eq!(candidate, Some("shell"));
    }

    #[test]
    fn title_source_candidate_keeps_predicted_explicit_when_shell_source_is_disabled() {
        let candidate = TerminalView::title_source_candidate(
            TabTitleSource::Explicit,
            None,
            Some("predicted"),
            true,
            false,
            Some("shell"),
            "fallback",
            false,
        );
        assert_eq!(candidate, Some("predicted"));
    }

    #[test]
    fn resolve_template_replaces_known_tokens_in_single_pass() {
        let resolved = TerminalView::resolve_template(
            "cwd={cwd} command={command}",
            Some("{command}"),
            Some("{cwd}"),
        );
        assert_eq!(resolved, "cwd={command} command={cwd}");
    }

    #[test]
    fn resolve_template_leaves_unknown_brace_tokens_unchanged() {
        let resolved =
            TerminalView::resolve_template("start {unknown} end", Some("cwd"), Some("cmd"));
        assert_eq!(resolved, "start {unknown} end");
    }

    #[test]
    fn resolve_prompt_title_formats_absolute_home_paths_relative() {
        let home = TerminalView::user_home_dir().expect("home dir");
        let cwd = home.join("projects").join("termy");
        let expected = expected_home_relative_path(&["projects", "termy"]);

        assert_eq!(
            TerminalView::resolve_prompt_title("{cwd}", Some(cwd.to_string_lossy().as_ref())),
            expected
        );
    }

    #[test]
    fn resolve_prompt_title_leaves_non_home_absolute_paths_unchanged() {
        assert_eq!(
            TerminalView::resolve_prompt_title("{cwd}", Some("/tmp/work")),
            "/tmp/work"
        );
    }

    #[test]
    fn is_shell_command_matches_fixed_shell_set_case_insensitively() {
        assert!(TerminalView::is_shell_command("zsh"));
        assert!(TerminalView::is_shell_command("PwSh"));
        assert!(TerminalView::is_shell_command("/bin/bash"));
        assert!(TerminalView::is_shell_command("-zsh"));
        assert!(TerminalView::is_shell_command("pwsh.exe"));
        assert!(TerminalView::is_shell_command("bash -l"));
        assert!(TerminalView::is_shell_command("cmd"));
        assert!(!TerminalView::is_shell_command("sleep"));
        assert!(!TerminalView::is_shell_command(""));
    }

    #[test]
    fn derive_tmux_shell_title_uses_prompt_format_for_shell_commands() {
        let tab_title = TabTitleConfig {
            prompt_format: "cwd:{cwd}".to_string(),
            ..Default::default()
        };
        let pane = pane_with("/tmp/work", "zsh");

        let title = TerminalView::derive_tmux_shell_title(&tab_title, &pane);
        assert_eq!(title.as_deref(), Some("cwd:/tmp/work"));
    }

    #[test]
    fn derive_tmux_shell_title_formats_home_paths_relative() {
        let tab_title = TabTitleConfig {
            prompt_format: "cwd:{cwd}".to_string(),
            ..Default::default()
        };
        let home = TerminalView::user_home_dir().expect("home dir");
        let pane = pane_with(
            home.join("work").join("project").to_string_lossy().as_ref(),
            "zsh",
        );
        let expected = format!("cwd:{}", expected_home_relative_path(&["work", "project"]));

        let title = TerminalView::derive_tmux_shell_title(&tab_title, &pane);
        assert_eq!(title.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn derive_tmux_shell_title_uses_command_format_for_non_shell_commands() {
        let tab_title = TabTitleConfig {
            command_format: "run:{command}".to_string(),
            ..Default::default()
        };
        let pane = pane_with("/tmp/work", "sleep");

        let title = TerminalView::derive_tmux_shell_title(&tab_title, &pane);
        assert_eq!(title.as_deref(), Some("run:sleep"));
    }

    #[test]
    fn derive_tmux_shell_title_returns_none_when_resolved_title_is_empty() {
        let tab_title = TabTitleConfig {
            command_format: " ".to_string(),
            ..Default::default()
        };
        let pane = pane_with("/tmp/work", "sleep");

        let title = TerminalView::derive_tmux_shell_title(&tab_title, &pane);
        assert!(title.is_none());
    }

    #[allow(clippy::unnecessary_wraps)]
    fn texts(left: &str, right: Option<&str>) -> Option<PaneLabelTexts> {
        Some(PaneLabelTexts {
            left: left.to_string(),
            right: right.map(str::to_string),
        })
    }

    #[test]
    fn pane_label_custom_title_goes_left_and_path_right() {
        assert_eq!(
            TerminalView::pane_label_texts(Some("build"), Some(r"C:\work\app")),
            texts("build", Some(r"C:\work\app"))
        );
    }

    #[test]
    fn pane_label_title_equal_to_path_counts_as_no_title() {
        // 大小写、分隔符、末尾分隔符不同，仍视为同一路径。
        assert_eq!(
            TerminalView::pane_label_texts(Some("c:/work/app/"), Some(r"C:\work\app")),
            texts(r"C:\work\app", None)
        );
    }

    #[test]
    fn pane_label_without_title_uses_path_as_title() {
        assert_eq!(
            TerminalView::pane_label_texts(None, Some(r"C:\work")),
            texts(r"C:\work", None)
        );
        assert_eq!(
            TerminalView::pane_label_texts(Some("  "), Some(r"C:\work")),
            texts(r"C:\work", None)
        );
    }

    #[test]
    fn pane_label_without_known_cwd_shows_title_only() {
        assert_eq!(
            TerminalView::pane_label_texts(Some("vim main.rs"), None),
            texts("vim main.rs", None)
        );
    }

    #[test]
    fn pane_label_is_none_without_any_text() {
        assert_eq!(TerminalView::pane_label_texts(None, None), None);
        assert_eq!(TerminalView::pane_label_texts(Some(""), Some(" ")), None);
    }

    fn map_of(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn manual_pane_title_wins_over_reported_title() {
        let manual = map_of(&[("p1", "build")]);
        let reported = map_of(&[("p1", "zsh"), ("p2", "vim")]);
        assert_eq!(
            TerminalView::resolve_pane_title(&manual, &reported, "p1"),
            Some("build")
        );
        assert_eq!(
            TerminalView::resolve_pane_title(&manual, &reported, "p2"),
            Some("vim")
        );
        assert_eq!(
            TerminalView::resolve_pane_title(&manual, &reported, "p3"),
            None
        );
    }

    #[test]
    fn clearing_manual_pane_title_falls_back_to_reported_title() {
        let mut manual = HashMap::new();
        let reported = map_of(&[("p1", "zsh")]);
        assert!(TerminalView::apply_manual_pane_title(
            &mut manual,
            "p1",
            "  build  "
        ));
        assert_eq!(
            TerminalView::resolve_pane_title(&manual, &reported, "p1"),
            Some("build")
        );
        // 重复提交相同名字不算变化。
        assert!(!TerminalView::apply_manual_pane_title(
            &mut manual,
            "p1",
            "build"
        ));
        // 空串清除手动名，恢复终端标题。
        assert!(TerminalView::apply_manual_pane_title(
            &mut manual,
            "p1",
            "   "
        ));
        assert_eq!(
            TerminalView::resolve_pane_title(&manual, &reported, "p1"),
            Some("zsh")
        );
        assert!(!TerminalView::apply_manual_pane_title(
            &mut manual,
            "p1",
            ""
        ));
    }

    #[test]
    fn manual_pane_title_is_truncated_and_single_line() {
        let mut manual = HashMap::new();
        let long = "a".repeat(MAX_TAB_TITLE_CHARS + 20);
        assert!(TerminalView::apply_manual_pane_title(
            &mut manual,
            "p1",
            &long
        ));
        assert_eq!(manual["p1"].chars().count(), MAX_TAB_TITLE_CHARS);
        assert!(TerminalView::apply_manual_pane_title(
            &mut manual,
            "p2",
            "a\nb"
        ));
        assert_eq!(manual["p2"], "a b");
    }

    #[test]
    fn msys_cwd_is_converted_to_windows_drive_path() {
        let cases = [
            ("/c/Users/jelon", r"C:\Users\jelon"),
            ("/d", r"D:\"),
            ("/cygdrive/e/work/api", r"E:\work\api"),
            ("/C/Program Files", r"C:\Program Files"),
        ];
        for (raw, expected) in cases {
            assert_eq!(TerminalView::normalize_msys_cwd(raw), expected, "{raw}");
        }
        // 不是盘符形式的路径保持原样：Windows 路径、UNC、WSL 内部路径、相对路径。
        for raw in [
            r"C:\Users\jelon",
            r"\wsl.localhost\Ubuntu\home",
            "/home/jelon",
            "/usr/bin",
            "relative/x",
            "",
        ] {
            assert_eq!(TerminalView::normalize_msys_cwd(raw), raw, "{raw}");
        }
    }

    #[test]
    fn manual_pane_title_without_cwd_shows_only_left() {
        let manual = map_of(&[("p1", "build")]);
        let reported = HashMap::new();
        let title = TerminalView::resolve_pane_title(&manual, &reported, "p1");
        assert_eq!(
            TerminalView::pane_label_texts(title, None),
            texts("build", None)
        );
    }
}
