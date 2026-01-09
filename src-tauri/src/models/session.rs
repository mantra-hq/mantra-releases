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
    /// OpenAI Codex CLI sessions
    pub const CODEX: &str = "codex";
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

/// Standardized tool type enumeration
///
/// Unifies tool semantics across different import sources (Claude, Gemini, Cursor, Codex),
/// eliminating the need for frontend compatibility code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StandardTool {
    /// Read file content
    FileRead {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_line: Option<u32>,
    },

    /// Write/create file
    FileWrite {
        path: String,
        content: String,
    },

    /// Edit file content
    FileEdit {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_string: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_string: Option<String>,
    },

    /// Execute shell command
    ShellExec {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },

    /// File search (Glob pattern matching)
    FileSearch {
        pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },

    /// Content search (Grep text search)
    ContentSearch {
        pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },

    /// Other/unknown tool (preserves original data)
    Other {
        name: String,
        input: serde_json::Value,
    },
}

/// Structured tool result data
///
/// Preserves structured information from tool execution results (e.g., Claude toolUseResult),
/// enabling frontend to display file paths, line numbers, and other semantic information
/// without parsing strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultData {
    /// File read result
    FileRead {
        file_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        num_lines: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_lines: Option<u32>,
    },

    /// File write result
    FileWrite { file_path: String },

    /// File edit result
    FileEdit {
        file_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_string: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_string: Option<String>,
    },

    /// Shell command execution result
    ShellExec {
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stdout: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stderr: Option<String>,
    },

    /// Other result (passthrough original data)
    Other { data: serde_json::Value },
}

/// Normalizes a tool call to a StandardTool variant.
///
/// Maps tool names from various sources (Claude, Gemini, Cursor, Codex) to
/// semantic StandardTool types, extracting standardized parameters from input.
///
/// # Arguments
/// * `name` - Original tool name (e.g., "Read", "read_file", "Bash")
/// * `input` - Tool input parameters as JSON Value
///
/// # Returns
/// Standardized tool type. Unknown tools return `StandardTool::Other`.
///
/// # Tool Name Mapping
/// | StandardTool    | Claude                  | Gemini            | Cursor           | Codex         |
/// |-----------------|-------------------------|-------------------|------------------|---------------|
/// | FileRead        | Read, read_file         | read_file         | read_file        | read_file     |
/// | FileWrite       | Write, write_file       | write_file        | write_file       | write_file    |
/// | FileEdit        | Edit, edit_file         | edit_file         | edit_file        | apply_diff    |
/// | ShellExec       | Bash, bash              | run_shell_command | run_terminal_cmd | shell         |
/// | FileSearch      | Glob, glob              | glob              | -                | search_files  |
/// | ContentSearch   | Grep, grep              | grep              | -                | -             |
pub fn normalize_tool(name: &str, input: &serde_json::Value) -> StandardTool {
    // Helper: extract path from input (supports file_path and path)
    let get_path = || -> String {
        input
            .get("file_path")
            .or_else(|| input.get("path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    // Helper: extract optional string field
    let get_str = |key: &str| -> Option<String> {
        input.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    // Helper: extract optional u32 field (supports both u32 and u64 JSON numbers)
    let get_u32 = |key: &str| -> Option<u32> {
        input.get(key).and_then(|v| v.as_u64()).map(|n| n as u32)
    };

    // Case-insensitive name matching
    match name.to_lowercase().as_str() {
        // FileRead: Read, read_file
        "read" | "read_file" => {
            let start = get_u32("start_line").or_else(|| get_u32("offset"));
            // end_line takes priority; if not present, calculate from offset + limit
            // Note: Claude uses offset (start line) + limit (line count), so end = offset + limit
            let end = get_u32("end_line").or_else(|| {
                let offset = get_u32("offset");
                let limit = get_u32("limit");
                match (offset, limit) {
                    (Some(o), Some(l)) => Some(o.saturating_add(l)),
                    _ => None,
                }
            });
            StandardTool::FileRead {
                path: get_path(),
                start_line: start,
                end_line: end,
            }
        }

        // FileWrite: Write, write_file
        "write" | "write_file" => StandardTool::FileWrite {
            path: get_path(),
            content: get_str("content").unwrap_or_default(),
        },

        // FileEdit: Edit, edit_file, apply_diff
        "edit" | "edit_file" | "apply_diff" => StandardTool::FileEdit {
            path: get_path(),
            old_string: get_str("old_string"),
            new_string: get_str("new_string").or_else(|| get_str("diff")),
        },

        // ShellExec: Bash, bash, run_shell_command, run_terminal_cmd, shell
        "bash" | "run_shell_command" | "run_terminal_cmd" | "shell" => StandardTool::ShellExec {
            command: get_str("command").unwrap_or_default(),
            cwd: get_str("cwd").or_else(|| get_str("working_dir")),
        },

        // FileSearch: Glob, glob, search_files
        "glob" | "search_files" => StandardTool::FileSearch {
            pattern: get_str("pattern").unwrap_or_default(),
            path: get_str("path"),
        },

        // ContentSearch: Grep, grep
        "grep" => StandardTool::ContentSearch {
            pattern: get_str("pattern").unwrap_or_default(),
            path: get_str("path"),
        },

        // Unknown tool: preserve original
        _ => StandardTool::Other {
            name: name.to_string(),
            input: input.clone(),
        },
    }
}

/// Content block types in a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text { text: String },

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

/// Git repository information for a session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitInfo {
    /// Git branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Git commit hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    /// Git repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
}

/// Token usage breakdown for a session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TokensBreakdown {
    /// Input tokens count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<u64>,

    /// Output tokens count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,

    /// Cached tokens count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<u64>,

    /// Thinking/reasoning tokens count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts: Option<u64>,

    /// Tool usage tokens count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<u64>,
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

    /// Unique message identifier (for message tree support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Parent message ID (for message tree support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Whether this message is part of a sidechain (branch conversation)
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_sidechain: bool,

    /// Source-specific metadata passthrough
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<serde_json::Value>,
}

/// Helper function for serde `skip_serializing_if` attribute.
///
/// Used to skip serializing boolean fields when they are `false`,
/// keeping the JSON output clean by omitting default values.
/// Example: `#[serde(default, skip_serializing_if = "is_false")]`
fn is_false(b: &bool) -> bool {
    !*b
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

    /// Git repository information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,

    /// Token usage breakdown (input, output, cached, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_breakdown: Option<TokensBreakdown>,

    /// System instructions/prompt (e.g., from Codex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Source-specific metadata passthrough (e.g., projectHash, unifiedMode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_metadata: Option<serde_json::Value>,
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

    /// Check if session is empty (no user AND no assistant messages)
    /// 
    /// Story 2.29: Empty session = user_message_count === 0 AND assistant_message_count === 0
    /// A session with only system messages is still considered empty.
    pub fn is_empty(&self) -> bool {
        let user_count = self.messages.iter().filter(|m| m.role == Role::User).count();
        let assistant_count = self.messages.iter().filter(|m| m.role == Role::Assistant).count();
        user_count == 0 && assistant_count == 0
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
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        }
    }

    /// Create a new message with mentioned files
    pub fn with_mentioned_files(role: Role, content_blocks: Vec<ContentBlock>, mentioned_files: Vec<String>) -> Self {
        Self {
            role,
            content_blocks,
            timestamp: Some(Utc::now()),
            mentioned_files,
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
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
                    subject: None,
                    timestamp: None,
                },
                ContentBlock::Text {
                    text: "I'll help you write the code.".to_string(),
                },
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "write_file".to_string(),
                    input: serde_json::json!({"path": "main.rs", "content": "fn main() {}"}),
                    correlation_id: Some("tool_1".to_string()),
                    standard_tool: None,
                    display_name: None,
                    description: None,
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

    #[test]
    fn test_git_info_serialization() {
        let git_info = GitInfo {
            branch: Some("main".to_string()),
            commit: Some("abc123".to_string()),
            repository_url: Some("https://github.com/user/repo".to_string()),
        };
        let json = serde_json::to_string(&git_info).unwrap();
        assert!(json.contains(r#""branch":"main""#));
        assert!(json.contains(r#""commit":"abc123""#));
        assert!(json.contains(r#""repository_url":"https://github.com/user/repo""#));

        let deserialized: GitInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, git_info);
    }

    #[test]
    fn test_git_info_skip_none_fields() {
        let git_info = GitInfo {
            branch: Some("main".to_string()),
            commit: None,
            repository_url: None,
        };
        let json = serde_json::to_string(&git_info).unwrap();
        assert!(json.contains(r#""branch":"main""#));
        assert!(!json.contains("commit"));
        assert!(!json.contains("repository_url"));
    }

    #[test]
    fn test_git_info_deserialize_partial() {
        let json = r#"{"branch":"feature"}"#;
        let git_info: GitInfo = serde_json::from_str(json).unwrap();
        assert_eq!(git_info.branch, Some("feature".to_string()));
        assert_eq!(git_info.commit, None);
        assert_eq!(git_info.repository_url, None);
    }

    #[test]
    fn test_tokens_breakdown_serialization() {
        let tokens = TokensBreakdown {
            input: Some(1000),
            output: Some(500),
            cached: Some(200),
            thoughts: Some(100),
            tool: Some(50),
        };
        let json = serde_json::to_string(&tokens).unwrap();
        assert!(json.contains(r#""input":1000"#));
        assert!(json.contains(r#""output":500"#));
        assert!(json.contains(r#""cached":200"#));
        assert!(json.contains(r#""thoughts":100"#));
        assert!(json.contains(r#""tool":50"#));

        let deserialized: TokensBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tokens);
    }

    #[test]
    fn test_tokens_breakdown_skip_none_fields() {
        let tokens = TokensBreakdown {
            input: Some(1000),
            output: None,
            cached: None,
            thoughts: None,
            tool: None,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        assert!(json.contains(r#""input":1000"#));
        assert!(!json.contains("output"));
        assert!(!json.contains("cached"));
        assert!(!json.contains("thoughts"));
        assert!(!json.contains("tool"));
    }

    #[test]
    fn test_tokens_breakdown_deserialize_partial() {
        let json = r#"{"input":500,"output":250}"#;
        let tokens: TokensBreakdown = serde_json::from_str(json).unwrap();
        assert_eq!(tokens.input, Some(500));
        assert_eq!(tokens.output, Some(250));
        assert_eq!(tokens.cached, None);
        assert_eq!(tokens.thoughts, None);
        assert_eq!(tokens.tool, None);
    }

    #[test]
    fn test_session_metadata_with_git_info() {
        let metadata = SessionMetadata {
            model: Some("claude-3-opus".to_string()),
            total_tokens: Some(1500),
            title: Some("Test Session".to_string()),
            original_path: None,
            git: Some(GitInfo {
                branch: Some("main".to_string()),
                commit: Some("abc123".to_string()),
                repository_url: Some("https://github.com/user/repo".to_string()),
            }),
            tokens_breakdown: None,
            instructions: None,
            source_metadata: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""git""#));
        assert!(json.contains(r#""branch":"main""#));
        assert!(!json.contains("original_path"));
        assert!(!json.contains("tokens_breakdown"));
        assert!(!json.contains("instructions"));
        assert!(!json.contains("source_metadata"));

        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert!(deserialized.git.is_some());
        assert_eq!(deserialized.git.as_ref().unwrap().branch, Some("main".to_string()));
    }

    #[test]
    fn test_session_metadata_with_tokens_breakdown() {
        let metadata = SessionMetadata {
            model: Some("gemini-pro".to_string()),
            total_tokens: None,
            title: None,
            original_path: None,
            git: None,
            tokens_breakdown: Some(TokensBreakdown {
                input: Some(1000),
                output: Some(500),
                cached: Some(200),
                thoughts: None,
                tool: None,
            }),
            instructions: None,
            source_metadata: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""tokens_breakdown""#));
        assert!(json.contains(r#""input":1000"#));
        assert!(!json.contains("thoughts"));

        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert!(deserialized.tokens_breakdown.is_some());
    }

    #[test]
    fn test_session_metadata_with_instructions() {
        let metadata = SessionMetadata {
            model: Some("codex".to_string()),
            total_tokens: None,
            title: None,
            original_path: None,
            git: None,
            tokens_breakdown: None,
            instructions: Some("You are a helpful coding assistant.".to_string()),
            source_metadata: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""instructions":"You are a helpful coding assistant.""#));

        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.instructions, Some("You are a helpful coding assistant.".to_string()));
    }

    #[test]
    fn test_session_metadata_with_source_metadata() {
        let metadata = SessionMetadata {
            model: Some("cursor".to_string()),
            total_tokens: None,
            title: None,
            original_path: None,
            git: None,
            tokens_breakdown: None,
            instructions: None,
            source_metadata: Some(serde_json::json!({
                "unifiedMode": true,
                "provider": "anthropic",
                "projectHash": "abc123"
            })),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""source_metadata""#));
        assert!(json.contains(r#""unifiedMode":true"#));
        assert!(json.contains(r#""projectHash":"abc123""#));

        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert!(deserialized.source_metadata.is_some());
        let source_meta = deserialized.source_metadata.unwrap();
        assert_eq!(source_meta["unifiedMode"], true);
        assert_eq!(source_meta["projectHash"], "abc123");
    }

    #[test]
    fn test_session_metadata_all_new_fields() {
        let metadata = SessionMetadata {
            model: Some("claude-3-opus".to_string()),
            total_tokens: Some(2000),
            title: Some("Complete Test".to_string()),
            original_path: Some("/path/to/session".to_string()),
            git: Some(GitInfo {
                branch: Some("feature-branch".to_string()),
                commit: Some("def456".to_string()),
                repository_url: None,
            }),
            tokens_breakdown: Some(TokensBreakdown {
                input: Some(1500),
                output: Some(500),
                cached: None,
                thoughts: Some(100),
                tool: Some(50),
            }),
            instructions: Some("Be concise.".to_string()),
            source_metadata: Some(serde_json::json!({"custom": "value"})),
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, metadata.model);
        assert_eq!(deserialized.total_tokens, metadata.total_tokens);
        assert!(deserialized.git.is_some());
        assert!(deserialized.tokens_breakdown.is_some());
        assert_eq!(deserialized.instructions, metadata.instructions);
        assert!(deserialized.source_metadata.is_some());
    }

    #[test]
    fn test_message_with_tree_structure() {
        let message = Message {
            role: Role::User,
            content_blocks: vec![ContentBlock::Text { text: "Hello".to_string() }],
            timestamp: None,
            mentioned_files: Vec::new(),
            message_id: Some("msg_123".to_string()),
            parent_id: Some("msg_122".to_string()),
            is_sidechain: false,
            source_metadata: None,
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""message_id":"msg_123""#));
        assert!(json.contains(r#""parent_id":"msg_122""#));
        assert!(!json.contains("is_sidechain")); // default false is serialized but may be omitted

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message_id, Some("msg_123".to_string()));
        assert_eq!(deserialized.parent_id, Some("msg_122".to_string()));
    }

    #[test]
    fn test_message_sidechain() {
        let message = Message {
            role: Role::Assistant,
            content_blocks: vec![ContentBlock::Text { text: "Branch response".to_string() }],
            timestamp: None,
            mentioned_files: Vec::new(),
            message_id: Some("msg_branch".to_string()),
            parent_id: Some("msg_123".to_string()),
            is_sidechain: true,
            source_metadata: None,
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""is_sidechain":true"#));

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_sidechain);
    }

    #[test]
    fn test_message_source_metadata() {
        let message = Message {
            role: Role::User,
            content_blocks: vec![ContentBlock::Text { text: "Test".to_string() }],
            timestamp: None,
            mentioned_files: Vec::new(),
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: Some(serde_json::json!({
                "claude_specific": "value",
                "context_mentions": ["file1.rs", "file2.rs"]
            })),
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains(r#""source_metadata""#));
        assert!(json.contains(r#""claude_specific":"value""#));

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert!(deserialized.source_metadata.is_some());
        let meta = deserialized.source_metadata.unwrap();
        assert_eq!(meta["claude_specific"], "value");
    }

    #[test]
    fn test_message_backward_compat_without_new_fields() {
        // Test deserializing old format without new fields
        let old_json = r#"{
            "role": "user",
            "content_blocks": [{"type": "text", "text": "Hello"}],
            "timestamp": null,
            "mentioned_files": []
        }"#;
        let message: Message = serde_json::from_str(old_json).unwrap();
        assert_eq!(message.role, Role::User);
        assert_eq!(message.message_id, None);
        assert_eq!(message.parent_id, None);
        assert!(!message.is_sidechain);
        assert!(message.source_metadata.is_none());
    }

    #[test]
    fn test_message_skip_none_fields() {
        let message = Message::new(Role::User, vec![ContentBlock::Text { text: "Hello".to_string() }]);
        let json = serde_json::to_string(&message).unwrap();
        // New fields with None values should be skipped
        assert!(!json.contains("message_id"));
        assert!(!json.contains("parent_id"));
        assert!(!json.contains("source_metadata"));
        // is_sidechain with false value is serialized (but may be present)
    }

    // ===== Task 5: 向后兼容性验证测试 =====

    #[test]
    fn test_session_metadata_backward_compat_old_format() {
        // Test deserializing old format SessionMetadata without new fields
        let old_json = r#"{
            "model": "claude-3-opus",
            "total_tokens": 1500,
            "title": "Test Session",
            "original_path": "/path/to/session"
        }"#;
        let metadata: SessionMetadata = serde_json::from_str(old_json).unwrap();
        assert_eq!(metadata.model, Some("claude-3-opus".to_string()));
        assert_eq!(metadata.total_tokens, Some(1500));
        assert_eq!(metadata.title, Some("Test Session".to_string()));
        assert_eq!(metadata.original_path, Some("/path/to/session".to_string()));
        // New fields should be None (default)
        assert!(metadata.git.is_none());
        assert!(metadata.tokens_breakdown.is_none());
        assert!(metadata.instructions.is_none());
        assert!(metadata.source_metadata.is_none());
    }

    #[test]
    fn test_session_metadata_backward_compat_minimal_old_format() {
        // Test with minimal old format (empty object)
        let old_json = r#"{}"#;
        let metadata: SessionMetadata = serde_json::from_str(old_json).unwrap();
        assert!(metadata.model.is_none());
        assert!(metadata.total_tokens.is_none());
        assert!(metadata.title.is_none());
        assert!(metadata.original_path.is_none());
        assert!(metadata.git.is_none());
        assert!(metadata.tokens_breakdown.is_none());
        assert!(metadata.instructions.is_none());
        assert!(metadata.source_metadata.is_none());
    }

    #[test]
    fn test_mantra_session_backward_compat_old_format() {
        // Full session with old format
        let old_json = r#"{
            "id": "session_old",
            "source": "claude",
            "cwd": "/home/user/project",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T01:00:00Z",
            "messages": [
                {
                    "role": "user",
                    "content_blocks": [{"type": "text", "text": "Hello"}],
                    "timestamp": "2025-01-01T00:00:00Z"
                },
                {
                    "role": "assistant",
                    "content_blocks": [{"type": "text", "text": "Hi there!"}],
                    "timestamp": "2025-01-01T00:01:00Z"
                }
            ],
            "metadata": {
                "model": "claude-3-opus",
                "title": "Old Session"
            }
        }"#;
        let session: MantraSession = serde_json::from_str(old_json).unwrap();
        assert_eq!(session.id, "session_old");
        assert_eq!(session.source, "claude");
        assert_eq!(session.messages.len(), 2);

        // Check old message fields work
        assert_eq!(session.messages[0].role, Role::User);

        // Check new message fields are default
        assert!(session.messages[0].message_id.is_none());
        assert!(session.messages[0].parent_id.is_none());
        assert!(!session.messages[0].is_sidechain);
        assert!(session.messages[0].source_metadata.is_none());

        // Check old metadata fields work
        assert_eq!(session.metadata.model, Some("claude-3-opus".to_string()));
        assert_eq!(session.metadata.title, Some("Old Session".to_string()));

        // Check new metadata fields are default
        assert!(session.metadata.git.is_none());
        assert!(session.metadata.tokens_breakdown.is_none());
    }

    #[test]
    fn test_serialization_skip_none_fields_comprehensive() {
        // Verify that serializing then deserializing produces equivalent result
        let metadata = SessionMetadata {
            model: Some("test-model".to_string()),
            total_tokens: None,
            title: None,
            original_path: None,
            git: None,
            tokens_breakdown: None,
            instructions: None,
            source_metadata: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();

        // Verify None fields are not in JSON
        assert!(!json.contains("total_tokens"));
        assert!(!json.contains("title"));
        assert!(!json.contains("original_path"));
        assert!(!json.contains("git"));
        assert!(!json.contains("tokens_breakdown"));
        assert!(!json.contains("instructions"));
        assert!(!json.contains("source_metadata"));

        // Verify only model is present
        assert!(json.contains(r#""model":"test-model""#));

        // Verify roundtrip works
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, metadata.model);
        assert!(deserialized.git.is_none());
    }

    // ===== StandardTool 序列化/反序列化测试 =====

    #[test]
    fn test_standard_tool_file_read_serialization() {
        let tool = StandardTool::FileRead {
            path: "/tmp/test.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""path":"/tmp/test.rs""#));
        assert!(json.contains(r#""start_line":10"#));
        assert!(json.contains(r#""end_line":20"#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_read_skip_none_lines() {
        let tool = StandardTool::FileRead {
            path: "/tmp/test.rs".to_string(),
            start_line: None,
            end_line: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""path":"/tmp/test.rs""#));
        assert!(!json.contains("start_line"));
        assert!(!json.contains("end_line"));
    }

    #[test]
    fn test_standard_tool_file_write_serialization() {
        let tool = StandardTool::FileWrite {
            path: "/tmp/output.txt".to_string(),
            content: "Hello World".to_string(),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_write""#));
        assert!(json.contains(r#""path":"/tmp/output.txt""#));
        assert!(json.contains(r#""content":"Hello World""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_edit_serialization() {
        let tool = StandardTool::FileEdit {
            path: "/tmp/edit.rs".to_string(),
            old_string: Some("old".to_string()),
            new_string: Some("new".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_edit""#));
        assert!(json.contains(r#""path":"/tmp/edit.rs""#));
        assert!(json.contains(r#""old_string":"old""#));
        assert!(json.contains(r#""new_string":"new""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_edit_skip_none_strings() {
        let tool = StandardTool::FileEdit {
            path: "/tmp/edit.rs".to_string(),
            old_string: None,
            new_string: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_edit""#));
        assert!(!json.contains("old_string"));
        assert!(!json.contains("new_string"));
    }

    #[test]
    fn test_standard_tool_shell_exec_serialization() {
        let tool = StandardTool::ShellExec {
            command: "ls -la".to_string(),
            cwd: Some("/home/user".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));
        assert!(json.contains(r#""command":"ls -la""#));
        assert!(json.contains(r#""cwd":"/home/user""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_shell_exec_skip_none_cwd() {
        let tool = StandardTool::ShellExec {
            command: "pwd".to_string(),
            cwd: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));
        assert!(json.contains(r#""command":"pwd""#));
        assert!(!json.contains("cwd"));
    }

    #[test]
    fn test_standard_tool_file_search_serialization() {
        let tool = StandardTool::FileSearch {
            pattern: "*.rs".to_string(),
            path: Some("/src".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_search""#));
        assert!(json.contains(r#""pattern":"*.rs""#));
        assert!(json.contains(r#""path":"/src""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_search_skip_none_path() {
        let tool = StandardTool::FileSearch {
            pattern: "*.txt".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_search""#));
        assert!(json.contains(r#""pattern":"*.txt""#));
        assert!(!json.contains(r#""path""#));
    }

    #[test]
    fn test_standard_tool_content_search_serialization() {
        let tool = StandardTool::ContentSearch {
            pattern: "TODO".to_string(),
            path: Some("/project".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"content_search""#));
        assert!(json.contains(r#""pattern":"TODO""#));
        assert!(json.contains(r#""path":"/project""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_content_search_skip_none_path() {
        let tool = StandardTool::ContentSearch {
            pattern: "FIXME".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"content_search""#));
        assert!(json.contains(r#""pattern":"FIXME""#));
        assert!(!json.contains(r#""path""#));
    }

    #[test]
    fn test_standard_tool_other_serialization() {
        let tool = StandardTool::Other {
            name: "custom_tool".to_string(),
            input: serde_json::json!({"key": "value", "number": 42}),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"other""#));
        assert!(json.contains(r#""name":"custom_tool""#));
        assert!(json.contains(r#""input""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_deserialize_file_read_partial() {
        // Deserialize without optional fields
        let json = r#"{"type":"file_read","path":"/tmp/test.rs"}"#;
        let tool: StandardTool = serde_json::from_str(json).unwrap();
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert!(start_line.is_none());
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead variant"),
        }
    }

    #[test]
    fn test_standard_tool_roundtrip_all_variants() {
        let tools = vec![
            StandardTool::FileRead {
                path: "/test".to_string(),
                start_line: Some(1),
                end_line: Some(10),
            },
            StandardTool::FileWrite {
                path: "/test".to_string(),
                content: "content".to_string(),
            },
            StandardTool::FileEdit {
                path: "/test".to_string(),
                old_string: Some("old".to_string()),
                new_string: Some("new".to_string()),
            },
            StandardTool::ShellExec {
                command: "cmd".to_string(),
                cwd: Some("/dir".to_string()),
            },
            StandardTool::FileSearch {
                pattern: "*.rs".to_string(),
                path: Some("/src".to_string()),
            },
            StandardTool::ContentSearch {
                pattern: "TODO".to_string(),
                path: Some("/".to_string()),
            },
            StandardTool::Other {
                name: "custom".to_string(),
                input: serde_json::json!({}),
            },
        ];

        for tool in tools {
            let json = serde_json::to_string(&tool).unwrap();
            let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, tool);
        }
    }

    // ===== Task 2: ToolUse 向后兼容性测试 =====

    #[test]
    fn test_tool_use_backward_compat_old_format() {
        // Old format without new fields (standard_tool, display_name, description)
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
                input,
                correlation_id,
                standard_tool,
                display_name,
                description,
            } => {
                assert_eq!(id, "tool_123");
                assert_eq!(name, "Read");
                assert_eq!(input["path"], "/tmp/test.txt");
                assert_eq!(correlation_id, Some("corr_123".to_string()));
                // New fields should be None (backward compat)
                assert!(standard_tool.is_none());
                assert!(display_name.is_none());
                assert!(description.is_none());
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn test_tool_use_backward_compat_minimal() {
        // Minimal old format without correlation_id
        let old_json = r#"{
            "type": "tool_use",
            "id": "tool_456",
            "name": "Write",
            "input": {}
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
                assert_eq!(id, "tool_456");
                assert_eq!(name, "Write");
                assert!(correlation_id.is_none());
                assert!(standard_tool.is_none());
                assert!(display_name.is_none());
                assert!(description.is_none());
            }
            _ => panic!("Expected ToolUse variant"),
        }
    }

    #[test]
    fn test_tool_use_with_new_fields() {
        let block = ContentBlock::ToolUse {
            id: "tool_789".to_string(),
            name: "read_file".to_string(),
            input: serde_json::json!({"path": "/test"}),
            correlation_id: Some("corr_789".to_string()),
            standard_tool: Some(StandardTool::FileRead {
                path: "/test".to_string(),
                start_line: None,
                end_line: None,
            }),
            display_name: Some("Read File".to_string()),
            description: Some("Reads file content".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""standard_tool""#));
        assert!(json.contains(r#""display_name":"Read File""#));
        assert!(json.contains(r#""description":"Reads file content""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_tool_use_skip_none_new_fields() {
        let block = ContentBlock::ToolUse {
            id: "tool_abc".to_string(),
            name: "Bash".to_string(),
            input: serde_json::json!({"command": "ls"}),
            correlation_id: None,
            standard_tool: None,
            display_name: None,
            description: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        // None fields should be skipped
        assert!(!json.contains("standard_tool"));
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
        assert!(!json.contains("correlation_id"));
    }

    // ===== Task 3: normalize_tool 函数测试 =====

    // --- Claude Tool Names ---

    #[test]
    fn test_normalize_tool_claude_read() {
        let input = serde_json::json!({"file_path": "/tmp/test.rs"});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert!(start_line.is_none());
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_read_with_lines() {
        // offset=10, limit=100 means: start at line 10, read 100 lines
        // end_line should be offset + limit = 110
        let input = serde_json::json!({"file_path": "/tmp/test.rs", "offset": 10, "limit": 100});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert_eq!(start_line, Some(10));
                assert_eq!(end_line, Some(110)); // 10 + 100 = 110
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_write() {
        let input = serde_json::json!({"file_path": "/tmp/output.txt", "content": "Hello World"});
        let tool = normalize_tool("Write", &input);
        match tool {
            StandardTool::FileWrite { path, content } => {
                assert_eq!(path, "/tmp/output.txt");
                assert_eq!(content, "Hello World");
            }
            _ => panic!("Expected FileWrite"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_edit() {
        let input = serde_json::json!({"file_path": "/tmp/edit.rs", "old_string": "old", "new_string": "new"});
        let tool = normalize_tool("Edit", &input);
        match tool {
            StandardTool::FileEdit { path, old_string, new_string } => {
                assert_eq!(path, "/tmp/edit.rs");
                assert_eq!(old_string, Some("old".to_string()));
                assert_eq!(new_string, Some("new".to_string()));
            }
            _ => panic!("Expected FileEdit"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_bash() {
        let input = serde_json::json!({"command": "ls -la"});
        let tool = normalize_tool("Bash", &input);
        match tool {
            StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "ls -la");
                assert!(cwd.is_none());
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_glob() {
        let input = serde_json::json!({"pattern": "*.rs", "path": "/src"});
        let tool = normalize_tool("Glob", &input);
        match tool {
            StandardTool::FileSearch { pattern, path } => {
                assert_eq!(pattern, "*.rs");
                assert_eq!(path, Some("/src".to_string()));
            }
            _ => panic!("Expected FileSearch"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_grep() {
        let input = serde_json::json!({"pattern": "TODO", "path": "/project"});
        let tool = normalize_tool("Grep", &input);
        match tool {
            StandardTool::ContentSearch { pattern, path } => {
                assert_eq!(pattern, "TODO");
                assert_eq!(path, Some("/project".to_string()));
            }
            _ => panic!("Expected ContentSearch"),
        }
    }

    // --- Gemini Tool Names ---

    #[test]
    fn test_normalize_tool_gemini_read_file() {
        let input = serde_json::json!({"path": "/tmp/test.rs"});
        let tool = normalize_tool("read_file", &input);
        match tool {
            StandardTool::FileRead { path, .. } => {
                assert_eq!(path, "/tmp/test.rs");
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_gemini_write_file() {
        let input = serde_json::json!({"path": "/tmp/output.txt", "content": "Hello"});
        let tool = normalize_tool("write_file", &input);
        match tool {
            StandardTool::FileWrite { path, content } => {
                assert_eq!(path, "/tmp/output.txt");
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected FileWrite"),
        }
    }

    #[test]
    fn test_normalize_tool_gemini_edit_file() {
        let input = serde_json::json!({"path": "/tmp/edit.rs", "old_string": "a", "new_string": "b"});
        let tool = normalize_tool("edit_file", &input);
        match tool {
            StandardTool::FileEdit { path, old_string, new_string } => {
                assert_eq!(path, "/tmp/edit.rs");
                assert_eq!(old_string, Some("a".to_string()));
                assert_eq!(new_string, Some("b".to_string()));
            }
            _ => panic!("Expected FileEdit"),
        }
    }

    #[test]
    fn test_normalize_tool_gemini_run_shell_command() {
        let input = serde_json::json!({"command": "pwd"});
        let tool = normalize_tool("run_shell_command", &input);
        match tool {
            StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "pwd");
                assert!(cwd.is_none());
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    // --- Cursor Tool Names ---

    #[test]
    fn test_normalize_tool_cursor_run_terminal_cmd() {
        let input = serde_json::json!({"command": "npm install", "cwd": "/project"});
        let tool = normalize_tool("run_terminal_cmd", &input);
        match tool {
            StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "npm install");
                assert_eq!(cwd, Some("/project".to_string()));
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    // --- Codex Tool Names ---

    #[test]
    fn test_normalize_tool_codex_shell() {
        let input = serde_json::json!({"command": "cargo build", "cwd": "/workspace"});
        let tool = normalize_tool("shell", &input);
        match tool {
            StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "cargo build");
                assert_eq!(cwd, Some("/workspace".to_string()));
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    #[test]
    fn test_normalize_tool_codex_apply_diff() {
        let input = serde_json::json!({"path": "/tmp/file.rs", "diff": "@@ -1,3 +1,4 @@"});
        let tool = normalize_tool("apply_diff", &input);
        match tool {
            StandardTool::FileEdit { path, old_string, new_string } => {
                assert_eq!(path, "/tmp/file.rs");
                assert!(old_string.is_none());
                assert_eq!(new_string, Some("@@ -1,3 +1,4 @@".to_string()));
            }
            _ => panic!("Expected FileEdit"),
        }
    }

    #[test]
    fn test_normalize_tool_codex_search_files() {
        let input = serde_json::json!({"pattern": "*.md", "path": "/docs"});
        let tool = normalize_tool("search_files", &input);
        match tool {
            StandardTool::FileSearch { pattern, path } => {
                assert_eq!(pattern, "*.md");
                assert_eq!(path, Some("/docs".to_string()));
            }
            _ => panic!("Expected FileSearch"),
        }
    }

    // --- Case Insensitive ---

    #[test]
    fn test_normalize_tool_case_insensitive() {
        // Test various case combinations
        let input = serde_json::json!({"file_path": "/tmp/test"});

        let tool1 = normalize_tool("READ", &input);
        let tool2 = normalize_tool("Read", &input);
        let tool3 = normalize_tool("read", &input);

        match (&tool1, &tool2, &tool3) {
            (
                StandardTool::FileRead { path: p1, .. },
                StandardTool::FileRead { path: p2, .. },
                StandardTool::FileRead { path: p3, .. },
            ) => {
                assert_eq!(p1, p2);
                assert_eq!(p2, p3);
            }
            _ => panic!("Expected all FileRead"),
        }
    }

    // --- Unknown Tool ---

    #[test]
    fn test_normalize_tool_unknown() {
        let input = serde_json::json!({"custom_param": "value"});
        let tool = normalize_tool("CustomTool", &input);
        match tool {
            StandardTool::Other { name, input: tool_input } => {
                assert_eq!(name, "CustomTool");
                assert_eq!(tool_input["custom_param"], "value");
            }
            _ => panic!("Expected Other"),
        }
    }

    #[test]
    fn test_normalize_tool_mcp_tool() {
        let input = serde_json::json!({"query": "test"});
        let tool = normalize_tool("mcp__deepwiki__ask_question", &input);
        match tool {
            StandardTool::Other { name, input: tool_input } => {
                assert_eq!(name, "mcp__deepwiki__ask_question");
                assert_eq!(tool_input["query"], "test");
            }
            _ => panic!("Expected Other"),
        }
    }

    // --- Edge Cases ---

    #[test]
    fn test_normalize_tool_empty_input() {
        let input = serde_json::json!({});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, ""); // Default empty string
                assert!(start_line.is_none());
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_path_priority() {
        // file_path should take priority over path
        let input = serde_json::json!({"file_path": "/from/file_path", "path": "/from/path"});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, .. } => {
                assert_eq!(path, "/from/file_path");
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_start_end_line_priority() {
        // start_line/end_line should take priority over offset/limit
        let input = serde_json::json!({
            "path": "/test",
            "start_line": 5,
            "end_line": 10,
            "offset": 1,
            "limit": 100
        });
        let tool = normalize_tool("read_file", &input);
        match tool {
            StandardTool::FileRead { start_line, end_line, .. } => {
                assert_eq!(start_line, Some(5));
                assert_eq!(end_line, Some(10));
            }
            _ => panic!("Expected FileRead"),
        }
    }

    // ===== Edge Case Tests (MEDIUM-4 coverage) =====

    #[test]
    fn test_normalize_tool_limit_only_without_offset() {
        // When only limit is present (no offset), end_line should be None
        // because we can't calculate end without knowing the start
        let input = serde_json::json!({"path": "/test", "limit": 100});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { start_line, end_line, .. } => {
                assert!(start_line.is_none());
                assert!(end_line.is_none()); // Cannot calculate without offset
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_offset_only_without_limit() {
        // When only offset is present (no limit), end_line should be None
        let input = serde_json::json!({"path": "/test", "offset": 10});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { start_line, end_line, .. } => {
                assert_eq!(start_line, Some(10));
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_string_number_ignored() {
        // String numbers should be ignored (not parsed)
        let input = serde_json::json!({"path": "/test", "start_line": "10", "end_line": "20"});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { start_line, end_line, .. } => {
                assert!(start_line.is_none()); // String "10" is not parsed as number
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_null_content() {
        // Explicit null content should result in empty string
        let input = serde_json::json!({"path": "/test", "content": null});
        let tool = normalize_tool("Write", &input);
        match tool {
            StandardTool::FileWrite { content, .. } => {
                assert_eq!(content, ""); // null becomes empty string
            }
            _ => panic!("Expected FileWrite"),
        }
    }

    #[test]
    fn test_normalize_tool_non_object_input() {
        // Non-object input (string) should still work, fields will be empty/None
        let input = serde_json::json!("not an object");
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, ""); // No path field
                assert!(start_line.is_none());
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_null_input() {
        // Null input should still work
        let input = serde_json::Value::Null;
        let tool = normalize_tool("Bash", &input);
        match tool {
            StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "");
                assert!(cwd.is_none());
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    #[test]
    fn test_normalize_tool_overflow_protection() {
        // Test saturating_add prevents overflow
        let input = serde_json::json!({"path": "/test", "offset": u32::MAX, "limit": 100});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { end_line, .. } => {
                assert_eq!(end_line, Some(u32::MAX)); // saturating_add caps at MAX
            }
            _ => panic!("Expected FileRead"),
        }
    }

    // ===== ToolResultData 序列化/反序列化测试 =====

    #[test]
    fn test_tool_result_data_file_read_serialization() {
        let data = ToolResultData::FileRead {
            file_path: "/tmp/test.rs".to_string(),
            start_line: Some(10),
            num_lines: Some(100),
            total_lines: Some(500),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""file_path":"/tmp/test.rs""#));
        assert!(json.contains(r#""start_line":10"#));
        assert!(json.contains(r#""num_lines":100"#));
        assert!(json.contains(r#""total_lines":500"#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_file_read_skip_none_fields() {
        let data = ToolResultData::FileRead {
            file_path: "/tmp/test.rs".to_string(),
            start_line: None,
            num_lines: None,
            total_lines: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""file_path":"/tmp/test.rs""#));
        assert!(!json.contains("start_line"));
        assert!(!json.contains("num_lines"));
        assert!(!json.contains("total_lines"));
    }

    #[test]
    fn test_tool_result_data_file_write_serialization() {
        let data = ToolResultData::FileWrite {
            file_path: "/tmp/output.txt".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_write""#));
        assert!(json.contains(r#""file_path":"/tmp/output.txt""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_file_edit_serialization() {
        let data = ToolResultData::FileEdit {
            file_path: "/tmp/edit.rs".to_string(),
            old_string: Some("old content".to_string()),
            new_string: Some("new content".to_string()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_edit""#));
        assert!(json.contains(r#""file_path":"/tmp/edit.rs""#));
        assert!(json.contains(r#""old_string":"old content""#));
        assert!(json.contains(r#""new_string":"new content""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_file_edit_skip_none_strings() {
        let data = ToolResultData::FileEdit {
            file_path: "/tmp/edit.rs".to_string(),
            old_string: None,
            new_string: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_edit""#));
        assert!(!json.contains("old_string"));
        assert!(!json.contains("new_string"));
    }

    #[test]
    fn test_tool_result_data_shell_exec_serialization() {
        let data = ToolResultData::ShellExec {
            exit_code: Some(0),
            stdout: Some("output".to_string()),
            stderr: Some("error".to_string()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));
        assert!(json.contains(r#""exit_code":0"#));
        assert!(json.contains(r#""stdout":"output""#));
        assert!(json.contains(r#""stderr":"error""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_shell_exec_skip_none_fields() {
        let data = ToolResultData::ShellExec {
            exit_code: None,
            stdout: None,
            stderr: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));
        assert!(!json.contains("exit_code"));
        assert!(!json.contains("stdout"));
        assert!(!json.contains("stderr"));
    }

    #[test]
    fn test_tool_result_data_other_serialization() {
        let data = ToolResultData::Other {
            data: serde_json::json!({"key": "value", "number": 42}),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"other""#));
        assert!(json.contains(r#""data""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_deserialize_partial_file_read() {
        // Deserialize without optional fields
        let json = r#"{"type":"file_read","file_path":"/tmp/test.rs"}"#;
        let data: ToolResultData = serde_json::from_str(json).unwrap();
        match data {
            ToolResultData::FileRead { file_path, start_line, num_lines, total_lines } => {
                assert_eq!(file_path, "/tmp/test.rs");
                assert!(start_line.is_none());
                assert!(num_lines.is_none());
                assert!(total_lines.is_none());
            }
            _ => panic!("Expected FileRead variant"),
        }
    }

    #[test]
    fn test_tool_result_data_roundtrip_all_variants() {
        let variants = vec![
            ToolResultData::FileRead {
                file_path: "/test".to_string(),
                start_line: Some(1),
                num_lines: Some(10),
                total_lines: Some(100),
            },
            ToolResultData::FileWrite {
                file_path: "/test".to_string(),
            },
            ToolResultData::FileEdit {
                file_path: "/test".to_string(),
                old_string: Some("old".to_string()),
                new_string: Some("new".to_string()),
            },
            ToolResultData::ShellExec {
                exit_code: Some(0),
                stdout: Some("out".to_string()),
                stderr: Some("err".to_string()),
            },
            ToolResultData::Other {
                data: serde_json::json!({"custom": true}),
            },
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    // ===== ToolResult 扩展字段测试 =====

    #[test]
    fn test_tool_result_with_structured_result() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_123".to_string(),
            content: "File read successfully".to_string(),
            is_error: false,
            correlation_id: Some("tool_123".to_string()),
            structured_result: Some(ToolResultData::FileRead {
                file_path: "/tmp/test.rs".to_string(),
                start_line: Some(1),
                num_lines: Some(100),
                total_lines: Some(500),
            }),
            display_content: None,
            render_as_markdown: None,
            user_decision: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""structured_result""#));
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""file_path":"/tmp/test.rs""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_tool_result_with_display_content() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_456".to_string(),
            content: "raw content".to_string(),
            is_error: false,
            correlation_id: None,
            structured_result: None,
            display_content: Some("**Formatted** content".to_string()),
            render_as_markdown: Some(true),
            user_decision: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""display_content":"**Formatted** content""#));
        assert!(json.contains(r#""render_as_markdown":true"#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_tool_result_with_user_decision() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_789".to_string(),
            content: "Approved action result".to_string(),
            is_error: false,
            correlation_id: None,
            structured_result: None,
            display_content: None,
            render_as_markdown: None,
            user_decision: Some("approved".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""user_decision":"approved""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_tool_result_backward_compat() {
        // Old format JSON without new fields
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
                correlation_id,
                structured_result,
                display_content,
                render_as_markdown,
                user_decision,
            } => {
                assert_eq!(tool_use_id, "123");
                assert_eq!(content, "file content");
                assert!(!is_error);
                assert!(correlation_id.is_none());
                // New fields should be None (backward compat)
                assert!(structured_result.is_none());
                assert!(display_content.is_none());
                assert!(render_as_markdown.is_none());
                assert!(user_decision.is_none());
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_tool_result_backward_compat_with_correlation_id() {
        // Old format with correlation_id but without new fields
        let old_json = r#"{
            "type": "tool_result",
            "tool_use_id": "456",
            "content": "result",
            "is_error": true,
            "correlation_id": "corr_456"
        }"#;
        let block: ContentBlock = serde_json::from_str(old_json).unwrap();
        match block {
            ContentBlock::ToolResult {
                correlation_id,
                structured_result,
                display_content,
                render_as_markdown,
                user_decision,
                ..
            } => {
                assert_eq!(correlation_id, Some("corr_456".to_string()));
                assert!(structured_result.is_none());
                assert!(display_content.is_none());
                assert!(render_as_markdown.is_none());
                assert!(user_decision.is_none());
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_tool_result_skip_none_new_fields() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "tool_abc".to_string(),
            content: "result".to_string(),
            is_error: false,
            correlation_id: None,
            structured_result: None,
            display_content: None,
            render_as_markdown: None,
            user_decision: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        // None fields should be skipped
        assert!(!json.contains("structured_result"));
        assert!(!json.contains("display_content"));
        assert!(!json.contains("render_as_markdown"));
        assert!(!json.contains("user_decision"));
        assert!(!json.contains("correlation_id"));
    }

    #[test]
    fn test_tool_result_all_new_fields() {
        let block = ContentBlock::ToolResult {
            tool_use_id: "complete_tool".to_string(),
            content: "raw result".to_string(),
            is_error: false,
            correlation_id: Some("complete_tool".to_string()),
            structured_result: Some(ToolResultData::ShellExec {
                exit_code: Some(0),
                stdout: Some("output".to_string()),
                stderr: None,
            }),
            display_content: Some("Formatted output".to_string()),
            render_as_markdown: Some(false),
            user_decision: Some("approved".to_string()),
        };
        let json = serde_json::to_string_pretty(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    // ===== Story 8.4: Thinking 块扩展字段测试 =====

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
    fn test_thinking_skip_none_fields() {
        let block = ContentBlock::Thinking {
            thinking: "Just thinking...".to_string(),
            subject: None,
            timestamp: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"thinking""#));
        assert!(json.contains(r#""thinking":"Just thinking...""#));
        // None fields should be skipped
        assert!(!json.contains("subject"));
        assert!(!json.contains("timestamp"));
    }

    #[test]
    fn test_thinking_backward_compat() {
        // Old format JSON without new fields (subject, timestamp)
        let old_json = r#"{
            "type": "thinking",
            "thinking": "Let me think about this..."
        }"#;
        let block: ContentBlock = serde_json::from_str(old_json).unwrap();
        match block {
            ContentBlock::Thinking { thinking, subject, timestamp } => {
                assert_eq!(thinking, "Let me think about this...");
                assert!(subject.is_none());
                assert!(timestamp.is_none());
            }
            _ => panic!("Expected Thinking"),
        }
    }

    #[test]
    fn test_thinking_partial_fields() {
        // JSON with only subject, no timestamp
        let json_subject_only = r#"{
            "type": "thinking",
            "thinking": "Analyzing...",
            "subject": "Code Analysis"
        }"#;
        let block: ContentBlock = serde_json::from_str(json_subject_only).unwrap();
        match block {
            ContentBlock::Thinking { thinking, subject, timestamp } => {
                assert_eq!(thinking, "Analyzing...");
                assert_eq!(subject, Some("Code Analysis".to_string()));
                assert!(timestamp.is_none());
            }
            _ => panic!("Expected Thinking"),
        }

        // JSON with only timestamp, no subject
        let json_timestamp_only = r#"{
            "type": "thinking",
            "thinking": "Processing...",
            "timestamp": "2025-12-30T20:00:00.000Z"
        }"#;
        let block: ContentBlock = serde_json::from_str(json_timestamp_only).unwrap();
        match block {
            ContentBlock::Thinking { thinking, subject, timestamp } => {
                assert_eq!(thinking, "Processing...");
                assert!(subject.is_none());
                assert_eq!(timestamp, Some("2025-12-30T20:00:00.000Z".to_string()));
            }
            _ => panic!("Expected Thinking"),
        }
    }

    #[test]
    fn test_thinking_roundtrip_all_fields() {
        let block = ContentBlock::Thinking {
            thinking: "**Deep Analysis**\nLet me carefully analyze this complex problem step by step.".to_string(),
            subject: Some("Deep Analysis".to_string()),
            timestamp: Some("2025-12-30T20:05:30.123Z".to_string()),
        };
        let json = serde_json::to_string_pretty(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_thinking_empty_strings() {
        // Edge case: empty strings for optional fields
        let block = ContentBlock::Thinking {
            thinking: "".to_string(),
            subject: Some("".to_string()),
            timestamp: Some("".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_thinking_unicode_content() {
        // Edge case: Unicode content in all fields
        let block = ContentBlock::Thinking {
            thinking: "**问题分析**\n让我仔细分析这个问题...".to_string(),
            subject: Some("问题分析".to_string()),
            timestamp: Some("2025-12-30T20:00:55.000Z".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("问题分析"));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    // ===== Story 8.5: CodeSuggestion 块测试 =====

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
        assert!(json.contains(r#""code":"fn main() {}\n""#));
        assert!(json.contains(r#""language":"rust""#));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_skip_none_language() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/src/main.rs".to_string(),
            code: "fn main() {}".to_string(),
            language: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"code_suggestion""#));
        assert!(json.contains(r#""file_path":"/src/main.rs""#));
        assert!(!json.contains("language"));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_roundtrip() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/path/to/file.ts".to_string(),
            code: "export const foo = 42;".to_string(),
            language: Some("typescript".to_string()),
        };
        let json = serde_json::to_string_pretty(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_deserialize_without_language() {
        let json = r#"{"type":"code_suggestion","file_path":"/test.py","code":"print('hello')"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::CodeSuggestion { file_path, code, language } => {
                assert_eq!(file_path, "/test.py");
                assert_eq!(code, "print('hello')");
                assert!(language.is_none());
            }
            _ => panic!("Expected CodeSuggestion variant"),
        }
    }

    #[test]
    fn test_code_suggestion_multiline_code() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/src/lib.rs".to_string(),
            code: "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n".to_string(),
            language: Some("rust".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_unicode_content() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/src/i18n.ts".to_string(),
            code: "const greeting = '你好世界';".to_string(),
            language: Some("typescript".to_string()),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("你好世界"));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }

    #[test]
    fn test_code_suggestion_empty_code() {
        let block = ContentBlock::CodeSuggestion {
            file_path: "/empty.txt".to_string(),
            code: "".to_string(),
            language: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block);
    }
}
