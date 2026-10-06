//! Git 分支监听：监听各窗格所在仓库的 HEAD 变化，驱动标题里的 `{branch}` 刷新。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use gpui_kit::{AsyncApp, Context, WeakEntity};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::terminal_view::TerminalView;

/// 防抖间隔：一个轮询周期内的多次 HEAD 事件合并成一次读取。
const DEBOUNCE_MS: u64 = 150;
/// git 目录里需要关注的文件名。
const HEAD_FILE: &str = "HEAD";
/// 分离头指针时展示的短 hash 位数。
const SHORT_HASH_LEN: usize = 7;
/// `.git` 文件里 gitdir 指针的前缀。
const GITDIR_PREFIX: &str = "gitdir:";

/// 从 cwd 向上查找 git 目录；`.git` 是文件（worktree/子模块）时解析其 `gitdir:` 指针。
/// 找不到返回 None（此时不监听）。
pub(crate) fn find_git_dir(cwd: &Path) -> Option<PathBuf> {
    for dir in cwd.ancestors() {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let content = std::fs::read_to_string(&dot_git).ok()?;
            let target = content.trim().strip_prefix(GITDIR_PREFIX)?.trim();
            let target = Path::new(target);
            return Some(if target.is_absolute() {
                target.to_path_buf()
            } else {
                dir.join(target)
            });
        }
    }
    None
}

/// 解析 HEAD 文本：`ref: refs/heads/xxx` 返回分支名，否则视为 hash 取前 7 位。
pub(crate) fn parse_head_content(content: &str) -> String {
    let content = content.trim();
    match content.strip_prefix("ref:") {
        Some(r) => {
            let r = r.trim();
            r.strip_prefix("refs/heads/").unwrap_or(r).to_string()
        }
        None => content.chars().take(SHORT_HASH_LEN).collect(),
    }
}

/// 读取 git 目录下的 HEAD 并解析；读不到返回 None。
pub(crate) fn read_head(git_dir: &Path) -> Option<String> {
    std::fs::read_to_string(git_dir.join(HEAD_FILE))
        .ok()
        .map(|c| parse_head_content(&c))
        .filter(|s| !s.is_empty())
}

/// 标题与分支的连接符，格式固定为 `{title}::{branch}`。
const BRANCH_SEPARATOR: &str = "::";

/// 给标题追加分支后缀；没有分支（非 git 目录）时原样返回。
pub(crate) fn append_branch(title: &str, branch: Option<&str>) -> String {
    match branch.filter(|b| !b.is_empty()) {
        Some(branch) => format!("{title}{BRANCH_SEPARATOR}{branch}"),
        None => title.to_string(),
    }
}

/// 按 git 目录去重的引用计数表（纯逻辑，不碰文件系统）。
#[derive(Default, Debug)]
pub(crate) struct GitDirRefs {
    /// git 目录 -> 引用它的窗格数。
    counts: HashMap<PathBuf, usize>,
}

impl GitDirRefs {
    /// 用最新的引用集合（每个窗格一项，可重复）重算计数，
    /// 返回 (需要新建监听的目录, 需要释放的目录)。
    pub(crate) fn reconcile(
        &mut self,
        dirs: impl IntoIterator<Item = PathBuf>,
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let mut next: HashMap<PathBuf, usize> = HashMap::new();
        for d in dirs {
            *next.entry(d).or_default() += 1;
        }
        let added = next
            .keys()
            .filter(|d| !self.counts.contains_key(*d))
            .cloned()
            .collect();
        let removed = self
            .counts
            .keys()
            .filter(|d| !next.contains_key(*d))
            .cloned()
            .collect();
        self.counts = next;
        (added, removed)
    }

    /// 某个 git 目录当前的引用数。
    pub(crate) fn count(&self, dir: &Path) -> usize {
        self.counts.get(dir).copied().unwrap_or(0)
    }
}

/// 把一批 notify 事件路径合并成“HEAD 发生变化的 git 目录”集合（防抖合并的纯函数部分）。
pub(crate) fn coalesce_head_events<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
) -> HashSet<PathBuf> {
    paths
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n == HEAD_FILE))
        .filter_map(|p| p.parent().map(Path::to_path_buf))
        .collect()
}

/// 视图级 git 分支监听器：同一 git 目录只挂一个 watcher。
pub(crate) struct ViewGitWatcher {
    /// notify 监听器（非递归监听 git 目录本身）。
    watcher: RecommendedWatcher,
    /// 引用计数表。
    refs: GitDirRefs,
    /// 各 git 目录最近一次读到的 HEAD。
    heads: HashMap<PathBuf, String>,
    /// 窗格 cwd -> 所在 git 目录（渲染时 O(1) 查分支，避免逐帧读磁盘）。
    cwd_dirs: HashMap<String, PathBuf>,
    /// 事件接收端，交给后台任务轮询。
    rx: Option<Receiver<PathBuf>>,
}

impl ViewGitWatcher {
    /// 创建监听器；系统监听器初始化失败返回 None。
    pub(crate) fn new() -> Option<Self> {
        let (tx, rx): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                for dir in coalesce_head_events(event.paths.iter().map(PathBuf::as_path)) {
                    let _ = tx.send(dir);
                }
            }
        })
        .ok()?;
        Some(Self {
            watcher,
            refs: GitDirRefs::default(),
            heads: HashMap::new(),
            cwd_dirs: HashMap::new(),
            rx: Some(rx),
        })
    }

    /// 取走事件接收端（只能取一次，供后台任务使用）。
    pub(crate) fn take_receiver(&mut self) -> Option<Receiver<PathBuf>> {
        self.rx.take()
    }

    /// 按当前所有存活窗格的 cwd 重新对齐监听：新仓库开监听，没人用的仓库释放。
    pub(crate) fn sync<'a>(&mut self, cwds: impl IntoIterator<Item = &'a str>) {
        self.cwd_dirs = cwds
            .into_iter()
            .filter_map(|c| Some((c.to_string(), find_git_dir(Path::new(c))?)))
            .collect();
        let (added, removed) = self.refs.reconcile(self.cwd_dirs.values().cloned());
        for dir in removed {
            let _ = self.watcher.unwatch(&dir);
            self.heads.remove(&dir);
        }
        for dir in added {
            if self
                .watcher
                .watch(&dir, RecursiveMode::NonRecursive)
                .is_ok()
            {
                if let Some(head) = read_head(&dir) {
                    self.heads.insert(dir, head);
                }
            }
        }
    }

    /// 某个 cwd 当前所在的分支名（或短 hash）；非 git 目录或尚未同步返回 None。
    pub(crate) fn branch_for(&self, cwd: &str) -> Option<&str> {
        self.heads.get(self.cwd_dirs.get(cwd)?).map(String::as_str)
    }

    /// 重新读取给定目录的 HEAD，返回是否有目录的分支发生了变化。
    pub(crate) fn refresh(&mut self, dirs: &HashSet<PathBuf>) -> bool {
        let mut changed = false;
        for dir in dirs {
            if self.refs.count(dir) == 0 {
                continue;
            }
            if let Some(head) = read_head(dir) {
                if self.heads.insert(dir.clone(), head.clone()) != Some(head) {
                    changed = true;
                }
            }
        }
        changed
    }
}

impl TerminalView {
    /// 某个窗格 cwd 所在的分支（缓存查询，可在渲染路径调用）；无 git 或未监听返回 None。
    pub(crate) fn branch_for_pane_cwd(&self, cwd: &str) -> Option<&str> {
        self.git_watcher.as_ref()?.branch_for(cwd)
    }

    /// 对齐 git 监听（首次调用时创建 watcher 与后台防抖任务）。
    /// 监听范围是所有存活窗格已知的 cwd。
    pub(crate) fn sync_git_watch(&mut self, cx: &mut Context<Self>) {
        let live: HashSet<&str> = self
            .session
            .tabs
            .iter()
            .flat_map(|t| t.panes.iter().map(|p| p.id.as_str()))
            .collect();
        let cwds: Vec<String> = self
            .pane_cwds
            .iter()
            .filter(|(id, _)| live.contains(id.as_str()))
            .map(|(_, c)| c.clone())
            .collect();
        if self.git_watcher.is_none() {
            let Some(mut w) = ViewGitWatcher::new() else {
                return;
            };
            let rx = w.take_receiver();
            self.git_watcher = Some(w);
            if let Some(rx) = rx {
                Self::spawn_git_debounce(rx, cx);
            }
        }
        if let Some(w) = self.git_watcher.as_mut() {
            w.sync(cwds.iter().map(String::as_str));
        }
    }

    /// 后台防抖任务：每个周期把累积的 HEAD 事件合并后只读一次，变了就刷新标题。
    fn spawn_git_debounce(rx: Receiver<PathBuf>, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(DEBOUNCE_MS))
                    .await;
                let changed: HashSet<PathBuf> = rx.try_iter().collect();
                if changed.is_empty() {
                    continue;
                }
                let alive = cx
                    .update(|cx| {
                        this.update(cx, |view, cx| {
                            let any = view
                                .git_watcher
                                .as_mut()
                                .is_some_and(|w| w.refresh(&changed));
                            if any {
                                for i in 0..view.session.tabs.len() {
                                    view.refresh_tab_title(i);
                                }
                                cx.notify();
                            }
                        })
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_head_branch_and_hash() {
        assert_eq!(parse_head_content("ref: refs/heads/feat/x\n"), "feat/x");
        assert_eq!(parse_head_content("a1b2c3d4e5f6a7\n"), "a1b2c3d");
    }

    #[test]
    fn find_git_dir_found_and_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("a/b");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        assert_eq!(find_git_dir(&sub), Some(tmp.path().join(".git")));

        let other = tempfile::tempdir().unwrap();
        // 临时目录祖先里可能真有 .git，只在确实没有时断言
        if other.path().ancestors().all(|d| !d.join(".git").exists()) {
            assert_eq!(find_git_dir(other.path()), None);
        }
    }

    #[test]
    fn find_git_dir_follows_gitdir_file() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real");
        fs::create_dir(&real).unwrap();
        fs::write(real.join(HEAD_FILE), "ref: refs/heads/wt\n").unwrap();
        let wt = tmp.path().join("wt");
        fs::create_dir(&wt).unwrap();
        fs::write(wt.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();
        assert_eq!(find_git_dir(&wt), Some(real));
        assert_eq!(
            read_head(&find_git_dir(&wt).unwrap()).as_deref(),
            Some("wt")
        );
    }

    #[test]
    fn append_branch_formats_and_skips_empty() {
        assert_eq!(append_branch("build", Some("main")), "build::main");
        assert_eq!(append_branch("build", None), "build");
        assert_eq!(append_branch("build", Some("")), "build");
    }

    #[test]
    fn branch_for_tracks_cwd_and_head() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(
            git.join(HEAD_FILE),
            "ref: refs/heads/dev
",
        )
        .unwrap();
        let cwd = tmp.path().to_string_lossy().to_string();
        let mut w = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str()]);
        assert_eq!(w.branch_for(&cwd), Some("dev"));
        fs::write(
            git.join(HEAD_FILE),
            "ref: refs/heads/next
",
        )
        .unwrap();
        assert!(w.refresh(&HashSet::from([git])));
        assert_eq!(w.branch_for(&cwd), Some("next"));
        w.sync(std::iter::empty());
        assert_eq!(w.branch_for(&cwd), None);
    }

    #[test]
    fn refs_dedupe_and_release() {
        let a = PathBuf::from("/r/.git");
        let mut refs = GitDirRefs::default();
        let (added, removed) = refs.reconcile([a.clone(), a.clone()]);
        assert_eq!((added.len(), removed.len()), (1, 0));
        assert_eq!(refs.count(&a), 2);
        let (added, removed) = refs.reconcile([a.clone()]);
        assert!(added.is_empty() && removed.is_empty());
        assert_eq!(refs.count(&a), 1);
        let (_, removed) = refs.reconcile(Vec::new());
        assert_eq!(removed, vec![a]);
    }

    #[test]
    fn coalesce_keeps_only_head_and_merges() {
        let p = [
            Path::new("/r/.git/HEAD"),
            Path::new("/r/.git/HEAD"),
            Path::new("/r/.git/HEAD.lock"),
            Path::new("/r/.git/index"),
        ];
        let set = coalesce_head_events(p);
        assert_eq!(set.len(), 1);
        assert!(set.contains(Path::new("/r/.git")));
    }
}
