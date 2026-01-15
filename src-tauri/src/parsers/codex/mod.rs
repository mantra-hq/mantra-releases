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
use crate::models::{sources, normalize_tool, ContentBlock, GitInfo, MantraSession, Message, ParserInfo, Role, SessionMetadata, UnknownFormatEntry};

pub use path::{get_codex_dir, get_codex_sessions_dir, CodexPaths, CodexSessionFile};
pub use types::*;

/// Codex Parser version for compatibility tracking (Story 8.15)
pub const CODEX_PARSER_VERSION: &str = "1.1.0";

/// Supported response item types in Codex format
pub const SUPPORTED_RESPONSE_TYPES: &[&str] = &[
    "message",
    "reasoning",
    "local_shell_call",
    "function_call",
    "function_call_output",
    "custom_tool_call",
    "custom_tool_call_output",
    "web_search_call",
    "ghost_snapshot",
    "compaction",
];

/// Maximum raw JSON size to store in UnknownFormatEntry (1KB)
const MAX_RAW_JSON_SIZE: usize = 1024;

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
    /// Story 8.15: Enhanced with unknown format collection
    fn parse_jsonl(&self, content: &str, file_path: Option<&str>) -> Result<MantraSession, ParseError> {
        let mut session_meta: Option<CodexSessionMeta> = None;
        let mut messages: Vec<Message> = Vec::new();
        let mut pending_calls: HashMap<String, PendingFunctionCall> = HashMap::new();
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

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
                    // Story 8.15: Collect unknown formats from response items
                    if let Some(entry) = self.process_response_item(
                        rollout_line.payload,
                        &rollout_line.timestamp,
                        &mut messages,
                        &mut pending_calls,
                    )? {
                        unknown_formats.push(entry);
                    }
                }
                CodexLineType::EventMsg | CodexLineType::TurnContext => {
                    // Skip these line types
                }
                CodexLineType::Unknown => {
                    // Story 8.15: Record unknown line types
                    unknown_formats.push(UnknownFormatEntry {
                        source: "codex".to_string(),
                        type_name: "unknown_line_type".to_string(),
                        raw_json: truncate_raw_json_str(line),
                        timestamp: rollout_line.timestamp.clone(),
                    });
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

        // Story 8.15: Set parser_info and unknown_formats
        let parser_info = Some(ParserInfo {
            parser_version: CODEX_PARSER_VERSION.to_string(),
            supported_formats: SUPPORTED_RESPONSE_TYPES.iter().map(|s| s.to_string()).collect(),
            detected_source_version: meta.cli_version.clone(),
        });

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
                parser_info,  // Story 8.15: Set parser info
                unknown_formats: if unknown_formats.is_empty() { None } else { Some(unknown_formats) },  // Story 8.15
                ..Default::default()
            },
        })
    }

    /// Process a response_item payload
    /// Story 8.15: Now returns Option<UnknownFormatEntry> to collect unknown formats
    fn process_response_item(
        &self,
        payload: serde_json::Value,
        timestamp: &str,
        messages: &mut Vec<Message>,
        pending_calls: &mut HashMap<String, PendingFunctionCall>,
    ) -> Result<Option<UnknownFormatEntry>, ParseError> {
        let item: CodexResponseItem = serde_json::from_value(payload.clone())
            .map_err(|e| ParseError::invalid_format(format!("Invalid response_item: {}", e)))?;

        let ts = parse_timestamp(timestamp).ok();

        match item {
            CodexResponseItem::Message { role, content } => {
                // Skip empty messages
                if content.is_empty() {
                    return Ok(None);
                }

                // Skip environment_context messages (they start with <environment_context>)
                let first_text = content.first().map(|c| c.text()).unwrap_or("");
                if first_text.trim().starts_with("<environment_context>")
                    || first_text.trim().starts_with("# AGENTS.md")
                {
                    return Ok(None);
                }

                // Strip system reminder tags and filter empty blocks
                let content_blocks: Vec<ContentBlock> = content
                    .into_iter()
                    .filter_map(|c| {
                        let cleaned = crate::parsers::strip_system_reminders(c.text());
                        if cleaned.is_empty() {
                            None
                        } else {
                            Some(ContentBlock::Text { text: cleaned, is_degraded: None })
                        }
                    })
                    .collect();

                // Skip messages with no content after cleaning
                if content_blocks.is_empty() {
                    return Ok(None);
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

                // Get output string from payload
                let output_str = output.get_output();

                // Detect errors more robustly:
                // - Explicit error prefixes from Codex CLI
                // - Non-zero exit codes in shell output
                // - Common error patterns (Rust panics, compilation errors, etc.)
                let is_error = output_str.starts_with("Error:")
                    || output_str.starts_with("error:")
                    || output_str.starts_with("error[")  // Rust compiler errors
                    || output_str.starts_with("FAILED")
                    || output_str.starts_with("fatal:")  // Git fatal errors
                    || output_str.starts_with("panic:")  // Explicit panic messages
                    || output_str.contains("exit code: 1")
                    || output_str.contains("exit status: 1")
                    || output_str.contains("exited with code")  // Generic exit code pattern
                    || output_str.contains("thread 'main' panicked")  // Rust panic
                    || output_str.contains("thread '") && output_str.contains("' panicked")  // Any thread panic
                    || (output_str.starts_with("Command failed") && output_str.contains("error"))
                    || output.success == Some(false);  // Explicit failure flag

                // Strip system reminder tags from tool result content
                let cleaned_output = crate::parsers::strip_system_reminders(&output_str);
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

            CodexResponseItem::Reasoning { summary, content, .. } => {
                // Extract readable text from reasoning
                let mut reasoning_text = String::new();

                // Add summary text
                for s in &summary {
                    if !reasoning_text.is_empty() {
                        reasoning_text.push_str("\n");
                    }
                    reasoning_text.push_str(s.text());
                }

                // Add raw content if available
                if let Some(contents) = content {
                    for c in contents {
                        if !reasoning_text.is_empty() {
                            reasoning_text.push_str("\n");
                        }
                        reasoning_text.push_str(c.text());
                    }
                }

                // Only add if we have readable content
                if !reasoning_text.is_empty() {
                    messages.push(Message {
                        role: Role::Assistant,
                        content_blocks: vec![ContentBlock::Thinking {
                            thinking: reasoning_text,
                            subject: None,
                            timestamp: None,
                        }],
                        timestamp: ts,
                        mentioned_files: Vec::new(),
                        message_id: None,
                        parent_id: None,
                        is_sidechain: false,
                        source_metadata: None,
                    });
                }
            }

            CodexResponseItem::LocalShellCall { call_id, action, status } => {
                // Process local shell call as a tool use
                if let LocalShellAction::Exec { command, cwd, exit_code, output } = action {
                    let cmd_str = command
                        .map(|c| c.join(" "))
                        .unwrap_or_default();

                    let call_id_str = call_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                    // If we have output, it's a completed call - create both ToolUse and ToolResult
                    if let Some(output_content) = output {
                        let is_completed = matches!(status, Some(LocalShellStatus::Completed));
                        let is_error = exit_code.map(|c| c != 0).unwrap_or(false);

                        // Create ToolUse
                        let input = serde_json::json!({
                            "command": cmd_str,
                            "cwd": cwd
                        });

                        messages.push(Message {
                            role: Role::Assistant,
                            content_blocks: vec![ContentBlock::ToolUse {
                                id: call_id_str.clone(),
                                name: "shell".to_string(),
                                input: input.clone(),
                                correlation_id: Some(call_id_str.clone()),
                                standard_tool: Some(normalize_tool("shell", &input)),
                                display_name: None,
                                description: None,
                            }],
                            timestamp: ts,
                            mentioned_files: Vec::new(),
                            message_id: None,
                            parent_id: None,
                            is_sidechain: false,
                            source_metadata: None,
                        });

                        // Create ToolResult if completed
                        if is_completed {
                            let cleaned_output = crate::parsers::strip_system_reminders(&output_content);
                            messages.push(Message {
                                role: Role::Assistant,
                                content_blocks: vec![ContentBlock::ToolResult {
                                    tool_use_id: call_id_str.clone(),
                                    content: cleaned_output,
                                    is_error,
                                    correlation_id: Some(call_id_str),
                                    structured_result: None,
                                    display_content: None,
                                    render_as_markdown: None,
                                    user_decision: None,
                                }],
                                timestamp: ts,
                                mentioned_files: Vec::new(),
                                message_id: None,
                                parent_id: None,
                                is_sidechain: false,
                                source_metadata: None,
                            });
                        }
                    } else {
                        // Just the call, no output yet
                        let input = serde_json::json!({
                            "command": cmd_str,
                            "cwd": cwd
                        });

                        pending_calls.insert(
                            call_id_str.clone(),
                            PendingFunctionCall {
                                name: "shell".to_string(),
                                arguments: serde_json::to_string(&input).unwrap_or_default(),
                                timestamp: ts,
                                mentioned_files: Vec::new(),
                            },
                        );

                        messages.push(Message {
                            role: Role::Assistant,
                            content_blocks: vec![ContentBlock::ToolUse {
                                id: call_id_str.clone(),
                                name: "shell".to_string(),
                                input: input.clone(),
                                correlation_id: Some(call_id_str),
                                standard_tool: Some(normalize_tool("shell", &input)),
                                display_name: None,
                                description: None,
                            }],
                            timestamp: ts,
                            mentioned_files: Vec::new(),
                            message_id: None,
                            parent_id: None,
                            is_sidechain: false,
                            source_metadata: None,
                        });
                    }
                }
            }

            CodexResponseItem::CustomToolCall { call_id, name, input, .. } => {
                // Store pending call
                pending_calls.insert(
                    call_id.clone(),
                    PendingFunctionCall {
                        name: name.clone(),
                        arguments: input.clone(),
                        timestamp: ts,
                        mentioned_files: Vec::new(),
                    },
                );

                let input_value = serde_json::from_str(&input).unwrap_or(serde_json::Value::Null);

                messages.push(Message {
                    role: Role::Assistant,
                    content_blocks: vec![ContentBlock::ToolUse {
                        id: call_id.clone(),
                        name: name.clone(),
                        input: input_value.clone(),
                        correlation_id: Some(call_id),
                        standard_tool: Some(normalize_tool(&name, &input_value)),
                        display_name: None,
                        description: None,
                    }],
                    timestamp: ts,
                    mentioned_files: Vec::new(),
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }

            CodexResponseItem::CustomToolCallOutput { call_id, output } => {
                let pending = pending_calls.remove(&call_id);
                let cleaned_output = crate::parsers::strip_system_reminders(&output);
                let is_error = output.starts_with("Error:") || output.starts_with("error:");

                messages.push(Message {
                    role: Role::Assistant,
                    content_blocks: vec![ContentBlock::ToolResult {
                        tool_use_id: call_id.clone(),
                        content: cleaned_output,
                        is_error,
                        correlation_id: Some(call_id),
                        structured_result: None,
                        display_content: None,
                        render_as_markdown: None,
                        user_decision: None,
                    }],
                    timestamp: pending.as_ref().and_then(|p| p.timestamp).or(ts),
                    mentioned_files: pending.map(|p| p.mentioned_files).unwrap_or_default(),
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }

            CodexResponseItem::WebSearchCall { action, .. } => {
                if let WebSearchAction::Search { query } = action {
                    if let Some(q) = query {
                        let input = serde_json::json!({ "query": q });
                        messages.push(Message {
                            role: Role::Assistant,
                            content_blocks: vec![ContentBlock::ToolUse {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: "web_search".to_string(),
                                input: input.clone(),
                                correlation_id: None,
                                standard_tool: Some(normalize_tool("web_search", &input)),
                                display_name: None,
                                description: None,
                            }],
                            timestamp: ts,
                            mentioned_files: Vec::new(),
                            message_id: None,
                            parent_id: None,
                            is_sidechain: false,
                            source_metadata: None,
                        });
                    }
                }
            }

            // GhostSnapshot and Compaction are metadata/internal types
            // We skip them as they don't represent user-visible conversation content
            CodexResponseItem::GhostSnapshot { .. }
            | CodexResponseItem::Compaction { .. } => {
                // Skip these types - they are internal to Codex
            }

            // Story 8.15: Unknown response item types are recorded for monitoring
            CodexResponseItem::Other => {
                return Ok(Some(UnknownFormatEntry {
                    source: "codex".to_string(),
                    type_name: "unknown_response_type".to_string(),
                    raw_json: truncate_raw_json(&payload),
                    timestamp: timestamp.to_string(),
                }));
            }
        }

        Ok(None)
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

/// Truncate raw JSON to maximum size for storage (Story 8.15)
fn truncate_raw_json(json: &serde_json::Value) -> String {
    let raw = serde_json::to_string(json).unwrap_or_default();
    if raw.len() <= MAX_RAW_JSON_SIZE {
        raw
    } else {
        format!("{}... [truncated]", &raw[..MAX_RAW_JSON_SIZE])
    }
}

/// Truncate raw JSON string to maximum size for storage (Story 8.15)
fn truncate_raw_json_str(json_str: &str) -> String {
    if json_str.len() <= MAX_RAW_JSON_SIZE {
        json_str.to_string()
    } else {
        format!("{}... [truncated]", &json_str[..MAX_RAW_JSON_SIZE])
    }
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
mod tests;
