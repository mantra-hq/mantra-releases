//! Content block types
//!
//! Contains the ContentBlock enum for representing different types of content
//! within messages (text, thinking, tool usage, images, etc.)

use serde::{Deserialize, Serialize};
use super::standard_tool::{StandardTool, ToolResultData};

/// Content block types in a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text {
        text: String,
        /// Whether this content was degraded from an unknown format
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_degraded: Option<bool>,
    },

    /// AI thinking/reasoning content (extended thinking)
    Thinking {
        thinking: String,

        // === New: Standardized fields ===
        /// Thinking subject/title (Gemini: thought.subject)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subject: Option<String>,

        /// Thinking timestamp (Gemini: thought.timestamp)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Tool usage request from AI
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        /// Unified correlation ID for pairing with ToolResult
        #[serde(skip_serializing_if = "Option::is_none")]
        correlation_id: Option<String>,

        // === New: Standardized fields ===
        /// Semantic tool type for unified frontend handling
        #[serde(default, skip_serializing_if = "Option::is_none")]
        standard_tool: Option<StandardTool>,

        /// UI display name (used by Gemini)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,

        /// Tool description (used by Gemini)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
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

        // === New: Structured result fields ===
        /// Structured tool result data (from Claude toolUseResult, etc.)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        structured_result: Option<ToolResultData>,

        /// UI display content (Gemini: resultDisplay)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_content: Option<String>,

        /// Whether to render display_content as Markdown (Gemini: renderOutputAsMarkdown)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        render_as_markdown: Option<bool>,

        /// User decision for tool execution (Cursor: approved/rejected)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user_decision: Option<String>,
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

    /// Image content (Base64 encoded image data)
    Image {
        /// MIME type (e.g., "image/png", "image/jpeg", "image/gif", "image/webp")
        media_type: String,
        /// Base64 encoded image data
        data: String,
        /// Source type indicating the origin of the image data
        /// - "base64": Raw Base64 encoded data (Claude, Gemini)
        /// - "url": Reference to external URL (Cursor, if applicable)
        #[serde(skip_serializing_if = "Option::is_none")]
        source_type: Option<String>,
        /// Optional alternative text for accessibility
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
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

    /// Code suggestion from AI (Cursor suggestedCodeBlocks)
    CodeSuggestion {
        /// Target file path for the suggestion
        file_path: String,
        /// Suggested code content
        code: String,
        /// Programming language
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_text_serialization() {
        let block = ContentBlock::Text {
            text: "Hello world".to_string(),
            is_degraded: None,
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
            subject: None,
            timestamp: None,
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
            standard_tool: None,
            display_name: None,
            description: None,
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
            structured_result: None,
            display_content: None,
            render_as_markdown: None,
            user_decision: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"tool_result""#));
        assert!(json.contains(r#""tool_use_id":"tool_123""#));
        assert!(json.contains(r#""correlation_id":"corr_123""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_tool_use_backward_compat_old_format() {
        let old_json = r#"{
            "type": "tool_use",
            "id": "tool_123",
            "name": "Read",
            "input": {"path": "/tmp/test.txt"},
            "correlation_id": "corr_123"
        }"#;
        let block: ContentBlock = serde_json::from_str(old_json).unwrap();
        match block {
            ContentBlock::ToolUse {
                id,
                name,
                correlation_id,
                standard_tool,
                display_name,
                description,
                ..
            } => {
                assert_eq!(id, "tool_123");
                assert_eq!(name, "Read");
                assert_eq!(correlation_id, Some("corr_123".to_string()));
                assert!(standard_tool.is_none());
                assert!(display_name.is_none());
                assert!(description.is_none());
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn test_tool_result_backward_compat() {
        let old_json = r#"{
            "type": "tool_result",
            "tool_use_id": "123",
            "content": "file content",
            "is_error": false
        }"#;
        let block: ContentBlock = serde_json::from_str(old_json).unwrap();
        match block {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
                structured_result,
                display_content,
                render_as_markdown,
                user_decision,
                ..
            } => {
                assert_eq!(tool_use_id, "123");
                assert_eq!(content, "file content");
                assert!(!is_error);
                assert!(structured_result.is_none());
                assert!(display_content.is_none());
                assert!(render_as_markdown.is_none());
                assert!(user_decision.is_none());
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_thinking_with_subject_timestamp() {
        let block = ContentBlock::Thinking {
            thinking: "**Problem Analysis**\nAnalyzing the code...".to_string(),
            subject: Some("Problem Analysis".to_string()),
            timestamp: Some("2025-12-30T20:00:55.000Z".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"thinking""#));
        assert!(json.contains(r#""subject":"Problem Analysis""#));
        assert!(json.contains(r#""timestamp":"2025-12-30T20:00:55.000Z""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_serialization() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/src/main.rs".to_string(),
            code: "fn main() {}\n".to_string(),
            language: Some("rust".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"code_suggestion""#));
        assert!(json.contains(r#""file_path":"/src/main.rs""#));
        assert!(json.contains(r#""language":"rust""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_image_block_serialization() {
        let block = ContentBlock::Image {
            media_type: "image/png".to_string(),
            data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            source_type: Some("base64".to_string()),
            alt_text: Some("A test image".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"image""#));
        assert!(json.contains(r#""media_type":"image/png""#));
        assert!(json.contains(r#""data":"iVBORw0KGgo"#));
        assert!(json.contains(r#""source_type":"base64""#));
        assert!(json.contains(r#""alt_text":"A test image""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_image_block_minimal() {
        // Test with only required fields
        let block = ContentBlock::Image {
            media_type: "image/jpeg".to_string(),
            data: "/9j/4AAQSkZJRg==".to_string(),
            source_type: None,
            alt_text: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"image""#));
        assert!(json.contains(r#""media_type":"image/jpeg""#));
        assert!(json.contains(r#""data":"/9j/4AAQSkZJRg==""#));
        // Optional fields should not be in output
        assert!(!json.contains(r#""source_type""#));
        assert!(!json.contains(r#""alt_text""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_image_block_deserialization_from_json() {
        // Test deserializing from raw JSON (as parsers would produce)
        let json = r#"{
            "type": "image",
            "media_type": "image/webp",
            "data": "UklGRlYAAABXRUJQ",
            "source_type": "base64",
            "alt_text": "Screenshot"
        }"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Image {
                media_type,
                data,
                source_type,
                alt_text,
            } => {
                assert_eq!(media_type, "image/webp");
                assert_eq!(data, "UklGRlYAAABXRUJQ");
                assert_eq!(source_type, Some("base64".to_string()));
                assert_eq!(alt_text, Some("Screenshot".to_string()));
            }
            _ => panic!("Expected Image variant"),
        }
    }
}
