//! Storage module error types
//!
//! Provides error types for database operations.

use thiserror::Error;

/// Storage operation error type
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database connection or query error
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    /// Data serialization error
    #[error("数据序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Project not found
    #[error("项目不存在: {0}")]
    ProjectNotFound(String),

    /// Session not found
    #[error("会话不存在: {0}")]
    SessionNotFound(String),

    /// Generic not found error
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// Lock error when accessing database
    #[error("数据库锁错误")]
    LockError,

    /// Invalid input parameters
    #[error("无效输入: {0}")]
    InvalidInput(String),
}
