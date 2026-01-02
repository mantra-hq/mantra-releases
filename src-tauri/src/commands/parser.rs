//! Parser-related Tauri commands
//!
//! Provides IPC commands for parsing AI conversation logs.

use std::path::PathBuf;

use crate::error::AppError;
use crate::models::MantraSession;
use crate::parsers::{ClaudeParser, CursorParser, LogParser};

/// Parse a Claude Code log file and return a MantraSession
///
/// # Arguments
/// * `path` - Path to the Claude log file (JSON format)
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_claude_log(path: String) -> Result<MantraSession, AppError> {
    // Use spawn_blocking for file I/O to avoid blocking the async runtime
    let result = tokio::task::spawn_blocking(move || {
        let parser = ClaudeParser::new();
        parser.parse_file(&path)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse Claude log content from a string
///
/// # Arguments
/// * `content` - JSON content of a Claude conversation
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse error
#[tauri::command]
pub fn parse_claude_log_string(content: String) -> Result<MantraSession, AppError> {
    let parser = ClaudeParser::new();
    parser.parse_string(&content).map_err(AppError::from)
}

/// Parse Cursor conversations for a specific project
///
/// # Arguments
/// * `project_path` - Path to the project directory
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_cursor_log(project_path: String) -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = CursorParser::new();
        let path = PathBuf::from(&project_path);
        parser.parse_workspace(&path)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse all Cursor conversations from all workspaces
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions from all workspaces
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_cursor_all() -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = CursorParser::new();
        parser.parse_all()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_log_string() {
        let json = r#"{
            "id": "test_conv",
            "cwd": "/tmp",
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"}
            ]
        }"#;

        let result = parse_claude_log_string(json.to_string());
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "test_conv");
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_claude_log_string("{ invalid }".to_string());
        assert!(result.is_err());
    }
}
