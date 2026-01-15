//! Claude Code log parser
//!
//! Parses conversation logs exported from Claude Code into MantraSession format.
//! Claude Code stores conversations in JSONL files located at:
//! - ~/.claude/projects/<project-path>/<session-id>.jsonl
//! - Each line is a JSON object with message data

mod path;
mod types;

pub use path::{ClaudePaths, ClaudeSessionFile, decode_claude_path, extract_cwd_from_file_content, get_claude_dir, get_claude_projects_dir};
pub use types::{ClaudeConversation, ClaudeMessage, ClaudeContent, ClaudeContentBlock, ClaudeToolResultContent};

use std::fs;

use chrono::{DateTime, Utc};
use regex::Regex;

use super::{LogParser, ParseError};
use crate::models::{sources, ContentBlock, GitInfo, MantraSession, Message, ParserInfo, Role, SessionMetadata, ToolResultData, UnknownFormatEntry, normalize_tool};

/// Claude Parser version for compatibility tracking
pub const CLAUDE_PARSER_VERSION: &str = "1.1.0";

/// Supported content block types in Claude JSONL format
pub const SUPPORTED_CONTENT_TYPES: &[&str] = &["text", "thinking", "tool_use", "tool_result", "image"];

/// Supported message types in Claude JSONL format
pub const SUPPORTED_MESSAGE_TYPES: &[&str] = &["user", "assistant", "summary"];

/// Maximum raw JSON size to store in UnknownFormatEntry (1KB)
const MAX_RAW_JSON_SIZE: usize = 1024;

/// Strip line number prefixes from file read output (Story 8.12: AC5)
///
/// Claude Code's file read tool outputs include line number prefixes like:
/// - "   1|content" (padded number + pipe)
/// - "42|content" (unpadded number + pipe)
/// - "   1→content" (padded number + arrow)
/// - "  42→content" (padded number + arrow)
///
/// This function removes these prefixes while preserving the actual content.
/// Note: The original code indentation (spaces after the delimiter) is preserved.
fn strip_line_number_prefix(content: &str) -> String {
    use once_cell::sync::Lazy;

    // Regex for line number prefix pattern
    // Matches: optional whitespace + digits + (pipe or arrow)
    // Does NOT consume spaces after the delimiter - those are part of the code indentation
    // Examples: "   1|", "42|", "   1→", "  42→"
    static LINE_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\s*\d+[|→]").unwrap()
    });

    content
        .lines()
        .map(|line| {
            LINE_PREFIX_REGEX.replace(line, "").to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parser for Claude Code conversation logs
#[derive(Debug, Default)]
pub struct ClaudeParser;

impl ClaudeParser {
    /// Create a new ClaudeParser instance
    pub fn new() -> Self {
        Self
    }

    /// Create an empty session from file path (Story 2.29 V2)
    /// 
    /// When a file contains only system events or no valid conversation,
    /// we still import it as an empty session with is_empty = true
    fn create_empty_session_from_path(&self, path: &str) -> Result<MantraSession, ParseError> {
        use std::path::Path;
        
        let path_buf = Path::new(path);
        
        // Extract session ID from filename (e.g., "b7485bbe-3a7d-460c-8452-54ec4ce4a3a5.jsonl" -> "b7485bbe-3a7d-460c-8452-54ec4ce4a3a5")
        let session_id = path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("empty-{}", uuid::Uuid::new_v4()));
        
        // Try to extract cwd from file content first (read first few lines)
        // This handles the case where the file has some system events with cwd info
        let cwd = extract_cwd_from_file_content(path)
            .or_else(|| {
                // Fallback: decode the parent directory name
                // Claude stores sessions in ~/.claude/projects/<encoded-path>/<session-id>.jsonl
                // The encoded path looks like: -mnt-disk0-project-foo -> /mnt/disk0/project/foo
                path_buf
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .map(decode_claude_path)
            })
            .unwrap_or_default();
        
        // Create empty session
        let session = MantraSession::new(
            session_id,
            sources::CLAUDE.to_string(),
            cwd,
        );
        
        Ok(session)
    }

    /// Parse JSONL format (one JSON object per line)
    fn parse_jsonl(&self, content: &str) -> Result<MantraSession, ParseError> {
        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut messages: Vec<Message> = Vec::new();
        let mut first_timestamp: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut version: Option<String> = None;
        let mut summary: Option<String> = None;
        let mut git_branch: Option<String> = None;

        // Story 8.15: Collect unknown formats for monitoring
        let mut all_unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

        // Track what types of records we've seen for better error messages
        let mut has_system_events = false;
        let mut has_summary_only = false;
        let mut valid_line_count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse the line as a JSONL record
            let record: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // Skip invalid lines
            };

            valid_line_count += 1;
            let record_type = record.get("type").and_then(|t| t.as_str()).unwrap_or("");

            // Extract summary from summary records (Claude Code stores session title here)
            if record_type == "summary" {
                if let Some(s) = record.get("summary").and_then(|s| s.as_str()) {
                    summary = Some(s.to_string());
                    has_summary_only = true;
                }
                continue;
            }

            // Track system events (file-history-snapshot, etc.)
            if record_type != "user" && record_type != "assistant" {
                has_system_events = true;
                continue;
            }

            // We have a user or assistant message, so it's not summary-only
            has_summary_only = false;

            // Extract session metadata from first valid record
            if session_id.is_none() {
                session_id = record.get("sessionId").and_then(|s| s.as_str()).map(|s| s.to_string());
                cwd = record.get("cwd").and_then(|s| s.as_str()).map(|s| s.to_string());
                version = record.get("version").and_then(|s| s.as_str()).map(|s| s.to_string());
            }

            // Extract git branch from record (AC2)
            // We take the first non-empty gitBranch we encounter
            if git_branch.is_none() {
                if let Some(branch) = record.get("gitBranch").and_then(|v| v.as_str()) {
                    if !branch.is_empty() {
                        git_branch = Some(branch.to_string());
                    }
                }
            }

            // Parse timestamp
            let timestamp = record
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(|t| t.parse::<DateTime<Utc>>().ok());

            if timestamp.is_some() {
                if first_timestamp.is_none() {
                    first_timestamp = timestamp;
                }
                last_timestamp = timestamp;
            }

            // Extract message tree structure fields (AC1)
            let message_uuid = record.get("uuid").and_then(|v| v.as_str()).map(|s| s.to_string());
            let parent_uuid = record.get("parentUuid").and_then(|v| v.as_str()).map(|s| s.to_string());
            let is_sidechain = record.get("isSidechain").and_then(|v| v.as_bool()).unwrap_or(false);

            // Parse message
            if let Some(msg_obj) = record.get("message") {
                let role_str = msg_obj.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let role = match role_str {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    _ => continue,
                };

                // Parse content (Story 8.15: now returns unknown_formats too)
                let (mut content_blocks, unknown_formats) = if let Some(content) = msg_obj.get("content") {
                    parse_jsonl_content(content)
                } else {
                    (Vec::new(), Vec::new())
                };

                // Collect unknown formats
                all_unknown_formats.extend(unknown_formats);

                // Extract and apply toolUseResult if present (AC4)
                // toolUseResult provides structured information about tool execution results
                if let Some(tool_use_result) = record.get("toolUseResult") {
                    if let Some(structured_result) = parse_tool_use_result(tool_use_result) {
                        // Apply to the first ToolResult block (Claude typically has one per record)
                        for block in content_blocks.iter_mut() {
                            if let ContentBlock::ToolResult { structured_result: ref mut sr, .. } = block {
                                *sr = Some(structured_result.clone());
                                break; // Apply to first ToolResult only
                            }
                        }
                    }
                }

                // Skip messages with no content or only meta content
                let is_meta = record.get("isMeta").and_then(|m| m.as_bool()).unwrap_or(false);
                if is_meta || content_blocks.is_empty() {
                    continue;
                }

                messages.push(Message {
                    role,
                    content_blocks,
                    timestamp,
                    mentioned_files: Vec::new(),
                    message_id: message_uuid,
                    parent_id: parent_uuid,
                    is_sidechain,
                    source_metadata: None,
                });
            }
        }

        // Provide better error messages based on what we found
        if session_id.is_none() {
            // No session ID found - determine why
            if valid_line_count == 0 {
                // File has content but no valid JSON lines
                return Err(ParseError::EmptyFile);
            } else if has_summary_only && !has_system_events {
                // Only summary records (e.g., "Invalid API key" messages)
                return Err(ParseError::NoValidConversation);
            } else if has_system_events {
                // Only system events (file-history-snapshot, etc.)
                return Err(ParseError::SystemEventsOnly);
            } else {
                return Err(ParseError::missing_field("sessionId"));
            }
        }

        let id = session_id.unwrap();

        // Build the session
        let mut session = MantraSession::new(
            id.clone(),
            sources::CLAUDE.to_string(),
            cwd.unwrap_or_default(),
        );

        if let Some(ts) = first_timestamp {
            session.created_at = ts;
        }
        if let Some(ts) = last_timestamp {
            session.updated_at = ts;
        }

        session.messages = messages;
        session.metadata = SessionMetadata {
            model: version.clone(), // Use version as model info for now
            title: summary, // Use summary from summary record as title
            total_tokens: None,
            original_path: None,
            git: git_branch.map(|branch| GitInfo {
                branch: Some(branch),
                commit: None,
                repository_url: None,
            }),
            // Story 8.15: Add parser info and unknown formats
            parser_info: Some(ParserInfo {
                parser_version: CLAUDE_PARSER_VERSION.to_string(),
                supported_formats: SUPPORTED_CONTENT_TYPES.iter().map(|s| s.to_string()).collect(),
                detected_source_version: version,
            }),
            unknown_formats: if all_unknown_formats.is_empty() {
                None
            } else {
                Some(all_unknown_formats)
            },
            ..Default::default()
        };

        Ok(session)
    }
}

/// Parse content from JSONL message
/// Returns (content_blocks, unknown_format_entries)
fn parse_jsonl_content(content: &serde_json::Value) -> (Vec<ContentBlock>, Vec<UnknownFormatEntry>) {
    match content {
        serde_json::Value::String(s) => {
            // Strip system reminder tags from text content
            let cleaned = super::strip_system_reminders(s);
            if cleaned.is_empty() {
                (Vec::new(), Vec::new())
            } else {
                (vec![ContentBlock::Text { text: cleaned, is_degraded: None }], Vec::new())
            }
        }
        serde_json::Value::Array(arr) => {
            let mut blocks = Vec::new();
            let mut unknown_formats = Vec::new();
            for item in arr {
                let (block, unknown) = parse_jsonl_content_block(item);
                if let Some(b) = block {
                    blocks.push(b);
                }
                if let Some(u) = unknown {
                    unknown_formats.push(u);
                }
            }
            (blocks, unknown_formats)
        }
        _ => (Vec::new(), Vec::new()),
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

/// Parse a single content block from JSONL
/// Returns (content_block, unknown_format_entry)
/// If block type is unknown, creates degraded Text block and records unknown format
fn parse_jsonl_content_block(block: &serde_json::Value) -> (Option<ContentBlock>, Option<UnknownFormatEntry>) {
    let block_type = match block.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return (None, None), // No type field, skip silently
    };

    match block_type {
        "text" => {
            let raw_text = match block.get("text").and_then(|t| t.as_str()) {
                Some(t) => t,
                None => return (None, None),
            };
            // Strip system reminder tags from text content
            let text = super::strip_system_reminders(raw_text);
            if text.is_empty() {
                (None, None)
            } else {
                (Some(ContentBlock::Text { text, is_degraded: None }), None)
            }
        }
        "thinking" => {
            let thinking = match block.get("thinking").and_then(|t| t.as_str()) {
                Some(t) => t.to_string(),
                None => return (None, None),
            };
            (Some(ContentBlock::Thinking { thinking, subject: None, timestamp: None }), None)
        }
        "tool_use" => {
            let id = match block.get("id").and_then(|i| i.as_str()) {
                Some(i) => i.to_string(),
                None => return (None, None),
            };
            let name = match block.get("name").and_then(|n| n.as_str()) {
                Some(n) => n.to_string(),
                None => return (None, None),
            };
            let input = match block.get("input") {
                Some(i) => i.clone(),
                None => return (None, None),
            };
            // Call normalize_tool() to get standardized tool type (AC3)
            let standard_tool = Some(normalize_tool(&name, &input));
            // Use id as correlation_id (Claude's tool_use_id is the correlation key)
            (Some(ContentBlock::ToolUse {
                id: id.clone(),
                name,
                input,
                correlation_id: Some(id),
                standard_tool,
                display_name: None,
                description: None,
            }), None)
        }
        "tool_result" => {
            let tool_use_id = match block.get("tool_use_id").and_then(|t| t.as_str()) {
                Some(t) => t.to_string(),
                None => return (None, None),
            };
            let raw_content = if let Some(c) = block.get("content") {
                if let Some(s) = c.as_str() {
                    s.to_string()
                } else {
                    c.to_string()
                }
            } else {
                String::new()
            };
            // Story 8.12: Strip line number prefixes from tool result content (AC5)
            // This is applied to all tool results as it's a safe operation
            // (only affects lines matching the line number pattern)
            let stripped = strip_line_number_prefix(&raw_content);
            // Also strip system reminder tags from tool result content
            let content = super::strip_system_reminders(&stripped);
            let is_error = block.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
            // Use tool_use_id as correlation_id
            (Some(ContentBlock::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content,
                is_error,
                correlation_id: Some(tool_use_id),
                structured_result: None,
                display_content: None,
                render_as_markdown: None,
                user_decision: None,
            }), None)
        }
        "image" => {
            // Story 8.16: Parse image content block (AC2)
            // Claude image format: { "type": "image", "source": { "media_type": "image/png", "data": "base64..." } }
            let source = match block.get("source") {
                Some(s) => s,
                None => return (None, None),
            };
            let media_type = match source.get("media_type").and_then(|m| m.as_str()) {
                Some(m) => m.to_string(),
                None => return (None, None),
            };
            let data = match source.get("data").and_then(|d| d.as_str()) {
                Some(d) => d.to_string(),
                None => return (None, None),
            };
            // source_type is always "base64" for Claude
            (Some(ContentBlock::Image {
                media_type,
                data,
                source_type: Some("base64".to_string()),
                alt_text: None,
            }), None)
        }
        unknown_type => {
            // Story 8.15: Unknown format - create degraded Text block
            // Note: Unknown type is recorded in UnknownFormatEntry for monitoring

            // Create degraded Text block with original JSON
            let degraded_text = format!("[无法解析的内容块: {}]\n{}", unknown_type, truncate_raw_json(block));
            let degraded_block = ContentBlock::Text {
                text: degraded_text,
                is_degraded: Some(true),
            };

            // Record unknown format entry
            let unknown_entry = UnknownFormatEntry {
                source: "claude".to_string(),
                type_name: unknown_type.to_string(),
                raw_json: truncate_raw_json(block),
                timestamp: Utc::now().to_rfc3339(),
            };

            (Some(degraded_block), Some(unknown_entry))
        }
    }
}

/// Parse toolUseResult from Claude JSONL record into ToolResultData (AC4)
///
/// Claude provides structured tool result information in the toolUseResult field.
/// This function converts it to our standardized ToolResultData format.
fn parse_tool_use_result(tool_use_result: &serde_json::Value) -> Option<ToolResultData> {
    // Check for file result (AC4: ToolResultData::FileRead)
    if let Some(file) = tool_use_result.get("file") {
        let file_path = file.get("filePath")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let start_line = file.get("startLine").and_then(|v| v.as_u64()).map(|n| n as u32);
        let num_lines = file.get("numLines").and_then(|v| v.as_u64()).map(|n| n as u32);
        let total_lines = file.get("totalLines").and_then(|v| v.as_u64()).map(|n| n as u32);
        
        return Some(ToolResultData::FileRead {
            file_path,
            start_line,
            num_lines,
            total_lines,
        });
    }
    
    // Other results: passthrough as Other (AC4)
    if !tool_use_result.is_null() && !tool_use_result.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return Some(ToolResultData::Other {
            data: tool_use_result.clone(),
        });
    }
    
    None
}

impl LogParser for ClaudeParser {
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError> {
        let content = fs::read_to_string(path)?;

        // Check for empty file
        if content.trim().is_empty() {
            // Story 2.29 V2: Return empty session instead of error
            return self.create_empty_session_from_path(path);
        }

        // Detect format: JSONL vs JSON
        // JSONL: each line is a separate JSON object with "type" field
        // JSON: single object with "id" and "messages" fields
        let first_line = content.lines().next().unwrap_or("").trim();

        // Check if it looks like a JSONL record (has "type" field)
        // This handles both single-line and multi-line JSONL files
        let is_jsonl = first_line.starts_with('{') &&
            (first_line.contains("\"type\"") || content.lines().count() > 1);

        let result = if is_jsonl {
            self.parse_jsonl(&content)
        } else {
            self.parse_string(&content)
        };

        // Story 2.29 V2: Handle skippable errors by returning empty session
        match result {
            Ok(session) => Ok(session),
            Err(e) if e.is_skippable() => {
                // Create an empty session from the file path
                self.create_empty_session_from_path(path)
            }
            Err(e) => Err(e),
        }
    }

    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError> {
        // Try to parse as a single conversation
        let conversation: ClaudeConversation = serde_json::from_str(content)?;

        // Validate required fields
        if conversation.id.is_empty() {
            return Err(ParseError::missing_field("id"));
        }

        // Convert to MantraSession
        let mut session = MantraSession::new(
            conversation.id.clone(),
            sources::CLAUDE.to_string(),
            conversation.cwd.unwrap_or_default(),
        );

        // Set timestamps if available
        if let Some(created) = conversation.created_at {
            session.created_at = created;
        }
        if let Some(updated) = conversation.updated_at {
            session.updated_at = updated;
        }

        // Set metadata
        session.metadata = SessionMetadata {
            model: conversation.model,
            title: conversation.title,
            total_tokens: None,
            original_path: None,
            ..Default::default()
        };

        // Parse messages
        for claude_msg in conversation.messages {
            let role = match claude_msg.role.to_lowercase().as_str() {
                "user" | "human" => Role::User,
                "assistant" | "ai" => Role::Assistant,
                _ => continue, // Skip unknown roles
            };

            let content_blocks = convert_content(&claude_msg.content);

            let message = Message {
                role,
                content_blocks,
                timestamp: claude_msg.timestamp,
                mentioned_files: Vec::new(),
                message_id: None,
                parent_id: None,
                is_sidechain: false,
                source_metadata: None,
            };

            session.messages.push(message);
        }

        Ok(session)
    }
}

/// Convert Claude content to MantraSession content blocks
fn convert_content(content: &ClaudeContent) -> Vec<ContentBlock> {
    match content {
        ClaudeContent::Text(text) => {
            // Strip system reminder tags from text content
            let cleaned = super::strip_system_reminders(text);
            if cleaned.is_empty() {
                vec![]
            } else {
                vec![ContentBlock::Text { text: cleaned, is_degraded: None }]
            }
        }
        ClaudeContent::Blocks(blocks) => blocks.iter().filter_map(convert_block).collect(),
    }
}

/// Convert a single Claude content block to MantraSession ContentBlock
fn convert_block(block: &ClaudeContentBlock) -> Option<ContentBlock> {
    match block {
        ClaudeContentBlock::Text { text } => {
            // Strip system reminder tags from text content
            let cleaned = super::strip_system_reminders(text);
            if cleaned.is_empty() {
                None
            } else {
                Some(ContentBlock::Text { text: cleaned, is_degraded: None })
            }
        }
        ClaudeContentBlock::Thinking { thinking } => Some(ContentBlock::Thinking {
            thinking: thinking.clone(),
            subject: None,
            timestamp: None,
        }),
        ClaudeContentBlock::ToolUse { id, name, input } => Some(ContentBlock::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
            correlation_id: Some(id.clone()),
            standard_tool: Some(normalize_tool(name, input)),
            display_name: None,
            description: None,
        }),
        ClaudeContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            // Story 8.12: Strip line number prefixes from tool result content (AC5)
            let stripped = strip_line_number_prefix(&content.as_string());
            // Also strip system reminder tags from tool result content
            let cleaned_content = super::strip_system_reminders(&stripped);
            Some(ContentBlock::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content: cleaned_content,
                is_error: *is_error,
                correlation_id: Some(tool_use_id.clone()),
                structured_result: None,
                display_content: None,
                render_as_markdown: None,
                user_decision: None,
            })
        },
        ClaudeContentBlock::Image { source } => {
            // Story 8.16: Convert Claude image block (AC2)
            Some(ContentBlock::Image {
                media_type: source.media_type.clone(),
                data: source.data.clone(),
                source_type: Some("base64".to_string()),
                alt_text: None,
            })
        },
    }
}



#[cfg(test)]
mod tests;
