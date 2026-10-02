//! 界面多语言（英文 / 中文）。
//!
//! 约定：**英文原文就是 key**，代码里写 `t!("New Tab")`；语言为中文时去译文表里查，
//! 查不到就原样显示英文，所以漏翻译不会出错，只是那一条还是英文。
//!
//! - 静态文案：`t!("New Tab")`，返回 `&'static str`。
//! - 带变量的文案：`t!("Failed to open {path}", path = path)`，返回 `String`。
//!   占位符一律用命名形式 `{name}`；译文里的占位符必须和原文完全一致（有单元测试检查）。
//! - 译文表按区域分文件：`zh_core`（配置项与命令）、`zh_terminal`（终端视图）、
//!   `zh_settings`（设置窗口）、`zh_app`（其余）、`zh_new`（后加的功能）。
//!
//! 语言来自配置项 `language = auto | en | zh`；`auto` 跟随系统界面语言。

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use termy_core::config_core::AppLanguage;

mod zh_app;
mod zh_core;
mod zh_new;
mod zh_settings;
mod zh_terminal;
mod zh_terminal2;
mod zh_terminal3;

/// 当前生效的界面语言（`auto` 已解析成具体语言）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

/// 当前语言：0 = 英文，1 = 中文。
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// 全部译文表，后面的表覆盖前面的同名条目。
const ZH_TABLES: &[&[(&str, &str)]] = &[
    zh_core::ENTRIES,
    zh_terminal::ENTRIES,
    zh_terminal2::ENTRIES,
    zh_terminal3::ENTRIES,
    zh_settings::ENTRIES,
    zh_app::ENTRIES,
    zh_new::ENTRIES,
];

/// 按配置设置界面语言，返回语言是否发生了变化。
pub fn set_language(setting: AppLanguage) -> bool {
    let language = match setting {
        AppLanguage::English => Language::English,
        AppLanguage::Chinese => Language::Chinese,
        AppLanguage::Auto => detect_system_language(),
    };
    let value = u8::from(language == Language::Chinese);
    CURRENT.swap(value, Ordering::Relaxed) != value
}

/// 当前界面语言。
pub fn language() -> Language {
    if CURRENT.load(Ordering::Relaxed) == 1 {
        Language::Chinese
    } else {
        Language::English
    }
}

/// 中文译文表（英文原文 -> 译文），首次使用时构建。
fn zh_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        ZH_TABLES
            .iter()
            .flat_map(|table| table.iter().copied())
            .collect()
    })
}

/// 按指定语言翻译一条文案；没有译文时返回原文。
pub fn translate_with(language: Language, text: &'static str) -> &'static str {
    match language {
        Language::English => text,
        Language::Chinese => zh_map().get(text).copied().unwrap_or(text),
    }
}

/// 按当前语言翻译一条静态文案。
pub fn tr(text: &'static str) -> &'static str {
    translate_with(language(), text)
}

/// 按指定语言翻译带命名占位符的模板，并代入参数。
pub fn translate_named_with(
    language: Language,
    text: &'static str,
    args: &[(&str, &dyn Display)],
) -> String {
    let mut out = translate_with(language, text).to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), &value.to_string());
    }
    out
}

/// 按当前语言翻译带命名占位符的模板，并代入参数。
pub fn tr_named(text: &'static str, args: &[(&str, &dyn Display)]) -> String {
    translate_named_with(language(), text, args)
}

/// 界面文案翻译宏，见模块说明。
#[macro_export]
macro_rules! t {
    ($text:literal) => {
        $crate::i18n::tr($text)
    };
    ($text:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::tr_named(
            $text,
            &[$((stringify!($name), &$value as &dyn ::std::fmt::Display)),+],
        )
    };
}

/// 检测系统界面语言：中文系统返回 `Chinese`，其余一律英文。
fn detect_system_language() -> Language {
    match system_ui_language_tag() {
        Some(tag) if tag.trim().to_ascii_lowercase().starts_with("zh") => Language::Chinese,
        _ => Language::English,
    }
}

/// 系统界面语言标签，例如 `zh-CN`、`en-US`；取不到返回 `None`。
#[cfg(target_os = "windows")]
fn system_ui_language_tag() -> Option<String> {
    // 优先取用户首选界面语言，其次取区域设置。
    read_user_registry_string(r"Control Panel\Desktop", "PreferredUILanguages")
        .or_else(|| read_user_registry_string(r"Control Panel\International", "LocaleName"))
}

/// 系统界面语言标签，例如 `zh_CN.UTF-8`；取不到返回 `None`。
#[cfg(not(target_os = "windows"))]
fn system_ui_language_tag() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.trim().is_empty())
}

/// 读取当前用户注册表里的字符串值（`REG_SZ` / `REG_MULTI_SZ` 只取第一项）。
#[cfg(target_os = "windows")]
fn read_user_registry_string(subkey: &str, name: &str) -> Option<String> {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW,
        RegQueryValueExW,
    };
    use windows::core::PCWSTR;

    let wide = |text: &str| -> Vec<u16> { text.encode_utf16().chain(std::iter::once(0)).collect() };
    let subkey_wide = wide(subkey);
    let name_wide = wide(name);

    let mut key = HKEY::default();
    // SAFETY: 传入的指针都指向以 NUL 结尾、在调用期间有效的宽字符串。
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
    };
    if opened != ERROR_SUCCESS {
        return None;
    }

    let mut value_type = REG_VALUE_TYPE::default();
    let mut byte_len: u32 = 0;
    // SAFETY: 第一次调用只查询所需字节数，数据缓冲区传 None。
    let sized = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(name_wide.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut byte_len),
        )
    };
    let result = if sized == ERROR_SUCCESS && byte_len >= 2 {
        let mut buffer = vec![0u8; byte_len as usize];
        // SAFETY: 缓冲区长度与传入的 byte_len 一致。
        let read = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(name_wide.as_ptr()),
                None,
                Some(&mut value_type),
                Some(buffer.as_mut_ptr()),
                Some(&mut byte_len),
            )
        };
        (read == ERROR_SUCCESS).then(|| {
            let units: Vec<u16> = buffer[..byte_len as usize]
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .take_while(|unit| *unit != 0)
                .collect();
            String::from_utf16_lossy(&units)
        })
    } else {
        None
    };
    // SAFETY: key 由上面成功打开，这里只关闭一次。
    let _ = unsafe { RegCloseKey(key) };
    result.filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// 取出模板里的 `{name}` 占位符集合。
    fn placeholders(text: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut rest = text;
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            let Some(end) = after.find('}') else { break };
            let name = &after[..end];
            if !name.is_empty() && name.chars().all(|ch| ch.is_alphanumeric() || ch == '_') {
                found.insert(name.to_string());
            }
            rest = &after[end + 1..];
        }
        found
    }

    #[test]
    fn english_returns_the_original_text() {
        assert_eq!(translate_with(Language::English, "New Tab"), "New Tab");
    }

    #[test]
    fn missing_translation_falls_back_to_english() {
        let text = "this text has no translation entry";
        assert_eq!(translate_with(Language::Chinese, text), text);
    }

    #[test]
    fn chinese_uses_the_table() {
        assert_ne!(translate_with(Language::Chinese, "New Tab"), "New Tab");
    }

    #[test]
    fn named_arguments_are_substituted_in_both_languages() {
        let path: &dyn Display = &"C:\\work";
        for language in [Language::English, Language::Chinese] {
            let text = translate_named_with(language, "Failed to open {path}", &[("path", path)]);
            assert!(text.contains("C:\\work"), "{language:?}: {text}");
            assert!(!text.contains("{path}"), "{language:?}: {text}");
        }
    }

    #[test]
    fn tables_have_no_conflicting_keys_and_no_empty_translations() {
        // 不同区域各自翻译了同一个词是允许的，但译文必须一致。
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for table in ZH_TABLES {
            for (english, chinese) in table.iter() {
                assert!(!english.is_empty(), "空的原文 key");
                assert!(!chinese.trim().is_empty(), "译文为空: {english}");
                if let Some(previous) = seen.insert(*english, *chinese) {
                    assert_eq!(previous, *chinese, "同一原文有两种译文: {english}");
                }
            }
        }
    }

    #[test]
    fn translations_keep_exactly_the_same_placeholders() {
        for table in ZH_TABLES {
            for (english, chinese) in table.iter() {
                assert_eq!(
                    placeholders(english),
                    placeholders(chinese),
                    "占位符不一致: {english} -> {chinese}"
                );
            }
        }
    }
}
