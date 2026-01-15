use super::*;

#[test]
fn test_error_display() {
    let err = ParseError::missing_field("id");
    assert_eq!(err.to_string(), "缺少必需字段: id");

    let err = ParseError::invalid_format("unexpected structure");
    assert_eq!(err.to_string(), "无效的数据格式: unexpected structure");

    let err = ParseError::EmptyConversation;
    assert_eq!(err.to_string(), "未找到任何对话记录");

    let err = ParseError::database_error("connection failed");
    assert_eq!(err.to_string(), "数据库错误: connection failed");

    let err = ParseError::workspace_not_found("/path/to/project");
    assert_eq!(err.to_string(), "工作区未找到: /path/to/project");
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

#[test]
fn test_is_skippable() {
    // Skippable errors - empty sessions that should be silently skipped
    assert!(ParseError::EmptyFile.is_skippable());
    assert!(ParseError::SystemEventsOnly.is_skippable());
    assert!(ParseError::NoValidConversation.is_skippable());

    // Non-skippable errors - real failures that should be reported
    assert!(!ParseError::EmptyConversation.is_skippable());
    assert!(!ParseError::missing_field("id").is_skippable());
    assert!(!ParseError::invalid_format("bad").is_skippable());
    assert!(!ParseError::database_error("fail").is_skippable());
    assert!(!ParseError::workspace_not_found("/path").is_skippable());
}
