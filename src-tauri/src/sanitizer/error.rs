//! Sanitizer 错误类型定义

use thiserror::Error;

/// Sanitizer 错误类型
#[derive(Debug, Error)]
pub enum SanitizerError {
    /// 无效的正则表达式模式
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),

    /// 序列化错误
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 配置文件读写错误
    #[error("Config error: {0}")]
    ConfigError(String),

    /// 正则表达式验证错误 (用于用户输入验证)
    #[error("{0}")]
    InvalidRegex(String),

    /// 通用验证错误
    #[error("Validation error: {0}")]
    ValidationError(String),
}
