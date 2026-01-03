//! Session data models for Mantra
//!
//! Defines the MantraSession structure and related types for representing
//! AI conversation sessions from various AI coding tools.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session source is now a String for unlimited extensibility.
/// Use constants from the `sources` module for known sources.
pub type SessionSource = String;

/// Known session source constants
pub mod sources {
    /// Claude Code sessions
    pub const CLAUDE: &str = "claude";
    /// Gemini CLI sessions
    pub const GEMINI: &str = "gemini";
    /// Cursor IDE sessions
    pub const CURSOR: &str = "cursor";
    /// GitHub Copilot sessions
    pub const COPILOT: &str = "copilot";
    /// Aider sessions
    pub const AIDER: &str = "aider";
    /// Unknown/unrecognized source
    pub const UNKNOWN: &str = "unknown";
}

/// Role in the conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

/// Content block types in a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text { text: String },

    /// AI thinking/reasoning content (extended thinking)
    Thinking { thinking: String },

    /// Tool usage request from AI
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        /// Unified correlation ID for pairing with ToolResult
        #[serde(skip_serializing_if = "Option::is_none")]
        correlation_id: Option<String>,
    },

    /// Result from tool execution
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
        /// Unified correlation ID for pairing with ToolUse
        #[serde(skip_serializing_if = "Option::is_none")]
        correlation_id: Option<String>,
    },

    /// Code diff content (new code changes)
    CodeDiff {
        /// File path the diff applies to
        file_path: String,
        /// Diff content (unified diff format or similar)
        diff: String,
        /// Programming language
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    /// Image content
    Image {
        /// Image data URL or path
        source: String,
        /// MIME type (e.g., "image/png", "image/jpeg")
        #[serde(skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
        /// Alt text or description
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },

    /// Code reference (file snippet or symbol reference)
    Reference {
        /// File path
        file_path: String,
        /// Start line (1-indexed)
        #[serde(skip_serializing_if = "Option::is_none")]
        start_line: Option<u32>,
        /// End line (1-indexed, inclusive)
        #[serde(skip_serializing_if = "Option::is_none")]
        end_line: Option<u32>,
        /// Referenced content snippet
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Symbol name (function, class, etc.)
        #[serde(skip_serializing_if = "Option::is_none")]
        symbol: Option<String>,
    },
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// Role of the message sender
    pub role: Role,

    /// Content blocks in this message
    pub content_blocks: Vec<ContentBlock>,

    /// Timestamp of the message (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    /// Mentioned files in this message (extracted from context)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentioned_files: Vec<String>,
}

/// Optional metadata for a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionMetadata {
    /// Model name used in the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Total tokens used (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,

    /// Session title (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Original file path (for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_path: Option<String>,
}

/// A complete conversation session from an AI tool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MantraSession {
    /// Unique session identifier
    pub id: String,

    /// Source of this session (Claude, Gemini, Cursor)
    pub source: SessionSource,

    /// Working directory where the session was created
    pub cwd: String,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session was last updated
    pub updated_at: DateTime<Utc>,

    /// Messages in this session
    pub messages: Vec<Message>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: SessionMetadata,
}

impl MantraSession {
    /// Create a new empty session
    pub fn new(id: String, source: SessionSource, cwd: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            source,
            cwd,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            metadata: SessionMetadata::default(),
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content_blocks: Vec<ContentBlock>) -> Self {
        Self {
            role,
            content_blocks,
            timestamp: Some(Utc::now()),
            mentioned_files: Vec::new(),
        }
    }

    /// Create a new message with mentioned files
    pub fn with_mentioned_files(role: Role, content_blocks: Vec<ContentBlock>, mentioned_files: Vec<String>) -> Self {
        Self {
            role,
            content_blocks,
            timestamp: Some(Utc::now()),
            mentioned_files,
        }
    }

    /// Create a text-only message
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self::new(role, vec![ContentBlock::Text { text: text.into() }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_source_serialization() {
        let source: SessionSource = sources::CLAUDE.to_string();
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#""claude""#);

        let deserialized: SessionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, sources::CLAUDE);
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""user""#);

        let role = Role::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""assistant""#);
    }

    #[test]
    fn test_content_block_text_serialization() {
        let block = ContentBlock::Text {
            text: "Hello world".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""text":"Hello world""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_content_block_thinking_serialization() {
        let block = ContentBlock::Thinking {
            thinking: "Let me think...".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"thinking""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_content_block_tool_use_serialization() {
        let block = ContentBlock::ToolUse {
            id: "tool_123".to_string(),
            name: "read_file".to_string(),
            input: serde_json::json!({"path": "/tmp/test.txt"}),
            correlation_id: Some("corr_123".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"tool_use""#));
        assert!(json.contains(r#""id":"tool_123""#));
        assert!(json.contains(r#""name":"read_file""#));
        assert!(json.contains(r#""correlation_id":"corr_123""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_content_block_tool_result_serialization() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: "File content here".to_string(),
            is_error: false,
            correlation_id: Some("corr_123".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"tool_result""#));
        assert!(json.contains(r#""tool_use_id":"tool_123""#));
        assert!(json.contains(r#""correlation_id":"corr_123""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_message_serialization() {
        let message = Message::text(Role::User, "Hello");
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""role":"user""#));
        assert!(json.contains("content_blocks"));

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, Role::User);
        assert_eq!(deserialized.content_blocks.len(), 1);
    }

    #[test]
    fn test_mantra_session_serialization() {
        let session = MantraSession::new(
            "session_123".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/project".to_string(),
        );
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains(r#""id":"session_123""#));
        assert!(json.contains(r#""source":"claude""#));
        assert!(json.contains(r#""cwd":"/home/user/project""#));

        let deserialized: MantraSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "session_123");
        assert_eq!(deserialized.source, sources::CLAUDE);
    }

    #[test]
    fn test_session_add_message() {
        let mut session = MantraSession::new(
            "test".to_string(),
            sources::CLAUDE.to_string(),
            "/tmp".to_string(),
        );
        assert_eq!(session.messages.len(), 0);

        session.add_message(Message::text(Role::User, "Hello"));
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_full_session_roundtrip() {
        let mut session = MantraSession::new(
            "test_session".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/project".to_string(),
        );

        // Add user message
        session.add_message(Message::new(
            Role::User,
            vec![ContentBlock::Text {
                text: "Please help me write code".to_string(),
            }],
        ));

        // Add assistant message with multiple content blocks
        session.add_message(Message::new(
            Role::Assistant,
            vec![
                ContentBlock::Thinking {
                    thinking: "The user wants code help...".to_string(),
                },
                ContentBlock::Text {
                    text: "I'll help you write the code.".to_string(),
                },
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "write_file".to_string(),
                    input: serde_json::json!({"path": "main.rs", "content": "fn main() {}"}),
                    correlation_id: Some("tool_1".to_string()),
                },
            ],
        ));

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&session).unwrap();
        let deserialized: MantraSession = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.messages.len(), 2);
        assert_eq!(deserialized.messages[1].content_blocks.len(), 3);
    }
}
