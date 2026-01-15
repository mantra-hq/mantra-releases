//! Error types for log parsing
//!
//! Defines comprehensive error types for handling various failure
//! scenarios during log parsing.

use thiserror::Error;

/// Errors that can occur during log parsing
#[derive(Error, Debug)]
pub enum ParseError {
    /// Failed to read the log file
    #[error("无法读取文件: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing failed
    #[error("JSON 格式无效: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// Required field is missing
    #[error("缺少必需字段: {0}")]
    MissingField(String),

    /// Unsupported log format version
    #[error("不支持的格式版本: {0}")]
    UnsupportedVersion(String),

    /// Invalid data format
    #[error("无效的数据格式: {0}")]
    InvalidFormat(String),

    /// Empty or no conversations found
    #[error("未找到任何对话记录")]
    EmptyConversation,

    /// SQLite database error
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// Workspace not found
    #[error("工作区未找到: {0}")]
    WorkspaceNotFound(String),

    /// Empty file (0 bytes)
    #[error("跳过: 空会话文件")]
    EmptyFile,

    /// File contains only system events (no conversation)
    #[error("跳过: 仅包含系统事件，无对话内容")]
    SystemEventsOnly,

    /// File contains only error/status messages (e.g., "Invalid API key")
    #[error("跳过: 无有效对话 (仅包含状态消息)")]
    NoValidConversation,
}

impl ParseError {
    /// Create a MissingField error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField(field.into())
    }

    /// Create an InvalidFormat error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a DatabaseError
    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    /// Create a WorkspaceNotFound error
    pub fn workspace_not_found(path: impl Into<String>) -> Self {
        Self::WorkspaceNotFound(path.into())
    }

    /// Check if this error represents a skippable condition
    /// (empty sessions that should be silently skipped, not treated as failures)
    pub fn is_skippable(&self) -> bool {
        matches!(
            self,
            Self::EmptyFile | Self::SystemEventsOnly | Self::NoValidConversation
        )
    }
}


#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
