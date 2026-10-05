//! 译文表：其余区域：main.rs、app_actions、auto_update、ui/**、workspace_store、theme_store、config/**、ssh、native_sdk、terminal_ui、deeplink、menus 等。
//! 格式：（英文原文，中文译文）；带 `{name}` 占位符的，译文里的占位符必须与原文一致。

pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "No main window available for new tab deeplink",
        "没有可用的主窗口来处理新标签页链接",
    ),
    (
        "Failed to open new tab from deeplink: {error}",
        "通过链接打开新标签页失败：{error}",
    ),
    (
        "Failed to focus settings window: {error}",
        "聚焦设置窗口失败：{error}",
    ),
    (
        "Failed to open settings window: {error}",
        "打开设置窗口失败：{error}",
    ),
    (
        "Invalid Arxk deeplink: {error}",
        "无效的 Arxk 链接：{error}",
    ),
    (
        "Unsupported deeplink scheme \"{scheme}\"; expected termy://",
        "不支持的链接协议 \"{scheme}\"，应为 termy://",
    ),
    (
        "Arxk deeplinks do not support user info or ports",
        "Arxk 链接不支持用户信息或端口",
    ),
    (
        "Theme install deeplink requires ?slug=<theme-slug>",
        "主题安装链接需要带上 ?slug=<theme-slug>",
    ),
    (
        "Unsupported Arxk deeplink route: {route}",
        "不支持的 Arxk 链接路径：{route}",
    ),
    (
        "Arxk deeplink {name} value is too long",
        "Arxk 链接的 {name} 值过长",
    ),
    (
        "Arxk deeplink {name} value contains unsupported control characters",
        "Arxk 链接的 {name} 值包含不支持的控制字符",
    ),
    (
        "Open the installed Arxk.app to set it as your default terminal.",
        "请打开已安装的 Arxk.app，再把它设为默认终端。",
    ),
    (
        "Could not locate the Arxk app bundle.",
        "找不到 Arxk 应用包。",
    ),
    (
        "Could not register Arxk with macOS (error {status}).",
        "无法向 macOS 注册 Arxk（错误码 {status}）。",
    ),
    (
        "Could not change the default terminal (macOS error {status}).",
        "无法更改默认终端（macOS 错误码 {status}）。",
    ),
    (
        "macOS did not apply the default terminal change. Try again.",
        "macOS 没有应用默认终端的更改，请重试。",
    ),
    ("is not installed", "未安装"),
    ("resolved to a proportional font", "解析为非等宽字体"),
    (
        "Font \"{requested}\" {reason}; using {fallback}",
        "字体 \"{requested}\" {reason}，改用 {fallback}",
    ),
    ("tmux preflight failed: {error}", "tmux 预检失败：{error}"),
    (
        "tmux runtime is unsupported on this platform",
        "当前平台不支持 tmux 运行时",
    ),
    (
        "Failed to open main window: {error}",
        "打开主窗口失败：{error}",
    ),
    ("Fetching theme \"{slug}\"...", "正在获取主题 \"{slug}\"..."),
    (
        "Theme install deeplink requires a slug",
        "主题安装链接需要提供 slug",
    ),
    ("config directory is missing", "缺少配置目录"),
    (
        "Cannot start the built-in multiplexer: {error}",
        "无法启动内置多路复用器：{error}",
    ),
    (
        "Cannot restore multiplexer layout: {error}",
        "无法恢复多路复用器布局：{error}",
    ),
    (
        "Unsupported multiplexer layout version",
        "不支持的多路复用器布局版本",
    ),
    (
        "This window was created by another client",
        "该窗口由其他客户端创建",
    ),
    (
        "This window was removed by another client",
        "该窗口已被其他客户端移除",
    ),
    (
        "The session layout is changing; try saving again",
        "会话布局正在变化，请再保存一次",
    ),
    (
        "Unable to resolve the Arxk configuration directory",
        "无法确定 Arxk 配置目录",
    ),
    (
        "The Arxk configuration path has no parent directory",
        "Arxk 配置路径没有上级目录",
    ),
    ("Verify SSH Host Key", "验证 SSH 主机密钥"),
    ("tmux preflight failed", "tmux 预检失败"),
    (
        "tmux is unavailable ({reason}: {error}); starting in native mode",
        "tmux 不可用（{reason}：{error}），改用原生模式启动",
    ),
    (
        "Arxk cannot continue because it failed to open the main window.\n\nError:\n{error}\n\nRecovery:\n- Restart Arxk and try again.\n- If this was launched from a terminal, keep this stderr message for support.\n- If the problem repeats, include your OS, display/GPU setup, and recent Arxk logs in the bug report.",
        "Arxk 无法继续运行，因为打开主窗口失败。\n\n错误：\n{error}\n\n恢复办法：\n- 重启 Arxk 后再试。\n- 如果是从终端启动的，请保留这段 stderr 输出以便排查。\n- 如果问题反复出现，请在反馈中附上操作系统、显示器/GPU 配置和最近的 Arxk 日志。",
    ),
    (
        "Arxk cannot continue because {reason}.\n\nError:\n{error}\n\nRecovery:\n- Open your config and set tmux_enabled = false to start in native mode.\n- Finder/DMG launches use a minimal environment; set tmux_binary to an absolute path (for example /opt/homebrew/bin/tmux) if tmux is not on the default PATH.\n- If tmux integration is desired, ensure tmux 3.3 or newer is installed.\n- Save the config and restart Arxk, then use tmux Sessions… when ready.",
        "Arxk 无法继续运行，因为{reason}。\n\n错误：\n{error}\n\n恢复办法：\n- 打开配置，设置 tmux_enabled = false，以原生模式启动。\n- 从 Finder/DMG 启动时环境变量很精简；如果默认 PATH 里找不到 tmux，请把 tmux_binary 设为绝对路径（例如 /opt/homebrew/bin/tmux）。\n- 想使用 tmux 集成，请确认已安装 tmux 3.3 或更新版本。\n- 保存配置并重启 Arxk，准备好后再使用 tmux 会话…。",
    ),
    ("Arxk Startup Error", "Arxk 启动错误"),
    (
        "Server returned 304 Not Modified but no matching local cache exists",
        "服务器返回 304（未修改），但本地没有匹配的缓存",
    ),
    (
        "Invalid theme registry response: {error}",
        "主题仓库返回的数据无效：{error}",
    ),
    (
        "Failed to fetch store themes: {error}",
        "获取主题商店列表失败：{error}",
    ),
    (
        "Theme '{slug}' was not found in the theme registry",
        "主题仓库中找不到主题 '{slug}'",
    ),
    (
        "Failed to logout from theme store: {error}",
        "退出主题商店登录失败：{error}",
    ),
    (
        "Failed to clear auth session: {error}",
        "清除登录会话失败：{error}",
    ),
    ("Config path unavailable", "配置路径不可用"),
    (
        "Invalid installed-theme metadata path",
        "已安装主题的元数据路径无效",
    ),
    (
        "Failed to create metadata directory: {error}",
        "创建元数据目录失败：{error}",
    ),
    (
        "Failed to serialize installed themes: {error}",
        "序列化已安装主题失败：{error}",
    ),
    (
        "Failed to write installed themes metadata: {error}",
        "写入已安装主题元数据失败：{error}",
    ),
    (
        "Theme '{slug}' has no downloadable file URL",
        "主题 '{slug}' 没有可下载的文件地址",
    ),
    (
        "Failed to download theme '{slug}': {error}",
        "下载主题 '{slug}' 失败：{error}",
    ),
    (
        "Failed to read theme '{slug}': {error}",
        "读取主题 '{slug}' 失败：{error}",
    ),
    (
        "Failed to validate theme '{name}': {error}",
        "校验主题 '{name}' 失败：{error}",
    ),
    ("Invalid installed theme path", "已安装主题的路径无效"),
    (
        "Failed to create installed theme directory: {error}",
        "创建已安装主题目录失败：{error}",
    ),
    (
        "Failed to write installed theme file: {error}",
        "写入已安装主题文件失败：{error}",
    ),
    ("Installed theme '{name}'", "已安装主题 '{name}'"),
    (
        "Failed to remove installed theme file: {error}",
        "删除已安装主题文件失败：{error}",
    ),
    (
        "Theme install deeplink is missing a slug",
        "主题安装链接缺少 slug",
    ),
    ("Invalid theme slug '{slug}'", "无效的主题 slug '{slug}'"),
    (
        "A saved layout named \"{name}\" already exists",
        "已存在名为 \"{name}\" 的布局",
    ),
    (
        "Saved layout \"{name}\" was not found",
        "找不到已保存的布局 \"{name}\"",
    ),
    ("Failed to download installer", "下载安装包失败"),
    ("Failed to create installer file", "创建安装包文件失败"),
    ("Failed to read download stream", "读取下载数据流失败"),
    (
        "Release is missing a checksum asset for {asset_name}",
        "发行版缺少 {asset_name} 的校验文件",
    ),
    (
        "Checksum file did not contain an entry for {asset_name}",
        "校验文件中没有 {asset_name} 的条目",
    ),
    (
        "Checksum mismatch for {asset_name}: expected {expected}, got {actual}",
        "{asset_name} 校验不一致：应为 {expected}，实际为 {actual}",
    ),
    ("Failed to download checksum file", "下载校验文件失败"),
    ("Failed to read checksum file", "读取校验文件失败"),
    (
        "Failed to open downloaded installer",
        "打开已下载的安装包失败",
    ),
    (
        "Failed to read downloaded installer for checksum",
        "读取已下载安装包以进行校验失败",
    ),
    ("Failed to mount DMG", "挂载 DMG 失败"),
    (
        "hdiutil attach failed: {error}",
        "hdiutil 挂载失败：{error}",
    ),
    (
        "Could not determine mounted volume from hdiutil output: {output}",
        "无法从 hdiutil 输出中确定挂载的卷：{output}",
    ),
    ("Failed to read mounted volume", "读取已挂载的卷失败"),
    (
        "No .app bundle found inside mounted DMG",
        "挂载的 DMG 中找不到 .app 应用包",
    ),
    (
        "Mounted app bundle is missing file name",
        "已挂载的应用包缺少文件名",
    ),
    (
        "Failed to remove old app bundle in /Applications",
        "删除 /Applications 中的旧应用包失败",
    ),
    (
        "failed removing existing app: {error}",
        "删除现有应用失败：{error}",
    ),
    (
        "Failed to copy app bundle to /Applications",
        "复制应用包到 /Applications 失败",
    ),
    ("ditto failed: {error}", "ditto 执行失败：{error}"),
    ("Failed to launch MSI installer", "启动 MSI 安装程序失败"),
    ("Failed to launch EXE installer", "启动 EXE 安装程序失败"),
    (
        "Unsupported installer format: {extension}",
        "不支持的安装包格式：{extension}",
    ),
    (
        "ShellExecuteW failed with code {code}",
        "ShellExecuteW 失败，错误码 {code}",
    ),
    ("HOME environment variable not set", "未设置 HOME 环境变量"),
    ("Failed to create ~/.local/bin", "创建 ~/.local/bin 失败"),
    (
        "Failed to create temp extraction directory",
        "创建临时解压目录失败",
    ),
    ("Failed to extract tarball", "解压 tar 包失败"),
    ("tar extraction failed: {error}", "tar 解压失败：{error}"),
    ("Failed to read temp directory", "读取临时目录失败"),
    (
        "Could not find arxk binary in extracted tarball",
        "在解压出的 tar 包中找不到 termy 可执行文件",
    ),
    (
        "Failed to copy binary to install directory",
        "复制可执行文件到安装目录失败",
    ),
    (
        "Auto-install is only supported on macOS, Windows, and Linux",
        "自动安装仅支持 macOS、Windows 和 Linux",
    ),
    (
        "Download or verification failed: {error}",
        "下载或校验失败：{error}",
    ),
    ("Install failed: {error}", "安装失败：{error}"),
    (
        "Unable to determine config file path",
        "无法确定配置文件路径",
    ),
    (
        "Invalid config file path: {path}",
        "配置文件路径无效：{path}",
    ),
    (
        "Failed to create config directory '{path}': {source}",
        "创建配置目录 '{path}' 失败：{source}",
    ),
    (
        "Failed to read config file '{path}': {source}",
        "读取配置文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to write config file '{path}': {source}",
        "写入配置文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to create temp config file near '{path}': {source}",
        "在 '{path}' 附近创建临时配置文件失败：{source}",
    ),
    (
        "Failed to persist config file '{path}': {source}",
        "保存配置文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to launch '{command}' for '{path}': {source}",
        "用 '{command}' 打开 '{path}' 失败：{source}",
    ),
    (
        "'{command}' failed for '{path}' with status {status}",
        "用 '{command}' 打开 '{path}' 失败，状态 {status}",
    ),
    (
        "Config has {count} warning(s). First: line {line} [{kind}] {message}",
        "配置有 {count} 条警告。第一条：第 {line} 行 [{kind}] {message}",
    ),
    ("Fix", "修复"),
    (
        "Config update lock was poisoned by a previous failed update",
        "配置更新锁因上一次更新失败而失效",
    ),
    (
        "Failed to open config lock file '{path}': {source}",
        "打开配置锁文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to lock config lock file '{path}': {source}",
        "锁定配置锁文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to lock config file '{path}': {source}",
        "锁定配置文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to unlock config file '{path}': {source}",
        "解锁配置文件 '{path}' 失败：{source}",
    ),
    (
        "Failed to unlock config lock file '{path}': {source}",
        "解锁配置锁文件 '{path}' 失败：{source}",
    ),
    ("Invalid theme id", "无效的主题 ID"),
    ("Theme set to {theme}", "主题已设为 {theme}"),
    (
        "Invalid hex color for '{key}': {hex}",
        "'{key}' 的十六进制颜色无效：{hex}",
    ),
    ("Task name is required", "任务名称不能为空"),
    ("Task command is required", "任务命令不能为空"),
    ("Failed to read file: {error}", "读取文件失败：{error}"),
    ("Invalid JSON: {error}", "无效的 JSON：{error}"),
    (
        "Color '{key}' must be a hex string",
        "颜色 '{key}' 必须是十六进制字符串",
    ),
    ("No valid colors found in JSON", "JSON 中没有找到有效的颜色"),
    ("Imported {count} colors", "已导入 {count} 个颜色"),
    ("Set", "设置"),
    ("Search settings", "搜索设置"),
    (
        "Ignored {count} keybind line",
        "已忽略 {count} 行快捷键配置",
    ),
    (
        "Ignored {count} keybind lines",
        "已忽略 {count} 行快捷键配置",
    ),
    ("OK", "确定"),
    ("Copy Buffer Position", "复制缓冲区位置"),
    ("Deny", "拒绝"),
    ("Allow Once", "允许一次"),
    ("Always Allow", "始终允许"),
    (
        "{message}\n\nYes: Allow Once\nNo: Always Allow\nCancel: Deny",
        "{message}\n\n是：允许一次\n否：始终允许\n取消：拒绝",
    ),
    (
        "{message}\n\nYes: Allow Once\nNo: Deny",
        "{message}\n\n是：允许一次\n否：拒绝",
    ),
    (
        "These windows already share the same tmux session",
        "这些窗口已经在使用同一个 tmux 会话",
    ),
    (
        "tmux command exited with status {status}",
        "tmux 命令退出，状态 {status}",
    ),
    ("signal", "信号"),
    ("failed to execute '{binary}' -V", "执行 '{binary}' -V 失败"),
    ("'{binary} -V' failed", "'{binary} -V' 执行失败"),
    (
        "unable to parse tmux version output: '{output}'",
        "无法解析 tmux 版本输出：'{output}'",
    ),
    (
        "unsupported tmux version format: '{version}'",
        "不支持的 tmux 版本格式：'{version}'",
    ),
    (
        "tmux {major}.{minor}+ required, found {version}",
        "需要 tmux {major}.{minor} 及以上版本，当前为 {version}",
    ),
    (
        "tmux current session name cannot be empty",
        "tmux 当前会话名称不能为空",
    ),
    (
        "tmux new session name cannot be empty",
        "tmux 新会话名称不能为空",
    ),
    ("Latest", "最新"),
    ("Pre-release", "预发布"),
    (
        "Only http(s) images can be loaded.",
        "只能加载 http(s) 图片。",
    ),
    ("Could not read image: {error}", "无法读取图片：{error}"),
    ("Image is larger than 5 MB.", "图片超过 5 MB。"),
    ("Could not decode image: {error}", "无法解码图片：{error}"),
    ("Version {version} is ready", "版本 {version} 已就绪"),
    (
        "Install now to get the latest Arxk.",
        "立即安装，用上最新的 Arxk。",
    ),
    ("View release notes", "查看发行说明"),
    ("Later", "稍后"),
    ("{done} of {total}", "{done} / {total}"),
    ("{size} so far", "已下载 {size}"),
    ("Downloading", "下载中"),
    ("Fetching version {version}", "正在获取版本 {version}"),
    ("Keeping Arxk current.", "让 Arxk 保持最新。"),
    ("Downloaded", "已下载"),
    (
        "Version {version} is ready to install",
        "版本 {version} 可以安装了",
    ),
    ("Starting the installer…", "正在启动安装程序…"),
    ("Download complete", "下载完成"),
    ("Installing", "安装中"),
    ("Installing version {version}", "正在安装版本 {version}"),
    (
        "Finishing the last update steps…",
        "正在完成最后的更新步骤…",
    ),
    ("This usually takes a moment", "通常只需片刻"),
    ("Installer", "安装程序"),
    (
        "Version {version} installer launched",
        "版本 {version} 的安装程序已启动",
    ),
    (
        "Arxk will quit and reopen when setup finishes.",
        "安装完成后 Arxk 会退出并自动重新打开。",
    ),
    ("Version {version} is installed", "版本 {version} 已安装"),
    (
        "Restart Arxk to start using it.",
        "重启 Arxk 即可开始使用。",
    ),
    ("Restart", "重启"),
    ("Dismiss", "关闭"),
    ("Failed", "失败"),
    ("Update failed", "更新失败"),
];
