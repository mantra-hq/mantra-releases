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

    /// Tool results from this message
    #[serde(default)]
    pub tool_results: Vec<ToolResult>,

    /// Suggested code blocks
    #[serde(default)]
    pub suggested_code_blocks: Vec<SuggestedCodeBlock>,

    /// Context mentions in this message
    #[serde(default)]
    pub context: Option<BubbleContext>,

    /// Timestamp (may be present in some bubbles)
    pub timestamp: Option<i64>,
}

/// Tool result in a bubble
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
mod tests {
    use super::*;

    #[test]
    fn test_cursor_role_from_bubble_type() {
        assert_eq!(CursorRole::from(1), CursorRole::User);
        assert_eq!(CursorRole::from(2), CursorRole::Assistant);
        assert_eq!(CursorRole::from(0), CursorRole::Unknown);
        assert_eq!(CursorRole::from(99), CursorRole::Unknown);
    }

    #[test]
    fn test_cursor_role_to_mantra_role() {
        assert_eq!(
            CursorRole::User.to_mantra_role(),
            Some(crate::models::Role::User)
        );
        assert_eq!(
            CursorRole::Assistant.to_mantra_role(),
            Some(crate::models::Role::Assistant)
        );
        assert_eq!(CursorRole::Unknown.to_mantra_role(), None);
    }

    #[test]
    fn test_deserialize_bubble_header() {
        let json = r#"{"bubbleId": "abc-123", "type": 1}"#;
        let header: BubbleHeader = serde_json::from_str(json).unwrap();
        assert_eq!(header.bubble_id, "abc-123");
        assert_eq!(header.bubble_type, 1);
    }

    #[test]
    fn test_deserialize_cursor_bubble() {
        let json = r#"{
            "_v": 3,
            "bubbleId": "bubble-123",
            "type": 2,
            "text": "Here is the code you requested.",
            "isAgentic": true,
            "toolResults": [],
            "suggestedCodeBlocks": []
        }"#;

        let bubble: CursorBubble = serde_json::from_str(json).unwrap();
        assert_eq!(bubble.version, Some(3));
        assert_eq!(bubble.bubble_id, Some("bubble-123".to_string()));
        assert_eq!(bubble.bubble_type, 2);
        assert_eq!(bubble.text, Some("Here is the code you requested.".to_string()));
        assert!(bubble.is_agentic);
    }

    #[test]
    fn test_deserialize_cursor_composer() {
        let json = r#"{
            "_v": 2,
            "composerId": "comp-456",
            "fullConversationHeadersOnly": [
                {"bubbleId": "b1", "type": 1},
                {"bubbleId": "b2", "type": 2}
            ],
            "createdAt": 1704067200000,
            "unifiedMode": "agent"
        }"#;

        let composer: CursorComposer = serde_json::from_str(json).unwrap();
        assert_eq!(composer.version, Some(2));
        assert_eq!(composer.composer_id, Some("comp-456".to_string()));
        assert_eq!(composer.full_conversation_headers_only.len(), 2);
        assert_eq!(composer.created_at, Some(1704067200000));
        assert_eq!(composer.unified_mode, Some("agent".to_string()));
    }
}
