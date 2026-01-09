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
use crate::models::{sources, normalize_tool, ContentBlock, GitInfo, MantraSession, Message, Role, SessionMetadata};

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

        // Extract Git info (AC1: Git 信息)
        let git = meta.git.as_ref().map(|g| GitInfo {
            branch: g.branch.clone(),
            commit: g.commit_hash.clone(),
            repository_url: g.repository_url.clone(),
        });

        // Extract instructions (AC2: 系统指令)
        let instructions = meta.instructions.clone();

        // Extract source_metadata (AC4: source_metadata 透传)
        let source_metadata = {
            let mut sm = serde_json::Map::new();

            if let Some(version) = &meta.cli_version {
                sm.insert("cli_version".to_string(), serde_json::json!(version));
            }

            if let Some(originator) = &meta.originator {
                sm.insert("originator".to_string(), serde_json::json!(originator));
            }

            if let Some(source) = &meta.source {
                sm.insert("source".to_string(), serde_json::json!(source));
            }

            if !sm.is_empty() {
                Some(serde_json::Value::Object(sm))
            } else {
                None
            }
        };

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
                git,  // AC1: Set git info
                instructions,  // AC2: Set instructions
                source_metadata,  // AC4: Set source_metadata
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

                // Strip system reminder tags and filter empty blocks
                let content_blocks: Vec<ContentBlock> = content
                    .into_iter()
                    .filter_map(|c| {
                        let cleaned = crate::parsers::strip_system_reminders(c.text());
                        if cleaned.is_empty() {
                            None
                        } else {
                            Some(ContentBlock::Text { text: cleaned })
                        }
                    })
                    .collect();

                // Skip messages with no content after cleaning
                if content_blocks.is_empty() {
                    return Ok(());
                }

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

                // AC3: Preprocess input for Codex-specific formats, then call normalize_tool
                let normalized_input = preprocess_codex_tool_input(&name, &input);
                let standard_tool = Some(normalize_tool(&name, &normalized_input));

                let content_blocks = vec![ContentBlock::ToolUse {
                    id: call_id.clone(),
                    name,
                    input,
                    correlation_id: Some(call_id),
                    standard_tool,  // AC3: Set standard_tool
                    display_name: None,
                    description: None,
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
                // - Common error patterns (Rust panics, compilation errors, etc.)
                let is_error = output.starts_with("Error:")
                    || output.starts_with("error:")
                    || output.starts_with("error[")  // Rust compiler errors
                    || output.starts_with("FAILED")
                    || output.starts_with("fatal:")  // Git fatal errors
                    || output.starts_with("panic:")  // Explicit panic messages
                    || output.contains("exit code: 1")
                    || output.contains("exit status: 1")
                    || output.contains("exited with code")  // Generic exit code pattern
                    || output.contains("thread 'main' panicked")  // Rust panic
                    || output.contains("thread '") && output.contains("' panicked")  // Any thread panic
                    || (output.starts_with("Command failed") && output.contains("error"));

                // Strip system reminder tags from tool result content
                let cleaned_output = crate::parsers::strip_system_reminders(&output);
                let content_blocks = vec![ContentBlock::ToolResult {
                    tool_use_id: call_id.clone(),
                    content: cleaned_output,
                    is_error,
                    correlation_id: Some(call_id),
                    structured_result: None,
                    display_content: None,
                    render_as_markdown: None,
                    user_decision: None,
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

/// Preprocess Codex tool input to normalize for StandardTool mapping (AC3)
///
/// Codex uses specific argument formats that differ from other tools:
/// - `shell` command: `{"command": ["bash", "-lc", "actual_command"]}` → needs to extract the actual command string
///
/// This function transforms Codex-specific formats to match what normalize_tool() expects.
fn preprocess_codex_tool_input(name: &str, input: &serde_json::Value) -> serde_json::Value {
    match name.to_lowercase().as_str() {
        "shell" => {
            // Codex shell: {"command": ["bash", "-lc", "ls -la"]}
            // normalize_tool expects: {"command": "ls -la"}
            if let Some(cmd_array) = input.get("command").and_then(|v| v.as_array()) {
                // Extract the actual command from array (typically at index 2+)
                // Format: ["bash", "-lc", "actual_command"]
                let actual_command = cmd_array
                    .iter()
                    .skip(2)
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                let mut normalized = serde_json::Map::new();
                normalized.insert("command".to_string(), serde_json::json!(actual_command));

                // Preserve other fields like cwd if present
                if let Some(obj) = input.as_object() {
                    for (key, value) in obj.iter() {
                        if key != "command" {
                            normalized.insert(key.clone(), value.clone());
                        }
                    }
                }

                return serde_json::Value::Object(normalized);
            }
            // Fallback: return as-is
            input.clone()
        }
        // Other tools: pass through unchanged
        _ => input.clone(),
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
    fn test_is_error_detection_rust_panic() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-panic","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"cargo run\"]}","call_id":"call_panic"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_panic","output":"thread 'main' panicked at src/main.rs:10"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_result = session.messages.iter()
            .find_map(|m| match m.content_blocks.first() {
                Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
                _ => None,
            })
            .expect("Should have ToolResult");

        assert!(tool_result, "Rust panic should be detected as error");
    }

    #[test]
    fn test_is_error_detection_rust_compiler() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-compile","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"cargo build\"]}","call_id":"call_compile"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_compile","output":"error[E0308]: mismatched types"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_result = session.messages.iter()
            .find_map(|m| match m.content_blocks.first() {
                Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
                _ => None,
            })
            .expect("Should have ToolResult");

        assert!(tool_result, "Rust compiler error should be detected as error");
    }

    #[test]
    fn test_is_error_detection_git_fatal() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"git push\"]}","call_id":"call_git"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_git","output":"fatal: not a git repository"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_result = session.messages.iter()
            .find_map(|m| match m.content_blocks.first() {
                Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
                _ => None,
            })
            .expect("Should have ToolResult");

        assert!(tool_result, "Git fatal error should be detected as error");
    }

    #[test]
    fn test_is_error_detection_success() {
        // Normal output should NOT be detected as error
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-ok","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls\"]}","call_id":"call_ok"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_ok","output":"file1.txt\nfile2.txt"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_result = session.messages.iter()
            .find_map(|m| match m.content_blocks.first() {
                Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
                _ => None,
            })
            .expect("Should have ToolResult");

        assert!(!tool_result, "Normal output should NOT be detected as error");
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

    // ====== AC1: Git Info Tests ======

    #[test]
    fn test_parse_git_info() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","git":{"commit_hash":"abc123def456","branch":"main","repository_url":"https://github.com/user/repo"}}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.git.is_some());
        let git = session.metadata.git.unwrap();
        assert_eq!(git.commit, Some("abc123def456".to_string()));
        assert_eq!(git.branch, Some("main".to_string()));
        assert_eq!(git.repository_url, Some("https://github.com/user/repo".to_string()));
    }

    #[test]
    fn test_parse_git_info_partial() {
        // Only branch, no commit or url
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git-partial","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","git":{"branch":"feature/test"}}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.git.is_some());
        let git = session.metadata.git.unwrap();
        assert_eq!(git.branch, Some("feature/test".to_string()));
        assert!(git.commit.is_none());
        assert!(git.repository_url.is_none());
    }

    #[test]
    fn test_parse_git_info_none() {
        // No git field at all (backward compatibility)
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.git.is_none());
    }

    // ====== AC2: Instructions Tests ======

    #[test]
    fn test_parse_instructions() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-inst","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","instructions":"You are a helpful assistant. Follow coding best practices."}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert_eq!(session.metadata.instructions, Some("You are a helpful assistant. Follow coding best practices.".to_string()));
    }

    #[test]
    fn test_parse_instructions_none() {
        // No instructions field (backward compatibility)
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-inst","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.instructions.is_none());
    }

    // ====== AC3: StandardTool Mapping Tests ======

    #[test]
    fn test_parse_standard_tool_shell() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-shell","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls -la\"]}","call_id":"call_shell_1"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        // Find the ToolUse message
        let tool_msg = session.messages.iter()
            .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
            .expect("Should have ToolUse message");

        if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
            assert_eq!(name, "shell");
            assert!(standard_tool.is_some(), "standard_tool should be set");
            match standard_tool {
                Some(crate::models::StandardTool::ShellExec { command, .. }) => {
                    assert_eq!(command, "ls -la");
                }
                _ => panic!("Expected ShellExec, got {:?}", standard_tool),
            }
        } else {
            panic!("Expected ToolUse block");
        }
    }

    #[test]
    fn test_parse_standard_tool_read_file() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-read","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"read_file","arguments":"{\"path\":\"/src/main.rs\"}","call_id":"call_read_1"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_msg = session.messages.iter()
            .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
            .expect("Should have ToolUse message");

        if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
            assert_eq!(name, "read_file");
            assert!(standard_tool.is_some());
            match standard_tool {
                Some(crate::models::StandardTool::FileRead { path, .. }) => {
                    assert_eq!(path, "/src/main.rs");
                }
                _ => panic!("Expected FileRead, got {:?}", standard_tool),
            }
        } else {
            panic!("Expected ToolUse block");
        }
    }

    #[test]
    fn test_parse_standard_tool_apply_diff() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-diff","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"apply_diff","arguments":"{\"path\":\"/src/lib.rs\",\"diff\":\"--- a/lib.rs\\n+++ b/lib.rs\"}","call_id":"call_diff_1"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_msg = session.messages.iter()
            .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
            .expect("Should have ToolUse message");

        if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
            assert_eq!(name, "apply_diff");
            assert!(standard_tool.is_some());
            match standard_tool {
                Some(crate::models::StandardTool::FileEdit { path, new_string, .. }) => {
                    assert_eq!(path, "/src/lib.rs");
                    // apply_diff uses the "diff" field which maps to new_string
                    assert_eq!(new_string, &Some("--- a/lib.rs\n+++ b/lib.rs".to_string()));
                }
                _ => panic!("Expected FileEdit, got {:?}", standard_tool),
            }
        } else {
            panic!("Expected ToolUse block");
        }
    }

    #[test]
    fn test_parse_standard_tool_write_file() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-write","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"write_file","arguments":"{\"path\":\"/src/new.rs\",\"content\":\"fn main() {}\"}","call_id":"call_write_1"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_msg = session.messages.iter()
            .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
            .expect("Should have ToolUse message");

        if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
            assert_eq!(name, "write_file");
            assert!(standard_tool.is_some());
            match standard_tool {
                Some(crate::models::StandardTool::FileWrite { path, content }) => {
                    assert_eq!(path, "/src/new.rs");
                    assert_eq!(content, "fn main() {}");
                }
                _ => panic!("Expected FileWrite, got {:?}", standard_tool),
            }
        } else {
            panic!("Expected ToolUse block");
        }
    }

    #[test]
    fn test_parse_standard_tool_update_plan() {
        // update_plan is not a standard tool, should map to Other
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-plan","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"update_plan","arguments":"{\"plan\":\"Step 1: Read code\"}","call_id":"call_plan_1"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        let tool_msg = session.messages.iter()
            .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
            .expect("Should have ToolUse message");

        if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
            assert_eq!(name, "update_plan");
            assert!(standard_tool.is_some());
            match standard_tool {
                Some(crate::models::StandardTool::Unknown { name: tool_name, .. }) => {
                    assert_eq!(tool_name, "update_plan");
                }
                _ => panic!("Expected Unknown, got {:?}", standard_tool),
            }
        } else {
            panic!("Expected ToolUse block");
        }
    }

    #[test]
    fn test_preprocess_codex_tool_input_shell() {
        let input = serde_json::json!({
            "command": ["bash", "-lc", "cargo build"]
        });

        let result = preprocess_codex_tool_input("shell", &input);

        assert_eq!(result.get("command").and_then(|v| v.as_str()), Some("cargo build"));
    }

    #[test]
    fn test_preprocess_codex_tool_input_shell_with_cwd() {
        let input = serde_json::json!({
            "command": ["bash", "-lc", "npm install"],
            "cwd": "/project"
        });

        let result = preprocess_codex_tool_input("shell", &input);

        assert_eq!(result.get("command").and_then(|v| v.as_str()), Some("npm install"));
        assert_eq!(result.get("cwd").and_then(|v| v.as_str()), Some("/project"));
    }

    #[test]
    fn test_preprocess_codex_tool_input_passthrough() {
        // Non-shell tools should pass through unchanged
        let input = serde_json::json!({
            "path": "/src/main.rs"
        });

        let result = preprocess_codex_tool_input("read_file", &input);

        assert_eq!(result, input);
    }

    #[test]
    fn test_preprocess_codex_tool_input_shell_empty_array() {
        // Edge case: empty command array
        let input = serde_json::json!({
            "command": []
        });

        let result = preprocess_codex_tool_input("shell", &input);

        // Should return empty command string
        assert_eq!(result.get("command").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn test_preprocess_codex_tool_input_shell_short_array() {
        // Edge case: command array with only bash (no actual command)
        let input = serde_json::json!({
            "command": ["bash", "-lc"]
        });

        let result = preprocess_codex_tool_input("shell", &input);

        // Should return empty command string (skip(2) yields nothing)
        assert_eq!(result.get("command").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn test_preprocess_codex_tool_input_shell_not_array() {
        // Edge case: command is a string instead of array
        let input = serde_json::json!({
            "command": "ls -la"
        });

        let result = preprocess_codex_tool_input("shell", &input);

        // Should return input as-is (fallback)
        assert_eq!(result, input);
    }

    // ====== AC4: source_metadata Tests ======

    #[test]
    fn test_parse_source_metadata_full() {
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-sm","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","cli_version":"0.77.0","originator":"codex_cli_rs","source":"cli"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.source_metadata.is_some());
        let sm = session.metadata.source_metadata.unwrap();
        assert_eq!(sm.get("cli_version").and_then(|v| v.as_str()), Some("0.77.0"));
        assert_eq!(sm.get("originator").and_then(|v| v.as_str()), Some("codex_cli_rs"));
        assert_eq!(sm.get("source").and_then(|v| v.as_str()), Some("cli"));
    }

    #[test]
    fn test_parse_source_metadata_partial() {
        // Only cli_version present
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-sm-partial","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","cli_version":"0.75.0"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.source_metadata.is_some());
        let sm = session.metadata.source_metadata.unwrap();
        assert_eq!(sm.get("cli_version").and_then(|v| v.as_str()), Some("0.75.0"));
        assert!(sm.get("originator").is_none());
        assert!(sm.get("source").is_none());
    }

    #[test]
    fn test_parse_source_metadata_none() {
        // No source metadata fields at all (backward compatibility)
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-sm","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        assert!(session.metadata.source_metadata.is_none());
    }

    // ====== AC5: Backward Compatibility Test ======

    #[test]
    fn test_backward_compatibility_no_new_fields() {
        // Old format log without any new fields (git, instructions, cli_version, etc.)
        let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"old-session-123","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/home/user/project"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

        let parser = CodexParser::new();
        let session = parser.parse_string(jsonl).unwrap();

        // All new fields should be None
        assert!(session.metadata.git.is_none());
        assert!(session.metadata.instructions.is_none());
        assert!(session.metadata.source_metadata.is_none());

        // Existing functionality still works
        assert_eq!(session.id, "old-session-123");
        assert_eq!(session.cwd, "/home/user/project");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::User);
    }
}
