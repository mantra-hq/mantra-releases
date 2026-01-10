//! Codex CLI data types and structures
//!
//! Defines types for parsing Codex CLI's JSONL conversation files.
//! Aligned with official Codex protocol: codex-rs/protocol/src/models.rs
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
//! - `response_item`: Messages, function calls, reasoning, web search, etc.
//! - `event_msg`: Event messages (skipped)
//! - `turn_context`: Turn context (skipped)
//!
//! ## ResponseItem Types (from official Codex protocol)
//!
//! - `message`: User or assistant message
//! - `reasoning`: Reasoning model's thinking process (summary + encrypted content)
//! - `local_shell_call`: Local shell command execution
//! - `function_call`: Function/tool call
//! - `function_call_output`: Function call result
//! - `custom_tool_call`: Custom tool call
//! - `custom_tool_call_output`: Custom tool result
//! - `web_search_call`: Web search action
//! - `ghost_snapshot`: Ghost commit snapshot
//! - `compaction`: Compacted context summary

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
/// Aligned with official Codex protocol: codex-rs/protocol/src/models.rs
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

    /// Reasoning model's thinking process (o1, o3, gpt-5, etc.)
    /// Contains summary (readable) and encrypted_content (opaque, for API continuation)
    Reasoning {
        /// Summary of reasoning steps (readable text)
        #[serde(default)]
        summary: Vec<ReasoningSummary>,
        /// Raw reasoning content (when available, readable)
        #[serde(default)]
        content: Option<Vec<ReasoningContent>>,
        /// Encrypted reasoning content (opaque, cannot be decrypted locally)
        /// Used for: 1) token estimation, 2) session resume/fork with OpenAI API
        #[serde(default)]
        encrypted_content: Option<String>,
    },

    /// Local shell command execution
    LocalShellCall {
        /// Call ID
        #[serde(default)]
        call_id: Option<String>,
        /// Shell status
        #[serde(default)]
        status: Option<LocalShellStatus>,
        /// Shell action
        action: LocalShellAction,
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
        /// Output content (can be string or structured)
        #[serde(flatten)]
        output: FunctionCallOutputPayload,
    },

    /// Custom tool call
    CustomToolCall {
        /// Call ID
        call_id: String,
        /// Tool name
        name: String,
        /// Input (JSON string)
        input: String,
        /// Status
        #[serde(default)]
        status: Option<String>,
    },

    /// Custom tool call output
    CustomToolCallOutput {
        /// Call ID
        call_id: String,
        /// Output content
        output: String,
    },

    /// Web search action
    WebSearchCall {
        /// Search action details
        action: WebSearchAction,
        /// Status
        #[serde(default)]
        status: Option<String>,
    },

    /// Ghost commit snapshot (for version control integration)
    GhostSnapshot {
        /// Ghost commit information
        ghost_commit: GhostCommit,
    },

    /// Compacted context summary (for long conversations)
    #[serde(alias = "compaction_summary")]
    Compaction {
        /// Encrypted compaction content
        encrypted_content: String,
    },

    /// Unknown response item type (for forward compatibility)
    #[serde(other)]
    Other,
}

/// Reasoning summary entry
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReasoningSummary {
    /// Summary text
    SummaryText {
        /// The summary text content
        text: String,
    },
}

impl ReasoningSummary {
    /// Get the text content
    pub fn text(&self) -> &str {
        match self {
            ReasoningSummary::SummaryText { text } => text,
        }
    }
}

/// Reasoning content entry (when raw content is available)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReasoningContent {
    /// Reasoning text
    ReasoningText {
        /// The reasoning text content
        text: String,
    },
    /// Plain text
    Text {
        /// The text content
        text: String,
    },
}

impl ReasoningContent {
    /// Get the text content
    pub fn text(&self) -> &str {
        match self {
            ReasoningContent::ReasoningText { text } | ReasoningContent::Text { text } => text,
        }
    }
}

/// Local shell status
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalShellStatus {
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Unknown status
    #[serde(other)]
    Unknown,
}

/// Local shell action
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalShellAction {
    /// Execute command
    Exec {
        /// Command to execute
        #[serde(default)]
        command: Option<Vec<String>>,
        /// Working directory
        #[serde(default)]
        cwd: Option<String>,
        /// Exit code
        #[serde(default)]
        exit_code: Option<i32>,
        /// Output
        #[serde(default)]
        output: Option<String>,
    },
    /// Unknown action
    #[serde(other)]
    Unknown,
}

/// Function call output payload
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCallOutputPayload {
    /// Output content (string form)
    #[serde(default)]
    pub output: Option<String>,
    /// Success flag
    #[serde(default)]
    pub success: Option<bool>,
}

impl FunctionCallOutputPayload {
    /// Get the output string, extracting from JSON wrapper if present.
    ///
    /// Codex shell output can be in two formats:
    /// 1. JSON format: `{"metadata": {...}, "output": "actual content"}`
    /// 2. Structured text: `Exit code: 0\nWall time: ...\nOutput:\nactual content`
    ///
    /// This method extracts the actual output content from either format.
    pub fn get_output(&self) -> String {
        let raw = self.output.clone().unwrap_or_default();

        // Try to parse as JSON and extract the "output" field
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(output) = parsed.get("output").and_then(|v| v.as_str()) {
                return output.to_string();
            }
        }

        raw
    }
}

/// Web search action
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSearchAction {
    /// Search query
    Search {
        /// The search query
        #[serde(default)]
        query: Option<String>,
    },
    /// Unknown action
    #[serde(other)]
    Unknown,
}

/// Ghost commit information
#[derive(Debug, Clone, Deserialize)]
pub struct GhostCommit {
    /// Commit hash
    #[serde(default)]
    pub commit_hash: Option<String>,
    /// Commit message
    #[serde(default)]
    pub message: Option<String>,
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
                assert!(output.get_output().contains("file1.txt"));
            }
            _ => panic!("Expected FunctionCallOutput"),
        }
    }

    #[test]
    fn test_deserialize_function_call_output_with_success() {
        let json = r#"{
            "type": "function_call_output",
            "call_id": "call_456",
            "output": "Success!",
            "success": true
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::FunctionCallOutput { call_id, output } => {
                assert_eq!(call_id, "call_456");
                assert_eq!(output.get_output(), "Success!");
                assert_eq!(output.success, Some(true));
            }
            _ => panic!("Expected FunctionCallOutput"),
        }
    }

    #[test]
    fn test_deserialize_reasoning() {
        let json = r#"{
            "type": "reasoning",
            "summary": [{"type": "summary_text", "text": "**Thinking about the problem**"}],
            "content": null,
            "encrypted_content": "gAAAAABo4kai..."
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::Reasoning { summary, content, encrypted_content } => {
                assert_eq!(summary.len(), 1);
                assert_eq!(summary[0].text(), "**Thinking about the problem**");
                assert!(content.is_none());
                assert!(encrypted_content.is_some());
            }
            _ => panic!("Expected Reasoning"),
        }
    }

    #[test]
    fn test_deserialize_reasoning_with_content() {
        let json = r#"{
            "type": "reasoning",
            "summary": [{"type": "summary_text", "text": "Step 1"}],
            "content": [
                {"type": "reasoning_text", "text": "First, let me analyze..."},
                {"type": "text", "text": "Then, I'll implement..."}
            ]
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::Reasoning { summary, content, .. } => {
                assert_eq!(summary.len(), 1);
                assert!(content.is_some());
                let contents = content.unwrap();
                assert_eq!(contents.len(), 2);
                assert_eq!(contents[0].text(), "First, let me analyze...");
                assert_eq!(contents[1].text(), "Then, I'll implement...");
            }
            _ => panic!("Expected Reasoning"),
        }
    }

    #[test]
    fn test_deserialize_web_search_call() {
        let json = r#"{
            "type": "web_search_call",
            "status": "completed",
            "action": {"type": "search", "query": "Rust async programming"}
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::WebSearchCall { action, status } => {
                assert_eq!(status, Some("completed".to_string()));
                if let WebSearchAction::Search { query } = action {
                    assert_eq!(query, Some("Rust async programming".to_string()));
                } else {
                    panic!("Expected Search action");
                }
            }
            _ => panic!("Expected WebSearchCall"),
        }
    }

    #[test]
    fn test_deserialize_custom_tool_call() {
        let json = r#"{
            "type": "custom_tool_call",
            "call_id": "custom_123",
            "name": "my_tool",
            "input": "{\"param\": \"value\"}"
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::CustomToolCall { call_id, name, input, .. } => {
                assert_eq!(call_id, "custom_123");
                assert_eq!(name, "my_tool");
                assert!(input.contains("param"));
            }
            _ => panic!("Expected CustomToolCall"),
        }
    }

    #[test]
    fn test_deserialize_compaction() {
        let json = r#"{
            "type": "compaction",
            "encrypted_content": "ENCRYPTED_SUMMARY_CONTENT"
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::Compaction { encrypted_content } => {
                assert_eq!(encrypted_content, "ENCRYPTED_SUMMARY_CONTENT");
            }
            _ => panic!("Expected Compaction"),
        }
    }

    #[test]
    fn test_deserialize_unknown_type() {
        let json = r#"{
            "type": "future_new_type",
            "some_field": "value"
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        assert!(matches!(item, CodexResponseItem::Other));
    }

    #[test]
    fn test_deserialize_local_shell_call() {
        let json = r#"{
            "type": "local_shell_call",
            "call_id": "shell_123",
            "status": "completed",
            "action": {
                "type": "exec",
                "command": ["bash", "-c", "ls -la"],
                "cwd": "/tmp",
                "exit_code": 0,
                "output": "file1.txt\nfile2.txt"
            }
        }"#;

        let item: CodexResponseItem = serde_json::from_str(json).unwrap();
        match item {
            CodexResponseItem::LocalShellCall { call_id, status, action } => {
                assert_eq!(call_id, Some("shell_123".to_string()));
                assert!(matches!(status, Some(LocalShellStatus::Completed)));
                if let LocalShellAction::Exec { command, cwd, exit_code, output } = action {
                    assert_eq!(command, Some(vec!["bash".to_string(), "-c".to_string(), "ls -la".to_string()]));
                    assert_eq!(cwd, Some("/tmp".to_string()));
                    assert_eq!(exit_code, Some(0));
                    assert!(output.unwrap().contains("file1.txt"));
                } else {
                    panic!("Expected Exec action");
                }
            }
            _ => panic!("Expected LocalShellCall"),
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

    #[test]
    fn test_function_call_output_extracts_json_output() {
        // JSON format: {"metadata": {...}, "output": "actual content"}
        let payload = FunctionCallOutputPayload {
            output: Some(r#"{"metadata":{"exit_code":0,"duration_seconds":0.1},"output":"file1.txt\nfile2.txt"}"#.to_string()),
            success: Some(true),
        };
        assert_eq!(payload.get_output(), "file1.txt\nfile2.txt");
    }

    #[test]
    fn test_function_call_output_returns_raw_for_plain_text() {
        // Plain text format: Exit code: 0\nWall time: ...\nOutput:\nactual content
        let payload = FunctionCallOutputPayload {
            output: Some("Exit code: 0\nWall time: 0.1 seconds\nOutput:\nfile1.txt\nfile2.txt".to_string()),
            success: Some(true),
        };
        assert_eq!(payload.get_output(), "Exit code: 0\nWall time: 0.1 seconds\nOutput:\nfile1.txt\nfile2.txt");
    }

    #[test]
    fn test_function_call_output_returns_raw_for_non_shell_json() {
        // JSON without "output" field should return as-is
        let payload = FunctionCallOutputPayload {
            output: Some(r#"{"status":"ok","data":"some value"}"#.to_string()),
            success: Some(true),
        };
        assert_eq!(payload.get_output(), r#"{"status":"ok","data":"some value"}"#);
    }

    #[test]
    fn test_function_call_output_handles_empty() {
        let payload = FunctionCallOutputPayload {
            output: None,
            success: None,
        };
        assert_eq!(payload.get_output(), "");
    }
}
