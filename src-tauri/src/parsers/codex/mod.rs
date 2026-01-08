//! Codex CLI log parser
//!
//! Parses conversation logs from Codex CLI into MantraSession format.
//! Codex CLI stores conversations in JSONL files located at:
//! - ~/.codex/sessions/YYYY/MM/DD/rollout-{timestamp}-{session_id}.jsonl
//!
//! ## Supported Features
//!
//! - User and assistant messages with full content
//! - Function calls (shell, update_plan, etc.) as ToolUse/ToolResult
//! - Timestamp preservation
//! - Cross-platform path resolution

pub mod path;
pub mod types;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::{DateTime, Utc};

use super::{LogParser, ParseError};
use crate::models::{sources, ContentBlock, MantraSession, Message, Role, SessionMetadata};

pub use path::{get_codex_dir, get_codex_sessions_dir, CodexPaths, CodexSessionFile};
pub use types::*;

/// Parser for Codex CLI conversation logs
#[derive(Debug, Default)]
pub struct CodexParser {
    /// Optional project path override for cwd
    project_path: Option<String>,
}

impl CodexParser {
    /// Create a new CodexParser instance
    pub fn new() -> Self {
        Self { project_path: None }
    }

    /// Create a parser with a specific project path for cwd
    pub fn with_project_path(project_path: String) -> Self {
        Self {
            project_path: Some(project_path),
        }
    }

    /// Parse a Codex JSONL file
    fn parse_jsonl(&self, content: &str, file_path: Option<&str>) -> Result<MantraSession, ParseError> {
        let mut session_meta: Option<CodexSessionMeta> = None;
        let mut messages: Vec<Message> = Vec::new();
        let mut pending_calls: HashMap<String, PendingFunctionCall> = HashMap::new();
        let mut last_timestamp: Option<DateTime<Utc>> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let rollout_line: CodexRolloutLine = serde_json::from_str(line)
                .map_err(|e| ParseError::invalid_format(format!("Invalid JSONL line: {}", e)))?;

            // Update last timestamp
            if let Ok(ts) = parse_timestamp(&rollout_line.timestamp) {
                last_timestamp = Some(ts);
            }

            match rollout_line.line_type {
                CodexLineType::SessionMeta => {
                    let meta: CodexSessionMeta = serde_json::from_value(rollout_line.payload)
                        .map_err(|e| ParseError::invalid_format(format!("Invalid session_meta: {}", e)))?;
                    session_meta = Some(meta);
                }
                CodexLineType::ResponseItem => {
                    self.process_response_item(
                        rollout_line.payload,
                        &rollout_line.timestamp,
                        &mut messages,
                        &mut pending_calls,
                    )?;
                }
                CodexLineType::EventMsg | CodexLineType::TurnContext => {
                    // Skip these line types
                }
                CodexLineType::Unknown => {
                    // Log unknown line types for debugging (future Codex versions may add new types)
                    #[cfg(debug_assertions)]
                    eprintln!("Warning: Unknown Codex line type encountered, skipping");
                }
            }
        }

        // Validate we have session metadata
        let meta = session_meta.ok_or_else(|| ParseError::missing_field("session_meta"))?;

        // Validate session ID
        if meta.id.is_empty() {
            return Err(ParseError::missing_field("session_meta.id"));
        }

        // Parse timestamps
        let created_at = parse_timestamp(&meta.timestamp)?;
        let updated_at = last_timestamp.unwrap_or(created_at);

        // Determine cwd - use project_path if available
        let cwd = self.project_path.clone().unwrap_or_else(|| meta.cwd.clone());

        Ok(MantraSession {
            id: meta.id,
            source: sources::CODEX.to_string(),
            cwd,
            created_at,
            updated_at,
            messages,
            metadata: SessionMetadata {
                model: meta.model_provider,
                title: None,
                original_path: file_path.map(String::from),
                total_tokens: None,
                ..Default::default()
            },
        })
    }

    /// Process a response_item payload
    fn process_response_item(
        &self,
        payload: serde_json::Value,
        timestamp: &str,
        messages: &mut Vec<Message>,
        pending_calls: &mut HashMap<String, PendingFunctionCall>,
    ) -> Result<(), ParseError> {
        let item: CodexResponseItem = serde_json::from_value(payload)
            .map_err(|e| ParseError::invalid_format(format!("Invalid response_item: {}", e)))?;

        let ts = parse_timestamp(timestamp).ok();

        match item {
            CodexResponseItem::Message { role, content } => {
                // Skip empty messages
                if content.is_empty() {
                    return Ok(());
                }

                // Skip environment_context messages (they start with <environment_context>)
                let first_text = content.first().map(|c| c.text()).unwrap_or("");
                if first_text.trim().starts_with("<environment_context>")
                    || first_text.trim().starts_with("# AGENTS.md")
                {
                    return Ok(());
                }

                let content_blocks: Vec<ContentBlock> = content
                    .into_iter()
                    .map(|c| ContentBlock::Text { text: c.text().to_string() })
                    .collect();

                messages.push(Message {
                    role: role.to_mantra_role(),
                    content_blocks,
                    timestamp: ts,
                    mentioned_files: Vec::new(),
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }

            CodexResponseItem::FunctionCall { name, arguments, call_id } => {
                // Parse arguments to extract mentioned files
                let mut mentioned_files = Vec::new();
                if let Ok(args) = serde_json::from_str::<serde_json::Value>(&arguments) {
                    extract_file_paths(&args, &mut mentioned_files);
                }

                // Store pending call for later matching with output
                pending_calls.insert(
                    call_id.clone(),
                    PendingFunctionCall {
                        name: name.clone(),
                        arguments: arguments.clone(),
                        timestamp: ts,
                        mentioned_files: mentioned_files.clone(),
                    },
                );

                // Create ToolUse block
                let input = serde_json::from_str(&arguments).unwrap_or(serde_json::Value::Null);
                let content_blocks = vec![ContentBlock::ToolUse {
                    id: call_id.clone(),
                    name,
                    input,
                    correlation_id: Some(call_id),
                }];

                messages.push(Message {
                    role: Role::Assistant,
                    content_blocks,
                    timestamp: ts,
                    mentioned_files,
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }

            CodexResponseItem::FunctionCallOutput { call_id, output } => {
                // Find pending call
                let pending = pending_calls.remove(&call_id);

                // Detect errors more robustly:
                // - Explicit error prefixes from Codex CLI
                // - Non-zero exit codes in shell output
                // - Common error patterns
                let is_error = output.starts_with("Error:")
                    || output.starts_with("error:")
                    || output.starts_with("FAILED")
                    || output.contains("exit code: 1")
                    || output.contains("exit status: 1")
                    || (output.starts_with("Command failed") && output.contains("error"));

                let content_blocks = vec![ContentBlock::ToolResult {
                    tool_use_id: call_id.clone(),
                    content: output,
                    is_error,
                    correlation_id: Some(call_id),
                }];

                // Use pending call's timestamp if available
                let msg_ts = pending.as_ref().and_then(|p| p.timestamp).or(ts);

                messages.push(Message {
                    role: Role::Assistant,
                    content_blocks,
                    timestamp: msg_ts,
                    mentioned_files: pending.map(|p| p.mentioned_files).unwrap_or_default(),
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }
        }

        Ok(())
    }

    /// Parse all sessions from the Codex CLI directory
    pub fn parse_all(&self) -> Result<Vec<MantraSession>, ParseError> {
        let paths = CodexPaths::detect()?;
        let session_files = paths.scan_all_sessions()?;

        let mut sessions = Vec::new();
        for session_file in session_files {
            match self.parse_file(session_file.path.to_string_lossy().as_ref()) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse {}: {}",
                        session_file.path.display(),
                        e
                    );
                }
            }
        }

        Ok(sessions)
    }

    /// Parse sessions for a specific project (by cwd)
    pub fn parse_project(&self, project_cwd: &str) -> Result<Vec<MantraSession>, ParseError> {
        let all_sessions = self.parse_all()?;

        Ok(all_sessions
            .into_iter()
            .filter(|s| s.cwd == project_cwd)
            .collect())
    }
}

/// Pending function call waiting for output
struct PendingFunctionCall {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    arguments: String,
    timestamp: Option<DateTime<Utc>>,
    mentioned_files: Vec<String>,
}

impl LogParser for CodexParser {
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError> {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ParseError::invalid_format(format!("File not found: {}", path)));
        }

        let file = fs::File::open(path_obj)?;
        let reader = BufReader::new(file);

        let mut content = String::new();
        for line in reader.lines() {
            let line = line?;
            content.push_str(&line);
            content.push('\n');
        }

        self.parse_jsonl(&content, Some(path))
    }

    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError> {
        self.parse_jsonl(content, None)
    }
}

/// Parse an ISO 8601 timestamp string to DateTime<Utc>
fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.3f")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| ParseError::invalid_format(format!("Invalid timestamp '{}': {}", timestamp, e)))
}

/// Extract file paths from function call arguments
fn extract_file_paths(args: &serde_json::Value, files: &mut Vec<String>) {
    const PATH_FIELDS: &[&str] = &[
        "path", "file_path", "target_file", "source_file", "filename",
        "file", "target", "source", "destination", "cwd", "directory",
    ];

    if let Some(obj) = args.as_object() {
        for field in PATH_FIELDS {
            if let Some(value) = obj.get(*field) {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() && (s.starts_with('/') || s.starts_with('.') || s.contains('/')) {
                        files.push(s.to_string());
                    }
                }
            }
        }
        // Handle "command" field for shell calls
        if let Some(cmd) = obj.get("command") {
            if let Some(arr) = cmd.as_array() {
                // Command array format: ["bash", "-lc", "..."]
                for item in arr.iter().skip(2) {
                    if let Some(s) = item.as_str() {
                        // Extract file paths from shell commands
                        for word in s.split_whitespace() {
                            if word.starts_with('/') || word.starts_with("./") || word.starts_with("../") {
                                files.push(word.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SESSION: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-session-123","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/home/user/project","cli_version":"0.77.0","originator":"codex_cli_rs","source":"cli"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Help me with this code"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"I'll help you with that."}]}}"#;

    const SESSION_WITH_FUNCTION_CALL: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"func-session","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"List files"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls -la\"]}","call_id":"call_123"}}
{"timestamp":"2025-12-30T20:00:03.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_123","output":"file1.txt\nfile2.txt"}}
{"timestamp":"2025-12-30T20:00:04.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Here are the files."}]}}"#;

    const SESSION_WITH_EVENTS: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"event-session","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"event_msg","payload":{"type":"user_message","message":"Hello"}}
{"timestamp":"2025-12-30T20:00:03.000Z","type":"turn_context","payload":{"cwd":"/tmp","model":"gpt-5"}}
{"timestamp":"2025-12-30T20:00:04.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Hi!"}]}}"#;

    #[test]
    fn test_parse_simple_session() {
        let parser = CodexParser::new();
        let session = parser.parse_string(SIMPLE_SESSION).unwrap();

        assert_eq!(session.id, "test-session-123");
        assert_eq!(session.source, sources::CODEX);
        assert_eq!(session.cwd, "/home/user/project");
        assert_eq!(session.messages.len(), 2);

        // Check user message
        assert_eq!(session.messages[0].role, Role::User);
        match &session.messages[0].content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "Help me with this code"),
            _ => panic!("Expected Text block"),
        }

        // Check assistant message
        assert_eq!(session.messages[1].role, Role::Assistant);
        match &session.messages[1].content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "I'll help you with that."),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_parse_session_with_function_call() {
        let parser = CodexParser::new();
        let session = parser.parse_string(SESSION_WITH_FUNCTION_CALL).unwrap();

        assert_eq!(session.id, "func-session");
        // user + function_call + function_call_output + assistant = 4 messages
        assert_eq!(session.messages.len(), 4);

        // Check function call (ToolUse)
        let tool_use_msg = &session.messages[1];
        assert_eq!(tool_use_msg.role, Role::Assistant);
        match &tool_use_msg.content_blocks[0] {
            ContentBlock::ToolUse { id, name, .. } => {
                assert_eq!(id, "call_123");
                assert_eq!(name, "shell");
            }
            _ => panic!("Expected ToolUse block"),
        }

        // Check function output (ToolResult)
        let tool_result_msg = &session.messages[2];
        match &tool_result_msg.content_blocks[0] {
            ContentBlock::ToolResult { tool_use_id, content, .. } => {
                assert_eq!(tool_use_id, "call_123");
                assert!(content.contains("file1.txt"));
            }
            _ => panic!("Expected ToolResult block"),
        }
    }

    #[test]
    fn test_events_filtered() {
        let parser = CodexParser::new();
        let session = parser.parse_string(SESSION_WITH_EVENTS).unwrap();

        // Only user and assistant messages, no event_msg or turn_context
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_parse_with_project_path() {
        let parser = CodexParser::with_project_path("/custom/path".to_string());
        let session = parser.parse_string(SIMPLE_SESSION).unwrap();

        assert_eq!(session.cwd, "/custom/path");
    }

    #[test]
    fn test_parse_empty_session_id_fails() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

        let parser = CodexParser::new();
        let result = parser.parse_string(jsonl);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json_fails() {
        let parser = CodexParser::new();
        let result = parser.parse_string("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_session_meta_fails() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

        let parser = CodexParser::new();
        let result = parser.parse_string(jsonl);
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_environment_context() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>\n  <cwd>/tmp</cwd>\n</environment_context>"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        // Only the "Hello" message should be included
        assert_eq!(session.messages.len(), 1);
        match &session.messages[0].content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_parse_timestamp_formats() {
        // RFC 3339 with Z
        let ts = parse_timestamp("2025-12-30T20:11:00.000Z").unwrap();
        assert_eq!(ts.year(), 2025);

        // RFC 3339 with offset
        let ts = parse_timestamp("2025-12-30T20:11:00.000+08:00").unwrap();
        assert_eq!(ts.year(), 2025);
    }

    use chrono::Datelike;

    #[test]
    fn test_extract_file_paths_from_shell() {
        let args = serde_json::json!({
            "command": ["bash", "-lc", "cat /etc/passwd ./local/file"]
        });

        let mut files = Vec::new();
        extract_file_paths(&args, &mut files);

        assert!(files.contains(&"/etc/passwd".to_string()));
        assert!(files.contains(&"./local/file".to_string()));
    }
}
