//! Cursor data types and structures
//!
//! Defines types for parsing Cursor's composer data and bubble content
//! from the state.vscdb database.

use serde::Deserialize;

/// Cursor composer metadata from composerData:{id}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorComposer {
    /// Schema version
    #[serde(rename = "_v")]
    pub version: Option<i32>,

    /// Composer unique ID
    pub composer_id: Option<String>,

    /// Conversation headers (bubble references, not full content)
    #[serde(default)]
    pub full_conversation_headers_only: Vec<BubbleHeader>,

    /// Context information (mentioned files, selections)
    #[serde(default)]
    pub context: Option<CursorContext>,

    /// Model configuration
    #[serde(default)]
    pub model: Option<ModelConfig>,

    /// Creation timestamp (epoch milliseconds)
    pub created_at: Option<i64>,

    /// Unified mode (e.g., "agent", "chat")
    pub unified_mode: Option<String>,
}

/// Bubble header reference from fullConversationHeadersOnly
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleHeader {
    /// Bubble unique ID
    pub bubble_id: String,

    /// Message type: 1 = User, 2 = Assistant
    #[serde(rename = "type")]
    pub bubble_type: i32,
}

/// Context information from composer
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorContext {
    /// Mentioned files/symbols (complex nested object, use Value for flexibility)
    #[serde(default)]
    pub mentions: serde_json::Value,

    /// File selections (direct array in context)
    #[serde(default)]
    pub file_selections: Vec<FileSelection>,
}

/// Mentioned file or symbol
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorMention {
    /// Mention type (e.g., "file", "symbol")
    #[serde(rename = "type")]
    pub mention_type: Option<String>,

    /// File path or URI
    pub uri: Option<String>,

    /// Symbol name (for symbol mentions)
    pub name: Option<String>,

    /// Text content
    pub text: Option<String>,
}

/// File selection in context
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSelection {
    /// File URI
    pub uri: Option<String>,

    /// Selection range
    pub range: Option<SelectionRange>,
}

/// Selection range in a file
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRange {
    pub start_line: Option<i32>,
    pub start_column: Option<i32>,
    pub end_line: Option<i32>,
    pub end_column: Option<i32>,
}

/// Model configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    /// Model name
    pub model_name: Option<String>,

    /// Model ID
    pub model_id: Option<String>,

    /// Provider (e.g., "anthropic", "openai")
    pub provider: Option<String>,
}

/// Bubble content from bubbleId:{composerId}:{bubbleId}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorBubble {
    /// Schema version
    #[serde(rename = "_v")]
    pub version: Option<i32>,

    /// Bubble unique ID
    pub bubble_id: Option<String>,

    /// Message type: 1 = User, 2 = Assistant
    #[serde(rename = "type")]
    pub bubble_type: i32,

    /// Plain text content
    pub text: Option<String>,

    /// Rich text content (Lexical format)
    pub rich_text: Option<serde_json::Value>,

    /// Whether this is an agentic message
    #[serde(default)]
    pub is_agentic: bool,

    /// Tool results from this message (legacy, usually empty)
    #[serde(default)]
    pub tool_results: Vec<ToolResult>,

    /// Tool call data - PRIMARY field for tool interactions
    /// Contains actual tool call data (name, rawArgs, result, status)
    #[serde(default)]
    pub tool_former_data: Option<ToolFormerData>,

    /// Suggested code blocks
    #[serde(default)]
    pub suggested_code_blocks: Vec<SuggestedCodeBlock>,

    /// Context mentions in this message
    #[serde(default)]
    pub context: Option<BubbleContext>,

    /// Timestamp (may be present in some bubbles)
    pub timestamp: Option<i64>,

    /// Story 8.16: Images attached to this message
    /// Format: Array of image objects with base64 data or URLs
    #[serde(default)]
    pub images: Vec<CursorImage>,

    /// Story 8.17: All thinking blocks from this message
    /// Contains AI reasoning/thinking content before response
    #[serde(default, alias = "allThinkingBlocks")]
    pub all_thinking_blocks: Vec<CursorThinkingBlock>,
}

/// Story 8.17: Thinking block data in Cursor bubbles
/// Cursor stores thinking blocks in allThinkingBlocks array
/// The structure can be either a simple string or an object with text and metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CursorThinkingBlock {
    /// Simple text thinking block
    Text(String),
    /// Structured thinking block with metadata
    Structured {
        /// The thinking content
        #[serde(alias = "content")]
        text: Option<String>,
        /// Optional timestamp (epoch milliseconds)
        timestamp: Option<i64>,
        /// Optional subject/topic
        subject: Option<String>,
    },
}

impl CursorThinkingBlock {
    /// Extract the thinking text content
    pub fn get_text(&self) -> Option<&str> {
        match self {
            CursorThinkingBlock::Text(s) => Some(s.as_str()),
            CursorThinkingBlock::Structured { text, .. } => text.as_deref(),
        }
    }

    /// Extract the timestamp if available
    pub fn get_timestamp(&self) -> Option<i64> {
        match self {
            CursorThinkingBlock::Text(_) => None,
            CursorThinkingBlock::Structured { timestamp, .. } => *timestamp,
        }
    }

    /// Extract the subject if available
    pub fn get_subject(&self) -> Option<&str> {
        match self {
            CursorThinkingBlock::Text(_) => None,
            CursorThinkingBlock::Structured { subject, .. } => subject.as_deref(),
        }
    }
}

/// Story 8.16: Image data in Cursor bubbles
/// Cursor may store images as base64 data or URLs
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorImage {
    /// MIME type (e.g., "image/png", "image/jpeg")
    #[serde(default)]
    pub mime_type: Option<String>,

    /// Base64 encoded image data
    #[serde(default)]
    pub data: Option<String>,

    /// Image URL (alternative to base64 data)
    #[serde(default)]
    pub url: Option<String>,

    /// Optional alternative text for accessibility
    #[serde(default)]
    pub alt: Option<String>,
}

/// Tool call data from Cursor's toolFormerData field
/// 
/// This is the PRIMARY source for tool call information in Cursor.
/// The `toolResults` field is typically empty; use this instead.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolFormerData {
    /// Tool type enum (e.g., 38 for edit_file)
    pub tool: Option<i32>,

    /// Tool index in conversation
    pub tool_index: Option<i32>,

    /// Tool call ID (use as correlation_id)
    pub tool_call_id: Option<String>,

    /// Model call ID
    pub model_call_id: Option<String>,

    /// Execution status: pending/running/completed/failed
    pub status: Option<String>,

    /// Tool name (e.g., "read_file", "edit_file", "run_terminal_cmd")
    pub name: Option<String>,

    /// Raw arguments JSON string
    pub raw_args: Option<String>,

    /// Parsed parameters JSON string
    pub params: Option<String>,

    /// Execution result JSON string
    pub result: Option<String>,

    /// Additional data
    pub additional_data: Option<serde_json::Value>,

    /// User decision (for approval flows)
    pub user_decision: Option<String>,
}

/// Tool result in a bubble (legacy, usually empty)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Tool name
    pub name: Option<String>,

    /// Tool ID
    pub id: Option<String>,

    /// Result content
    pub result: Option<serde_json::Value>,

    /// Whether the tool execution failed
    #[serde(default)]
    pub is_error: bool,
}

/// Suggested code block in assistant response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedCodeBlock {
    /// File path
    pub file_path: Option<String>,

    /// Code content
    pub code: Option<String>,

    /// Language identifier
    pub language: Option<String>,
}

/// Context information within a bubble
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleContext {
    /// Mentioned files (complex nested object, use Value for flexibility)
    #[serde(default)]
    pub mentions: serde_json::Value,
}

/// Role mapping for bubble types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorRole {
    User,
    Assistant,
    Unknown,
}

impl From<i32> for CursorRole {
    fn from(bubble_type: i32) -> Self {
        match bubble_type {
            1 => CursorRole::User,
            2 => CursorRole::Assistant,
            _ => CursorRole::Unknown,
        }
    }
}

impl CursorRole {
    /// Convert to MantraSession Role
    pub fn to_mantra_role(self) -> Option<crate::models::Role> {
        match self {
            CursorRole::User => Some(crate::models::Role::User),
            CursorRole::Assistant => Some(crate::models::Role::Assistant),
            CursorRole::Unknown => None,
        }
    }
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
