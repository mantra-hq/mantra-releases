//! Claude Code log parser
//!
//! Parses conversation logs exported from Claude Code into MantraSession format.
//! Claude Code stores conversations in JSONL files located at:
//! - ~/.claude/projects/<project-path>/<session-id>.jsonl
//! - Each line is a JSON object with message data

use std::fs;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{LogParser, ParseError};
use crate::models::{ContentBlock, MantraSession, Message, Role, SessionMetadata, SessionSource};

/// Parser for Claude Code conversation logs
#[derive(Debug, Default)]
pub struct ClaudeParser;

impl ClaudeParser {
    /// Create a new ClaudeParser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse JSONL format (one JSON object per line)
    fn parse_jsonl(&self, content: &str) -> Result<MantraSession, ParseError> {
        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut messages: Vec<Message> = Vec::new();
        let mut first_timestamp: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut version: Option<String> = None;

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

            // Skip non-message records (file-history-snapshot, etc.)
            let record_type = record.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if record_type != "user" && record_type != "assistant" {
                continue;
            }

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
                });
            }
        }

        // Validate we got a session ID
        let id = session_id.ok_or_else(|| ParseError::missing_field("sessionId"))?;

        // Build the session
        let mut session = MantraSession::new(
            id.clone(),
            SessionSource::Claude,
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
            title: None,
            total_tokens: None,
            original_path: None,
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
            Some(ContentBlock::ToolUse { id, name, input })
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
            Some(ContentBlock::ToolResult { tool_use_id, content, is_error })
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
        
        // Detect format: JSONL (lines) vs JSON (single object)
        // JSONL files typically start with { on first line and have multiple lines
        let first_line = content.lines().next().unwrap_or("").trim();
        let is_jsonl = first_line.starts_with('{') && content.lines().count() > 1;
        
        if is_jsonl {
            self.parse_jsonl(&content)
        } else {
            self.parse_string(&content)
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
            SessionSource::Claude,
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
        },
        ClaudeContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => ContentBlock::ToolResult {
            tool_use_id: tool_use_id.clone(),
            content: content.as_string(),
            is_error: *is_error,
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
        assert_eq!(session.source, SessionSource::Claude);
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
        assert_eq!(session.messages.len(), 3);

        // Check assistant message has multiple blocks
        assert_eq!(session.messages[1].content_blocks.len(), 3);

        // Verify thinking block
        match &session.messages[1].content_blocks[0] {
            ContentBlock::Thinking { thinking } => {
                assert!(thinking.contains("user wants me to read"));
            }
            _ => panic!("Expected Thinking block"),
        }

        // Verify tool use block
        match &session.messages[1].content_blocks[2] {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "tool_1");
                assert_eq!(name, "read_file");
                assert_eq!(input["path"], "main.rs");
            }
            _ => panic!("Expected ToolUse block"),
        }

        // Verify tool result
        match &session.messages[2].content_blocks[0] {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => {
                assert_eq!(tool_use_id, "tool_1");
                assert_eq!(content, "fn main() {}");
                assert!(!is_error);
            }
            _ => panic!("Expected ToolResult block"),
        }
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
}
