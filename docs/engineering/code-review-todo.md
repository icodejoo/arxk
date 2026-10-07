# 全量代码审查待办

> 状态：进行中（2026-10-06 起）。来源：全仓 `/code-review max` 与未提交改动专项审查。
> 4 个子代理的结论已合并（见「子代理补充」）；后续统一转入待办事项。
> 明天（2026-10-07）继续。

## 已修复（待测试确认）

- 标签颜色写入会话库（`tabs.color` 列 + 旧库迁移 + 三方合并）。
- 改名 termarx 的遗漏：基准工具、`check-gpui-launch-idle.sh`、`APP_ID`、Linux 文件管理器菜单文件名与图标。
- git 监听在 Linux 上被自己的读取触发而自激（过滤 `is_access()`）。
- 盘符回退：`/E%3A/x`、`/E:\x`、`/E:%5Cx` 等 OSC 7 形式。

## 待完成

### 高

- [x] 【决定保持现状，需要增强时在 ci.yml 上增强】CI 基本被拆空：`ci.yml` 只在 v* tag/手动触发时运行，需重建 PR/push 的 clippy、fmt、边界检查、文档同步、跨平台测试。
- [x] 设置页 `state_apply.rs:68` 用 `strip_prefix("Theme set to ")` 反解已翻译文案，中文界面主题 id 退回原始输入；改为让 `set_theme_in_config` 直接返回规范 id。
- [x] bash 的 PROMPT_COMMAND 注入（`runtime.rs:939`）冲掉 `$?`；先保存再还原，或放到原命令之后。

### 中

- [x] `record_pane_cwd` 把真实目录当标题截到 96 字符并折叠空白，`%XX` 不解码，影响显示、持久化恢复、git 查找；截断只留给渲染。
- [x] 窗格关闭时 `pane_titles`/`pane_cwds` 不回收，复用 `%native-pane-N` 会继承旧路径和分支后缀。
- [x] 跨窗口移动标签不迁移 `tab_colors`、窗格手动名、`pane_cwds`、`pane_titles`（`tabs/transfer.rs`）。
- [x] 切换活动窗格不刷新标签标题，`::分支` 停在上一个窗格的值（`tabs/lifecycle.rs:1462`）。
- [x] 标题高频变化时共用 `native_persist_revision`，80ms 保存被 1500ms 标题保存不断作废（`persistence.rs:1292`）。
- [x] 右键菜单已打开时在另一窗格再右键，Close/Rename Pane 作用在旧活动窗格（`render.rs:2229`）。
- [x] `sync_git_watch` 缺口：切换工作区、取消缩放、关闭窗格/标签、会话恢复缩放标签后不重新对齐监听。
- [x] `cwd_dirs` 缓存只记命中项且永不校验：嵌套 `git init`、删 `.git` 后显示旧分支；非仓库 cwd 每次 sync 在 UI 线程逐层 stat。
- [x] 被删除/重建的 `.git` 监听被 notify 摘除后不会重挂。
- [x] `last_prompt_cwd` 仍存 OSC 7 原文：Windows 上新建标签/分屏不继承目录，`tab_branch` 回退键不匹配；与 `pane_cwds` 共用同一套规范化。
- [x] reftable 仓库的 HEAD 是占位 `ref: refs/heads/.invalid`，显示 `::.invalid` 且切分支无事件。（决定保持现状：只做到不显示 `.invalid`；reftable 真实分支读取暂不做，需要时用后台线程调 `git symbolic-ref --short HEAD`）
- [x] 含 `/` 的分支名（feature/x）被标题压缩当成路径碎片，仓库目录名和 `::` 丢失。
- [x] macOS 符号链接路径下事件路径与 `watched` 键不一致（FSEvents 返回真实路径），需 canonicalize；Linux 别名共用同一 wd 同类。
- [x] 窗格标签宽度按字符数估算，CJK 全角约 2 倍宽，左右两段重叠（`render.rs:3630`）。

### 低

- [x] `normalize_msys_cwd` 文档注释过期（现也处理 `/E:/x`），函数名里的 msys 偏窄；盘符/分隔符字面量重复。
- [x] `render.rs:3611` 的 `mem::take` 多余，可直接 `append_branch(texts.left, branch)`。
- [x] `ViewGitWatcher::sync` 整表重建克隆偏多；可用 `retain` 增量维护并顺带去重非仓库 cwd 的重复遍历；`dir_diff`/`watched`/`heads` 可精简。
- [x] `append_branch` 文档缺参数/返回值/示例，且写了内部实现（“不额外分配”）。
- [x] `cwd_to_path` 手写 `%XX` 解码，可换 `percent-encoding`（需给 desktop_app 新增依赖，先征得同意）；对 OSC 9;9 原样路径中字面 `%XX` 会误解码。
- [x] 测试缺口：多窗格同仓库去重/释放、cwd→gitdir 缓存复用、watch 失败重试、`normalize_msys_cwd`→`cwd_to_path`→`find_git_dir` 串联。（已补 `unavailable_git_watch_is_never_retried`）
- [x] 通知失败（创建 watcher 失败）后每个提示符都重试创建，且无日志。
- [x] 回调丢弃 notify 的 `Err` 且不打日志（`config/io.rs` 有 `log::warn!`）。
- [ ] 其他被上限挤掉的：`duplicate_tab_by_id` 无 tmux 守卫（`persistence.rs:1057`）、命令面板中文界面关键词排序（`command_palette/mod.rs:227`）、bash 注入把 `$PWD` 原样写入 OSC 9;9 可被 BEL/ESC 注入（`runtime.rs:878`）、`translate_named_with` 二次替换（`i18n/mod.rs:102`）、`interaction/actions.rs:443` 英文硬编码 toast、`TOP_STRIP_TERMY_BRANDING_TEXT` 仍是 "termy"、README/文档里过期的 DMG 签名、`cd termy`、`yay -S termy-bin`。（已做：duplicate_tab 守卫、$PWD 注入、translate_named_with、toast、branding、README DMG/cd；未做：命令面板排序见下、`yay -S termy-bin` 待 AUR 包改名）
- [x] 标签背景色其余存储路径（multiplexer 持久化）复查。

### 未跟踪脚本（不要提交）

- [x] `scripts/precise_search.py`、`scripts/search_mem.py`：硬编码本机不存在的路径、裸 `except`、无注释，不属于 `scripts/` 职责；删除，或改用 `rg`/CodeGraph。
- [x] `.codegraph/`、`.dev-appdata/` 不应被 `git add -A` 带入，确认已在 `.gitignore`。

## 子代理补充（4 个，已复审并去重）

复审方式：与上文重复的并入原条目，其余对照代码抽查；「已核」= 我读过对应代码，「未核」= 仅采信子代理结论。

重复并入原条目（不再单列）：设置页主题 id 反解、右键菜单切窗格误作用、切活动窗格不刷新标题、`tab_branch` 回退键不匹配、改名遗漏（benchmark/`APP_ID`）、`git_watcher` 创建失败反复重试、快捷键弹窗未接入模态状态。

### 中

- [x] 【已核】macOS 自动更新缺校验和时只告警并继续安装，而 DMG 是未签名构建（`auto_update/engine.rs:169`，`checksum_required_for_current_platform` 只在 Windows 为真）；macOS 也应强制校验。
- [x] 【已核】启动期 toast（配置警告、tmux 回退）在 `set_language` 之前生成，`language=zh` 时仍是英文（`main.rs:731/739`，`set_language` 在 `main.rs:280` 才调用）。
- [x] 【未核】ssh askpass 是独立进程，从不调用 `set_language`，主机密钥确认框标题永远英文（`ssh.rs:59`）。
- [x] 【已核】`duplicate_tab_by_id` 在 index+1 插入标签、改 `active_tab`，不平移 `renaming_tab`，行内重命名时复制别的标签会把名字写到错误标签（`persistence.rs:1096`；对照 `lifecycle.rs:229` 关闭路径有调整）。
- [x] 【未核】命令面板中文界面：标题命中会压掉只靠英文 keywords 命中的内置命令（`command_palette/mod.rs:227`，`state.rs:864`）。
- [x] 【未核】设置页 6 个分区的副标题译文缺失（`sections.rs:242/270/580/606`、`ssh.rs:1009`、`keybinds.rs:546`）；设置搜索不含译文（`search.rs:359`）。（核查不成立：渲染用 `settings_section_subtitle`，译文已齐）
- [x] 【未核】设置窗口先自调 `set_language`，使终端窗口变更检测为 false 而不刷新；重置语言后文案可能不更新（`state_apply.rs:138`、`mod.rs:3754`）。

### 低

- [x] 【未核】设置页数值输入不检查 `is_finite`，NaN/inf 会写入文件、重载时静默回退（`state_apply.rs:186/223/664/722/735`）。
- [x] 【未核】设置窗口标题栏文字创建时固化，切语言不更新（`app_actions.rs:194/227`）。
- [x] 【未核】关闭标签的收缩动画忽略自定义背景色（`tab_strip/render_horizontal.rs:376`）。
- [x] 【未核】测试 `linux_and_windows_titlebar_height_collapses_when_tab_strip_hidden` 只测硬编码辅助函数，真实 `window_titlebar_height_for` 在 Windows 无覆盖（`interaction/chrome.rs:134`）。
- [x] 【未核】Linux `cache_installer_path` 目录仍叫 `termy`，且违反 `map_unwrap_or`/`uninlined_format_args`（`auto_update/engine.rs:78/80`）。
- [x] 【未核】Linux `do_install` 只拷启动脚本、不拷 `termarx-bin`/`termarx-cli`；目前 Linux 不支持自动更新所以不可达（`engine.rs:514`）。
- [x] 【未核】`begin_rename_pane` 无活动窗格时静默 `return false`，与注释承诺的提示不符（`command_palette/mod.rs:1692`）。
- [x] 【未核，低置信】macOS 中文界面菜单名 "Window" 被翻译后可能让系统窗口菜单失效（`menus.rs:48`）。（核查无依据，未改）
- [ ] 未验证：WSL `\wsl.localhost` 路径下 notify 是否收到 HEAD 事件。
