use super::{
    NativePaneLayoutTree, NativePaneZoomSnapshot, TabId, TerminalTab, workspaces::WorkspaceEntry,
};
use std::collections::{HashMap, HashSet};

/// Mutable terminal-session state that must stay coherent across tab,
/// workspace, and native-pane operations.
pub(super) struct SessionState {
    pub(super) tabs: Vec<TerminalTab>,
    /// All workspaces in sidebar order. The entry at `active_workspace` always
    /// has an empty `tabs` vec: the active workspace's tabs live in `tabs` so
    /// the existing tab machinery only ever sees the visible strip.
    pub(super) workspaces: Vec<WorkspaceEntry>,
    pub(super) active_workspace: usize,
    pub(super) next_workspace_id: u64,
    pub(super) native_pane_zoom_snapshots: HashMap<TabId, NativePaneZoomSnapshot>,
    pub(super) native_pane_layout_trees: HashMap<TabId, NativePaneLayoutTree>,
    pub(super) next_tab_id: TabId,
    pub(super) active_tab: usize,
}

impl SessionState {
    pub(super) fn new() -> Self {
        Self {
            tabs: Vec::new(),
            workspaces: vec![WorkspaceEntry::new(1)],
            active_workspace: 0,
            next_workspace_id: 2,
            native_pane_zoom_snapshots: HashMap::new(),
            native_pane_layout_trees: HashMap::new(),
            next_tab_id: 1,
            active_tab: 0,
        }
    }

    /// 所有仍存活的窗格 id：当前标签条、已暂存工作区的标签，以及缩放标签里被藏起来的窗格。
    ///
    /// 返回借用的 id 集合，用来判断按窗格 id 存的侧表条目是否已成孤儿。
    pub(super) fn live_pane_ids(&self) -> HashSet<&str> {
        let zoomed = self
            .native_pane_zoom_snapshots
            .values()
            .flat_map(|snapshot| snapshot.other_panes.iter());
        self.tabs
            .iter()
            .chain(self.workspaces.iter().flat_map(|entry| entry.tabs.iter()))
            .flat_map(|tab| tab.panes.iter())
            .chain(zoomed)
            .map(|pane| pane.id.as_str())
            .collect()
    }
}
