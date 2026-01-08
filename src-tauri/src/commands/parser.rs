//! Parser-related Tauri commands
//!
//! Provides IPC commands for parsing AI conversation logs.

use std::path::PathBuf;

use crate::error::AppError;
use crate::models::MantraSession;
use crate::parsers::{ClaudeParser, CodexParser, CursorParser, GeminiParser, LogParser};

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

/// Parse a Gemini CLI log file and return a MantraSession
///
/// # Arguments
/// * `path` - Path to the Gemini session JSON file
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_gemini_log(path: String) -> Result<MantraSession, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = GeminiParser::new();
        parser.parse_file(&path)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse Gemini log content from a string
///
/// # Arguments
/// * `content` - JSON content of a Gemini conversation
/// * `project_path` - Optional project path to use as cwd
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse error
#[tauri::command]
pub fn parse_gemini_log_string(
    content: String,
    project_path: Option<String>,
) -> Result<MantraSession, AppError> {
    let parser = match project_path {
        Some(path) => GeminiParser::with_project_path(path),
        None => GeminiParser::new(),
    };
    parser.parse_string(&content).map_err(AppError::from)
}

/// Parse all Gemini CLI conversations from all discovered projects
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_gemini_all() -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = GeminiParser::new();
        parser.parse_all()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse Gemini CLI conversations for a specific project hash
///
/// # Arguments
/// * `project_hash` - The project hash (directory name under ~/.gemini/tmp/)
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions for this project
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_gemini_project(project_hash: String) -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = GeminiParser::new();
        parser.parse_project(&project_hash)
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

    #[test]
    fn test_parse_gemini_log_string() {
        let json = r#"{
            "sessionId": "gemini-test",
            "projectHash": "abc123",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:05:00.000Z",
            "messages": [
                {"id": "m1", "timestamp": "2025-12-30T20:00:10.000Z", "type": "user", "content": "Hello"},
                {"id": "m2", "timestamp": "2025-12-30T20:01:00.000Z", "type": "gemini", "content": "Hi there!"}
            ]
        }"#;

        let result = parse_gemini_log_string(json.to_string(), None);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "gemini-test");
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_parse_gemini_with_project_path() {
        let json = r#"{
            "sessionId": "gemini-test",
            "projectHash": "abc123",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:05:00.000Z",
            "messages": []
        }"#;

        let result = parse_gemini_log_string(
            json.to_string(),
            Some("/home/user/my-project".to_string()),
        );
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.cwd, "/home/user/my-project");
    }
}

/// Parse a Codex CLI log file and return a MantraSession
///
/// # Arguments
/// * `path` - Path to the Codex session JSONL file
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_codex_log(path: String) -> Result<MantraSession, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = CodexParser::new();
        parser.parse_file(&path)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse Codex log content from a string
///
/// # Arguments
/// * `content` - JSONL content of a Codex conversation
/// * `project_path` - Optional project path to use as cwd
///
/// # Returns
/// * `Ok(MantraSession)` - Successfully parsed session
/// * `Err(AppError)` - Parse error
#[tauri::command]
pub fn parse_codex_log_string(
    content: String,
    project_path: Option<String>,
) -> Result<MantraSession, AppError> {
    let parser = match project_path {
        Some(path) => CodexParser::with_project_path(path),
        None => CodexParser::new(),
    };
    parser.parse_string(&content).map_err(AppError::from)
}

/// Parse all Codex CLI conversations from all discovered sessions
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_codex_all() -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = CodexParser::new();
        parser.parse_all()
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

/// Parse Codex CLI conversations for a specific project (by cwd)
///
/// # Arguments
/// * `project_cwd` - The project working directory
///
/// # Returns
/// * `Ok(Vec<MantraSession>)` - Successfully parsed sessions for this project
/// * `Err(AppError)` - Parse or IO error
#[tauri::command]
pub async fn parse_codex_project(project_cwd: String) -> Result<Vec<MantraSession>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        let parser = CodexParser::new();
        parser.parse_project(&project_cwd)
    })
    .await
    .map_err(|e| AppError::internal(format!("Task join error: {}", e)))?;

    result.map_err(AppError::from)
}

#[cfg(test)]
mod codex_tests {
    use super::*;

    #[test]
    fn test_parse_codex_log_string() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"codex-test","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Hi there!"}]}}"#;

        let result = parse_codex_log_string(jsonl.to_string(), None);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "codex-test");
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_parse_codex_with_project_path() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"codex-test","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

        let result = parse_codex_log_string(
            jsonl.to_string(),
            Some("/home/user/my-project".to_string()),
        );
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.cwd, "/home/user/my-project");
    }
}
