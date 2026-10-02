//! Desktop presentation and native integration modules.

// i18n 必须排在最前面：它导出的 `t!` 宏要让后面的模块直接可用。
#[macro_use]
pub mod i18n;
pub mod auto_update;
pub mod design_system;
pub mod native_sdk;
pub mod terminal_ui;
