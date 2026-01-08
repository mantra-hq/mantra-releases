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
}
