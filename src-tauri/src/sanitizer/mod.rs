//! Sanitizer 模块 - 敏感信息脱敏引擎
//!
//! 提供正则表达式驱动的敏感信息识别和脱敏功能。
//! 支持内置规则 (API Key, IP, Token 等) 和自定义规则。

mod engine;
mod error;
mod patterns;
mod record;
mod scanner;

pub use engine::{Sanitizer, SanitizationResult, SanitizationStats};
pub use error::SanitizerError;
pub use patterns::{SanitizationRule, SensitiveType, Severity, BUILTIN_RULES};
pub use record::{InterceptionRecord, InterceptionSource, InterceptionStats, PaginatedRecords, UserAction};
pub use scanner::{PrivacyScanner, ScanMatch, ScanResult, ScanStats};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod extended_tests;
