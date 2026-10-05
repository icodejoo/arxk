//! 译文表：终端视图（第二部分：runtime/tabs/interaction/titles/plugin_ui/persistence/workspaces/inspector/search/update_overlay 等）。
//! 格式：（英文原文，中文译文）；带 `{name}` 占位符的，译文里的占位符必须与原文一致。

pub(super) const ENTRIES: &[(&str, &str)] = &[
    // 剪贴板权限（backend）
    ("A terminal application", "某个终端应用"),
    (
        "{name} wants to read {formats} from your {location}.",
        "{name} 想从你的{location}读取 {formats}。",
    ),
    ("Allow clipboard access?", "允许访问剪贴板？"),
    ("clipboard", "剪贴板"),
    ("primary selection", "主选区"),
    // 检查器
    ("Input", "输入"),
    ("Render", "渲染"),
    ("Pane {id}{suffix}", "窗格 {id}{suffix}"),
    ("  (active)", "  （当前）"),
    ("Key", "按键"),
    ("Mods", "修饰键"),
    ("Char", "字符"),
    ("Routed To", "路由目标"),
    ("Press keys to record events…", "按下按键以记录事件…"),
    (
        "Cache and dirty-span counters require a debug build.",
        "缓存与脏区计数需要调试构建。",
    ),
    (
        "Callback interval is idle/activity cadence, not latency; ~500 ms can be healthy while idle. CPU view build excludes GPUI layout/paint, GPU work, and presentation.",
        "回调间隔反映的是空闲/活跃节奏，不是延迟；空闲时约 500 ms 属于正常。CPU 视图构建耗时不含 GPUI 布局/绘制、GPU 工作和上屏呈现。",
    ),
    ("Clear", "清除"),
    // 工作区持久化与布局
    ("The workspace store is unavailable", "工作区存储不可用"),
    (
        "Saved layouts are only available in the native runtime",
        "已保存的布局仅在原生运行时可用",
    ),
    ("Layout name is required", "请填写布局名称"),
    ("There is no native layout to save", "没有可保存的原生布局"),
    (
        "Saved layout \"{layout_name}\" was not found",
        "未找到已保存的布局“{layout_name}”",
    ),
    // 插件视图
    (
        "Plugin view lost its terminal window",
        "插件视图丢失了所属的终端窗口",
    ),
    (
        "Plugin view returned a response from the wrong revision",
        "插件视图返回了来自错误版本的响应",
    ),
    (
        "Plugin view returned a response for the wrong params",
        "插件视图返回了与参数不匹配的响应",
    ),
    ("Plugin view is no longer available", "插件视图已不可用"),
    (
        "Plugin changed while its view was running; reopen the view",
        "视图运行期间插件发生了变化，请重新打开视图",
    ),
    (
        "Input `{id}` must be at most {limit} characters",
        "输入项 `{id}` 最多 {limit} 个字符",
    ),
    ("Loading view…", "正在加载视图…"),
    ("Working…", "处理中…"),
    (
        "Plugin view {plugin_id}.{view_id} is unavailable",
        "插件视图 {plugin_id}.{view_id} 不可用",
    ),
    (
        "Plugin changed before its view could open; try again",
        "视图打开前插件发生了变化，请重试",
    ),
    (
        "Plugin returned view.replace without an open view",
        "插件在没有打开视图的情况下返回了 view.replace",
    ),
    (
        "Plugin cannot replace another plugin's view",
        "插件不能替换其他插件的视图",
    ),
    (
        "Plugin changed before its view could be replaced",
        "视图被替换前插件发生了变化",
    ),
    ("Select…", "请选择…"),
    ("Type to filter…", "输入以筛选…"),
    ("Loading…", "加载中…"),
    // 终端内查找
    ("Invalid pattern", "无效的匹配模式"),
    ("Searching…", "查找中…"),
    ("No matches", "无匹配项"),
    ("{current} of {total}+", "第 {current} 项，共 {total}+ 项"),
    ("Find in terminal", "在终端中查找"),
    ("Case", "区分大小写"),
    ("Regex", "正则"),
    (
        "↵ next  ·  ⇧↵ prev  ·  esc",
        "↵ 下一个  ·  ⇧↵ 上一个  ·  esc",
    ),
    // 更新浮层与发行说明
    ("Restart failed: {error}", "重启失败：{error}"),
    ("Version {version}", "版本 {version}"),
    (
        "Fetching release notes from GitHub…",
        "正在从 GitHub 获取发行说明…",
    ),
    ("Try again", "重试"),
    ("This release has no written notes.", "此版本没有发行说明。"),
    ("Release notes", "发行说明"),
    ("Could not open link", "无法打开链接"),
    ("Image", "图片"),
    ("Loading {name}…", "正在加载{name}…"),
    // 更新提示
    ("Update v{version} available", "有新版本 v{version} 可更新"),
    ("Installing v{version}", "正在安装 v{version}"),
    (
        "Installer launched for v{version}; Arxk will reopen when setup finishes",
        "已启动 v{version} 的安装程序，安装完成后 Arxk 会自动重新打开",
    ),
    ("Update failed: {message}", "更新失败：{message}"),
    (
        "v{version} installed \u{2014} reopen from /Applications",
        "v{version} 已安装\u{2014}\u{2014}请从“应用程序”文件夹重新打开",
    ),
    (
        "v{version} installed \u{2014} restart to apply",
        "v{version} 已安装\u{2014}\u{2014}重启后生效",
    ),
    (
        "v{version} installed to ~/.local/bin \u{2014} restart to apply",
        "v{version} 已安装到 ~/.local/bin\u{2014}\u{2014}重启后生效",
    ),
    // 工作区
    ("Failed to start saved workspace", "无法启动已保存的工作区"),
    (
        "Could not start the replacement workspace",
        "无法启动替代的工作区",
    ),
    (
        "Could not close workspace session: {error}",
        "无法关闭工作区会话：{error}",
    ),
    (
        "Workspaces are not available with the tmux runtime",
        "tmux 运行时下不支持工作区",
    ),
    ("Could not merge saved workspaces", "无法合并已保存的工作区"),
    // 文件拖放（macOS）
    (
        "Only Finder file drops are supported here.",
        "这里只支持从访达拖入文件。",
    ),
    (
        "Finder drop data was not valid UTF-8.",
        "访达拖入的数据不是有效的 UTF-8。",
    ),
    (
        "Finder drop did not contain a valid file URL: {url}",
        "访达拖入的内容不含有效的文件 URL：{url}",
    ),
    (
        "Failed to access the macOS window handle.",
        "无法获取 macOS 窗口句柄。",
    ),
    (
        "macOS file drop bridge requires an AppKit window handle.",
        "macOS 文件拖放桥接需要 AppKit 窗口句柄。",
    ),
    (
        "macOS file drop bridge requires a live NSView.",
        "macOS 文件拖放桥接需要一个可用的 NSView。",
    ),
    (
        "Failed to create the macOS file drop overlay view.",
        "无法创建 macOS 文件拖放浮层视图。",
    ),
    // tmux 运行时
    (
        "failed to start tmux control runtime",
        "无法启动 tmux 控制运行时",
    ),
    (
        "failed to fetch initial tmux snapshot",
        "无法获取初始 tmux 快照",
    ),
    ("{prefix}: {error}", "{prefix}：{error}"),
    ("Input write failed: {error}", "写入输入失败：{error}"),
    ("Failed to resize pane", "调整窗格大小失败"),
    ("Failed to reorder tabs", "调整标签页顺序失败"),
    ("Failed to switch tab", "切换标签页失败"),
    (
        "Failed to create tab: active tmux window is unavailable",
        "新建标签页失败：当前 tmux 窗口不可用",
    ),
    ("Failed to create tab", "新建标签页失败"),
    (
        "Failed to create tab: new tmux terminal is unavailable",
        "新建标签页失败：新的 tmux 终端不可用",
    ),
    ("Failed to close tab", "关闭标签页失败"),
    ("Failed to rename tab", "重命名标签页失败"),
    ("Failed to focus pane", "切换窗格焦点失败"),
    ("Failed to split pane", "分屏失败"),
    ("Failed to close pane", "关闭窗格失败"),
    ("Failed to toggle pane zoom", "切换窗格缩放失败"),
    ("tmux sync failed: {error}", "tmux 同步失败：{error}"),
    ("tmux control mode exited", "tmux 控制模式已退出"),
    (
        "tmux pane restore degraded for {count} pane(s): {preview}{suffix}",
        "{count} 个窗格的 tmux 恢复不完整：{preview}{suffix}",
    ),
    (
        "Disable the built-in multiplexer and restart Arxk to switch to tmux",
        "请先停用内置多路复用器并重启 Arxk，再切换到 tmux",
    ),
    (
        "failed to start tmux control runtime: {error}",
        "无法启动 tmux 控制运行时：{error}",
    ),
    (
        "failed to fetch tmux snapshot: {error}; cleanup failed: {cleanup_error}",
        "无法获取 tmux 快照：{error}；清理也失败了：{cleanup_error}",
    ),
    (
        "failed to fetch tmux snapshot: {error}",
        "无法获取 tmux 快照：{error}",
    ),
    (
        "failed to cleanup previous tmux client before attach: {error}",
        "连接前清理旧 tmux 客户端失败：{error}",
    ),
    (
        "failed to cleanup previous tmux client before attach: {error}; failed to cleanup new tmux client: {cleanup_error}",
        "连接前清理旧 tmux 客户端失败：{error}；清理新 tmux 客户端也失败了：{cleanup_error}",
    ),
    (
        "Failed to start native runtime: {error}",
        "无法启动原生运行时：{error}",
    ),
    (
        "tmux_exclusive keeps Arxk in control mode; disable it to detach to a classic terminal",
        "tmux_exclusive 会让 Arxk 一直处于控制模式；停用它才能断开并回到经典终端",
    ),
    (
        "Failed to detach tmux session: {error}",
        "断开 tmux 会话失败：{error}",
    ),
    (
        "{reason}; restarting tmux control mode (tmux_exclusive)",
        "{reason}；正在重启 tmux 控制模式（tmux_exclusive）",
    ),
    (
        "tmux control mode exited; restarting (tmux_exclusive)",
        "tmux 控制模式已退出，正在重启（tmux_exclusive）",
    ),
    (
        "tmux exclusive restart failed: {error}",
        "tmux 独占模式重启失败：{error}",
    ),
    (
        "tmux exclusive restart failed to start control mode: {error}",
        "tmux 独占模式重启失败，无法启动控制模式：{error}",
    ),
    (
        "tmux exclusive restart failed to fetch snapshot: {error}",
        "tmux 独占模式重启失败，无法获取快照：{error}",
    ),
    (
        "tmux reconnect failed while cleaning previous client: {error}",
        "tmux 重新连接失败，清理旧客户端时出错：{error}",
    ),
    (
        "tmux reconnect failed while cleaning previous client: {error}; failed to cleanup new client: {cleanup_error}",
        "tmux 重新连接失败，清理旧客户端时出错：{error}；清理新客户端也失败了：{cleanup_error}",
    ),
    (
        "tmux reconnect failed: {error}",
        "tmux 重新连接失败：{error}",
    ),
    ("tmux resize failed: {error}", "tmux 调整大小失败：{error}"),
    // 标签页与窗格生命周期
    (
        "Structured program launches are not supported in tmux tabs",
        "tmux 标签页不支持结构化程序启动",
    ),
    (
        "Structured program launches are not supported in tmux panes",
        "tmux 窗格不支持结构化程序启动",
    ),
    (
        "Failed to send the plugin command to the new tmux tab",
        "无法把插件命令发送到新的 tmux 标签页",
    ),
    ("Failed to create tab: {error}", "新建标签页失败：{error}"),
    (
        "SSH hosts can only be opened from the native terminal runtime",
        "SSH 主机只能在原生终端运行时中打开",
    ),
    (
        "Invalid SSH host “{name}”: {error}",
        "SSH 主机“{name}”无效：{error}",
    ),
    (
        "Could not read the saved credential for “{name}”; SSH will prompt in the terminal: {error}",
        "无法读取“{name}”已保存的凭据，SSH 会在终端里提示输入：{error}",
    ),
    (
        "Unable to locate the Arxk executable: {error}",
        "找不到 Arxk 可执行文件：{error}",
    ),
    (
        "Could not prepare the saved credential for “{name}”; SSH will prompt in the terminal: {error}",
        "无法准备“{name}”已保存的凭据，SSH 会在终端里提示输入：{error}",
    ),
    (
        "Could not start SSH session “{name}”: {error}. Check that OpenSSH is installed and ssh is on PATH.",
        "无法启动 SSH 会话“{name}”：{error}。请确认已安装 OpenSSH，且 ssh 在 PATH 中。",
    ),
    ("Failed to split pane: {error}", "分屏失败：{error}"),
    (
        "Pane needs at least {count} columns to split vertically",
        "窗格至少需要 {count} 列才能竖向分屏",
    ),
    (
        "Pane needs at least {count} rows to split horizontally",
        "窗格至少需要 {count} 行才能横向分屏",
    ),
    ("The source window was closed", "来源窗口已关闭"),
    ("Drop onto a terminal window", "请拖放到终端窗口上"),
    (
        "Move this tab into a window using the same terminal runtime",
        "请把此标签页移到使用相同终端运行时的窗口中",
    ),
    ("The new window could not start tmux", "新窗口无法启动 tmux"),
    ("The source tab was closed", "来源标签页已关闭"),
    // 交互与应用动作
    (
        "Configure tmux_command_prefix to use tmux on Windows",
        "在 Windows 上使用 tmux 需要先配置 tmux_command_prefix",
    ),
    ("Command unavailable", "命令不可用"),
    (
        "Load saved layout \"{layout_name}\" before running task \"{task_name}\"",
        "运行任务“{task_name}”前，请先载入已保存的布局“{layout_name}”",
    ),
    (
        "Arxk v{version} | {os}-{arch} | config: {config_path}",
        "Arxk v{version} | {os}-{arch} | 配置：{config_path}",
    ),
    (
        "Auto updates are only available on macOS and Windows",
        "自动更新仅支持 macOS 和 Windows",
    ),
    ("Checking for updates", "正在检查更新"),
    (
        "Buffer Position: Line {line}, Column {column}",
        "缓冲区位置：第 {line} 行，第 {column} 列",
    ),
    ("Copied image", "已复制图片"),
    ("Copied buffer position", "已复制缓冲区位置"),
    (
        "Failed to prepare clipboard image for paste: {error}",
        "无法准备要粘贴的剪贴板图片：{error}",
    ),
    (
        "CLI installed to {path}. Updated {profile} and activated PATH in this shell.",
        "命令行工具已安装到 {path}。已更新 {profile}，并在当前 Shell 中生效 PATH。",
    ),
    (
        "CLI installed to {path}. {profile} already configures Arxk PATH; activated PATH in this shell.",
        "命令行工具已安装到 {path}。{profile} 已配置好 Arxk 的 PATH，并已在当前 Shell 中生效。",
    ),
    (
        "CLI installed to {path}. Add {dir} to PATH: setx PATH \"%PATH%;{dir}\"",
        "命令行工具已安装到 {path}。请把 {dir} 加入 PATH：setx PATH \"%PATH%;{dir}\"",
    ),
    ("CLI installed to {path}", "命令行工具已安装到 {path}"),
    ("Failed to open link", "无法打开链接"),
    (
        "The last workspace cannot be deleted",
        "最后一个工作区不能删除",
    ),
    (
        "Pinned workspaces must be unpinned before deleting",
        "固定的工作区需要先取消固定才能删除",
    ),
    (
        "Pinned tabs must be unpinned before closing",
        "固定的标签页需要先取消固定才能关闭",
    ),
    // 退出 / 关闭确认
    ("Quit Arxk?", "退出 Arxk？"),
    ("Close Window?", "关闭窗口？"),
    ("Close Tab?", "关闭标签页？"),
    ("Delete Workspace?", "删除工作区？"),
    ("Quit anyway?", "仍要退出吗？"),
    ("Close this window anyway?", "仍要关闭此窗口吗？"),
    ("Close it anyway?", "仍要关闭吗？"),
    ("Delete this workspace anyway?", "仍要删除此工作区吗？"),
    ("Delete Workspace", "删除工作区"),
    ("Close Window", "关闭窗口"),
    (
        "This tab is running a command or fullscreen terminal app:",
        "此标签页正在运行命令或全屏终端应用：",
    ),
    ("Close this tab anyway?", "仍要关闭此标签页吗？"),
    (
        "{count} tab has running a command or fullscreen terminal app:",
        "有 {count} 个标签页正在运行命令或全屏终端应用：",
    ),
    (
        "{count} tabs have running a command or fullscreen terminal app:",
        "有 {count} 个标签页正在运行命令或全屏终端应用：",
    ),
];
