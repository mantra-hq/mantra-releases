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
#[path = "types_tests.rs"]
mod tests;
