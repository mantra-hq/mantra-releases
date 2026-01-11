//! Session type definitions
//!
//! Contains basic types, enums, and helper structures used throughout the session module.

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

/// Unknown format entry for tracking unrecognized content during parsing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UnknownFormatEntry {
    /// Source parser (claude, gemini, cursor, codex)
    pub source: String,

    /// Unknown type name encountered
    pub type_name: String,

    /// Original JSON (truncated to 1KB)
    pub raw_json: String,

    /// Timestamp when detected
    pub timestamp: String,
}

/// Parser information for version compatibility tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ParserInfo {
    /// Parser version (e.g., "1.0.0")
    pub parser_version: String,

    /// List of supported format types
    pub supported_formats: Vec<String>,

    /// Detected source tool version (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_source_version: Option<String>,
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

    /// Unknown formats encountered during parsing (for monitoring)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_formats: Option<Vec<UnknownFormatEntry>>,

    /// Parser information for version compatibility tracking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser_info: Option<ParserInfo>,
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// Role of the message sender
    pub role: Role,

    /// Content blocks in this message
    pub content_blocks: Vec<super::ContentBlock>,

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
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
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
        !self.messages.iter().any(|m| m.role == Role::User || m.role == Role::Assistant)
    }
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content_blocks: Vec<super::ContentBlock>) -> Self {
        Self {
            role,
            content_blocks,
            timestamp: None,
            mentioned_files: Vec::new(),
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        }
    }

    /// Create a new message with mentioned files
    pub fn with_mentioned_files(role: Role, content_blocks: Vec<super::ContentBlock>, mentioned_files: Vec<String>) -> Self {
        Self {
            role,
            content_blocks,
            timestamp: None,
            mentioned_files,
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        }
    }

    /// Create a text-only message
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self::new(role, vec![super::ContentBlock::Text { text: text.into(), is_degraded: None }])
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
            unknown_formats: None,
            parser_info: None,
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
            unknown_formats: None,
            parser_info: None,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""tokens_breakdown""#));
        assert!(json.contains(r#""input":1000"#));
        assert!(!json.contains("thoughts"));

        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert!(deserialized.tokens_breakdown.is_some());
    }

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
}
