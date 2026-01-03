//! Unified application error types
//!
//! Provides a single error type for the entire application,
//! suitable for returning from Tauri commands.

use serde::Serialize;
use thiserror::Error;

use crate::git::GitError;
use crate::parsers::ParseError;
use crate::storage::StorageError;

/// Application-level error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Log parsing error
    #[error("解析错误: {0}")]
    Parse(#[from] ParseError),

    /// File operation error
    #[error("文件操作错误: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error
    #[error("内部错误: {0}")]
    Internal(String),

    /// Git operation error
    #[error("Git 错误: {0}")]
    Git(#[from] GitError),

    /// Storage/database error
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),

    /// Lock error
    #[error("锁获取失败")]
    LockError,

    /// Resource not found error (Story 2.19)
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// Validation error (Story 2.19)
    #[error("验证错误: {0}")]
    Validation(String),
}

/// Serializable error response for Tauri IPC
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code for client-side handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

/// 将 GitError 转换为精确的错误码
fn git_error_code(err: &GitError) -> &'static str {
    match err {
        GitError::FileNotFound { .. } => "FILE_NOT_FOUND",
        GitError::CommitNotFound(_) => "COMMIT_NOT_FOUND",
        GitError::NotARepository(_) => "NOT_A_REPOSITORY",
        GitError::InvalidUtf8 => "INVALID_UTF8",
        GitError::RepositoryError(_) => "GIT_ERROR",
    }
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        let (code, message) = match &err {
            AppError::Parse(e) => ("PARSE_ERROR".to_string(), e.to_string()),
            AppError::Io(e) => ("IO_ERROR".to_string(), e.to_string()),
            AppError::Internal(msg) => ("INTERNAL_ERROR".to_string(), msg.clone()),
            AppError::Git(e) => (git_error_code(e).to_string(), e.to_string()),
            AppError::Storage(e) => ("STORAGE_ERROR".to_string(), e.to_string()),
            AppError::LockError => ("LOCK_ERROR".to_string(), "锁获取失败".to_string()),
            AppError::NotFound(msg) => ("NOT_FOUND".to_string(), msg.clone()),
            AppError::Validation(msg) => ("VALIDATION_ERROR".to_string(), msg.clone()),
        };
        Self { code, message }
    }
}

// Implement Serialize for AppError to make it work with Tauri commands
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Directly create ErrorResponse from self without cloning
        let (code, message) = match self {
            Self::Parse(e) => ("PARSE_ERROR".to_string(), e.to_string()),
            Self::Io(e) => ("IO_ERROR".to_string(), e.to_string()),
            Self::Internal(msg) => ("INTERNAL_ERROR".to_string(), msg.clone()),
            Self::Git(e) => (git_error_code(e).to_string(), e.to_string()),
            Self::Storage(e) => ("STORAGE_ERROR".to_string(), e.to_string()),
            Self::LockError => ("LOCK_ERROR".to_string(), "锁获取失败".to_string()),
            Self::NotFound(msg) => ("NOT_FOUND".to_string(), msg.clone()),
            Self::Validation(msg) => ("VALIDATION_ERROR".to_string(), msg.clone()),
        };
        ErrorResponse { code, message }.serialize(serializer)
    }
}

impl AppError {
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::internal("something went wrong");
        assert_eq!(err.to_string(), "内部错误: something went wrong");
    }

    #[test]
    fn test_error_serialization() {
        let err = AppError::internal("test error");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("INTERNAL_ERROR"));
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_git_file_not_found_serialization() {
        let git_err = GitError::FileNotFound {
            commit: "abc123".to_string(),
            path: "src/main.rs".to_string(),
        };
        let err = AppError::Git(git_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("FILE_NOT_FOUND"));
        assert!(json.contains("src/main.rs"));
    }

    #[test]
    fn test_git_commit_not_found_serialization() {
        let git_err = GitError::CommitNotFound("before 2020".to_string());
        let err = AppError::Git(git_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("COMMIT_NOT_FOUND"));
    }
}
