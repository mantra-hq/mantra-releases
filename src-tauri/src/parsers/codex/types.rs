//! Codex CLI data types and structures
//!
//! Defines types for parsing Codex CLI's JSONL conversation files.
//!
//! ## Session File Structure
//!
//! ```text
//! ~/.codex/sessions/YYYY/MM/DD/rollout-{timestamp}-{session_id}.jsonl
//! ```
//!
//! ## JSONL Line Types
//!
//! - `session_meta`: Session metadata (id, cwd, timestamp, cli_version)
//! - `response_item`: Messages and function calls
//! - `event_msg`: Event messages (skipped)
//! - `turn_context`: Turn context (skipped)

use serde::Deserialize;

/// A single line in a Codex rollout JSONL file
#[derive(Debug, Clone, Deserialize)]
pub struct CodexRolloutLine {
    /// Timestamp of this line
    pub timestamp: String,

    /// Line type: session_meta, response_item, event_msg, turn_context
    #[serde(rename = "type")]
    pub line_type: CodexLineType,

    /// Payload content (varies by line type)
    pub payload: serde_json::Value,
}

/// Types of lines in a Codex rollout file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexLineType {
    /// Session metadata
    SessionMeta,
    /// Response item (message, function_call, function_call_output)
    ResponseItem,
    /// Event message (skipped)
    EventMsg,
    /// Turn context (skipped)
    TurnContext,
    /// Unknown type (for forward compatibility with new Codex versions)
    #[serde(other)]
    Unknown,
}

impl CodexLineType {
    /// Check if this line type should be processed
    pub fn should_process(self) -> bool {
        matches!(self, CodexLineType::SessionMeta | CodexLineType::ResponseItem)
    }

    /// Check if this is an unknown type
    pub fn is_unknown(self) -> bool {
        matches!(self, CodexLineType::Unknown)
    }
}

/// Session metadata from session_meta line
#[derive(Debug, Clone, Deserialize)]
pub struct CodexSessionMeta {
    /// Session unique ID
    pub id: String,

    /// Session timestamp (ISO 8601)
    pub timestamp: String,

    /// Working directory
    pub cwd: String,

    /// CLI version
    #[serde(default)]
    pub cli_version: Option<String>,

    /// Originator (e.g., "codex_cli_rs")
    #[serde(default)]
    pub originator: Option<String>,

    /// Source (e.g., "cli")
    #[serde(default)]
    pub source: Option<String>,

    /// Model provider
    #[serde(default)]
    pub model_provider: Option<String>,

    /// Instructions (system prompt)
    #[serde(default)]
    pub instructions: Option<String>,

    /// Git information
    #[serde(default)]
    pub git: Option<CodexGitInfo>,
}

/// Git repository information
#[derive(Debug, Clone, Deserialize)]
pub struct CodexGitInfo {
    /// Commit hash
    #[serde(default)]
    pub commit_hash: Option<String>,

    /// Branch name
    #[serde(default)]
    pub branch: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository_url: Option<String>,
}

/// Response item from response_item line
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexResponseItem {
    /// User or assistant message
    Message {
        /// Role: user or assistant
        role: CodexRole,
        /// Content blocks
        content: Vec<CodexContentItem>,
    },

    /// Function call
    FunctionCall {
        /// Function name
        name: String,
        /// Function arguments (JSON string)
        arguments: String,
        /// Call ID
        call_id: String,
    },

    /// Function call output
    FunctionCallOutput {
        /// Call ID
        call_id: String,
        /// Output content
        output: String,
    },
}

/// Role in Codex conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodexRole {
    /// User message
    User,
    /// Assistant message
    Assistant,
}

impl CodexRole {
    /// Convert to MantraSession Role
    pub fn to_mantra_role(self) -> crate::models::Role {
        match self {
            CodexRole::User => crate::models::Role::User,
            CodexRole::Assistant => crate::models::Role::Assistant,
        }
    }
}

/// Content item in a message
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CodexContentItem {
    /// User input text
    InputText {
        /// Text content
        text: String,
    },

    /// Assistant output text
    OutputText {
        /// Text content
        text: String,
    },
}

impl CodexContentItem {
    /// Get the text content
    pub fn text(&self) -> &str {
        match self {
            CodexContentItem::InputText { text } => text,
            CodexContentItem::OutputText { text } => text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_type_should_process() {
        assert!(CodexLineType::SessionMeta.should_process());
        assert!(CodexLineType::ResponseItem.should_process());
        assert!(!CodexLineType::EventMsg.should_process());
        assert!(!CodexLineType::TurnContext.should_process());
    }

    #[test]
    fn test_role_to_mantra_role() {
        assert_eq!(CodexRole::User.to_mantra_role(), crate::models::Role::User);
        assert_eq!(CodexRole::Assistant.to_mantra_role(), crate::models::Role::Assistant);
    }

    #[test]
    fn test_content_item_text() {
        let input = CodexContentItem::InputText { text: "hello".to_string() };
        assert_eq!(input.text(), "hello");

        let output = CodexContentItem::OutputText { text: "world".to_string() };
        assert_eq!(output.text(), "world");
    }

    #[test]
    fn test_deserialize_session_meta() {
        let json = r#"{
            "id": "test-123",
            "timestamp": "2025-12-30T20:00:00.000Z",
            "cwd": "/home/user/project",
            "cli_version": "0.77.0",
            "originator": "codex_cli_rs",
            "source": "cli"
        }"#;

        let meta: CodexSessionMeta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "test-123");
        assert_eq!(meta.cwd, "/home/user/project");
        assert_eq!(meta.cli_version, Some("0.77.0".to_string()));
    }

    #[test]
    fn test_deserialize_rollout_line() {
        let json = r#"{
            "timestamp": "2025-12-30T20:00:00.000Z",
            "type": "session_meta",
            "payload": {"id": "test-123", "timestamp": "2025-12-30T20:00:00.000Z", "cwd": "/tmp"}
        }"#;

        let line: CodexRolloutLine = serde_json::from_str(json).unwrap();
        assert_eq!(line.line_type, CodexLineType::SessionMeta);
    }

    #[test]
    fn test_deserialize_user_message() {
        let json = r#"{
            "type": "message",
            "role": "user",
            "content": [{"type": "input_text", "text": "Hello"}]
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::Message { role, content } => {
                assert_eq!(role, CodexRole::User);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text(), "Hello");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_deserialize_assistant_message() {
        let json = r#"{
            "type": "message",
            "role": "assistant",
            "content": [{"type": "output_text", "text": "I'll help you."}]
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::Message { role, content } => {
                assert_eq!(role, CodexRole::Assistant);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text(), "I'll help you.");
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn test_deserialize_function_call() {
        let json = r#"{
            "type": "function_call",
            "name": "shell",
            "arguments": "{\"command\": \"ls\"}",
            "call_id": "call_123"
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::FunctionCall { name, arguments, call_id } => {
                assert_eq!(name, "shell");
                assert_eq!(call_id, "call_123");
                assert!(arguments.contains("ls"));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_deserialize_function_call_output() {
        let json = r#"{
            "type": "function_call_output",
            "call_id": "call_123",
            "output": "file1.txt\nfile2.txt"
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::FunctionCallOutput { call_id, output } => {
                assert_eq!(call_id, "call_123");
                assert!(output.contains("file1.txt"));
            }
            _ => panic!("Expected FunctionCallOutput"),
        }
    }

    #[test]
    fn test_deserialize_git_info() {
        let json = r#"{
            "id": "test-123",
            "timestamp": "2025-12-30T20:00:00.000Z",
            "cwd": "/home/user/project",
            "git": {
                "commit_hash": "abc123",
                "branch": "main",
                "repository_url": "https://github.com/user/repo"
            }
        }"#;

        let meta: CodexSessionMeta = serde_json::from_str(json).unwrap();
        assert!(meta.git.is_some());
        let git = meta.git.unwrap();
        assert_eq!(git.commit_hash, Some("abc123".to_string()));
        assert_eq!(git.branch, Some("main".to_string()));
    }
}
