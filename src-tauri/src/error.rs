//! Unified application error types
//!
//! Provides a single error type for the entire application,
//! suitable for returning from Tauri commands.

use serde::Serialize;
use thiserror::Error;

use crate::parsers::ParseError;

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
}

/// Serializable error response for Tauri IPC
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code for client-side handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        let (code, message) = match &err {
            AppError::Parse(e) => ("PARSE_ERROR".to_string(), e.to_string()),
            AppError::Io(e) => ("IO_ERROR".to_string(), e.to_string()),
            AppError::Internal(msg) => ("INTERNAL_ERROR".to_string(), msg.clone()),
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
}
