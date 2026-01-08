//! Gemini CLI log parser
//!
//! Parses conversation logs from Gemini CLI into MantraSession format.
//! Gemini CLI stores conversations in JSON files located at:
//! - ~/.gemini/tmp/{projectHash}/chats/session-{date}-{uuid}.json
//!
//! ## Supported Features
//!
//! - User and Gemini messages with full content
//! - Thoughts/reasoning (extended thinking)
//! - Tool calls with results
//! - Timestamp preservation
//! - Cross-platform path resolution

pub mod path;
pub mod types;

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

use super::{LogParser, ParseError};
use crate::models::{sources, ContentBlock, MantraSession, Message, SessionMetadata};

pub use path::{get_gemini_dir, get_gemini_tmp_dir, GeminiPaths, GeminiSessionFile};
pub use types::*;

/// Parser for Gemini CLI conversation logs
#[derive(Debug, Default)]
pub struct GeminiParser {
    /// Optional project path override for cwd
    project_path: Option<String>,
}

impl GeminiParser {
    /// Create a new GeminiParser instance
    pub fn new() -> Self {
        Self { project_path: None }
    }

    /// Create a parser with a specific project path for cwd
    pub fn with_project_path(project_path: String) -> Self {
        Self {
            project_path: Some(project_path),
        }
    }

    /// Parse a Gemini conversation JSON file
    fn parse_json(&self, content: &str, file_path: Option<&str>) -> Result<MantraSession, ParseError> {
        let conversation: GeminiConversation =
            serde_json::from_str(content).map_err(|e| ParseError::invalid_format(format!("Invalid JSON: {}", e)))?;

        // Validate required fields
        if conversation.session_id.is_empty() {
            return Err(ParseError::missing_field("sessionId"));
        }

        // Parse timestamps
        let created_at = parse_timestamp(&conversation.start_time)?;
        let updated_at = parse_timestamp(&conversation.last_updated)?;

        // Determine cwd - use project_path if available, otherwise use project_hash as fallback
        let cwd = self
            .project_path
            .clone()
            .unwrap_or_else(|| format!("gemini-project:{}", conversation.project_hash));

        // Build messages and aggregate tokens
        let mut messages = Vec::new();
        let mut last_model: Option<String> = None;
        let mut total_tokens: u64 = 0;

        for gemini_msg in &conversation.messages {
            // Skip non-includable messages (info, error, warning)
            if !gemini_msg.msg_type.should_include() {
                continue;
            }

            // Track model from gemini messages
            if let Some(model) = &gemini_msg.model {
                last_model = Some(model.clone());
            }

            // Aggregate tokens from gemini messages
            // Prefer the authoritative 'total' field if available
            if let Some(tokens) = &gemini_msg.tokens {
                if let Some(total) = tokens.total {
                    total_tokens += total.max(0) as u64;
                } else {
                    // Fallback: sum input + output if total not available
                    if let Some(input) = tokens.input {
                        total_tokens += input.max(0) as u64;
                    }
                    if let Some(output) = tokens.output {
                        total_tokens += output.max(0) as u64;
                    }
                }
            }

            let converted = self.convert_message(gemini_msg)?;
            messages.extend(converted);
        }

        let mut session = MantraSession {
            id: conversation.session_id,
            source: sources::GEMINI.to_string(),
            cwd,
            created_at,
            updated_at,
            messages,
            metadata: SessionMetadata {
                model: last_model,
                title: conversation.summary,
                original_path: file_path.map(String::from),
                total_tokens: if total_tokens > 0 { Some(total_tokens) } else { None },
                ..Default::default()
            },
        };

        // Update updated_at to match the last message timestamp if available
        if let Some(last_msg) = session.messages.last() {
            if let Some(ts) = last_msg.timestamp {
                session.updated_at = ts;
            }
        }

        Ok(session)
    }


    /// Convert a Gemini message to Mantra Messages
    ///
    /// Mantra 消息结构规范：
    /// 1. 文本消息 (thinking + text) → 一条消息
    /// 2. 工具调用消息 (tool_use + tool_result) → 每个工具调用一条独立消息
    ///
    /// 这样设计确保每个消息是语义完整的单元，便于前端渲染和理解。
    fn convert_message(&self, gemini_msg: &GeminiMessage) -> Result<Vec<Message>, ParseError> {
        let role = match gemini_msg.msg_type.to_mantra_role() {
            Some(r) => r,
            None => return Ok(Vec::new()), // Skip messages with unknown role
        };

        let mut messages = Vec::new();
        let timestamp = parse_timestamp(&gemini_msg.timestamp).ok();

        // === 消息 1: 思考 + 文本内容 ===
        let mut text_blocks = Vec::new();

        // Add thoughts first (for Gemini messages)
        if let Some(thoughts) = &gemini_msg.thoughts {
            for thought in thoughts {
                text_blocks.push(ContentBlock::Thinking {
                    thinking: thought.as_formatted_string(),
                });
            }
        }

        // Add text content
        if !gemini_msg.content.is_empty() {
            let text = gemini_msg.content.as_text();
            if !text.is_empty() {
                text_blocks.push(ContentBlock::Text { text });
            }
        }

        // Create text message if we have content
        if !text_blocks.is_empty() {
            messages.push(Message {
                role: role.clone(),
                content_blocks: text_blocks,
                timestamp,
                mentioned_files: Vec::new(),
                message_id: None,
                parent_id: None,
                is_sidechain: false,
                source_metadata: None,
            });
        }

        // === 消息 2+: 每个工具调用作为独立消息 ===
        if let Some(tool_calls) = &gemini_msg.tool_calls {
            for tool_call in tool_calls {
                let mut tool_blocks = Vec::new();
                let mut mentioned_files = Vec::new();

                // Extract file paths from tool call arguments
                Self::extract_file_paths(&tool_call.args, &mut mentioned_files);

                // Generate correlation_id using tool_call.id (deterministic)
                let correlation_id = Some(tool_call.id.clone());

                // Add ToolUse
                tool_blocks.push(ContentBlock::ToolUse {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    input: tool_call.args.clone(),
                    correlation_id: correlation_id.clone(),
                    standard_tool: None,
                    display_name: None,
                    description: None,
                });

                // Add ToolResult if available
                if let Some(results) = &tool_call.result {
                    for result_wrapper in results {
                        let response = &result_wrapper.function_response;
                        let is_error = tool_call.status == "error";
                        let content = response.response.as_content();

                        tool_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            content,
                            is_error,
                            correlation_id: correlation_id.clone(),
                        });
                    }
                }

                // Parse tool call timestamp if available, fallback to message timestamp
                let tool_timestamp = tool_call.timestamp.as_deref()
                    .and_then(|ts| parse_timestamp(ts).ok())
                    .or(timestamp);

                // Create tool action message
                messages.push(Message {
                    role: role.clone(),
                    content_blocks: tool_blocks,
                    timestamp: tool_timestamp,
                    mentioned_files,
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                });
            }
        }

        Ok(messages)
    }

    /// Extract file paths from tool call arguments
    fn extract_file_paths(args: &serde_json::Value, files: &mut Vec<String>) {
        // Common field names for file paths in Gemini CLI tools
        const PATH_FIELDS: &[&str] = &[
            "path", "file_path", "target_file", "source_file", "filename",
            "file", "target", "source", "destination", "cwd", "directory",
        ];

        if let Some(obj) = args.as_object() {
            for field in PATH_FIELDS {
                if let Some(value) = obj.get(*field) {
                    if let Some(s) = value.as_str() {
                        if !s.is_empty() && (s.starts_with('/') || s.starts_with('.') || s.contains('/')) {
                            files.push(s.to_string());
                        }
                    }
                }
            }
            // Handle "paths" array field
            if let Some(paths) = obj.get("paths") {
                if let Some(arr) = paths.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            if !s.is_empty() {
                                files.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }


    /// Parse all sessions from the Gemini CLI directory
    pub fn parse_all(&self) -> Result<Vec<MantraSession>, ParseError> {
        let paths = GeminiPaths::detect()?;
        let sessions_files = paths.scan_all_sessions()?;

        let mut sessions = Vec::new();
        for session_file in sessions_files {
            match self.parse_file(session_file.path.to_string_lossy().as_ref()) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    // Log but continue with other sessions
                    eprintln!(
                        "Warning: Failed to parse {}: {}",
                        session_file.path.display(),
                        e
                    );
                }
            }
        }

        Ok(sessions)
    }

    /// Parse all sessions for a specific project hash
    pub fn parse_project(&self, project_hash: &str) -> Result<Vec<MantraSession>, ParseError> {
        let paths = GeminiPaths::detect()?;
        let session_files = paths.scan_sessions(project_hash)?;

        let mut sessions = Vec::new();
        for session_file in session_files {
            match self.parse_file(session_file.path.to_string_lossy().as_ref()) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse {}: {}",
                        session_file.path.display(),
                        e
                    );
                }
            }
        }

        Ok(sessions)
    }
}

impl LogParser for GeminiParser {
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError> {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ParseError::invalid_format(format!("File not found: {}", path)));
        }

        let content = fs::read_to_string(path_obj)?;

        self.parse_json(&content, Some(path))
    }

    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError> {
        self.parse_json(content, None)
    }
}

/// Parse an ISO 8601 timestamp string to DateTime<Utc>
fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, ParseError> {
    // Try parsing with different formats
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try alternative format without timezone
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.3f")
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|_| {
            // Try format with Z suffix
            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| ParseError::invalid_format(format!("Invalid timestamp '{}': {}", timestamp, e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Role;
    use chrono::Datelike;

    const SIMPLE_CONVERSATION: &str = r#"{
        "sessionId": "test-session-123",
        "projectHash": "abc456def789",
        "startTime": "2025-12-30T20:11:00.000Z",
        "lastUpdated": "2025-12-30T20:15:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:11:18.207Z",
                "type": "user",
                "content": "Help me with this code"
            },
            {
                "id": "msg-2",
                "timestamp": "2025-12-30T20:13:18.207Z",
                "type": "gemini",
                "content": "I'll help you with that.",
                "model": "gemini-3-pro-preview"
            }
        ]
    }"#;

    const CONVERSATION_WITH_THOUGHTS: &str = r#"{
        "sessionId": "thought-session",
        "projectHash": "project123",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:05:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "user",
                "content": "Explain this problem"
            },
            {
                "id": "msg-2",
                "timestamp": "2025-12-30T20:01:00.000Z",
                "type": "gemini",
                "content": "Let me analyze this.",
                "thoughts": [
                    {
                        "subject": "Problem Analysis",
                        "description": "The user is asking about understanding a problem.",
                        "timestamp": "2025-12-30T20:00:55.000Z"
                    }
                ]
            }
        ]
    }"#;

    const CONVERSATION_WITH_TOOL_CALLS: &str = r#"{
        "sessionId": "tool-session",
        "projectHash": "tools123",
        "startTime": "2025-12-30T21:00:00.000Z",
        "lastUpdated": "2025-12-30T21:05:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T21:00:10.000Z",
                "type": "user",
                "content": "List files in current directory"
            },
            {
                "id": "msg-2",
                "timestamp": "2025-12-30T21:01:00.000Z",
                "type": "gemini",
                "content": "I'll list the files for you.",
                "toolCalls": [
                    {
                        "id": "run_shell_command-123",
                        "name": "run_shell_command",
                        "args": {"command": "ls -la"},
                        "result": [
                            {
                                "functionResponse": {
                                    "id": "run_shell_command-123",
                                    "name": "run_shell_command",
                                    "response": {
                                        "output": "total 0\ndrwxr-xr-x 2 user user 40 Dec 30 21:00 ."
                                    }
                                }
                            }
                        ],
                        "status": "success",
                        "timestamp": "2025-12-30T21:01:05.000Z"
                    }
                ]
            }
        ]
    }"#;

    const CONVERSATION_WITH_SYSTEM_MESSAGES: &str = r#"{
        "sessionId": "system-session",
        "projectHash": "sys123",
        "startTime": "2025-12-30T22:00:00.000Z",
        "lastUpdated": "2025-12-30T22:05:00.000Z",
        "messages": [
            {
                "id": "info-1",
                "timestamp": "2025-12-30T22:00:00.000Z",
                "type": "info",
                "content": "Session started"
            },
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T22:00:10.000Z",
                "type": "user",
                "content": "Hello"
            },
            {
                "id": "error-1",
                "timestamp": "2025-12-30T22:00:20.000Z",
                "type": "error",
                "content": "API rate limit exceeded"
            },
            {
                "id": "msg-2",
                "timestamp": "2025-12-30T22:01:00.000Z",
                "type": "gemini",
                "content": "Hello! How can I help?"
            },
            {
                "id": "warning-1",
                "timestamp": "2025-12-30T22:02:00.000Z",
                "type": "warning",
                "content": "Token limit approaching"
            }
        ]
    }"#;

    #[test]
    fn test_parse_simple_conversation() {
        let parser = GeminiParser::new();
        let session = parser.parse_string(SIMPLE_CONVERSATION).unwrap();

        assert_eq!(session.id, "test-session-123");
        assert_eq!(session.source, sources::GEMINI);
        assert!(session.cwd.contains("abc456def789"));
        assert_eq!(session.messages.len(), 2);

        // Check user message
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[0].content_blocks.len(), 1);
        match &session.messages[0].content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "Help me with this code"),
            _ => panic!("Expected Text block"),
        }

        // Check gemini message
        assert_eq!(session.messages[1].role, Role::Assistant);
        assert_eq!(session.metadata.model, Some("gemini-3-pro-preview".to_string()));
    }

    #[test]
    fn test_parse_conversation_with_thoughts() {
        let parser = GeminiParser::new();
        let session = parser.parse_string(CONVERSATION_WITH_THOUGHTS).unwrap();

        assert_eq!(session.messages.len(), 2);

        // Check gemini message with thoughts
        let gemini_msg = &session.messages[1];
        assert_eq!(gemini_msg.role, Role::Assistant);
        assert!(gemini_msg.content_blocks.len() >= 2);

        // First block should be thinking
        match &gemini_msg.content_blocks[0] {
            ContentBlock::Thinking { thinking } => {
                assert!(thinking.contains("Problem Analysis"));
                assert!(thinking.contains("understanding a problem"));
            }
            _ => panic!("Expected Thinking block"),
        }

        // Second block should be text
        match &gemini_msg.content_blocks[1] {
            ContentBlock::Text { text } => assert_eq!(text, "Let me analyze this."),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_parse_conversation_with_tool_calls() {
        let parser = GeminiParser::new();
        let session = parser.parse_string(CONVERSATION_WITH_TOOL_CALLS).unwrap();

        // 新结构：user + assistant text + assistant tool_action
        assert_eq!(session.messages.len(), 3);

        // 消息 0: user
        assert_eq!(session.messages[0].role, Role::User);

        // 消息 1: assistant 文本消息
        let text_msg = &session.messages[1];
        assert_eq!(text_msg.role, Role::Assistant);
        assert_eq!(text_msg.content_blocks.len(), 1);
        match &text_msg.content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "I'll list the files for you."),
            _ => panic!("Expected Text block"),
        }

        // 消息 2: assistant 工具调用消息 (tool_use + tool_result)
        let tool_msg = &session.messages[2];
        assert_eq!(tool_msg.role, Role::Assistant);
        assert_eq!(tool_msg.content_blocks.len(), 2); // tool_use + tool_result

        // Check ToolUse block
        match &tool_msg.content_blocks[0] {
            ContentBlock::ToolUse { id, name, input, .. } => {
                assert_eq!(id, "run_shell_command-123");
                assert_eq!(name, "run_shell_command");
                assert_eq!(input["command"], "ls -la");
            }
            _ => panic!("Expected ToolUse block"),
        }

        // Check ToolResult block
        match &tool_msg.content_blocks[1] {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                is_error,
                ..
            } => {
                assert_eq!(tool_use_id, "run_shell_command-123");
                assert!(content.contains("drwxr-xr-x"));
                assert!(!is_error);
            }
            _ => panic!("Expected ToolResult block"),
        }
    }

    #[test]
    fn test_system_messages_filtered() {
        let parser = GeminiParser::new();
        let session = parser.parse_string(CONVERSATION_WITH_SYSTEM_MESSAGES).unwrap();

        // Only user and gemini messages should be included
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[1].role, Role::Assistant);
    }

    #[test]
    fn test_parse_with_project_path() {
        let parser = GeminiParser::with_project_path("/home/user/my-project".to_string());
        let session = parser.parse_string(SIMPLE_CONVERSATION).unwrap();

        assert_eq!(session.cwd, "/home/user/my-project");
    }

    #[test]
    fn test_parse_empty_session_id_fails() {
        let json = r#"{
            "sessionId": "",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": []
        }"#;

        let parser = GeminiParser::new();
        let result = parser.parse_string(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json_fails() {
        let parser = GeminiParser::new();
        let result = parser.parse_string("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_messages_ok() {
        let json = r#"{
            "sessionId": "test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z"
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_parse_empty_content_message() {
        let json = r#"{
            "sessionId": "test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [
                {
                    "id": "msg-1",
                    "timestamp": "2025-12-30T20:00:10.000Z",
                    "type": "gemini",
                    "content": "",
                    "toolCalls": [
                        {
                            "id": "tool-1",
                            "name": "read_file",
                            "args": {"path": "/tmp/test"},
                            "status": "success"
                        }
                    ]
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();
        // Message should exist with just the tool call
        assert_eq!(session.messages.len(), 1);
        // Should have ToolUse but no Text block
        let msg = &session.messages[0];
        assert!(msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. })));
        assert!(!msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::Text { .. })));
    }

    #[test]
    fn test_parse_timestamp_formats() {
        // RFC 3339 with timezone
        let ts = parse_timestamp("2025-12-30T20:11:00.000Z").unwrap();
        assert_eq!(ts.year(), 2025);

        // RFC 3339 with offset
        let ts = parse_timestamp("2025-12-30T20:11:00.000+08:00").unwrap();
        assert_eq!(ts.year(), 2025);
    }

    #[test]
    fn test_parse_error_tool_call() {
        let json = r#"{
            "sessionId": "error-test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [
                {
                    "id": "msg-1",
                    "timestamp": "2025-12-30T20:00:10.000Z",
                    "type": "gemini",
                    "content": "Let me try that.",
                    "toolCalls": [
                        {
                            "id": "tool-1",
                            "name": "run_command",
                            "args": {"command": "rm -rf /"},
                            "result": [
                                {
                                    "functionResponse": {
                                        "id": "tool-1",
                                        "name": "run_command",
                                        "response": {
                                            "error": "Permission denied"
                                        }
                                    }
                                }
                            ],
                            "status": "error"
                        }
                    ]
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();

        // 新结构：消息 0 是文本，消息 1 是工具调用
        assert_eq!(session.messages.len(), 2);

        let tool_msg = &session.messages[1];
        let tool_result = tool_msg.content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolResult { .. }));
        assert!(tool_result.is_some());

        match tool_result.unwrap() {
            ContentBlock::ToolResult {
                is_error, content, ..
            } => {
                assert!(is_error);
                assert!(content.contains("Permission denied"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_summary_becomes_title() {
        let json = r#"{
            "sessionId": "summary-test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [],
            "summary": "Discussion about Rust parsing"
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();
        assert_eq!(
            session.metadata.title,
            Some("Discussion about Rust parsing".to_string())
        );
    }

    #[test]
    fn test_content_with_parts_array() {
        let json = r#"{
            "sessionId": "parts-test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [
                {
                    "id": "msg-1",
                    "timestamp": "2025-12-30T20:00:10.000Z",
                    "type": "gemini",
                    "content": [
                        {"text": "First part. "},
                        {"text": "Second part."}
                    ]
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();

        match &session.messages[0].content_blocks[0] {
            ContentBlock::Text { text } => assert_eq!(text, "First part. Second part."),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_mentioned_files_extracted_from_tool_calls() {
        let json = r#"{
            "sessionId": "files-test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [
                {
                    "id": "msg-1",
                    "timestamp": "2025-12-30T20:00:10.000Z",
                    "type": "gemini",
                    "content": "I'll read the file.",
                    "toolCalls": [
                        {
                            "id": "read-1",
                            "name": "read_file",
                            "args": {"path": "/src/main.rs"},
                            "status": "success"
                        },
                        {
                            "id": "write-1",
                            "name": "write_file",
                            "args": {"target_file": "/src/lib.rs", "content": "test"},
                            "status": "success"
                        }
                    ]
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();

        // 新结构：消息 0 是文本，消息 1 和 2 是工具调用
        assert_eq!(session.messages.len(), 3);

        // 工具调用消息各自包含自己的 mentioned_files
        let tool_msg_1 = &session.messages[1];
        assert!(tool_msg_1.mentioned_files.contains(&"/src/main.rs".to_string()));

        let tool_msg_2 = &session.messages[2];
        assert!(tool_msg_2.mentioned_files.contains(&"/src/lib.rs".to_string()));
    }

    #[test]
    fn test_total_tokens_aggregated() {
        let json = r#"{
            "sessionId": "tokens-test",
            "projectHash": "abc",
            "startTime": "2025-12-30T20:00:00.000Z",
            "lastUpdated": "2025-12-30T20:00:00.000Z",
            "messages": [
                {
                    "id": "msg-1",
                    "timestamp": "2025-12-30T20:00:10.000Z",
                    "type": "user",
                    "content": "Hello"
                },
                {
                    "id": "msg-2",
                    "timestamp": "2025-12-30T20:01:00.000Z",
                    "type": "gemini",
                    "content": "Hi there!",
                    "tokens": {"input": 100, "output": 50}
                },
                {
                    "id": "msg-3",
                    "timestamp": "2025-12-30T20:02:00.000Z",
                    "type": "gemini",
                    "content": "Here's more info.",
                    "tokens": {"input": 200, "output": 150}
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();

        // Should aggregate: 100 + 50 + 200 + 150 = 500
        assert_eq!(session.metadata.total_tokens, Some(500));
    }

    #[test]
    fn test_parse_real_world_tool_call_format() {
        // Based on actual Gemini CLI output format
        let json = r#"{
            "sessionId": "8c9a7d96-6b65-4e36-9540-0484bc3e3eb2",
            "projectHash": "3f39cb10e8c4f4196f80e8daa74c4d17f88708c17dcf1ae0d56da82a11a9f941",
            "startTime": "2025-12-30T20:11:51.773Z",
            "lastUpdated": "2025-12-30T20:23:26.927Z",
            "messages": [
                {
                    "id": "61a5e2cb-2840-4249-93cb-3406903fa0e1",
                    "timestamp": "2025-12-30T20:11:51.774Z",
                    "type": "user",
                    "content": "请帮我分析这个问题"
                },
                {
                    "id": "b20dd04d-74a5-47dd-913c-8232f30231b9",
                    "timestamp": "2025-12-30T20:12:02.838Z",
                    "type": "gemini",
                    "content": "我来分析一下",
                    "thoughts": [
                        {
                            "subject": "Examining System",
                            "description": "Analyzing the problem",
                            "timestamp": "2025-12-30T20:11:55.073Z"
                        }
                    ],
                    "tokens": {
                        "input": 6926,
                        "output": 246,
                        "cached": 0,
                        "thoughts": 651,
                        "tool": 0,
                        "total": 7823
                    },
                    "model": "gemini-3-pro-preview",
                    "toolCalls": [
                        {
                            "id": "run_shell_command-1767125522547",
                            "name": "run_shell_command",
                            "args": {
                                "command": "hostnamectl"
                            },
                            "result": [
                                {
                                    "functionResponse": {
                                        "id": "run_shell_command-1767125522547",
                                        "name": "run_shell_command",
                                        "response": {
                                            "output": "Static hostname: test-machine"
                                        }
                                    }
                                }
                            ],
                            "status": "success",
                            "timestamp": "2025-12-30T20:12:41.239Z",
                            "displayName": "Shell",
                            "description": "Execute shell commands",
                            "resultDisplay": "Static hostname: test-machine",
                            "renderOutputAsMarkdown": false
                        }
                    ]
                }
            ]
        }"#;

        let parser = GeminiParser::new();
        let session = parser.parse_string(json).unwrap();

        assert_eq!(session.id, "8c9a7d96-6b65-4e36-9540-0484bc3e3eb2");
        // 新结构：user + assistant text/thinking + assistant tool_action
        assert_eq!(session.messages.len(), 3);

        // 消息 1: 文本消息，包含 thinking
        let text_msg = &session.messages[1];
        let has_thinking = text_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::Thinking { .. }));
        assert!(has_thinking, "Should have thinking block");

        // 消息 2: 工具调用消息
        let tool_msg = &session.messages[2];
        let has_tool_use = tool_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }));
        let has_tool_result = tool_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolResult { .. }));
        assert!(has_tool_use, "Should have tool use block");
        assert!(has_tool_result, "Should have tool result block");

        // Check tokens
        assert_eq!(session.metadata.total_tokens, Some(7823));
    }
}

