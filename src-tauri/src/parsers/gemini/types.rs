//! Gemini CLI data types and structures
//!
//! Defines types for parsing Gemini CLI's conversation JSON files.
//!
//! ## Session File Structure
//!
//! ```text
//! ~/.gemini/tmp/{projectHash}/chats/session-{date}-{uuid}.json
//! ```
//!
//! ## File Format
//!
//! ```json
//! {
//!   "sessionId": "uuid",
//!   "projectHash": "sha256hex",
//!   "startTime": "ISO8601",
//!   "lastUpdated": "ISO8601",
//!   "messages": [...],
//!   "summary": "optional summary"
//! }
//! ```

use serde::Deserialize;

/// Gemini CLI conversation record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiConversation {
    /// Session unique ID
    pub session_id: String,

    /// Project hash (SHA256 of project root path)
    pub project_hash: String,

    /// Session start time (ISO 8601)
    pub start_time: String,

    /// Last update time (ISO 8601)
    pub last_updated: String,

    /// Messages in the conversation
    #[serde(default)]
    pub messages: Vec<GeminiMessage>,

    /// Optional summary
    pub summary: Option<String>,
}

/// Gemini CLI message record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMessage {
    /// Message unique ID
    pub id: String,

    /// Message timestamp (ISO 8601)
    pub timestamp: String,

    /// Message content (string or array of parts)
    #[serde(default)]
    pub content: GeminiContent,

    /// Message type: user, gemini, info, error, warning
    #[serde(rename = "type")]
    pub msg_type: GeminiMessageType,

    /// Tool calls made by Gemini (only for gemini type)
    #[serde(default)]
    pub tool_calls: Option<Vec<GeminiToolCall>>,

    /// Gemini's thoughts/reasoning (only for gemini type)
    #[serde(default)]
    pub thoughts: Option<Vec<GeminiThought>>,

    /// Token usage (only for gemini type)
    #[serde(default)]
    pub tokens: Option<GeminiTokens>,

    /// Model used (only for gemini type)
    #[serde(default)]
    pub model: Option<String>,
}

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeminiMessageType {
    /// User message
    User,
    /// Gemini assistant message
    Gemini,
    /// System info message (skip)
    Info,
    /// Error message (skip)
    Error,
    /// Warning message (skip)
    Warning,
}

impl GeminiMessageType {
    /// Check if this message type should be included in the session
    pub fn should_include(self) -> bool {
        matches!(self, GeminiMessageType::User | GeminiMessageType::Gemini)
    }

    /// Convert to MantraSession Role
    pub fn to_mantra_role(self) -> Option<crate::models::Role> {
        match self {
            GeminiMessageType::User => Some(crate::models::Role::User),
            GeminiMessageType::Gemini => Some(crate::models::Role::Assistant),
            _ => None,
        }
    }
}

/// Content can be either a simple string or an array of parts (PartListUnion)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum GeminiContent {
    /// Simple text content
    Text(String),
    /// Array of content parts
    Parts(Vec<GeminiPart>),
}

impl Default for GeminiContent {
    fn default() -> Self {
        GeminiContent::Text(String::new())
    }
}

impl GeminiContent {
    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            GeminiContent::Text(s) => s.is_empty(),
            GeminiContent::Parts(parts) => parts.is_empty(),
        }
    }

    /// Get text content as string
    pub fn as_text(&self) -> String {
        match self {
            GeminiContent::Text(s) => s.clone(),
            GeminiContent::Parts(parts) => {
                parts
                    .iter()
                    .filter_map(|p| p.text.as_ref())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }
}

/// Content part (from @google/genai Part type)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPart {
    /// Text content
    pub text: Option<String>,

    /// Inline data (e.g., images)
    pub inline_data: Option<GeminiInlineData>,

    /// Function call
    pub function_call: Option<GeminiFunctionCall>,

    /// Function response
    pub function_response: Option<GeminiFunctionResponse>,
}

/// Inline data (e.g., base64 encoded images)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineData {
    /// MIME type
    pub mime_type: Option<String>,
    /// Base64 encoded data
    pub data: Option<String>,
}

/// Function call in content
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments
    #[serde(default)]
    pub args: serde_json::Value,
}

/// Function response in content
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionResponse {
    /// Function name
    pub name: String,
    /// Response content
    #[serde(default)]
    pub response: serde_json::Value,
}

/// Tool call record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolCall {
    /// Tool call ID
    pub id: String,

    /// Tool/function name
    pub name: String,

    /// Arguments passed to the tool
    #[serde(default)]
    pub args: serde_json::Value,

    /// Tool results
    #[serde(default)]
    pub result: Option<Vec<GeminiToolResultWrapper>>,

    /// Execution status
    #[serde(default)]
    pub status: String,

    /// Execution timestamp
    pub timestamp: Option<String>,

    /// Human-readable tool name for display (e.g., "Shell", "Edit File")
    #[serde(default)]
    pub display_name: Option<String>,

    /// Tool description for UI display
    #[serde(default)]
    pub description: Option<String>,

    /// Pre-formatted result for display (cleaner than raw output)
    #[serde(default)]
    pub result_display: Option<String>,

    /// Whether to render output as markdown
    #[serde(default)]
    pub render_output_as_markdown: Option<bool>,
}

/// Wrapper for tool result containing functionResponse
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolResultWrapper {
    /// Function response object
    pub function_response: GeminiFunctionResponseResult,
}

/// Function response result
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionResponseResult {
    /// Function call ID
    pub id: String,

    /// Function name
    pub name: String,

    /// Response content
    #[serde(default)]
    pub response: GeminiToolResponse,
}

/// Tool response content
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolResponse {
    /// Output text
    pub output: Option<String>,

    /// Error message if failed
    pub error: Option<String>,

    /// Additional fields stored as raw Value
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl GeminiToolResponse {
    /// Get the output or error as a string
    pub fn as_content(&self) -> String {
        if let Some(output) = &self.output {
            output.clone()
        } else if let Some(error) = &self.error {
            format!("Error: {}", error)
        } else {
            String::new()
        }
    }
}

/// Thought/reasoning summary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiThought {
    /// Thought subject/title
    pub subject: String,

    /// Thought description/content
    pub description: String,

    /// Thought timestamp
    pub timestamp: Option<String>,
}

impl GeminiThought {
    /// Format thought as a string for ContentBlock::Thinking
    pub fn as_formatted_string(&self) -> String {
        format!("**{}** {}", self.subject, self.description)
    }
}

/// Token usage summary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTokens {
    /// Input/prompt tokens
    pub input: Option<i64>,

    /// Output/completion tokens
    pub output: Option<i64>,

    /// Cached tokens (from Gemini CLI: cachedContentTokenCount)
    pub cached: Option<i64>,

    /// Thoughts tokens (optional)
    pub thoughts: Option<i64>,

    /// Tool use tokens (optional)
    pub tool: Option<i64>,

    /// Total tokens (the authoritative sum)
    pub total: Option<i64>,

    /// Cache read tokens (legacy field)
    pub cache_read: Option<i64>,

    /// Cache write tokens (legacy field)
    pub cache_write: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_should_include() {
        assert!(GeminiMessageType::User.should_include());
        assert!(GeminiMessageType::Gemini.should_include());
        assert!(!GeminiMessageType::Info.should_include());
        assert!(!GeminiMessageType::Error.should_include());
        assert!(!GeminiMessageType::Warning.should_include());
    }

    #[test]
    fn test_message_type_to_role() {
        assert_eq!(
            GeminiMessageType::User.to_mantra_role(),
            Some(crate::models::Role::User)
        );
        assert_eq!(
            GeminiMessageType::Gemini.to_mantra_role(),
            Some(crate::models::Role::Assistant)
        );
        assert_eq!(GeminiMessageType::Info.to_mantra_role(), None);
    }

    #[test]
    fn test_content_text() {
        let content = GeminiContent::Text("Hello".to_string());
        assert_eq!(content.as_text(), "Hello");
        assert!(!content.is_empty());
    }

    #[test]
    fn test_content_default() {
        let content = GeminiContent::default();
        assert!(content.is_empty());
    }

    #[test]
    fn test_content_parts() {
        let content = GeminiContent::Parts(vec![
            GeminiPart {
                text: Some("Hello ".to_string()),
                inline_data: None,
                function_call: None,
                function_response: None,
            },
            GeminiPart {
                text: Some("World".to_string()),
                inline_data: None,
                function_call: None,
                function_response: None,
            },
        ]);
        assert_eq!(content.as_text(), "Hello World");
    }

    #[test]
    fn test_thought_format() {
        let thought = GeminiThought {
            subject: "Analysis".to_string(),
            description: "Analyzing the code structure".to_string(),
            timestamp: None,
        };
        assert_eq!(
            thought.as_formatted_string(),
            "**Analysis** Analyzing the code structure"
        );
    }

    #[test]
    fn test_tool_response_content() {
        let response = GeminiToolResponse {
            output: Some("Command output".to_string()),
            error: None,
            extra: serde_json::Map::new(),
        };
        assert_eq!(response.as_content(), "Command output");

        let error_response = GeminiToolResponse {
            output: None,
            error: Some("Failed".to_string()),
            extra: serde_json::Map::new(),
        };
        assert_eq!(error_response.as_content(), "Error: Failed");
    }

    #[test]
    fn test_deserialize_conversation() {
        let json = r#"{
            "sessionId": "test-123",
            "projectHash": "abc456",
            "startTime": "2025-12-30T20:11:00.000Z",
            "lastUpdated": "2025-12-30T20:15:00.000Z",
            "messages": [],
            "summary": "Test session"
        }"#;

        let conv: GeminiConversation = serde_json::from_str(json).unwrap();
        assert_eq!(conv.session_id, "test-123");
        assert_eq!(conv.project_hash, "abc456");
        assert!(conv.messages.is_empty());
        assert_eq!(conv.summary, Some("Test session".to_string()));
    }

    #[test]
    fn test_deserialize_user_message() {
        let json = r#"{
            "id": "msg-1",
            "timestamp": "2025-12-30T20:11:00.000Z",
            "type": "user",
            "content": "Hello, help me with this code"
        }"#;

        let msg: GeminiMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "msg-1");
        assert_eq!(msg.msg_type, GeminiMessageType::User);
        assert_eq!(msg.content.as_text(), "Hello, help me with this code");
    }

    #[test]
    fn test_deserialize_gemini_message_with_thoughts() {
        let json = r#"{
            "id": "msg-2",
            "timestamp": "2025-12-30T20:13:00.000Z",
            "type": "gemini",
            "content": "I'll help you with that.",
            "thoughts": [
                {
                    "subject": "Analysis",
                    "description": "User needs help with code",
                    "timestamp": "2025-12-30T20:12:58.000Z"
                }
            ],
            "model": "gemini-3-pro-preview"
        }"#;

        let msg: GeminiMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, "msg-2");
        assert_eq!(msg.msg_type, GeminiMessageType::Gemini);
        assert!(msg.thoughts.is_some());
        let thoughts = msg.thoughts.unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].subject, "Analysis");
        assert_eq!(msg.model, Some("gemini-3-pro-preview".to_string()));
    }

    #[test]
    fn test_deserialize_tool_call() {
        let json = r#"{
            "id": "run_shell_command-123",
            "name": "run_shell_command",
            "args": {"command": "ls -la"},
            "result": [
                {
                    "functionResponse": {
                        "id": "run_shell_command-123",
                        "name": "run_shell_command",
                        "response": {
                            "output": "total 0\ndrwxr-xr-x 2 user user 40 Dec 30 20:11 ."
                        }
                    }
                }
            ],
            "status": "success",
            "timestamp": "2025-12-30T20:13:20.000Z"
        }"#;

        let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
        assert_eq!(tool_call.id, "run_shell_command-123");
        assert_eq!(tool_call.name, "run_shell_command");
        assert_eq!(tool_call.status, "success");
        assert!(tool_call.result.is_some());
        let result = tool_call.result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].function_response.name,
            "run_shell_command"
        );
    }

    #[test]
    fn test_deserialize_tool_call_with_display_fields() {
        let json = r#"{
            "id": "run_shell_command-123",
            "name": "run_shell_command",
            "args": {"command": "ls -la"},
            "result": [
                {
                    "functionResponse": {
                        "id": "run_shell_command-123",
                        "name": "run_shell_command",
                        "response": {
                            "output": "file1.txt\nfile2.txt"
                        }
                    }
                }
            ],
            "status": "success",
            "timestamp": "2025-12-30T20:13:20.000Z",
            "displayName": "Shell",
            "description": "Execute shell commands",
            "resultDisplay": "file1.txt\nfile2.txt",
            "renderOutputAsMarkdown": false
        }"#;

        let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
        assert_eq!(tool_call.display_name, Some("Shell".to_string()));
        assert_eq!(tool_call.description, Some("Execute shell commands".to_string()));
        assert_eq!(tool_call.result_display, Some("file1.txt\nfile2.txt".to_string()));
        assert_eq!(tool_call.render_output_as_markdown, Some(false));
    }

    #[test]
    fn test_deserialize_message_with_parts_content() {
        let json = r#"{
            "id": "msg-3",
            "timestamp": "2025-12-30T20:14:00.000Z",
            "type": "gemini",
            "content": [
                {"text": "Here is "},
                {"text": "the answer"}
            ]
        }"#;

        let msg: GeminiMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.content.as_text(), "Here is the answer");
    }

    #[test]
    fn test_deserialize_info_message() {
        let json = r#"{
            "id": "msg-info",
            "timestamp": "2025-12-30T20:10:00.000Z",
            "type": "info",
            "content": "Session started"
        }"#;

        let msg: GeminiMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, GeminiMessageType::Info);
        assert!(!msg.msg_type.should_include());
    }
}
