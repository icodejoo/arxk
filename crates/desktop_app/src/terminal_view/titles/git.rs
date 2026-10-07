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
pub(crate) const BRANCH_SEPARATOR: &str = "🔱";
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
///
/// 只接受绝对路径：相对路径的 `.git` 会按应用进程自己的工作目录解析，
/// 可能误命中应用所在仓库，导致非仓库目录也显示分支。
fn find_git_dir(cwd: &Path) -> Option<PathBuf> {
    if !cwd.is_absolute() {
        return None;
    }
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

/// 给标题追加 `🔱分支` 后缀。
///
/// - `title`：原标题（取得所有权，直接在其上追加）。
/// - `branch`：分支名；为 `None` 或空串（非 git 目录）时不追加。
///
/// 返回追加后的标题。
///
/// ```ignore
/// assert_eq!(append_branch("build".into(), Some("main")), "build🔱main");
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

/// 一次“cwd -> 仓库目录 + 分支”的后台解析请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolveRequest {
    /// 要解析的窗格 cwd。
    cwd: String,
    /// 请求序号；回填时与在途记录不一致就丢弃。
    seq: u64,
}

/// 后台解析的结果，由 UI 线程回填进缓存。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolveResult {
    /// 对应请求的 cwd。
    cwd: String,
    /// 对应请求的序号。
    seq: u64,
    /// 解析出的 git 目录；不在仓库里为 None。
    dir: Option<PathBuf>,
    /// 解析时读到的 HEAD 分支（仅在 `dir` 有值时有意义）。
    head: Option<String>,
}

/// 解析 cwd（会做磁盘 I/O，只能在后台线程调用）：向上找 git 目录并读一次 HEAD。
///
/// - `request`：解析请求。
///
/// 返回带同一序号的解析结果。
fn resolve_cwd(request: ResolveRequest) -> ResolveResult {
    let dir = find_git_dir(Path::new(&request.cwd));
    let head = dir.as_deref().and_then(read_head);
    ResolveResult {
        cwd: request.cwd,
        seq: request.seq,
        dir,
        head,
    }
}

/// 一次 HEAD 探测的结果（后台读取）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadProbe {
    /// 被探测的 git 目录。
    dir: PathBuf,
    /// 目录是否仍然存在。
    exists: bool,
    /// 当前 HEAD 分支（读不到为 None）。
    head: Option<String>,
}

/// 探测 git 目录是否还在以及当前 HEAD（会做磁盘 I/O，只能在后台线程调用）。
fn probe_head(dir: PathBuf) -> HeadProbe {
    let exists = dir.is_dir();
    let head = if exists { read_head(&dir) } else { None };
    HeadProbe { dir, exists, head }
}

/// HEAD 探测的处理结果。
#[derive(Debug, Default, PartialEq, Eq)]
struct HeadOutcome {
    /// 分支发生变化（或目录消失）的 cwd。
    changed: HashSet<String>,
    /// 探测期间又来了新事件，需要再探测一次。
    rerun: bool,
}

/// 某个 git 目录的 HEAD 探测状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeState {
    /// 已有探测在后台执行。
    InFlight,
    /// 执行期间又收到事件，结束后需要重跑。
    Dirty,
}

/// 某个 cwd 的仓库查找结果（含“不在仓库里”）及查找时间。
struct CwdEntry {
    /// 所在 git 目录；不在仓库里为 None。
    dir: Option<PathBuf>,
    /// 查找时间，用来判断是否过期。
    resolved_at: Instant,
}

impl CwdEntry {
    /// 条目是否未过期（只比较时间，不碰磁盘）。
    fn is_fresh(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.resolved_at) < CWD_CACHE_TTL
    }
}

/// 视图级 git 分支监听器：同一 git 目录只挂一个 watcher。
/// 所有磁盘读取都在后台完成，这里只维护缓存与在途请求。
pub(crate) struct ViewGitWatcher {
    /// notify 监听器（非递归监听 git 目录本身）。
    watcher: RecommendedWatcher,
    /// 已挂上监听的 git 目录 -> 最近一次读到的 HEAD（读不到为 None）。
    watched: HashMap<PathBuf, Option<String>>,
    /// 窗格 cwd -> 所在 git 目录的查找结果（渲染时 O(1) 查分支，避免逐帧读磁盘；含非仓库 cwd 的负缓存）。
    cwd_dirs: HashMap<String, CwdEntry>,
    /// 在途的 cwd 解析请求：cwd -> 序号（同一 cwd 只保留一个）。
    pending: HashMap<String, u64>,
    /// 在途的 HEAD 探测：git 目录 -> 状态。
    head_probes: HashMap<PathBuf, ProbeState>,
    /// 下一个请求序号。
    next_seq: u64,
}

impl ViewGitWatcher {
    /// 创建监听器并返回 HEAD 事件接收端（交给后台防抖任务）；系统监听器初始化失败返回 None 并记日志。
    fn new() -> Option<(Self, flume::Receiver<PathBuf>)> {
        let (tx, rx) = flume::unbounded();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            match res {
                // 丢弃读取类事件：读取 HEAD 自己打开文件在 Linux 上会触发 Access(Open)，造成自激。
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
            pending: HashMap::new(),
            head_probes: HashMap::new(),
            next_seq: 0,
        };
        Some((this, rx))
    }

    /// 该 cwd 是否需要发起（或等待）后台解析：缓存缺失或已过期，且没有在途请求。
    fn is_stale(&self, cwd: &str, now: Instant) -> bool {
        !self.pending.contains_key(cwd)
            && self
                .cwd_dirs
                .get(cwd)
                .is_none_or(|entry| !entry.is_fresh(now))
    }

    /// 按当前所有存活窗格的 cwd 对齐缓存（不做任何磁盘 I/O）：
    /// 丢弃不再存活的 cwd 缓存与在途请求，为缺失或过期且无在途请求的 cwd 生成解析请求，
    /// 释放没人用的仓库监听。过期条目的旧值继续生效，直到新结果回填。
    ///
    /// - `cwds`：存活窗格的 cwd（可重复）。
    /// - `now`：当前时间。
    ///
    /// 返回需要交给后台执行的解析请求。
    fn sync<'a>(
        &mut self,
        cwds: impl IntoIterator<Item = &'a str>,
        now: Instant,
    ) -> Vec<ResolveRequest> {
        let live: HashSet<&str> = cwds.into_iter().collect();
        self.cwd_dirs.retain(|cwd, _| live.contains(cwd.as_str()));
        self.pending.retain(|cwd, _| live.contains(cwd.as_str()));
        let mut requests = Vec::new();
        for cwd in live {
            if self.is_stale(cwd, now) {
                let seq = self.next_seq;
                self.next_seq += 1;
                self.pending.insert(cwd.to_string(), seq);
                requests.push(ResolveRequest {
                    cwd: cwd.to_string(),
                    seq,
                });
            }
        }
        self.release_unwanted();
        requests
    }

    /// 释放不再被任何 cwd 引用的仓库监听。
    fn release_unwanted(&mut self) {
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
        self.head_probes
            .retain(|dir, _| self.watched.contains_key(dir));
    }

    /// 回填一次后台解析结果。序号不匹配（已过期或 cwd 已不存活）直接丢弃。
    /// 新仓库在这里挂监听并记录解析时读到的 HEAD。
    ///
    /// 返回分支因此变化的 cwd 集合。
    fn apply_resolved(&mut self, result: ResolveResult, now: Instant) -> HashSet<String> {
        if self.pending.get(&result.cwd) != Some(&result.seq) {
            return HashSet::new();
        }
        self.pending.remove(&result.cwd);
        let before = self.branch_for(&result.cwd).map(str::to_owned);
        self.cwd_dirs.insert(
            result.cwd.clone(),
            CwdEntry {
                dir: result.dir.clone(),
                resolved_at: now,
            },
        );
        if let Some(dir) = result.dir
            && !self.watched.contains_key(&dir)
        {
            match self.watcher.watch(&dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    self.watched.insert(dir, result.head);
                }
                Err(error) => log::warn!("Failed to watch git dir {}: {error}", dir.display()),
            }
        }
        // 旧仓库可能已无人引用。
        self.release_unwanted();
        let after = self.branch_for(&result.cwd).map(str::to_owned);
        if before == after {
            HashSet::new()
        } else {
            HashSet::from([result.cwd])
        }
    }

    /// 登记一次对该 git 目录的 HEAD 探测请求，同一目录的在途探测会合并。
    ///
    /// 返回 true 表示调用方需要真正起后台任务；目录未被监听或已有在途探测返回 false
    /// （后者会标记为“结束后重跑”，不丢事件）。
    fn request_head_probe(&mut self, dir: &Path) -> bool {
        if !self.watched.contains_key(dir) {
            return false;
        }
        if let Some(state) = self.head_probes.get_mut(dir) {
            *state = ProbeState::Dirty;
            false
        } else {
            self.head_probes
                .insert(dir.to_path_buf(), ProbeState::InFlight);
            true
        }
    }

    /// 某个 git 目录下的所有 cwd。
    fn cwds_in(&self, dir: &Path) -> HashSet<String> {
        self.cwd_dirs
            .iter()
            .filter(|(_, entry)| entry.dir.as_deref() == Some(dir))
            .map(|(cwd, _)| cwd.clone())
            .collect()
    }

    /// 处理一次 HEAD 探测结果：目录消失才释放监听并作废指向它的 cwd 缓存；
    /// 目录还在时只在 HEAD 变了才更新，不重挂监听。
    fn apply_head_probe(&mut self, probe: HeadProbe) -> HeadOutcome {
        let rerun = self.head_probes.remove(&probe.dir) == Some(ProbeState::Dirty);
        if !self.watched.contains_key(&probe.dir) {
            return HeadOutcome::default();
        }
        if !probe.exists {
            let _ = self.watcher.unwatch(&probe.dir);
            self.watched.remove(&probe.dir);
            let changed = self.cwds_in(&probe.dir);
            self.cwd_dirs
                .retain(|_, entry| entry.dir.as_ref() != Some(&probe.dir));
            return HeadOutcome {
                changed,
                rerun: false,
            };
        }
        let mut changed = HashSet::new();
        if let Some(slot) = self.watched.get_mut(&probe.dir)
            && *slot != probe.head
        {
            *slot = probe.head;
            changed = self.cwds_in(&probe.dir);
        }
        HeadOutcome { changed, rerun }
    }

    /// 某个 cwd 当前所在的分支名（或短 hash）；非 git 目录、尚未解析或无法确定分支返回 None。
    fn branch_for(&self, cwd: &str) -> Option<&str> {
        let dir = self.cwd_dirs.get(cwd)?.dir.as_ref()?;
        self.watched.get(dir)?.as_deref()
    }
}

impl TerminalView {
    /// 某个窗格 cwd 所在的分支（缓存查询，可在渲染路径调用）；无 git 或未监听返回 None。
    pub(crate) fn branch_for_pane_cwd(&self, cwd: &str) -> Option<&str> {
        self.git_watcher.as_ref()?.branch_for(cwd)
    }

    /// 某个窗格的 git 监听是否需要重新对齐：监听器还没建、或该窗格 cwd 的仓库缓存过期且无在途解析。
    /// 监听器创建失败过则恒为 false，避免每个提示符都重试。只读缓存，不碰磁盘。
    pub(crate) fn git_watch_needs_sync(&self, pane_id: &str) -> bool {
        if self.git_watch_unavailable {
            return false;
        }
        match (&self.git_watcher, self.pane_cwds.get(pane_id)) {
            (None, _) => true,
            (Some(watcher), Some(cwd)) => watcher.is_stale(cwd, Instant::now()),
            (Some(_), None) => false,
        }
    }

    /// 对齐 git 监听（首次调用时创建 watcher 与后台防抖任务），并为缺失/过期的 cwd 发起后台解析。
    /// 监听范围是所有存活窗格已知的 cwd；创建 watcher 失败后不再重试。本函数不做磁盘读取。
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
        let requests = watcher.sync(
            self.pane_cwds
                .iter()
                .filter(|(id, _)| live.contains(id.as_str()))
                .map(|(_, cwd)| cwd.as_str()),
            Instant::now(),
        );
        self.git_watcher = Some(watcher);
        for request in requests {
            Self::spawn_git_resolve(request, cx);
        }
    }

    /// 在后台线程解析一个 cwd，完成后回到 UI 线程回填缓存。
    fn spawn_git_resolve(request: ResolveRequest, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = cx
                .background_executor()
                .spawn(async move { resolve_cwd(request) })
                .await;
            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    let changed = view
                        .git_watcher
                        .as_mut()
                        .map(|w| w.apply_resolved(result, Instant::now()))
                        .unwrap_or_default();
                    view.finish_git_change(&changed, cx);
                })
            });
        })
        .detach();
    }

    /// 在后台线程探测一个 git 目录的 HEAD，完成后回到 UI 线程处理。
    fn spawn_git_head_probe(dir: PathBuf, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let probe = cx
                .background_executor()
                .spawn(async move { probe_head(dir) })
                .await;
            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    let Some(watcher) = view.git_watcher.as_mut() else {
                        return;
                    };
                    let dir = probe.dir.clone();
                    let outcome = watcher.apply_head_probe(probe);
                    if outcome.rerun && watcher.request_head_probe(&dir) {
                        Self::spawn_git_head_probe(dir, cx);
                    }
                    if !outcome.changed.is_empty() {
                        // 目录消失会作废缓存，重新对齐后才能挂上重建的仓库。
                        view.sync_git_watch(cx);
                    }
                    view.finish_git_change(&outcome.changed, cx);
                })
            });
        })
        .detach();
    }

    /// 只刷新当前 cwd 落在 `changed` 里的标签标题并重绘；集合为空什么都不做。
    fn finish_git_change(&mut self, changed: &HashSet<String>, cx: &mut Context<Self>) {
        if changed.is_empty() {
            return;
        }
        for index in 0..self.session.tabs.len() {
            if self.tab_cwd(index).is_some_and(|cwd| changed.contains(cwd)) {
                self.refresh_tab_title(index);
            }
        }
        cx.notify();
    }

    /// 后台防抖任务：阻塞等第一个 HEAD 事件，再等一个防抖间隔合并后续事件，
    /// 然后对每个受影响目录发起一次后台 HEAD 探测。空闲时不唤醒；监听器被释放（通道关闭）后退出。
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
                            let Some(watcher) = view.git_watcher.as_mut() else {
                                return;
                            };
                            for dir in changed {
                                if watcher.request_head_probe(&dir) {
                                    Self::spawn_git_head_probe(dir, cx);
                                }
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
    fn find_git_dir_rejects_relative_paths() {
        // 相对路径会按进程工作目录解析，不能因此命中应用所在仓库。
        assert_eq!(find_git_dir(Path::new("")), None);
        assert_eq!(find_git_dir(Path::new(".")), None);
        assert_eq!(find_git_dir(Path::new("some/relative/dir")), None);
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
        assert_eq!(
            append_branch("build".into(), Some("main")),
            format!("build{BRANCH_SEPARATOR}main")
        );
        assert_eq!(append_branch("build".into(), None), "build");
        assert_eq!(append_branch("build".into(), Some("")), "build");
    }

    /// 同步执行一轮：对齐缓存、在当前线程解析所有请求并回填（测试里不起后台线程）。
    fn settle<'a>(
        w: &mut ViewGitWatcher,
        cwds: impl IntoIterator<Item = &'a str>,
        now: Instant,
    ) -> HashSet<String> {
        let mut changed = HashSet::new();
        for request in w.sync(cwds, now) {
            changed.extend(w.apply_resolved(resolve_cwd(request), now));
        }
        changed
    }

    /// 同步探测并处理一个 git 目录的 HEAD。
    fn probe_now(w: &mut ViewGitWatcher, dir: &Path) -> HeadOutcome {
        assert!(w.request_head_probe(dir));
        w.apply_head_probe(probe_head(dir.to_path_buf()))
    }

    #[test]
    fn branch_for_tracks_cwd_and_head() {
        let (_tmp, git, cwd) = temp_repo("dev");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        assert_eq!(
            settle(&mut w, [cwd.as_str()], now),
            HashSet::from([cwd.clone()])
        );
        assert_eq!(w.branch_for(&cwd), Some("dev"));
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/next\n").unwrap();
        let outcome = probe_now(&mut w, &git);
        assert_eq!(outcome.changed, HashSet::from([cwd.clone()]));
        assert_eq!(w.branch_for(&cwd), Some("next"));
        w.sync(std::iter::empty(), now);
        assert_eq!(w.branch_for(&cwd), None);
    }

    #[test]
    fn sync_never_resolves_inline_and_keeps_stale_value_until_result() {
        let (_tmp, _git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        settle(&mut w, [cwd.as_str()], now);
        // 过期后只发请求，旧值继续显示。
        let later = now + CWD_CACHE_TTL;
        assert!(w.is_stale(&cwd, later));
        let requests = w.sync([cwd.as_str()], later);
        assert_eq!(requests.len(), 1);
        assert_eq!(w.branch_for(&cwd), Some("main"));
    }

    #[test]
    fn duplicate_requests_for_same_cwd_are_coalesced() {
        let (_tmp, _git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        let first = w.sync([cwd.as_str(), cwd.as_str()], now);
        assert_eq!(first.len(), 1);
        assert!(!w.is_stale(&cwd, now));
        assert!(w.sync([cwd.as_str()], now).is_empty());
    }

    #[test]
    fn stale_sequence_result_is_discarded() {
        let (_tmp, _git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        let old = w.sync([cwd.as_str()], now).remove(0);
        // 旧请求被作废（cwd 离开又回来，序号换新），旧结果不得回填。
        w.sync(std::iter::empty(), now);
        let new = w.sync([cwd.as_str()], now).remove(0);
        assert_ne!(old.seq, new.seq);
        assert!(w.apply_resolved(resolve_cwd(old), now).is_empty());
        assert_eq!(w.branch_for(&cwd), None);
        assert!(!w.apply_resolved(resolve_cwd(new), now).is_empty());
        assert_eq!(w.branch_for(&cwd), Some("main"));
    }

    #[test]
    fn result_for_closed_pane_is_not_filled() {
        let (_tmp, _git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        let request = w.sync([cwd.as_str()], now).remove(0);
        // 窗格关闭：存活 cwd 为空，在途请求被清掉。
        w.sync(std::iter::empty(), now);
        assert!(w.apply_resolved(resolve_cwd(request), now).is_empty());
        assert!(w.cwd_dirs.is_empty() && w.watched.is_empty());
    }

    #[test]
    fn panes_in_same_repo_share_one_watch_and_release_together() {
        let (_tmp, git, cwd) = temp_repo("main");
        let sub = format!("{cwd}{}sub", std::path::MAIN_SEPARATOR);
        fs::create_dir(&sub).unwrap();
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        settle(&mut w, [cwd.as_str(), sub.as_str(), cwd.as_str()], now);
        assert_eq!(w.watched.len(), 1);
        assert!(w.watched.contains_key(&git));
        assert_eq!(w.branch_for(&sub), Some("main"));
        // 还剩一个窗格时仓库保持监听；全部离开才释放。
        settle(&mut w, [sub.as_str()], now);
        assert_eq!(w.watched.len(), 1);
        settle(&mut w, std::iter::empty(), now);
        assert!(w.watched.is_empty());
    }

    #[test]
    fn fresh_cwd_cache_is_reused_without_rescanning() {
        let (_tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        settle(&mut w, [cwd.as_str()], now);
        let resolved_at = w.cwd_dirs[&cwd].resolved_at;
        assert!(!w.is_stale(&cwd, now));
        assert!(w.sync([cwd.as_str()], now).is_empty());
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
        let now = Instant::now();
        settle(&mut w, [cwd.as_str()], now);
        assert!(w.cwd_dirs[&cwd].dir.is_none());
        assert!(!w.is_stale(&cwd, now));
        // 之后在该目录 git init：缓存过期前仍是旧结果，过期后才被发现。
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/new\n").unwrap();
        settle(&mut w, [cwd.as_str()], now);
        assert_eq!(w.branch_for(&cwd), None);
        let later = now + CWD_CACHE_TTL;
        assert!(w.is_stale(&cwd, later));
        settle(&mut w, [cwd.as_str()], later);
        assert_eq!(w.branch_for(&cwd), Some("new"));
    }

    #[test]
    fn deleted_git_dir_is_released_and_recreated_dir_is_rewatched() {
        let (tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        let now = Instant::now();
        settle(&mut w, [cwd.as_str()], now);
        fs::remove_dir_all(&git).unwrap();
        let outcome = probe_now(&mut w, &git);
        assert_eq!(outcome.changed, HashSet::from([cwd.clone()]));
        assert!(w.watched.is_empty());
        assert_eq!(w.branch_for(&cwd), None);
        // 重建后重新对齐（缓存已作废）即可重新挂上监听。
        fs::create_dir(tmp.path().join(".git")).unwrap();
        fs::write(git.join(HEAD_FILE), "ref: refs/heads/again\n").unwrap();
        settle(&mut w, [cwd.as_str()], now);
        assert_eq!(w.branch_for(&cwd), Some("again"));
    }

    #[test]
    fn unchanged_head_reports_nothing_and_keeps_watch() {
        let (_tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        settle(&mut w, [cwd.as_str()], Instant::now());
        let outcome = probe_now(&mut w, &git);
        assert!(outcome.changed.is_empty() && !outcome.rerun);
        assert!(w.watched.contains_key(&git));
        assert_eq!(w.branch_for(&cwd), Some("main"));
    }

    #[test]
    fn head_probes_coalesce_and_rerun_when_dirty() {
        let (_tmp, git, cwd) = temp_repo("main");
        let (mut w, _rx) = ViewGitWatcher::new().unwrap();
        settle(&mut w, [cwd.as_str()], Instant::now());
        assert!(w.request_head_probe(&git));
        // 在途时再来事件：不重复起任务，但结束后要求重跑。
        assert!(!w.request_head_probe(&git));
        let outcome = w.apply_head_probe(probe_head(git.clone()));
        assert!(outcome.rerun);
        assert!(w.request_head_probe(&git));
        // 未监听的目录不起探测。
        assert!(!w.request_head_probe(Path::new("/nonexistent/.git")));
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
