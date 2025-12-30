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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ParseError::missing_field("id");
        assert_eq!(err.to_string(), "缺少必需字段: id");

        let err = ParseError::invalid_format("unexpected structure");
        assert_eq!(err.to_string(), "无效的数据格式: unexpected structure");

        let err = ParseError::EmptyConversation;
        assert_eq!(err.to_string(), "未找到任何对话记录");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let parse_err: ParseError = io_err.into();
        assert!(matches!(parse_err, ParseError::IoError(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let parse_err: ParseError = json_err.into();
        assert!(matches!(parse_err, ParseError::InvalidJson(_)));
    }
}
