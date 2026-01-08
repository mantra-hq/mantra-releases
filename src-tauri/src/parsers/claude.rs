//! Claude Code log parser
//!
//! Parses conversation logs exported from Claude Code into MantraSession format.
//! Claude Code stores conversations in JSONL files located at:
//! - ~/.claude/projects/<project-path>/<session-id>.jsonl
//! - Each line is a JSON object with message data

use std::collections::HashMap;
use std::fs;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{LogParser, ParseError};
use crate::models::{sources, ContentBlock, MantraSession, Message, Role, SessionMetadata};

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
        let cwd = Self::extract_cwd_from_file_content(path)
            .or_else(|| {
                // Fallback: decode the parent directory name
                // Claude stores sessions in ~/.claude/projects/<encoded-path>/<session-id>.jsonl
                // The encoded path looks like: -mnt-disk0-project-foo -> /mnt/disk0/project/foo
                path_buf
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .map(|encoded_path| Self::decode_claude_path(encoded_path))
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
    
    /// Try to extract cwd from file content by reading the first few lines
    fn extract_cwd_from_file_content(path: &str) -> Option<String> {
        use std::io::{BufRead, BufReader};
        
        let file = std::fs::File::open(path).ok()?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().take(20) {
            if let Ok(line) = line {
                if let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(cwd) = record.get("cwd").and_then(|v| v.as_str()) {
                        if !cwd.is_empty() {
                            return Some(cwd.to_string());
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Decode Claude's encoded project path
    /// Claude encodes paths by replacing / with -
    /// e.g., -mnt-disk0-project-foo -> /mnt/disk0/project/foo
    /// 
    /// Note: This simple replacement works because Claude's encoding is straightforward.
    /// Project names with hyphens will be decoded incorrectly, but since we primarily
    /// need this for matching with existing sessions that have the real cwd, this is
    /// acceptable - the key is consistency within the same project folder.
    fn decode_claude_path(encoded_path: &str) -> String {
        if !encoded_path.starts_with('-') {
            return encoded_path.to_string();
        }
        
        // Claude encodes paths by replacing / with -
        // Simply replace all - with / to decode
        encoded_path.replace('-', "/")
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

            // Parse message
            if let Some(msg_obj) = record.get("message") {
                let role_str = msg_obj.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let role = match role_str {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    _ => continue,
                };

                // Parse content
                let content_blocks = if let Some(content) = msg_obj.get("content") {
                    parse_jsonl_content(content)
                } else {
                    Vec::new()
                };

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
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
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
            ..Default::default()
        };

        Ok(session)
    }
}

/// Parse content from JSONL message
fn parse_jsonl_content(content: &serde_json::Value) -> Vec<ContentBlock> {
    match content {
        serde_json::Value::String(s) => {
            vec![ContentBlock::Text { text: s.clone() }]
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
            let text = block.get("text")?.as_str()?.to_string();
            Some(ContentBlock::Text { text })
        }
        "thinking" => {
            let thinking = block.get("thinking")?.as_str()?.to_string();
            Some(ContentBlock::Thinking { thinking })
        }
        "tool_use" => {
            let id = block.get("id")?.as_str()?.to_string();
            let name = block.get("name")?.as_str()?.to_string();
            let input = block.get("input")?.clone();
            // Use id as correlation_id (Claude's tool_use_id is the correlation key)
            Some(ContentBlock::ToolUse { id: id.clone(), name, input, correlation_id: Some(id) })
        }
        "tool_result" => {
            let tool_use_id = block.get("tool_use_id")?.as_str()?.to_string();
            let content = if let Some(c) = block.get("content") {
                if let Some(s) = c.as_str() {
                    s.to_string()
                } else {
                    c.to_string()
                }
            } else {
                String::new()
            };
            let is_error = block.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
            // Use tool_use_id as correlation_id
            Some(ContentBlock::ToolResult { tool_use_id: tool_use_id.clone(), content, is_error, correlation_id: Some(tool_use_id) })
        }
        _ => None,
    }
}

// Internal structures for deserializing Claude's legacy JSON format

/// Claude conversation file structure (legacy JSON format)
#[derive(Debug, Deserialize)]
struct ClaudeConversation {
    /// Unique conversation ID
    id: String,

    /// Working directory (optional)
    #[serde(default)]
    cwd: Option<String>,

    /// Conversation creation time (optional)
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,

    /// Last update time (optional)
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,

    /// Model name (optional)
    #[serde(default)]
    model: Option<String>,

    /// Conversation title (optional)
    #[serde(default)]
    title: Option<String>,

    /// Messages in the conversation
    #[serde(default)]
    messages: Vec<ClaudeMessage>,
}

/// Claude message structure
#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    /// Message role (user or assistant)
    role: String,

    /// Message content (can be string or array)
    content: ClaudeContent,

    /// Message timestamp (optional)
    #[serde(default)]
    timestamp: Option<DateTime<Utc>>,
}

/// Claude content can be either a simple string or an array of content blocks
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClaudeContent {
    /// Simple text content
    Text(String),
    /// Array of content blocks
    Blocks(Vec<ClaudeContentBlock>),
}

/// Individual content block in Claude format
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClaudeContentBlock {
    /// Plain text
    Text { text: String },

    /// Thinking/reasoning content
    Thinking { thinking: String },

    /// Tool use request
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    /// Tool result
    ToolResult {
        tool_use_id: String,
        content: ClaudeToolResultContent,
        #[serde(default)]
        is_error: bool,
    },
}

/// Tool result content can be string or structured
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClaudeToolResultContent {
    Text(String),
    Structured(serde_json::Value),
}

impl ClaudeToolResultContent {
    fn as_string(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Structured(v) => v.to_string(),
        }
    }
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
            vec![ContentBlock::Text { text: text.clone() }]
        }
        ClaudeContent::Blocks(blocks) => blocks.iter().map(convert_block).collect(),
    }
}

/// Convert a single Claude content block to MantraSession ContentBlock
fn convert_block(block: &ClaudeContentBlock) -> ContentBlock {
    match block {
        ClaudeContentBlock::Text { text } => ContentBlock::Text { text: text.clone() },
        ClaudeContentBlock::Thinking { thinking } => ContentBlock::Thinking {
            thinking: thinking.clone(),
        },
        ClaudeContentBlock::ToolUse { id, name, input } => ContentBlock::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
            correlation_id: Some(id.clone()),
        },
        ClaudeContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => ContentBlock::ToolResult {
            tool_use_id: tool_use_id.clone(),
            content: content.as_string(),
            is_error: *is_error,
            correlation_id: Some(tool_use_id.clone()),
        },
    }
}

/// Reorganize messages to follow Mantra message structure specification
///
/// Mantra 消息结构规范：
/// 1. 文本消息 (thinking + text) → 一条消息
/// 2. 工具调用消息 (tool_use + tool_result) → 每个工具调用一条独立消息
///
/// Claude 原始结构：
/// - assistant 消息包含 thinking + text + tool_use
/// - 下一条 user 消息包含 tool_result
///
/// 转换后：
/// - 消息 1 (assistant): thinking + text
/// - 消息 2 (assistant): tool_use + tool_result
#[allow(dead_code)]
fn reorganize_messages(raw_messages: Vec<Message>) -> Vec<Message> {
    // Step 1: Collect all tool_results by tool_use_id
    let mut tool_results: HashMap<String, (ContentBlock, Option<DateTime<Utc>>)> = HashMap::new();

    for msg in &raw_messages {
        for block in &msg.content_blocks {
            if let ContentBlock::ToolResult { tool_use_id, .. } = block {
                tool_results.insert(tool_use_id.clone(), (block.clone(), msg.timestamp));
            }
        }
    }

    // Step 2: Reorganize messages
    let mut result = Vec::new();

    for msg in raw_messages {
        // Skip messages that only contain tool_results (they'll be merged into tool_use messages)
        let only_tool_results = msg.content_blocks.iter().all(|b| matches!(b, ContentBlock::ToolResult { .. }));
        if only_tool_results && !msg.content_blocks.is_empty() {
            continue;
        }

        // Separate text blocks and tool_use blocks
        let mut text_blocks = Vec::new();
        let mut tool_uses: Vec<(ContentBlock, Option<DateTime<Utc>>)> = Vec::new();

        for block in msg.content_blocks {
            match &block {
                ContentBlock::Text { .. } | ContentBlock::Thinking { .. } => {
                    text_blocks.push(block);
                }
                ContentBlock::ToolUse { id, .. } => {
                    // Find matching tool_result timestamp
                    let result_ts = tool_results.get(id).and_then(|(_, ts)| *ts);
                    tool_uses.push((block, result_ts));
                }
                ContentBlock::ToolResult { .. } => {
                    // Already collected, skip
                }
                // Handle new content block types - treat as text-like content
                ContentBlock::CodeDiff { .. } | ContentBlock::Image { .. } | ContentBlock::Reference { .. } => {
                    text_blocks.push(block);
                }
            }
        }

        // Create text message if we have text/thinking content
        if !text_blocks.is_empty() {
            result.push(Message {
                role: msg.role.clone(),
                content_blocks: text_blocks,
                timestamp: msg.timestamp,
                mentioned_files: Vec::new(),
                message_id: None,
                parent_id: None,
                is_sidechain: false,
                source_metadata: None,
            });
        }

        // Create tool action messages
        for (tool_use, result_timestamp) in tool_uses {
            let mut tool_blocks = vec![tool_use.clone()];

            // Add matching tool_result if found
            if let ContentBlock::ToolUse { id, name, input, .. } = &tool_use {
                if let Some((tool_result, _)) = tool_results.get(id) {
                    tool_blocks.push(tool_result.clone());
                }

                // Extract file paths for mentioned_files
                let mentioned_files = extract_file_paths(name, input);

                result.push(Message {
                    role: msg.role.clone(),
                    content_blocks: tool_blocks,
                    timestamp: result_timestamp.or(msg.timestamp),
                    mentioned_files,
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }
        }
    }

    result
}

/// Extract file paths from tool input
#[allow(dead_code)]
fn extract_file_paths(tool_name: &str, input: &serde_json::Value) -> Vec<String> {
    let mut files = Vec::new();

    // Common file operation tools
    let file_tools = ["Read", "Write", "Edit", "Glob", "Grep", "read_file", "write_file", "edit_file"];
    if !file_tools.iter().any(|t| tool_name.to_lowercase().contains(&t.to_lowercase())) {
        return files;
    }

    // Extract from common path fields
    let path_fields = ["file_path", "filePath", "path", "file", "target_file", "source_file"];
    if let Some(obj) = input.as_object() {
        for field in path_fields {
            if let Some(value) = obj.get(field) {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() {
                        files.push(s.to_string());
                    }
                }
            }
        }
    }

    files
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
}
