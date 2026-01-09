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
use crate::models::{sources, ContentBlock, GitInfo, MantraSession, Message, Role, SessionMetadata, ToolResultData, normalize_tool};

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
                    .map(|encoded_path| decode_claude_path(encoded_path))
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

                // Parse content
                let mut content_blocks = if let Some(content) = msg_obj.get("content") {
                    parse_jsonl_content(content)
                } else {
                    Vec::new()
                };

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
            model: version, // Use version as model info for now
            title: summary, // Use summary from summary record as title
            total_tokens: None,
            original_path: None,
            git: git_branch.map(|branch| GitInfo {
                branch: Some(branch),
                commit: None,
                repository_url: None,
            }),
            ..Default::default()
        };

        Ok(session)
    }
}

/// Parse content from JSONL message
fn parse_jsonl_content(content: &serde_json::Value) -> Vec<ContentBlock> {
    match content {
        serde_json::Value::String(s) => {
            // Strip system reminder tags from text content
            let cleaned = super::strip_system_reminders(s);
            if cleaned.is_empty() {
                Vec::new()
            } else {
                vec![ContentBlock::Text { text: cleaned }]
            }
        }
        serde_json::Value::Array(arr) => {
            arr.iter().filter_map(parse_jsonl_content_block).collect()
        }
        _ => Vec::new(),
    }
}

/// Parse a single content block from JSONL
fn parse_jsonl_content_block(block: &serde_json::Value) -> Option<ContentBlock> {
    let block_type = block.get("type")?.as_str()?;

    match block_type {
        "text" => {
            let raw_text = block.get("text")?.as_str()?;
            // Strip system reminder tags from text content
            let text = super::strip_system_reminders(raw_text);
            if text.is_empty() {
                None
            } else {
                Some(ContentBlock::Text { text })
            }
        }
        "thinking" => {
            let thinking = block.get("thinking")?.as_str()?.to_string();
            Some(ContentBlock::Thinking { thinking, subject: None, timestamp: None })
        }
        "tool_use" => {
            let id = block.get("id")?.as_str()?.to_string();
            let name = block.get("name")?.as_str()?.to_string();
            let input = block.get("input")?.clone();
            // Call normalize_tool() to get standardized tool type (AC3)
            let standard_tool = Some(normalize_tool(&name, &input));
            // Use id as correlation_id (Claude's tool_use_id is the correlation key)
            Some(ContentBlock::ToolUse { id: id.clone(), name, input, correlation_id: Some(id), standard_tool, display_name: None, description: None })
        }
        "tool_result" => {
            let tool_use_id = block.get("tool_use_id")?.as_str()?.to_string();
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
            Some(ContentBlock::ToolResult {
                tool_use_id: tool_use_id.clone(),
                content,
                is_error,
                correlation_id: Some(tool_use_id),
                structured_result: None,
                display_content: None,
                render_as_markdown: None,
                user_decision: None,
            })
        }
        _ => None,
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
                vec![ContentBlock::Text { text: cleaned }]
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
                Some(ContentBlock::Text { text: cleaned })
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
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_CONVERSATION: &str = r#"{
        "id": "conv_123",
        "cwd": "/home/user/project",
        "messages": [
            {
                "role": "user",
                "content": "Hello, please help me with my code."
            },
            {
                "role": "assistant",
                "content": "Of course! I'd be happy to help. What do you need?"
            }
        ]
    }"#;

    const CONVERSATION_WITH_BLOCKS: &str = r#"{
        "id": "conv_456",
        "cwd": "/tmp/test",
        "model": "claude-3-opus",
        "title": "Code Help Session",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Please read this file"}
                ]
            },
            {
                "role": "assistant",
                "content": [
                    {"type": "thinking", "thinking": "The user wants me to read a file..."},
                    {"type": "text", "text": "I'll read the file for you."},
                    {"type": "tool_use", "id": "tool_1", "name": "read_file", "input": {"path": "main.rs"}}
                ]
            },
            {
                "role": "user",
                "content": [
                    {"type": "tool_result", "tool_use_id": "tool_1", "content": "fn main() {}", "is_error": false}
                ]
            }
        ]
    }"#;

    #[test]
    fn test_parse_simple_conversation() {
        let parser = ClaudeParser::new();
        let result = parser.parse_string(SIMPLE_CONVERSATION);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "conv_123");
        assert_eq!(session.source, sources::CLAUDE);
        assert_eq!(session.cwd, "/home/user/project");
        assert_eq!(session.messages.len(), 2);

        // Check first message
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[0].content_blocks.len(), 1);

        // Check second message
        assert_eq!(session.messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_parse_conversation_with_blocks() {
        let parser = ClaudeParser::new();
        let result = parser.parse_string(CONVERSATION_WITH_BLOCKS);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "conv_456");
        assert_eq!(session.metadata.model, Some("claude-3-opus".to_string()));
        assert_eq!(
            session.metadata.title,
            Some("Code Help Session".to_string())
        );

        // 新结构：user + assistant text + assistant tool_action
        // 原始 tool_result user 消息被合并到工具调用消息中
        assert_eq!(session.messages.len(), 3);

        // 消息 0: user
        assert_eq!(session.messages[0].role, Role::User);

        // 消息 1: assistant (包含 thinking + text + tool_use 或单独的消息)
        // 由于消息结构可能变化，检查是否包含预期内容
        let assistant_msgs: Vec<_> = session.messages.iter().filter(|m| m.role == Role::Assistant).collect();
        assert!(assistant_msgs.len() >= 1);
        
        // 检查是否存在 thinking block
        let has_thinking = session.messages.iter().any(|m| {
            m.content_blocks.iter().any(|b| matches!(b, ContentBlock::Thinking { .. }))
        });
        assert!(has_thinking, "Should have thinking block");

        // 检查是否存在 text block
        let has_text = session.messages.iter().any(|m| {
            m.content_blocks.iter().any(|b| matches!(b, ContentBlock::Text { .. }))
        });
        assert!(has_text, "Should have text block");

        // 检查是否存在 tool_use block
        let has_tool_use = session.messages.iter().any(|m| {
            m.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }))
        });
        assert!(has_tool_use, "Should have tool_use block");

        // 检查是否存在 tool_result block
        let has_tool_result = session.messages.iter().any(|m| {
            m.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolResult { .. }))
        });
        assert!(has_tool_result, "Should have tool_result block");
    }

    #[test]
    fn test_parse_empty_id_fails() {
        let parser = ClaudeParser::new();
        let json = r#"{"id": "", "messages": []}"#;
        let result = parser.parse_string(json);
        assert!(matches!(result, Err(ParseError::MissingField(_))));
    }

    #[test]
    fn test_parse_invalid_json_fails() {
        let parser = ClaudeParser::new();
        let result = parser.parse_string("{ invalid json }");
        assert!(matches!(result, Err(ParseError::InvalidJson(_))));
    }

    #[test]
    fn test_parse_missing_messages_ok() {
        let parser = ClaudeParser::new();
        let json = r#"{"id": "test_123"}"#;
        let result = parser.parse_string(json);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "test_123");
        assert_eq!(session.messages.len(), 0);
    }

    #[test]
    fn test_unknown_role_skipped() {
        let parser = ClaudeParser::new();
        let json = r#"{
            "id": "test",
            "messages": [
                {"role": "system", "content": "You are an AI assistant"},
                {"role": "user", "content": "Hello"}
            ]
        }"#;
        let result = parser.parse_string(json);
        assert!(result.is_ok());

        let session = result.unwrap();
        // Only user message should be included, system role is skipped
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::User);
    }

    #[test]
    fn test_parse_jsonl_with_summary() {
        // Simulate Claude Code JSONL format with summary record
        let jsonl = r#"{"type":"summary","summary":"Test Session Title","leafUuid":"abc123"}
{"parentUuid":"root","isSidechain":false,"userType":"external","cwd":"/test/project","sessionId":"sess-001","version":"2.0.76","gitBranch":"","message":{"role":"user","content":"Hello"},"type":"user","uuid":"msg-1","timestamp":"2024-01-01T00:00:00Z"}
{"parentUuid":"msg-1","isSidechain":false,"userType":"external","cwd":"/test/project","sessionId":"sess-001","version":"2.0.76","gitBranch":"","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}]},"type":"assistant","uuid":"msg-2","timestamp":"2024-01-01T00:00:01Z"}"#;

        let parser = ClaudeParser::new();
        let result = parser.parse_jsonl(jsonl);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.id, "sess-001");
        assert_eq!(session.cwd, "/test/project");
        assert_eq!(session.metadata.title, Some("Test Session Title".to_string()));
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_parse_real_problematic_file() {
        let file_path = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-capsule/4fe9325e-4c69-4633-ac6f-d879ca16d6c5.jsonl";

        let content = std::fs::read_to_string(file_path).expect("Failed to read file");
        println!("\n=== DEBUG: File Info ===");
        println!("Content length: {} bytes", content.len());
        println!("Lines: {}", content.lines().count());

        // 使用 parse_file 而不是 parse_string（这是实际导入流程使用的方法）
        let parser = ClaudeParser::new();
        let result = parser.parse_file(file_path);

        match result {
            Ok(session) => {
                println!("\n=== DEBUG: Parse Result ===");
                println!("Session ID: {}", session.id);
                println!("Messages: {}", session.messages.len());

                for (i, msg) in session.messages.iter().enumerate() {
                    let block_types: Vec<&str> = msg.content_blocks.iter().map(|b| {
                        match b {
                            ContentBlock::Text { .. } => "text",
                            ContentBlock::Thinking { .. } => "thinking",
                            ContentBlock::ToolUse { .. } => "tool_use",
                            ContentBlock::ToolResult { .. } => "tool_result",
                            _ => "other",
                        }
                    }).collect();
                    println!("  Msg {}: {:?} - {:?}", i + 1, msg.role, block_types);
                }

                // 期望 12 条消息
                assert!(session.messages.len() >= 10, "Expected at least 10 messages, got {}", session.messages.len());
            }
            Err(e) => {
                panic!("Parse failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_parse_empty_session_files() {
        let parser = ClaudeParser::new();

        // Test file with only file-history-snapshot records
        // Story 2.29 V2: Returns empty session (not error) for system-events-only files
        let file1 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/1239d15e-5b17-4607-961f-ba103d232021.jsonl";
        if std::path::Path::new(file1).exists() {
            let result = parser.parse_file(file1);
            println!("\nFile 1 (file-history-snapshot only):");
            println!("  Result: {:?}", result);
            // Story 2.29 V2: Returns empty session instead of error
            assert!(result.is_ok(), "Should return empty session for file-history-snapshot only file");
            if let Ok(session) = result {
                assert!(session.messages.is_empty(), "Session should have no messages");
                assert!(session.is_empty(), "Session should be marked as empty");
            }
        }

        // Test file with only summary record
        // Story 2.29 V2: Returns empty session (not error) for summary-only files
        let file2 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/b7485bbe-3a7d-460c-8452-54ec4ce4a3a5.jsonl";
        if std::path::Path::new(file2).exists() {
            let result = parser.parse_file(file2);
            println!("\nFile 2 (summary only):");
            println!("  Result: {:?}", result);
            // Story 2.29 V2: Returns empty session instead of error
            assert!(result.is_ok(), "Should return empty session for summary only file");
            if let Ok(session) = result {
                assert!(session.messages.is_empty(), "Session should have no messages");
                assert!(session.is_empty(), "Session should be marked as empty");
            }
        }

        // Test file with actual conversation
        let file3 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/06e56ded-b41d-4904-9760-f83361dd76ae.jsonl";
        if std::path::Path::new(file3).exists() {
            let result = parser.parse_file(file3);
            println!("\nFile 3 (real conversation):");
            println!("  Result: {:?}", result.as_ref().map(|s| format!("Ok({} messages)", s.messages.len())));
            assert!(result.is_ok(), "Should successfully parse file with real conversation");
            if let Ok(session) = result {
                assert!(!session.messages.is_empty(), "Should have messages");
                println!("  Messages: {}", session.messages.len());
            }
        }
    }

    // ========== Story 8-6: Claude Parser Adaptation Tests ==========

    #[test]
    fn test_parse_jsonl_message_tree_structure() {
        // Test AC1: Message tree structure (uuid, parentUuid, isSidechain)
        let jsonl = r#"{"type":"user","sessionId":"s1","uuid":"msg-001","parentUuid":"msg-000","isSidechain":true,"cwd":"/test","message":{"role":"user","content":"Hello"},"timestamp":"2024-01-01T00:00:00Z"}
{"type":"assistant","sessionId":"s1","uuid":"msg-002","parentUuid":"msg-001","isSidechain":false,"cwd":"/test","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}]},"timestamp":"2024-01-01T00:00:01Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert_eq!(session.messages.len(), 2);

        // Check first message (user)
        assert_eq!(session.messages[0].message_id, Some("msg-001".to_string()));
        assert_eq!(session.messages[0].parent_id, Some("msg-000".to_string()));
        assert!(session.messages[0].is_sidechain);

        // Check second message (assistant)
        assert_eq!(session.messages[1].message_id, Some("msg-002".to_string()));
        assert_eq!(session.messages[1].parent_id, Some("msg-001".to_string()));
        assert!(!session.messages[1].is_sidechain);
    }

    #[test]
    fn test_parse_jsonl_message_tree_backward_compatible() {
        // Test AC5: Backward compatibility - missing tree fields default to None/false
        let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":"Hello"},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].message_id, None);
        assert_eq!(session.messages[0].parent_id, None);
        assert!(!session.messages[0].is_sidechain);
    }

    #[test]
    fn test_parse_jsonl_git_branch() {
        // Test AC2: Git information extraction
        let jsonl = r#"{"type":"user","sessionId":"s1","gitBranch":"feature/test-branch","cwd":"/test","message":{"role":"user","content":"Hi"},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert!(session.metadata.git.is_some());
        let git = session.metadata.git.unwrap();
        assert_eq!(git.branch, Some("feature/test-branch".to_string()));
        assert_eq!(git.commit, None);
        assert_eq!(git.repository_url, None);
    }

    #[test]
    fn test_parse_jsonl_git_branch_empty_ignored() {
        // Test AC5: Empty gitBranch should not create GitInfo
        let jsonl = r#"{"type":"user","sessionId":"s1","gitBranch":"","cwd":"/test","message":{"role":"user","content":"Hi"},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert!(session.metadata.git.is_none());
    }

    #[test]
    fn test_parse_jsonl_standard_tool_read() {
        // Test AC3: StandardTool mapping - Read
        let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/src/main.rs","offset":10,"limit":50}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert_eq!(session.messages.len(), 1);
        if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
            assert!(standard_tool.is_some());
            if let Some(crate::models::StandardTool::FileRead { path, start_line, end_line }) = standard_tool {
                assert_eq!(path, "/src/main.rs");
                assert_eq!(*start_line, Some(10));
                assert_eq!(*end_line, Some(60)); // offset 10 + limit 50
            } else {
                panic!("Expected StandardTool::FileRead");
            }
        } else {
            panic!("Expected ToolUse content block");
        }
    }

    #[test]
    fn test_parse_jsonl_standard_tool_bash() {
        // Test AC3: StandardTool mapping - Bash
        let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls -la","cwd":"/tmp"}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
            if let Some(crate::models::StandardTool::ShellExec { command, cwd }) = standard_tool {
                assert_eq!(command, "ls -la");
                assert_eq!(*cwd, Some("/tmp".to_string()));
            } else {
                panic!("Expected StandardTool::ShellExec");
            }
        }
    }

    #[test]
    fn test_parse_jsonl_standard_tool_glob_grep() {
        // Test AC3: StandardTool mapping - Glob and Grep
        let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Glob","input":{"pattern":"*.rs","path":"/src"}},{"type":"tool_use","id":"t2","name":"Grep","input":{"pattern":"TODO","path":"/project"}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        // Check Glob
        if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
            if let Some(crate::models::StandardTool::FileSearch { pattern, path }) = standard_tool {
                assert_eq!(pattern, "*.rs");
                assert_eq!(*path, Some("/src".to_string()));
            } else {
                panic!("Expected StandardTool::FileSearch");
            }
        }

        // Check Grep
        if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[1] {
            if let Some(crate::models::StandardTool::ContentSearch { pattern, path }) = standard_tool {
                assert_eq!(pattern, "TODO");
                assert_eq!(*path, Some("/project".to_string()));
            } else {
                panic!("Expected StandardTool::ContentSearch");
            }
        }
    }

    #[test]
    fn test_parse_tool_use_result_file_read() {
        // Test AC4: toolUseResult parsing - FileRead
        let tool_use_result = serde_json::json!({
            "file": {
                "filePath": "/src/main.rs",
                "startLine": 1,
                "numLines": 50,
                "totalLines": 100
            }
        });

        let result = parse_tool_use_result(&tool_use_result);
        assert!(result.is_some());

        if let Some(ToolResultData::FileRead { file_path, start_line, num_lines, total_lines }) = result {
            assert_eq!(file_path, "/src/main.rs");
            assert_eq!(start_line, Some(1));
            assert_eq!(num_lines, Some(50));
            assert_eq!(total_lines, Some(100));
        } else {
            panic!("Expected ToolResultData::FileRead");
        }
    }

    #[test]
    fn test_parse_tool_use_result_other() {
        // Test AC4: toolUseResult parsing - Other (passthrough)
        let tool_use_result = serde_json::json!({
            "custom": {
                "some_field": "some_value"
            }
        });

        let result = parse_tool_use_result(&tool_use_result);
        assert!(result.is_some());

        if let Some(ToolResultData::Other { data }) = result {
            assert_eq!(data.get("custom").unwrap().get("some_field").unwrap(), "some_value");
        } else {
            panic!("Expected ToolResultData::Other");
        }
    }

    #[test]
    fn test_parse_tool_use_result_empty() {
        // Test AC4: Empty toolUseResult returns None
        let tool_use_result = serde_json::json!({});
        let result = parse_tool_use_result(&tool_use_result);
        assert!(result.is_none());

        let tool_use_result_null = serde_json::Value::Null;
        let result_null = parse_tool_use_result(&tool_use_result_null);
        assert!(result_null.is_none());
    }

    #[test]
    fn test_parse_jsonl_with_tool_use_result() {
        // Test AC4: toolUseResult integration in JSONL parsing
        let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","toolUseResult":{"file":{"filePath":"/src/lib.rs","startLine":10,"numLines":20,"totalLines":200}},"message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"file content here","is_error":false}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

        let parser = ClaudeParser::new();
        let session = parser.parse_jsonl(jsonl).unwrap();

        assert_eq!(session.messages.len(), 1);
        if let ContentBlock::ToolResult { structured_result, .. } = &session.messages[0].content_blocks[0] {
            assert!(structured_result.is_some());
            if let Some(ToolResultData::FileRead { file_path, start_line, num_lines, total_lines }) = structured_result {
                assert_eq!(file_path, "/src/lib.rs");
                assert_eq!(*start_line, Some(10));
                assert_eq!(*num_lines, Some(20));
                assert_eq!(*total_lines, Some(200));
            } else {
                panic!("Expected ToolResultData::FileRead in structured_result");
            }
        } else {
            panic!("Expected ToolResult content block");
        }
    }

    #[test]
    fn test_convert_block_standard_tool() {
        // Test AC3: StandardTool mapping in legacy JSON format (convert_block)
        let block = ClaudeContentBlock::ToolUse {
            id: "t1".to_string(),
            name: "Write".to_string(),
            input: serde_json::json!({"file_path": "/out.txt", "content": "hello"}),
        };

        let result = convert_block(&block);
        if let Some(ContentBlock::ToolUse { standard_tool, .. }) = result {
            assert!(standard_tool.is_some());
            if let Some(crate::models::StandardTool::FileWrite { path, content }) = standard_tool {
                assert_eq!(path, "/out.txt");
                assert_eq!(content, "hello");
            } else {
                panic!("Expected StandardTool::FileWrite");
            }
        } else {
            panic!("Expected ToolUse content block");
        }
    }

    // Story 8.12: Tests for strip_line_number_prefix (AC5)
    #[test]
    fn test_strip_line_number_prefix_pipe_format() {
        // Test pipe format: "   1|content"
        let input = "   1|fn main() {\n   2|    println!(\"Hello\");\n   3|}";
        let expected = "fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(strip_line_number_prefix(input), expected);
    }

    #[test]
    fn test_strip_line_number_prefix_arrow_format() {
        // Test arrow format: "  42→content"
        let input = "  42→const x = 1;\n  43→const y = 2;";
        let expected = "const x = 1;\nconst y = 2;";
        assert_eq!(strip_line_number_prefix(input), expected);
    }

    #[test]
    fn test_strip_line_number_prefix_unpadded() {
        // Test unpadded numbers
        let input = "1|line one\n2|line two\n10|line ten";
        let expected = "line one\nline two\nline ten";
        assert_eq!(strip_line_number_prefix(input), expected);
    }

    #[test]
    fn test_strip_line_number_prefix_mixed() {
        // Test content without line numbers (should remain unchanged)
        let input = "Hello World\nNo line numbers here";
        assert_eq!(strip_line_number_prefix(input), input);
    }

    #[test]
    fn test_strip_line_number_prefix_empty() {
        // Test empty content
        let input = "";
        assert_eq!(strip_line_number_prefix(input), "");
    }

    #[test]
    fn test_strip_line_number_prefix_preserves_content_with_pipe() {
        // Test that pipes in content are preserved (not line number format)
        let input = "This is a | pipe in text\nAnother line | with pipe";
        // These don't match the pattern (no leading digits), so unchanged
        assert_eq!(strip_line_number_prefix(input), input);
    }

    #[test]
    fn test_strip_line_number_prefix_with_space_after_delimiter() {
        // Test format with space after delimiter: "1| content"
        // The space after the delimiter is preserved (part of code indentation)
        let input = "1| fn main() {";
        let expected = " fn main() {";
        assert_eq!(strip_line_number_prefix(input), expected);
    }
}
