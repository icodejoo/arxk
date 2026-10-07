//! Git 分支监听：监听各窗格所在仓库的 HEAD 变化，驱动标题里的 `{branch}` 刷新。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui_kit::{AsyncApp, Context, WeakEntity};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::terminal_view::TerminalView;

/// 防抖间隔：收到第一个 HEAD 事件后等这么久，把期间的多次事件合并成一次读取。
const DEBOUNCE_MS: u64 = 150;
/// git 目录里需要关注的文件名。
const HEAD_FILE: &str = "HEAD";
/// 分离头指针时展示的短 hash 位数。
const SHORT_HASH_LEN: usize = 7;
/// `.git` 文件里 gitdir 指针的前缀。
const GITDIR_PREFIX: &str = "gitdir:";
/// 标题与分支的连接符，格式固定为 `{title}::{branch}`。
pub(crate) const BRANCH_SEPARATOR: &str = "::";
/// HEAD 里分支引用的前缀。
const HEADS_REF_PREFIX: &str = "refs/heads/";
/// reftable 仓库的 HEAD 占位分支名（真实分支存在 reftable 里，HEAD 文件只是占位）。
const REFTABLE_PLACEHOLDER_BRANCH: &str = ".invalid";
/// cwd -> 仓库目录缓存的有效期：超时后重新向上查找，才能发现嵌套 `git init` 或被删除的 `.git`。
const CWD_CACHE_TTL: Duration = Duration::from_secs(5);

/// 把 git 目录规范成真实路径，让它与 notify 事件里的路径一致。
/// macOS 的 FSEvents 返回真实路径、Linux 别名共用同一个 inotify wd，不规范就对不上；
/// Windows 的 canonicalize 会变成 `\\?\` 前缀，与事件路径反而不一致，所以不处理。
fn canonical_git_dir(dir: PathBuf) -> PathBuf {
    if cfg!(windows) {
        dir
    } else {
        std::fs::canonicalize(&dir).unwrap_or(dir)
    }
}

/// 从 cwd 向上查找 git 目录；`.git` 是文件（worktree/子模块）时解析其 `gitdir:` 指针。
/// 找不到返回 None（此时不监听）。
fn find_git_dir(cwd: &Path) -> Option<PathBuf> {
    for dir in cwd.ancestors() {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(canonical_git_dir(dot_git));
        }
        if dot_git.is_file() {
            let content = std::fs::read_to_string(&dot_git).ok()?;
            let target = content.trim().strip_prefix(GITDIR_PREFIX)?.trim();
            let target = Path::new(target);
            return Some(canonical_git_dir(if target.is_absolute() {
                target.to_path_buf()
            } else {
                dir.join(target)
            }));
        }
    }
    None
}

/// 解析 HEAD 文本：`ref: refs/heads/xxx` 返回分支名，否则视为 hash 取前 7 位。
/// 内容为空或是 reftable 的占位 HEAD（拿不到真实分支）时返回 None。
fn parse_head_content(content: &str) -> Option<String> {
    let content = content.trim();
    let head = match content.strip_prefix("ref:") {
        Some(r) => {
            let r = r.trim();
            r.strip_prefix(HEADS_REF_PREFIX).unwrap_or(r).to_string()
        }
        None => content.chars().take(SHORT_HASH_LEN).collect(),
    };
    (!head.is_empty() && head != REFTABLE_PLACEHOLDER_BRANCH).then_some(head)
}

/// 读取 git 目录下的 HEAD 并解析；读不到或无法确定分支返回 None。
fn read_head(git_dir: &Path) -> Option<String> {
    std::fs::read_to_string(git_dir.join(HEAD_FILE))
        .ok()
        .and_then(|c| parse_head_content(&c))
}

/// 给标题追加 `::分支` 后缀。
///
/// - `title`：原标题（取得所有权，直接在其上追加）。
/// - `branch`：分支名；为 `None` 或空串（非 git 目录）时不追加。
///
/// 返回追加后的标题。
///
/// ```ignore
/// assert_eq!(append_branch("build".into(), Some("main")), "build::main");
/// assert_eq!(append_branch("build".into(), None), "build");
/// ```
pub(crate) fn append_branch(mut title: String, branch: Option<&str>) -> String {
    if let Some(branch) = branch.filter(|b| !b.is_empty()) {
        title.push_str(BRANCH_SEPARATOR);
        title.push_str(branch);
    }
    title
}

/// 把一批 notify 事件路径合并成“HEAD 发生变化的 git 目录”集合（防抖合并的纯函数部分）。
fn coalesce_head_events<'a>(paths: impl IntoIterator<Item = &'a Path>) -> HashSet<PathBuf> {
    paths
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n == HEAD_FILE))
        .filter_map(|p| p.parent().map(Path::to_path_buf))
        .collect()
}

/// 某个 cwd 的仓库查找结果（含“不在仓库里”）及查找时间。
struct CwdEntry {
    /// 所在 git 目录；不在仓库里为 None。
    dir: Option<PathBuf>,
    /// 查找时间，用来判断是否过期。
    resolved_at: Instant,
}

impl CwdEntry {
    /// 条目是否仍可信：未过期，且记录的 git 目录还在。
    fn is_fresh(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.resolved_at) < CWD_CACHE_TTL
            && self.dir.as_deref().is_none_or(Path::is_dir)
    }
}

/// 视图级 git 分支监听器：同一 git 目录只挂一个 watcher。
pub(crate) struct ViewGitWatcher {
    /// notify 监听器（非递归监听 git 目录本身）。
    watcher: RecommendedWatcher,
    /// 已挂上监听的 git 目录 -> 最近一次读到的 HEAD（读不到为 None）。
    watched: HashMap<PathBuf, Option<String>>,
    /// 窗格 cwd -> 所在 git 目录的查找结果（渲染时 O(1) 查分支，避免逐帧读磁盘；含非仓库 cwd 的负缓存）。
    cwd_dirs: HashMap<String, CwdEntry>,
}

impl ViewGitWatcher {
    /// 创建监听器并返回 HEAD 事件接收端（交给后台防抖任务）；系统监听器初始化失败返回 None 并记日志。
    fn new() -> Option<(Self, flume::Receiver<PathBuf>)> {
        let (tx, rx) = flume::unbounded();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            match res {
                // 丢弃读取类事件：read_head 自己打开 HEAD 在 Linux 上会触发 Access(Open)，造成自激。
                Ok(event) if !event.kind.is_access() => {
                    for dir in coalesce_head_events(event.paths.iter().map(PathBuf::as_path)) {
                        let _ = tx.send(dir);
                    }
                }
                Ok(_) => {}
                Err(error) => log::warn!("git HEAD watcher error: {error}"),
            }
        })
        .inspect_err(|error| log::warn!("Failed to create git HEAD watcher: {error}"))
        .ok()?;
        let this = Self {
            watcher,
            watched: HashMap::new(),
            cwd_dirs: HashMap::new(),
        };
        Some((this, rx))
    }

    /// 该 cwd 的仓库缓存是否缺失或已过期（需要重新对齐监听）。
    fn is_stale(&self, cwd: &str) -> bool {
        let now = Instant::now();
        self.cwd_dirs
            .get(cwd)
            .is_none_or(|entry| !entry.is_fresh(now))
    }

    /// 按当前所有存活窗格的 cwd 重新对齐监听：新仓库开监听，没人用的仓库释放。
    /// 仍有效的 cwd 查找结果（含非仓库）直接复用，过期或目录已消失的才重新向上遍历磁盘。
    fn sync<'a>(&mut self, cwds: impl IntoIterator<Item = &'a str>) {
        let now = Instant::now();
        let mut next: HashMap<String, CwdEntry> = HashMap::new();
        for cwd in cwds {
            if next.contains_key(cwd) {
                continue;
            }
            let entry = match self.cwd_dirs.remove(cwd) {
                Some(entry) if entry.is_fresh(now) => entry,
                _ => CwdEntry {
                    dir: find_git_dir(Path::new(cwd)),
                    resolved_at: now,
                },
            };
            next.insert(cwd.to_string(), entry);
        }
        self.cwd_dirs = next;

        let wanted: HashSet<&PathBuf> = self
            .cwd_dirs
            .values()
            .filter_map(|entry| entry.dir.as_ref())
            .collect();
        self.watched.retain(|dir, _| {
            let keep = wanted.contains(dir);
            if !keep {
                let _ = self.watcher.unwatch(dir);
            }
            keep
        });
        for dir in wanted {
            if self.watched.contains_key(dir) {
                continue;
            }
            match self.watcher.watch(dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    self.watched.insert(dir.clone(), read_head(dir));
                }
                Err(error) => log::warn!("Failed to watch git dir {}: {error}", dir.display()),
            }
        }
    }

    /// 某个 cwd 当前所在的分支名（或短 hash）；非 git 目录、尚未同步或无法确定分支返回 None。
    fn branch_for(&self, cwd: &str) -> Option<&str> {
        let dir = self.cwd_dirs.get(cwd)?.dir.as_ref()?;
        self.watched.get(dir)?.as_deref()
    }

    /// 重新读取给定目录的 HEAD，返回是否有目录的分支发生了变化（或目录消失）。
    ///
    /// 目录消失时释放其监听并作废指向它的 cwd 缓存；目录仍在则重挂一次监听，
    /// 因为 `.git` 被删后重建，旧监听已被系统摘除。
    fn refresh(&mut self, dirs: &HashSet<PathBuf>) -> bool {
        let mut changed = false;
        for dir in dirs {
            if !self.watched.contains_key(dir) {
                continue;
            }
            if !dir.is_dir() {
                let _ = self.watcher.unwatch(dir);
                self.watched.remove(dir);
                self.cwd_dirs
                    .retain(|_, entry| entry.dir.as_ref() != Some(dir));
                changed = true;
                continue;
            }
            let _ = self.watcher.unwatch(dir);
            if let Err(error) = self.watcher.watch(dir, RecursiveMode::NonRecursive) {
                log::warn!("Failed to re-watch git dir {}: {error}", dir.display());
            }
            let head = read_head(dir);
            if let Some(slot) = self.watched.get_mut(dir)
                && *slot != head
            {
                *slot = head;
                changed = true;
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

    /// 某个窗格的 git 监听是否需要重新对齐：监听器还没建、或该窗格 cwd 的仓库缓存过期。
    /// 监听器创建失败过则恒为 false，避免每个提示符都重试。
    pub(crate) fn git_watch_needs_sync(&self, pane_id: &str) -> bool {
        if self.git_watch_unavailable {
            return false;
        }
        match (&self.git_watcher, self.pane_cwds.get(pane_id)) {
            (None, _) => true,
            (Some(watcher), Some(cwd)) => watcher.is_stale(cwd),
            (Some(_), None) => false,
        }
    }

    /// 对齐 git 监听（首次调用时创建 watcher 与后台防抖任务）。
    /// 监听范围是所有存活窗格已知的 cwd；创建 watcher 失败后不再重试。
    pub(crate) fn sync_git_watch(&mut self, cx: &mut Context<Self>) {
        if self.git_watch_unavailable {
            return;
        }
        let live = self.session.live_pane_ids();
        // 还没有任何窗格上报过目录时不必创建系统监听器。
        if self.git_watcher.is_none() && !self.pane_cwds.keys().any(|id| live.contains(id.as_str()))
        {
            return;
        }
        let mut watcher = if let Some(watcher) = self.git_watcher.take() {
            watcher
        } else {
            let Some((watcher, rx)) = ViewGitWatcher::new() else {
                self.git_watch_unavailable = true;
                return;
            };
            Self::spawn_git_debounce(rx, cx);
            watcher
        };
        watcher.sync(
            self.pane_cwds
                .iter()
                .filter(|(id, _)| live.contains(id.as_str()))
                .map(|(_, cwd)| cwd.as_str()),
        );
        self.git_watcher = Some(watcher);
    }

    /// 后台防抖任务：阻塞等第一个 HEAD 事件，再等一个防抖间隔合并后续事件，
    /// 只读一次，变了就重新对齐监听并刷新标题。空闲时不唤醒；监听器被释放（通道关闭）后退出。
    fn spawn_git_debounce(rx: flume::Receiver<PathBuf>, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            while let Ok(first) = rx.recv_async().await {
                cx.background_executor()
                    .timer(Duration::from_millis(DEBOUNCE_MS))
                    .await;
                let mut changed: HashSet<PathBuf> = rx.try_iter().collect();
                changed.insert(first);
                let alive = cx
                    .update(|cx| {
                        this.update(cx, |view, cx| {
                            let any = view
                                .git_watcher
                                .as_mut()
                                .is_some_and(|w| w.refresh(&changed));
                            if any {
                                // 目录消失会作废缓存，重新对齐后才能挂上重建的仓库。
                                view.sync_git_watch(cx);
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

    /// 建一个带 `.git/HEAD` 的临时仓库，返回 (临时目录, git 目录, 仓库根路径文本)。
    fn temp_repo(branch: &str) -> (tempfile::TempDir, PathBuf, String) {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(git.join(HEAD_FILE), format!("ref: refs/heads/{branch}\n")).unwrap();
        let cwd = tmp.path().to_string_lossy().to_string();
        (tmp, canonical_git_dir(git), cwd)
    }

    #[test]
    fn parse_head_branch_and_hash() {
        assert_eq!(
            parse_head_content("ref: refs/heads/feat/x\n").as_deref(),
            Some("feat/x")
        );
        assert_eq!(
            parse_head_content("a1b2c3d4e5f6a7\n").as_deref(),
            Some("a1b2c3d")
        );
    }

    #[test]
    fn parse_head_ignores_empty_and_reftable_placeholder() {
        assert_eq!(parse_head_content("  \n"), None);
        assert_eq!(parse_head_content("ref: refs/heads/.invalid\n"), None);
    }

    #[test]
    fn find_git_dir_found_and_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("a/b");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        assert_eq!(
            find_git_dir(&sub),
            Some(canonical_git_dir(tmp.path().join(".git")))
        );

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
        let found = find_git_dir(&wt).unwrap();
        assert_eq!(found, canonical_git_dir(real));
        assert_eq!(read_head(&found).as_deref(), Some("wt"));
    }

    #[cfg(unix)]
    #[test]
    fn find_git_dir_resolves_symlinked_cwd_to_real_path() {
        let (tmp, git, _) = temp_repo("main");
        let link_holder = tempfile::tempdir().unwrap();
        let link = link_holder.path().join("link");
        std::os::unix::fs::symlink(tmp.path(), &link).unwrap();
        assert_eq!(find_git_dir(&link), Some(git));
    }

    #[test]
    fn append_branch_formats_and_skips_empty() {
        assert_eq!(append_branch("build".into(), Some("main")), "build::main");
        assert_eq!(append_branch("build".into(), None), "build");
        assert_eq!(append_branch("build".into(), Some("")), "build");
    }

    #[test]
    fn branch_for_tracks_cwd_and_head() {
        let (_tmp, git, cwd) = temp_repo("dev");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str()]);
        assert_eq!(w.branch_for(&cwd), Some("dev"));
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/next\n").unwrap();
        assert!(w.refresh(&HashSet::from([git])));
        assert_eq!(w.branch_for(&cwd), Some("next"));
        w.sync(std::iter::empty());
        assert_eq!(w.branch_for(&cwd), None);
    }

    #[test]
    fn panes_in_same_repo_share_one_watch_and_release_together() {
        let (_tmp, git, cwd) = temp_repo("main");
        let sub = format!("{cwd}{}sub", std::path::MAIN_SEPARATOR);
        fs::create_dir(&sub).unwrap();
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str(), sub.as_str(), cwd.as_str()]);
        assert_eq!(w.watched.len(), 1);
        assert!(w.watched.contains_key(&git));
        assert_eq!(w.branch_for(&sub), Some("main"));
        // 还剩一个窗格时仓库保持监听；全部离开才释放。
        w.sync([sub.as_str()]);
        assert_eq!(w.watched.len(), 1);
        w.sync(std::iter::empty());
        assert!(w.watched.is_empty());
    }

    #[test]
    fn fresh_cwd_cache_is_reused_without_rescanning() {
        let (_tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str()]);
        let resolved_at = w.cwd_dirs[&cwd].resolved_at;
        assert!(!w.is_stale(&cwd));
        w.sync([cwd.as_str()]);
        assert_eq!(w.cwd_dirs[&cwd].resolved_at, resolved_at);
        assert_eq!(w.cwd_dirs[&cwd].dir.as_deref(), Some(git.as_path()));
    }

    #[test]
    fn non_repo_cwd_is_negative_cached_then_rechecked_after_ttl() {
        let tmp = tempfile::tempdir().unwrap();
        // 临时目录祖先里有 .git 时无法构造“非仓库”，直接跳过。
        if tmp.path().ancestors().any(|d| d.join(".git").exists()) {
            return;
        }
        let cwd = tmp.path().to_string_lossy().to_string();
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str()]);
        assert!(w.cwd_dirs[&cwd].dir.is_none());
        assert!(!w.is_stale(&cwd));
        // 之后在该目录 git init：缓存过期前仍是旧结果，过期后才被发现。
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/new\n").unwrap();
        w.sync([cwd.as_str()]);
        assert_eq!(w.branch_for(&cwd), None);
        w.cwd_dirs.get_mut(&cwd).unwrap().resolved_at -= CWD_CACHE_TTL;
        assert!(w.is_stale(&cwd));
        w.sync([cwd.as_str()]);
        assert_eq!(w.branch_for(&cwd), Some("new"));
    }

    #[test]
    fn deleted_git_dir_is_released_and_recreated_dir_is_rewatched() {
        let (tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        w.sync([cwd.as_str()]);
        fs::remove_dir_all(&git).unwrap();
        assert!(w.refresh(&HashSet::from([git.clone()])));
        assert!(w.watched.is_empty());
        assert_eq!(w.branch_for(&cwd), None);
        // 重建后重新对齐即可重新挂上监听。
        fs::create_dir(tmp.path().join(".git")).unwrap();
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/again\n").unwrap();
        w.sync([cwd.as_str()]);
        assert_eq!(w.branch_for(&cwd), Some("again"));
    }

    #[cfg(windows)]
    #[test]
    fn msys_cwd_normalizes_into_a_findable_git_dir() {
        let (_tmp, git, cwd) = temp_repo("main");
        // 还原成 Git Bash 上报的 `/c/Users/...` 形式，再走 规范化 -> 路径 -> 查找 的完整链路。
        let mut chars = cwd.chars();
        let drive = chars.next().unwrap().to_ascii_lowercase();
        let rest = chars.as_str().trim_start_matches(':').replace('\\', "/");
        let reported = format!("/{drive}{rest}");
        let normalized = TerminalView::normalize_drive_style_cwd(&reported);
        assert_eq!(find_git_dir(Path::new(&normalized)), Some(git));
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
