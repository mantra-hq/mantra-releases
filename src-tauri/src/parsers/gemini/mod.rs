//! Gemini CLI log parser
//!
//! Parses conversation logs from Gemini CLI into MantraSession format.
//! Gemini CLI stores conversations in JSON files located at:
//! - ~/.gemini/tmp/{projectHash}/chats/session-{date}-{uuid}.json
//!
//! ## Supported Features
//!
//! - User and Gemini messages with full content
//! - Thoughts/reasoning (extended thinking)
//! - Tool calls with results
//! - Timestamp preservation
//! - Cross-platform path resolution

pub mod path;
pub mod types;

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

use super::{LogParser, ParseError};
use crate::models::{normalize_tool, sources, ContentBlock, MantraSession, Message, ParserInfo, SessionMetadata, TokensBreakdown, ToolResultData, UnknownFormatEntry};

pub use path::{get_gemini_dir, get_gemini_tmp_dir, GeminiPaths, GeminiSessionFile};
pub use types::*;

/// Gemini Parser version for compatibility tracking
pub const GEMINI_PARSER_VERSION: &str = "1.1.0";

/// Supported message types in Gemini format
pub const SUPPORTED_MESSAGE_TYPES: &[&str] = &["user", "gemini"];

/// Supported content part types in Gemini format
pub const SUPPORTED_PART_TYPES: &[&str] = &["text", "inline_data", "function_call", "function_response"];

/// Maximum raw JSON size to store in UnknownFormatEntry (1KB)
const MAX_RAW_JSON_SIZE: usize = 1024;

/// Parser for Gemini CLI conversation logs
#[derive(Debug, Default)]
pub struct GeminiParser {
    /// Optional project path override for cwd
    project_path: Option<String>,
}

impl GeminiParser {
    /// Create a new GeminiParser instance
    pub fn new() -> Self {
        Self { project_path: None }
    }

    /// Create a parser with a specific project path for cwd
    pub fn with_project_path(project_path: String) -> Self {
        Self {
            project_path: Some(project_path),
        }
    }

    /// Parse a Gemini conversation JSON file
    fn parse_json(&self, content: &str, file_path: Option<&str>) -> Result<MantraSession, ParseError> {
        let conversation: GeminiConversation =
            serde_json::from_str(content).map_err(|e| ParseError::invalid_format(format!("Invalid JSON: {}", e)))?;

        // Validate required fields
        if conversation.session_id.is_empty() {
            return Err(ParseError::missing_field("sessionId"));
        }

        // Parse timestamps
        let created_at = parse_timestamp(&conversation.start_time)?;
        let updated_at = parse_timestamp(&conversation.last_updated)?;

        // Determine cwd - use project_path if available, otherwise use project_hash as fallback
        let cwd = self
            .project_path
            .clone()
            .unwrap_or_else(|| format!("gemini-project:{}", conversation.project_hash));

        // Build messages and aggregate tokens
        let mut messages = Vec::new();
        let mut last_model: Option<String> = None;
        let mut total_tokens: u64 = 0;

        // Story 8.15: Collect unknown formats for monitoring
        let mut all_unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

        // Token breakdown accumulators (AC1)
        let mut tb_input: u64 = 0;
        let mut tb_output: u64 = 0;
        let mut tb_cached: u64 = 0;
        let mut tb_thoughts: u64 = 0;
        let mut tb_tool: u64 = 0;
        let mut has_breakdown = false;

        for gemini_msg in &conversation.messages {
            // Skip non-includable messages (info, error, warning)
            if !gemini_msg.msg_type.should_include() {
                continue;
            }

            // Track model from gemini messages
            if let Some(model) = &gemini_msg.model {
                last_model = Some(model.clone());
            }

            // Aggregate tokens from gemini messages
            // Prefer the authoritative 'total' field if available
            if let Some(tokens) = &gemini_msg.tokens {
                if let Some(total) = tokens.total {
                    total_tokens += total.max(0) as u64;
                } else {
                    // Fallback: sum input + output if total not available
                    if let Some(input) = tokens.input {
                        total_tokens += input.max(0) as u64;
                    }
                    if let Some(output) = tokens.output {
                        total_tokens += output.max(0) as u64;
                    }
                }

                // Accumulate token breakdown fields (AC1)
                if let Some(v) = tokens.input {
                    tb_input += v.max(0) as u64;
                    has_breakdown = true;
                }
                if let Some(v) = tokens.output {
                    tb_output += v.max(0) as u64;
                    has_breakdown = true;
                }
                if let Some(v) = tokens.cached {
                    tb_cached += v.max(0) as u64;
                    has_breakdown = true;
                }
                if let Some(v) = tokens.thoughts {
                    tb_thoughts += v.max(0) as u64;
                    has_breakdown = true;
                }
                if let Some(v) = tokens.tool {
                    tb_tool += v.max(0) as u64;
                    has_breakdown = true;
                }
            }

            let (converted, unknown_formats) = self.convert_message(gemini_msg)?;
            messages.extend(converted);
            all_unknown_formats.extend(unknown_formats);
        }

        // Build tokens_breakdown (AC1)
        let tokens_breakdown = if has_breakdown {
            Some(TokensBreakdown {
                input: if tb_input > 0 { Some(tb_input) } else { None },
                output: if tb_output > 0 { Some(tb_output) } else { None },
                cached: if tb_cached > 0 { Some(tb_cached) } else { None },
                thoughts: if tb_thoughts > 0 { Some(tb_thoughts) } else { None },
                tool: if tb_tool > 0 { Some(tb_tool) } else { None },
            })
        } else {
            None
        };

        // Build source_metadata (AC4)
        let source_metadata = if !conversation.project_hash.is_empty() {
            Some(serde_json::json!({
                "project_hash": conversation.project_hash
            }))
        } else {
            None
        };

        let mut session = MantraSession {
            id: conversation.session_id,
            source: sources::GEMINI.to_string(),
            cwd,
            created_at,
            updated_at,
            messages,
            metadata: SessionMetadata {
                model: last_model.clone(),
                title: conversation.summary,
                original_path: file_path.map(String::from),
                total_tokens: if total_tokens > 0 { Some(total_tokens) } else { None },
                tokens_breakdown,
                source_metadata,
                // Story 8.15: Add parser info and unknown formats
                parser_info: Some(ParserInfo {
                    parser_version: GEMINI_PARSER_VERSION.to_string(),
                    supported_formats: SUPPORTED_PART_TYPES.iter().map(|s| s.to_string()).collect(),
                    detected_source_version: last_model,
                }),
                unknown_formats: if all_unknown_formats.is_empty() {
                    None
                } else {
                    Some(all_unknown_formats)
                },
                ..Default::default()
            },
        };

        // Update updated_at to match the last message timestamp if available
        if let Some(last_msg) = session.messages.last() {
            if let Some(ts) = last_msg.timestamp {
                session.updated_at = ts;
            }
        }

        Ok(session)
    }


    /// Convert a Gemini message to Mantra Messages
    ///
    /// Mantra 消息结构规范：
    /// 1. 文本消息 (thinking + text) → 一条消息
    /// 2. 工具调用消息 (tool_use + tool_result) → 每个工具调用一条独立消息
    ///
    /// 这样设计确保每个消息是语义完整的单元，便于前端渲染和理解。
    /// Story 8.15: Returns (messages, unknown_format_entries) for monitoring
    fn convert_message(&self, gemini_msg: &GeminiMessage) -> Result<(Vec<Message>, Vec<UnknownFormatEntry>), ParseError> {
        let role = match gemini_msg.msg_type.to_mantra_role() {
            Some(r) => r,
            None => return Ok((Vec::new(), Vec::new())), // Skip messages with unknown role
        };

        let mut messages = Vec::new();
        let mut unknown_formats = Vec::new();
        let timestamp = parse_timestamp(&gemini_msg.timestamp).ok();

        // === 消息 1: 思考 + 文本内容 ===
        let mut text_blocks = Vec::new();

        // Add thoughts first (for Gemini messages)
        if let Some(thoughts) = &gemini_msg.thoughts {
            for thought in thoughts {
                text_blocks.push(ContentBlock::Thinking {
                    thinking: thought.as_formatted_string(),
                    subject: Some(thought.subject.clone()),
                    timestamp: thought.timestamp.clone(),
                });
            }
        }

        // Add text content and check for unknown fields in parts
        match &gemini_msg.content {
            GeminiContent::Text(s) => {
                let cleaned = crate::parsers::strip_system_reminders(s);
                if !cleaned.is_empty() {
                    text_blocks.push(ContentBlock::Text { text: cleaned, is_degraded: None });
                }
            }
            GeminiContent::Parts(parts) => {
                let mut text_parts = Vec::new();
                for part in parts {
                    // Collect known text content
                    if let Some(text) = &part.text {
                        text_parts.push(text.clone());
                    }
                    // Story 8.16: Parse inline_data as Image block (AC3)
                    if let Some(inline_data) = &part.inline_data {
                        if let (Some(mime_type), Some(data)) = (&inline_data.mime_type, &inline_data.data) {
                            // Only process if mime_type indicates an image
                            if mime_type.starts_with("image/") {
                                text_blocks.push(ContentBlock::Image {
                                    media_type: mime_type.clone(),
                                    data: data.clone(),
                                    source_type: Some("base64".to_string()),
                                    alt_text: None,
                                });
                            }
                        }
                    }
                    // Story 8.15: Check for unknown fields in GeminiPart
                    if part.has_unknown_fields() {
                        for field_name in part.unknown_field_names() {
                            let raw_value = part.unknown_fields.get(&field_name)
                                .map(truncate_raw_json)
                                .unwrap_or_default();

                            unknown_formats.push(UnknownFormatEntry {
                                source: "gemini".to_string(),
                                type_name: field_name.clone(),
                                raw_json: raw_value,
                                timestamp: Utc::now().to_rfc3339(),
                            });

                            // Create degraded text block for unknown field
                            let degraded_text = format!("[无法解析的内容: {}]", field_name);
                            text_blocks.push(ContentBlock::Text {
                                text: degraded_text,
                                is_degraded: Some(true),
                            });
                        }
                    }
                }
                if !text_parts.is_empty() {
                    let combined = text_parts.join("");
                    let cleaned = crate::parsers::strip_system_reminders(&combined);
                    if !cleaned.is_empty() {
                        text_blocks.push(ContentBlock::Text { text: cleaned, is_degraded: None });
                    }
                }
            }
        }

        // Create text message if we have content
        if !text_blocks.is_empty() {
            messages.push(Message {
                role: role.clone(),
                content_blocks: text_blocks,
                timestamp,
                mentioned_files: Vec::new(),
                message_id: None,
                parent_id: None,
                is_sidechain: false,
                source_metadata: None,
            });
        }

        // === 消息 2+: 每个工具调用作为独立消息 ===
        if let Some(tool_calls) = &gemini_msg.tool_calls {
            for tool_call in tool_calls {
                let mut tool_blocks = Vec::new();
                let mut mentioned_files = Vec::new();

                // Extract file paths from tool call arguments
                Self::extract_file_paths(&tool_call.args, &mut mentioned_files);

                // Generate correlation_id using tool_call.id (deterministic)
                let correlation_id = Some(tool_call.id.clone());

                // Add ToolUse with StandardTool mapping (AC3) and display metadata (AC2)
                let standard_tool = Some(normalize_tool(&tool_call.name, &tool_call.args));
                tool_blocks.push(ContentBlock::ToolUse {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    input: tool_call.args.clone(),
                    correlation_id: correlation_id.clone(),
                    standard_tool,
                    display_name: tool_call.display_name.clone(),
                    description: tool_call.description.clone(),
                });

                // Add ToolResult if available
                if let Some(results) = &tool_call.result {
                    for result_wrapper in results {
                        let response = &result_wrapper.function_response;
                        let is_error = tool_call.status == "error";

                        // Try to parse shell result format (Gemini CLI multi-line format)
                        let (content, structured_result, display_content) = if let Some(parsed) = response.response.parse_shell_result() {
                            // Shell result format detected: extract actual output
                            let shell_output = parsed.output.clone()
                                .or_else(|| parsed.stderr.clone())
                                .unwrap_or_default();

                            // Create structured result for shell command
                            let structured = Some(ToolResultData::ShellExec {
                                exit_code: parsed.exit_code,
                                stdout: parsed.output,
                                stderr: parsed.stderr,
                            });

                            // Use Gemini's resultDisplay if available, otherwise use extracted output
                            let display = tool_call.result_display.clone()
                                .or_else(|| if shell_output.is_empty() { None } else { Some(shell_output.clone()) });

                            (shell_output, structured, display)
                        } else {
                            // Not shell format: use original content
                            let raw_content = response.response.as_content();
                            let cleaned = crate::parsers::strip_system_reminders(&raw_content);
                            (cleaned, None, tool_call.result_display.clone())
                        };

                        // Add ToolResult with display metadata (AC2)
                        tool_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            content,
                            is_error,
                            correlation_id: correlation_id.clone(),
                            structured_result,
                            display_content,
                            render_as_markdown: tool_call.render_output_as_markdown,
                            user_decision: None,
                        });
                    }
                }

                // Parse tool call timestamp if available, fallback to message timestamp
                let tool_timestamp = tool_call.timestamp.as_deref()
                    .and_then(|ts| parse_timestamp(ts).ok())
                    .or(timestamp);

                // Create tool action message
                messages.push(Message {
                    role: role.clone(),
                    content_blocks: tool_blocks,
                    timestamp: tool_timestamp,
                    mentioned_files,
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }
        }

        Ok((messages, unknown_formats))
    }

    /// Extract file paths from tool call arguments
    fn extract_file_paths(args: &serde_json::Value, files: &mut Vec<String>) {
        // Common field names for file paths in Gemini CLI tools
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
            // Handle "paths" array field
            if let Some(paths) = obj.get("paths") {
                if let Some(arr) = paths.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            if !s.is_empty() {
                                files.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }


    /// Parse all sessions from the Gemini CLI directory
    pub fn parse_all(&self) -> Result<Vec<MantraSession>, ParseError> {
        let paths = GeminiPaths::detect()?;
        let sessions_files = paths.scan_all_sessions()?;

        let mut sessions = Vec::new();
        for session_file in sessions_files {
            match self.parse_file(session_file.path.to_string_lossy().as_ref()) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    // Log but continue with other sessions
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

    /// Parse all sessions for a specific project hash
    pub fn parse_project(&self, project_hash: &str) -> Result<Vec<MantraSession>, ParseError> {
        let paths = GeminiPaths::detect()?;
        let session_files = paths.scan_sessions(project_hash)?;

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
}

impl LogParser for GeminiParser {
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError> {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ParseError::invalid_format(format!("File not found: {}", path)));
        }

        let content = fs::read_to_string(path_obj)?;

        self.parse_json(&content, Some(path))
    }

    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError> {
        self.parse_json(content, None)
    }
}

/// Truncate raw JSON to maximum size for storage
fn truncate_raw_json(json: &serde_json::Value) -> String {
    let raw = serde_json::to_string(json).unwrap_or_default();
    if raw.len() <= MAX_RAW_JSON_SIZE {
        raw
    } else {
        format!("{}... [truncated]", &raw[..MAX_RAW_JSON_SIZE])
    }
}

/// Parse an ISO 8601 timestamp string to DateTime<Utc>
fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, ParseError> {
    // Try parsing with different formats
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try alternative format without timezone
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.3f")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            // Try format with Z suffix
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| ParseError::invalid_format(format!("Invalid timestamp '{}': {}", timestamp, e)))
}


#[cfg(test)]
mod tests;
