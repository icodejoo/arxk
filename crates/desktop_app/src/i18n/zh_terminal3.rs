//! 译文表：终端视图（第三部分：render.rs、mod.rs、tab_strip/** 等）。
//! 格式：（英文原文，中文译文）；带 `{name}` 占位符的，译文里的占位符必须与原文一致。

pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("Config fixed", "配置已修复"),
    ("Copied", "已复制"),
    ("Open Search", "打开查找"),
    ("Copy Image", "复制图片"),
    ("Default Shell", "默认 Shell"),
    ("New Terminal Tab", "新建终端标签页"),
    ("SSH HOSTS", "SSH 主机"),
    ("No tabs found in here", "这里没有找到标签页"),
    ("Configuration reloaded", "配置已重新加载"),
    (
        "Failed to load saved native tabs",
        "加载已保存的原生标签页失败",
    ),
    (
        "Failed to restore saved native tabs",
        "恢复已保存的原生标签页失败",
    ),
    (
        "Could not restore multiplexer tabs: {error}",
        "无法恢复多路复用器标签页：{error}",
    ),
    (
        "Background blur is unsupported in this session; using transparency",
        "当前会话不支持背景模糊，已改用透明效果",
    ),
    (
        "tmux startup default saved. Use Tmux Sessions to switch runtime now.",
        "tmux 启动默认值已保存。请通过 Tmux 会话立即切换运行时。",
    ),
    (
        "Built-in multiplexer setting saved. Restart Termy to apply it.",
        "内置多路复用器设置已保存。重启 Termy 后生效。",
    ),
    (
        "tmux on Windows requires tmux_command_prefix (for example, wsl.exe -e); using native runtime.",
        "Windows 上的 tmux 需要设置 tmux_command_prefix（例如 wsl.exe -e），已改用原生运行时。",
    ),
    ("up to 120Hz", "最高 120Hz"),
    ("system", "系统"),
    ("Display: {hint}", "显示：{hint}"),
    ("Render callbacks: {value}/s", "渲染回调：{value}/秒"),
    ("CPU: {value}%", "CPU：{value}%"),
    ("Process RSS: {memory}", "进程内存（RSS）：{memory}"),
    ("Drain passes: {count}", "排空次数：{count}"),
    ("Redraws: {count}", "重绘次数：{count}"),
    ("Alt fallback redraws: {count}", "备用屏回退重绘：{count}"),
];
