//! Claude Code log parser
//!
//! Parses conversation logs exported from Claude Code into MantraSession format.
//! Claude Code stores conversations in JSON files typically located at:
//! - macOS: ~/.claude/conversations/
//! - The structure contains conversation metadata and message arrays

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
}

// Internal structures for deserializing Claude's JSON format

/// Claude conversation file structure
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
        self.parse_string(&content)
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
