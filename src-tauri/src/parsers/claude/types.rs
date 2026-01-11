//! Claude-specific type definitions
//!
//! Contains deserialization structures for Claude Code's JSONL and JSON formats.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Claude conversation file structure (legacy JSON format)
#[derive(Debug, Deserialize)]
pub struct ClaudeConversation {
    /// Unique conversation ID
    pub id: String,

    /// Working directory (optional)
    #[serde(default)]
    pub cwd: Option<String>,

    /// Conversation creation time (optional)
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update time (optional)
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Model name (optional)
    #[serde(default)]
    pub model: Option<String>,

    /// Conversation title (optional)
    #[serde(default)]
    pub title: Option<String>,

    /// Messages in the conversation
    #[serde(default)]
    pub messages: Vec<ClaudeMessage>,
}

/// Claude message structure
#[derive(Debug, Deserialize)]
pub struct ClaudeMessage {
    /// Message role (user or assistant)
    pub role: String,

    /// Message content (can be string or array)
    pub content: ClaudeContent,

    /// Message timestamp (optional)
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Claude content can be either a simple string or an array of content blocks
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClaudeContent {
    /// Simple text content
    Text(String),
    /// Array of content blocks
    Blocks(Vec<ClaudeContentBlock>),
}

/// Individual content block in Claude format
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeContentBlock {
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

    /// Image content (base64 encoded)
    Image {
        source: ClaudeImageSource,
    },
}

/// Claude image source structure
/// Contains the base64-encoded image data and its MIME type
#[derive(Debug, Deserialize)]
pub struct ClaudeImageSource {
    /// MIME type of the image (e.g., "image/png", "image/jpeg")
    pub media_type: String,
    /// Base64-encoded image data
    pub data: String,
    /// Source type (always "base64" for Claude)
    #[serde(default, rename = "type")]
    pub source_type: Option<String>,
}

/// Tool result content can be string or structured
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClaudeToolResultContent {
    Text(String),
    Structured(serde_json::Value),
}

impl ClaudeToolResultContent {
    pub fn as_string(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Structured(v) => v.to_string(),
        }
    }
}
