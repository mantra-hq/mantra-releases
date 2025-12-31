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
}
