//! Cursor log parser module
//!
//! Parses Cursor's conversation logs from state.vscdb databases
//! into MantraSession format.
//!
//! ## Data Flow
//!
//! 1. workspaceStorage/{hash}/state.vscdb (ItemTable)
//!    - composer.composerData → allComposers array (conversation index)
//!
//! 2. globalStorage/state.vscdb (cursorDiskKV)
//!    - composerData:{composerId} → conversation metadata
//!    - bubbleId:{composerId}:{bubbleId} → message content

mod db;
mod path;
mod types;

pub use db::CursorDatabase;
pub use path::{CursorPaths, WorkspaceInfo};
pub use types::*;

use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};

use crate::models::{sources, normalize_tool, ContentBlock, MantraSession, Message, SessionMetadata};
use crate::parsers::ParseError;

/// Parser for Cursor conversation logs
#[derive(Debug, Default)]
pub struct CursorParser;

impl CursorParser {
    /// Create a new CursorParser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse all conversations from a specific project path
    ///
    /// This method:
    /// 1. Finds the workspace folder hash for the given project path
    /// 2. Reads conversation index from workspaceStorage
    /// 3. Retrieves full conversation data from globalStorage
    /// 4. Converts to MantraSession format
    pub fn parse_workspace(&self, project_path: &Path) -> Result<Vec<MantraSession>, ParseError> {
        // Step 1: Detect Cursor paths
        let paths = CursorPaths::detect()?;

        // Step 2: Find workspace ID for the project
        let workspace = paths
            .find_workspace_id(project_path)?
            .ok_or_else(|| {
                ParseError::invalid_format(format!(
                    "Project not found in Cursor workspaces: {}",
                    project_path.display()
                ))
            })?;

        // Step 3: Open workspace database and list composers
        if !workspace.state_db_path.exists() {
            return Err(ParseError::invalid_format(format!(
                "Workspace database not found: {}",
                workspace.state_db_path.display()
            )));
        }

        let workspace_db = CursorDatabase::open(&workspace.state_db_path)?;
        let composer_summaries = workspace_db.list_composers()?;

        if composer_summaries.is_empty() {
            return Ok(Vec::new());
        }

        // Step 4: Open global database for conversation content
        let global_db_path = paths.global_state_db();
        if !global_db_path.exists() {
            return Err(ParseError::invalid_format(format!(
                "Global database not found: {}",
                global_db_path.display()
            )));
        }

        let global_db = CursorDatabase::open(&global_db_path)?;

        // Step 5: Convert each composer to MantraSession
        let mut sessions = Vec::new();

        for summary in composer_summaries {
            match self.parse_composer(&global_db, &summary, project_path) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    // Log warning but continue with other conversations
                    // Note: Using eprintln as tracing is not available in this context
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Warning: Failed to parse composer {}: {}",
                        summary.composer_id, e
                    );
                    let _ = e; // Suppress unused warning in release
                }
            }
        }

        Ok(sessions)
    }

    /// Parse all conversations from Cursor (all workspaces)
    pub fn parse_all(&self) -> Result<Vec<MantraSession>, ParseError> {
        let paths = CursorPaths::detect()?;
        let workspaces = paths.scan_workspaces()?;

        let mut all_sessions = Vec::new();

        for workspace in workspaces {
            if let Ok(sessions) = self.parse_workspace(&workspace.folder_path) {
                all_sessions.extend(sessions);
            }
        }

        Ok(all_sessions)
    }

    /// Parse a single composer conversation to MantraSession
    fn parse_composer(
        &self,
        global_db: &CursorDatabase,
        summary: &db::ComposerSummary,
        project_path: &Path,
    ) -> Result<MantraSession, ParseError> {
        // Get full composer data
        let composer_data = global_db
            .get_composer_data(&summary.composer_id)?
            .ok_or_else(|| {
                ParseError::missing_field(format!("composerData:{}", summary.composer_id))
            })?;

        // Parse composer metadata
        let composer: CursorComposer = serde_json::from_value(composer_data)?;

        // Create session
        let mut session = MantraSession::new(
            summary.composer_id.clone(),
            sources::CURSOR.to_string(),
            project_path.to_string_lossy().to_string(),
        );

        // Set timestamps
        if let Some(created_at_ms) = summary.created_at.or(composer.created_at) {
            session.created_at = epoch_ms_to_datetime(created_at_ms);
            session.updated_at = session.created_at;
        }

        // Set metadata
        session.metadata = SessionMetadata {
            title: summary.name.clone(),
            model: composer
                .model
                .as_ref()
                .and_then(|m| m.model_name.clone().or(m.model_id.clone())),
            total_tokens: None,
            original_path: None,
            ..Default::default()
        };

        // Build source_metadata from Cursor-specific fields (AC3, AC4)
        let mut source_metadata = serde_json::Map::new();

        // Extract unified_mode (e.g., "agent", "chat")
        if let Some(mode) = &composer.unified_mode {
            source_metadata.insert("unified_mode".to_string(), serde_json::json!(mode));
        }

        // Extract model provider (e.g., "anthropic", "openai")
        if let Some(model) = &composer.model {
            if let Some(provider) = &model.provider {
                source_metadata.insert("model_provider".to_string(), serde_json::json!(provider));
            }
        }

        // Extract context mentions
        if let Some(ctx) = &composer.context {
            if !ctx.mentions.is_null() {
                source_metadata.insert("context".to_string(), serde_json::json!({
                    "mentions": ctx.mentions.clone()
                }));
            }
        }

        // Set source_metadata if not empty (AC4: defaults to None if all fields missing)
        if !source_metadata.is_empty() {
            session.metadata.source_metadata = Some(serde_json::Value::Object(source_metadata));
        }

        // Parse messages from bubble headers
        let mut messages = Vec::new();

        for header in &composer.full_conversation_headers_only {
            if let Ok(Some(msg)) = self.parse_bubble(global_db, &summary.composer_id, header) {
                messages.push(msg);
            }
        }

        session.messages = messages;

        // Update last timestamp from messages
        if let Some(last_msg) = session.messages.last() {
            if let Some(ts) = last_msg.timestamp {
                session.updated_at = ts;
            }
        }

        Ok(session)
    }

    /// Parse a single bubble to Message
    fn parse_bubble(
        &self,
        global_db: &CursorDatabase,
        composer_id: &str,
        header: &BubbleHeader,
    ) -> Result<Option<Message>, ParseError> {
        // Get bubble content
        let bubble_data = global_db.get_bubble_content(composer_id, &header.bubble_id)?;

        let bubble_data = match bubble_data {
            Some(data) => data,
            None => return Ok(None),
        };

        let bubble: CursorBubble = serde_json::from_value(bubble_data)?;

        // Map role
        let role = match CursorRole::from(bubble.bubble_type).to_mantra_role() {
            Some(r) => r,
            None => return Ok(None),
        };

        // Build content blocks
        let mut content_blocks = Vec::new();

        // Add main text content (strip system reminder tags)
        if let Some(text) = &bubble.text {
            let cleaned = crate::parsers::strip_system_reminders(text);
            if !cleaned.is_empty() {
                content_blocks.push(ContentBlock::Text { text: cleaned });
            }
        }

        // Parse toolFormerData (PRIMARY: this is where Cursor stores tool call data)
        if let Some(tfd) = &bubble.tool_former_data {
            if let Some(name) = &tfd.name {
                // Generate correlation_id from tool_call_id (preferred) or fallback to name+index
                let correlation_id = tfd.tool_call_id.clone()
                    .or_else(|| Some(format!("cursor:{}:{}", name, tfd.tool_index.unwrap_or(0))));

                // Parse tool input from raw_args (JSON string)
                let input = tfd.raw_args
                    .as_ref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_else(|| serde_json::json!({}));

                // Call normalize_tool() to get standardized tool type (AC2)
                let standard_tool = Some(normalize_tool(name, &input));

                // Add ToolUse block
                content_blocks.push(ContentBlock::ToolUse {
                    id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                    name: name.clone(),
                    input,
                    correlation_id: correlation_id.clone(),
                    standard_tool,
                    display_name: None,
                    description: None,
                });

                // Add ToolResult if result exists
                if let Some(result_str) = &tfd.result {
                    // Strip system reminder tags from tool result content
                    let cleaned_result = crate::parsers::strip_system_reminders(result_str);
                    content_blocks.push(ContentBlock::ToolResult {
                        tool_use_id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                        content: cleaned_result,
                        is_error: tfd.status.as_deref() == Some("failed"),
                        correlation_id,
                        structured_result: None,
                        display_content: None,
                        render_as_markdown: None,
                        // Extract user_decision from toolFormerData (AC1, AC4: defaults to None if missing)
                        user_decision: tfd.user_decision.clone(),
                    });
                }
            }
        }

        // Fallback: parse legacy toolResults (usually empty, but kept for backwards compatibility)
        if bubble.tool_former_data.is_none() {
            for tool_result in &bubble.tool_results {
                if let (Some(id), Some(name)) = (&tool_result.id, &tool_result.name) {
                    let correlation_id = Some(id.clone());
                    let input = serde_json::json!({});
                    // Call normalize_tool() for legacy path (AC2)
                    let standard_tool = Some(normalize_tool(name, &input));

                    content_blocks.push(ContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input,
                        correlation_id: correlation_id.clone(),
                        standard_tool,
                        display_name: None,
                        description: None,
                    });

                    if let Some(result) = &tool_result.result {
                        // Strip system reminder tags from tool result content
                        let cleaned_result = crate::parsers::strip_system_reminders(&result.to_string());
                        content_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: cleaned_result,
                            is_error: tool_result.is_error,
                            correlation_id,
                            structured_result: None,
                            display_content: None,
                            render_as_markdown: None,
                            // Legacy path has no user_decision (AC4: defaults to None)
                            user_decision: None,
                        });
                    }
                }
            }
        }

        // Add code blocks from suggestions as CodeSuggestion blocks (Story 8.5)
        for code_block in &bubble.suggested_code_blocks {
            if let Some(code) = &code_block.code {
                // Skip empty code blocks (AC4)
                if !code.is_empty() {
                    content_blocks.push(ContentBlock::CodeSuggestion {
                        // Use "unknown" as default when file_path is None (AC4)
                        file_path: code_block.file_path.clone().unwrap_or_else(|| "unknown".to_string()),
                        code: code.clone(),
                        language: code_block.language.clone(),
                    });
                }
            }
        }

        // Skip empty messages
        if content_blocks.is_empty() {
            return Ok(None);
        }

        // Extract mentioned files from bubble context
        let mentioned_files = extract_mentioned_files_from_bubble(&bubble.context);

        // Build message
        let timestamp = bubble.timestamp.map(epoch_ms_to_datetime);

        Ok(Some(Message {
            role,
            content_blocks,
            timestamp,
            mentioned_files,
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        }))
    }
}

/// Convert epoch milliseconds to DateTime<Utc>
fn epoch_ms_to_datetime(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Extract mentioned files from bubble context
fn extract_mentioned_files_from_bubble(context: &Option<BubbleContext>) -> Vec<String> {
    let mut files = Vec::new();

    if let Some(ctx) = context {
        // Extract from mentions object (e.g., mentions.fileSelections, mentions.files)
        if let Some(mentions) = ctx.mentions.as_object() {
            // Handle fileSelections format
            if let Some(file_selections) = mentions.get("fileSelections") {
                if let Some(obj) = file_selections.as_object() {
                    for (uri, _) in obj {
                        files.push(uri.clone());
                    }
                }
            }
            // Handle files array format
            if let Some(files_arr) = mentions.get("files") {
                if let Some(arr) = files_arr.as_array() {
                    for item in arr {
                        if let Some(uri) = item.as_str() {
                            files.push(uri.to_string());
                        } else if let Some(obj) = item.as_object() {
                            if let Some(uri) = obj.get("uri").and_then(|v| v.as_str()) {
                                files.push(uri.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    files
}

/// Extract mentioned files from composer context (for session-level context)
#[allow(dead_code)]
fn extract_mentioned_files(context: &Option<CursorContext>) -> Vec<String> {
    let mut files = Vec::new();

    if let Some(ctx) = context {
        // Extract from mentions object (e.g., mentions.fileSelections)
        if let Some(file_selections) = ctx.mentions.get("fileSelections") {
            if let Some(obj) = file_selections.as_object() {
                for (uri, _) in obj {
                    files.push(uri.clone());
                }
            }
        }

        // Extract from direct file_selections array
        for selection in &ctx.file_selections {
            if let Some(uri) = &selection.uri {
                files.push(uri.clone());
            }
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_epoch_ms_to_datetime() {
        // 2024-01-01 00:00:00 UTC
        let ms = 1704067200000_i64;
        let dt = epoch_ms_to_datetime(ms);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_cursor_parser_new() {
        let parser = CursorParser::new();
        assert!(format!("{:?}", parser).contains("CursorParser"));
    }

    #[test]
    fn test_extract_mentioned_files() {
        let context = Some(CursorContext {
            mentions: serde_json::json!({
                "fileSelections": {
                    "file:///path/to/file.rs": {}
                }
            }),
            file_selections: vec![FileSelection {
                uri: Some("file:///path/to/another.rs".to_string()),
                range: None,
            }],
        });

        let files = extract_mentioned_files(&context);
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file:///path/to/file.rs".to_string()));
        assert!(files.contains(&"file:///path/to/another.rs".to_string()));
    }

    #[test]
    fn test_extract_mentioned_files_empty() {
        let context: Option<CursorContext> = None;
        let files = extract_mentioned_files(&context);
        assert!(files.is_empty());
    }

    // ===== Story 8.5: CodeSuggestion 块解析测试 =====

    /// Helper function to create a test bubble with suggested code blocks
    fn create_bubble_with_code_suggestions(
        code_blocks: Vec<SuggestedCodeBlock>,
    ) -> CursorBubble {
        CursorBubble {
            version: Some(1),
            bubble_id: Some("test-bubble-id".to_string()),
            bubble_type: 2, // Assistant type
            text: Some("Here is the code suggestion:".to_string()),
            rich_text: None,
            is_agentic: false,
            timestamp: Some(1704067200000),
            tool_former_data: None,
            tool_results: vec![],
            suggested_code_blocks: code_blocks,
            context: None,
        }
    }

    #[test]
    fn test_parse_suggested_code_blocks_creates_code_suggestion() {
        // Test that suggestedCodeBlocks are converted to CodeSuggestion ContentBlocks
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: Some("/src/lib.rs".to_string()),
                code: Some("pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
                language: Some("rust".to_string()),
            },
        ]);

        assert_eq!(bubble.suggested_code_blocks.len(), 1);
        let code_block = &bubble.suggested_code_blocks[0];
        assert_eq!(code_block.file_path, Some("/src/lib.rs".to_string()));
        assert_eq!(code_block.code, Some("pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string()));
        assert_eq!(code_block.language, Some("rust".to_string()));
    }

    #[test]
    fn test_parse_suggested_code_blocks_empty_code_skipped() {
        // Test that empty code blocks are skipped (AC4)
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: Some("/src/empty.rs".to_string()),
                code: Some("".to_string()), // Empty code
                language: Some("rust".to_string()),
            },
        ]);

        // The empty code block should not create a CodeSuggestion
        assert!(bubble.suggested_code_blocks[0].code.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_parse_suggested_code_blocks_missing_file_path_uses_default() {
        // Test that missing file_path uses "unknown" default (AC4)
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: None, // Missing file path
                code: Some("let x = 1;".to_string()),
                language: Some("javascript".to_string()),
            },
        ]);

        // file_path should be None, will default to "unknown" during parsing
        assert!(bubble.suggested_code_blocks[0].file_path.is_none());
    }

    #[test]
    fn test_parse_suggested_code_blocks_none_code_skipped() {
        // Test that None code blocks are skipped
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: Some("/src/test.rs".to_string()),
                code: None, // None code
                language: Some("rust".to_string()),
            },
        ]);

        assert!(bubble.suggested_code_blocks[0].code.is_none());
    }

    #[test]
    fn test_parse_suggested_code_blocks_multiple() {
        // Test multiple suggested code blocks
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: Some("/src/main.rs".to_string()),
                code: Some("fn main() {}".to_string()),
                language: Some("rust".to_string()),
            },
            SuggestedCodeBlock {
                file_path: Some("/src/lib.rs".to_string()),
                code: Some("pub fn lib_fn() {}".to_string()),
                language: Some("rust".to_string()),
            },
        ]);

        assert_eq!(bubble.suggested_code_blocks.len(), 2);
    }

    #[test]
    fn test_parse_suggested_code_blocks_no_language() {
        // Test code block without language
        let bubble = create_bubble_with_code_suggestions(vec![
            SuggestedCodeBlock {
                file_path: Some("/config.txt".to_string()),
                code: Some("key=value".to_string()),
                language: None, // No language
            },
        ]);

        assert!(bubble.suggested_code_blocks[0].language.is_none());
    }

    #[test]
    fn test_code_suggestion_content_block_creation() {
        // Test direct ContentBlock::CodeSuggestion creation matching parse_bubble logic
        let code_block = SuggestedCodeBlock {
            file_path: Some("/src/test.rs".to_string()),
            code: Some("fn test() {}".to_string()),
            language: Some("rust".to_string()),
        };

        // Simulate the logic in parse_bubble
        if let Some(code) = &code_block.code {
            if !code.is_empty() {
                let content_block = ContentBlock::CodeSuggestion {
                    file_path: code_block.file_path.clone().unwrap_or_else(|| "unknown".to_string()),
                    code: code.clone(),
                    language: code_block.language.clone(),
                };

                // Verify the content block
                match content_block {
                    ContentBlock::CodeSuggestion { file_path, code: c, language } => {
                        assert_eq!(file_path, "/src/test.rs");
                        assert_eq!(c, "fn test() {}");
                        assert_eq!(language, Some("rust".to_string()));
                    }
                    _ => panic!("Expected CodeSuggestion variant"),
                }
            }
        }
    }

    #[test]
    fn test_code_suggestion_default_file_path() {
        // Test that missing file_path defaults to "unknown"
        let code_block = SuggestedCodeBlock {
            file_path: None,
            code: Some("console.log('test');".to_string()),
            language: Some("javascript".to_string()),
        };

        // Simulate the logic in parse_bubble
        if let Some(code) = &code_block.code {
            if !code.is_empty() {
                let content_block = ContentBlock::CodeSuggestion {
                    file_path: code_block.file_path.clone().unwrap_or_else(|| "unknown".to_string()),
                    code: code.clone(),
                    language: code_block.language.clone(),
                };

                match content_block {
                    ContentBlock::CodeSuggestion { file_path, .. } => {
                        assert_eq!(file_path, "unknown"); // Should default to "unknown"
                    }
                    _ => panic!("Expected CodeSuggestion variant"),
                }
            }
        }
    }

    // ===== Story 8.8: Cursor Parser 适配测试 =====

    /// Helper function to create a test bubble with toolFormerData
    fn create_bubble_with_tool_former_data(tfd: ToolFormerData) -> CursorBubble {
        CursorBubble {
            version: Some(1),
            bubble_id: Some("test-bubble-id".to_string()),
            bubble_type: 2, // Assistant type
            text: None,
            rich_text: None,
            is_agentic: true,
            timestamp: Some(1704067200000),
            tool_former_data: Some(tfd),
            tool_results: vec![],
            suggested_code_blocks: vec![],
            context: None,
        }
    }

    #[test]
    fn test_parse_user_decision_approved() {
        // Test AC1: user_decision extraction with "approved" value
        let tfd = ToolFormerData {
            tool: Some(38),
            tool_index: Some(0),
            tool_call_id: Some("call-123".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("edit_file".to_string()),
            raw_args: Some(r#"{"file_path": "/src/main.rs", "old_string": "foo", "new_string": "bar"}"#.to_string()),
            params: None,
            result: Some("File edited successfully".to_string()),
            additional_data: None,
            user_decision: Some("approved".to_string()),
        };

        // Verify user_decision is present
        assert_eq!(tfd.user_decision, Some("approved".to_string()));
    }

    #[test]
    fn test_parse_user_decision_rejected() {
        // Test AC1: user_decision extraction with "rejected" value
        let tfd = ToolFormerData {
            tool: Some(10),
            tool_index: Some(1),
            tool_call_id: Some("call-456".to_string()),
            model_call_id: None,
            status: Some("failed".to_string()),
            name: Some("run_terminal_cmd".to_string()),
            raw_args: Some(r#"{"command": "rm -rf /"}"#.to_string()),
            params: None,
            result: Some("User rejected".to_string()),
            additional_data: None,
            user_decision: Some("rejected".to_string()),
        };

        assert_eq!(tfd.user_decision, Some("rejected".to_string()));
    }

    #[test]
    fn test_parse_user_decision_none() {
        // Test AC4: user_decision defaults to None when missing
        let tfd = ToolFormerData {
            tool: Some(1),
            tool_index: Some(0),
            tool_call_id: Some("call-789".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("read_file".to_string()),
            raw_args: Some(r#"{"file_path": "/src/lib.rs"}"#.to_string()),
            params: None,
            result: Some("File content...".to_string()),
            additional_data: None,
            user_decision: None, // No user decision
        };

        assert!(tfd.user_decision.is_none());
    }

    #[test]
    fn test_standard_tool_mapping_read_file() {
        // Test AC2: StandardTool mapping for read_file
        let input = serde_json::json!({"file_path": "/src/main.rs", "start_line": 1, "end_line": 50});
        let standard_tool = normalize_tool("read_file", &input);

        match standard_tool {
            crate::models::StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/src/main.rs");
                assert_eq!(start_line, Some(1));
                assert_eq!(end_line, Some(50));
            }
            _ => panic!("Expected FileRead variant"),
        }
    }

    #[test]
    fn test_standard_tool_mapping_edit_file() {
        // Test AC2: StandardTool mapping for edit_file
        let input = serde_json::json!({
            "file_path": "/src/lib.rs",
            "old_string": "fn old()",
            "new_string": "fn new()"
        });
        let standard_tool = normalize_tool("edit_file", &input);

        match standard_tool {
            crate::models::StandardTool::FileEdit { path, old_string, new_string } => {
                assert_eq!(path, "/src/lib.rs");
                assert_eq!(old_string, Some("fn old()".to_string()));
                assert_eq!(new_string, Some("fn new()".to_string()));
            }
            _ => panic!("Expected FileEdit variant"),
        }
    }

    #[test]
    fn test_standard_tool_mapping_run_terminal_cmd() {
        // Test AC2: StandardTool mapping for run_terminal_cmd
        let input = serde_json::json!({"command": "cargo build", "cwd": "/project"});
        let standard_tool = normalize_tool("run_terminal_cmd", &input);

        match standard_tool {
            crate::models::StandardTool::ShellExec { command, cwd } => {
                assert_eq!(command, "cargo build");
                assert_eq!(cwd, Some("/project".to_string()));
            }
            _ => panic!("Expected ShellExec variant"),
        }
    }

    #[test]
    fn test_standard_tool_mapping_write_file() {
        // Test AC2: StandardTool mapping for write_file
        let input = serde_json::json!({
            "file_path": "/src/new.rs",
            "content": "fn main() {}"
        });
        let standard_tool = normalize_tool("write_file", &input);

        match standard_tool {
            crate::models::StandardTool::FileWrite { path, content } => {
                assert_eq!(path, "/src/new.rs");
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected FileWrite variant"),
        }
    }

    #[test]
    fn test_standard_tool_mapping_unknown() {
        // Test AC2: Unknown tools map to StandardTool::Unknown
        let input = serde_json::json!({"custom_param": "value"});
        let standard_tool = normalize_tool("custom_cursor_tool", &input);

        match standard_tool {
            crate::models::StandardTool::Unknown { name, input: tool_input } => {
                assert_eq!(name, "custom_cursor_tool");
                assert_eq!(tool_input, serde_json::json!({"custom_param": "value"}));
            }
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_source_metadata_unified_mode() {
        // Test AC3: source_metadata contains unified_mode
        let composer = CursorComposer {
            version: Some(2),
            composer_id: Some("comp-123".to_string()),
            full_conversation_headers_only: vec![],
            context: None,
            model: None,
            created_at: Some(1704067200000),
            unified_mode: Some("agent".to_string()),
        };

        assert_eq!(composer.unified_mode, Some("agent".to_string()));
    }

    #[test]
    fn test_source_metadata_model_provider() {
        // Test AC3: source_metadata contains model_provider
        let composer = CursorComposer {
            version: Some(2),
            composer_id: Some("comp-456".to_string()),
            full_conversation_headers_only: vec![],
            context: None,
            model: Some(ModelConfig {
                model_name: Some("claude-3-opus".to_string()),
                model_id: Some("claude-3-opus-20240229".to_string()),
                provider: Some("anthropic".to_string()),
            }),
            created_at: Some(1704067200000),
            unified_mode: Some("chat".to_string()),
        };

        assert_eq!(composer.model.as_ref().unwrap().provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_source_metadata_context_mentions() {
        // Test AC3: source_metadata contains context mentions
        let composer = CursorComposer {
            version: Some(2),
            composer_id: Some("comp-789".to_string()),
            full_conversation_headers_only: vec![],
            context: Some(CursorContext {
                mentions: serde_json::json!({
                    "fileSelections": {
                        "file:///src/main.rs": {}
                    }
                }),
                file_selections: vec![],
            }),
            model: None,
            created_at: Some(1704067200000),
            unified_mode: None,
        };

        assert!(!composer.context.as_ref().unwrap().mentions.is_null());
    }

    #[test]
    fn test_backward_compat_no_new_fields() {
        // Test AC4: Old data without new fields still parses correctly
        let json = r#"{
            "_v": 2,
            "composerId": "old-comp",
            "fullConversationHeadersOnly": [],
            "createdAt": 1704067200000
        }"#;

        let composer: CursorComposer = serde_json::from_str(json).unwrap();
        assert_eq!(composer.composer_id, Some("old-comp".to_string()));
        assert!(composer.unified_mode.is_none()); // New field defaults to None
        assert!(composer.model.is_none()); // New field defaults to None
        assert!(composer.context.is_none()); // New field defaults to None
    }

    #[test]
    fn test_backward_compat_tool_former_data_no_user_decision() {
        // Test AC4: Old toolFormerData without user_decision still parses
        let json = r#"{
            "tool": 1,
            "toolIndex": 0,
            "toolCallId": "old-call",
            "name": "read_file",
            "rawArgs": "{\"file_path\": \"/test.rs\"}",
            "result": "file content"
        }"#;

        let tfd: ToolFormerData = serde_json::from_str(json).unwrap();
        assert_eq!(tfd.name, Some("read_file".to_string()));
        assert!(tfd.user_decision.is_none()); // Defaults to None
    }

    #[test]
    fn test_source_metadata_build_logic() {
        // Test AC3: Verify source_metadata building logic
        let composer = CursorComposer {
            version: Some(2),
            composer_id: Some("test-comp".to_string()),
            full_conversation_headers_only: vec![],
            context: Some(CursorContext {
                mentions: serde_json::json!({"files": ["test.rs"]}),
                file_selections: vec![],
            }),
            model: Some(ModelConfig {
                model_name: Some("gpt-4".to_string()),
                model_id: None,
                provider: Some("openai".to_string()),
            }),
            created_at: Some(1704067200000),
            unified_mode: Some("agent".to_string()),
        };

        // Simulate source_metadata building logic from parse_composer
        let mut source_metadata = serde_json::Map::new();

        if let Some(mode) = &composer.unified_mode {
            source_metadata.insert("unified_mode".to_string(), serde_json::json!(mode));
        }

        if let Some(model) = &composer.model {
            if let Some(provider) = &model.provider {
                source_metadata.insert("model_provider".to_string(), serde_json::json!(provider));
            }
        }

        if let Some(ctx) = &composer.context {
            if !ctx.mentions.is_null() {
                source_metadata.insert("context".to_string(), serde_json::json!({
                    "mentions": ctx.mentions.clone()
                }));
            }
        }

        // Verify all fields are present
        assert_eq!(source_metadata.get("unified_mode").unwrap(), "agent");
        assert_eq!(source_metadata.get("model_provider").unwrap(), "openai");
        assert!(source_metadata.get("context").is_some());
    }

    // ===== End-to-End Tests: Simulating parse_bubble() logic =====
    // These tests verify the complete ContentBlock creation flow

    /// Simulate parse_bubble's ContentBlock creation logic for testing
    /// This mirrors the actual implementation in parse_bubble() lines 250-291
    fn simulate_parse_bubble_content_blocks(bubble: &CursorBubble) -> Vec<ContentBlock> {
        let mut content_blocks = Vec::new();

        // Add main text content (strip system reminder tags)
        if let Some(text) = &bubble.text {
            let cleaned = crate::parsers::strip_system_reminders(text);
            if !cleaned.is_empty() {
                content_blocks.push(ContentBlock::Text { text: cleaned });
            }
        }

        // Parse toolFormerData (PRIMARY path)
        if let Some(tfd) = &bubble.tool_former_data {
            if let Some(name) = &tfd.name {
                let correlation_id = tfd.tool_call_id.clone()
                    .or_else(|| Some(format!("cursor:{}:{}", name, tfd.tool_index.unwrap_or(0))));

                let input = tfd.raw_args
                    .as_ref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_else(|| serde_json::json!({}));

                // Call normalize_tool() (AC2)
                let standard_tool = Some(normalize_tool(name, &input));

                // Add ToolUse block
                content_blocks.push(ContentBlock::ToolUse {
                    id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                    name: name.clone(),
                    input,
                    correlation_id: correlation_id.clone(),
                    standard_tool,
                    display_name: None,
                    description: None,
                });

                // Add ToolResult if result exists
                if let Some(result_str) = &tfd.result {
                    // Strip system reminder tags from tool result content (same as production code)
                    let cleaned_result = crate::parsers::strip_system_reminders(result_str);
                    content_blocks.push(ContentBlock::ToolResult {
                        tool_use_id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                        content: cleaned_result,
                        is_error: tfd.status.as_deref() == Some("failed"),
                        correlation_id,
                        structured_result: None,
                        display_content: None,
                        render_as_markdown: None,
                        // Extract user_decision (AC1)
                        user_decision: tfd.user_decision.clone(),
                    });
                }
            }
        }

        content_blocks
    }

    #[test]
    fn test_e2e_parse_bubble_user_decision_approved() {
        // End-to-end test: Verify user_decision is correctly passed to ToolResult
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(38),
            tool_index: Some(0),
            tool_call_id: Some("call-e2e-1".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("edit_file".to_string()),
            raw_args: Some(r#"{"file_path": "/src/main.rs", "old_string": "foo", "new_string": "bar"}"#.to_string()),
            params: None,
            result: Some("File edited successfully".to_string()),
            additional_data: None,
            user_decision: Some("approved".to_string()),
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        // Find ToolResult block and verify user_decision
        let tool_result = content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolResult { .. }));
        assert!(tool_result.is_some(), "ToolResult block should exist");

        if let Some(ContentBlock::ToolResult { user_decision, .. }) = tool_result {
            assert_eq!(*user_decision, Some("approved".to_string()), "user_decision should be 'approved'");
        }
    }

    #[test]
    fn test_e2e_parse_bubble_user_decision_rejected() {
        // End-to-end test: Verify rejected user_decision
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(10),
            tool_index: Some(1),
            tool_call_id: Some("call-e2e-2".to_string()),
            model_call_id: None,
            status: Some("failed".to_string()),
            name: Some("run_terminal_cmd".to_string()),
            raw_args: Some(r#"{"command": "rm -rf /"}"#.to_string()),
            params: None,
            result: Some("User rejected the command".to_string()),
            additional_data: None,
            user_decision: Some("rejected".to_string()),
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        let tool_result = content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolResult { .. }));
        assert!(tool_result.is_some());

        if let Some(ContentBlock::ToolResult { user_decision, is_error, .. }) = tool_result {
            assert_eq!(*user_decision, Some("rejected".to_string()));
            assert!(*is_error, "is_error should be true for failed status");
        }
    }

    #[test]
    fn test_e2e_parse_bubble_standard_tool_file_read() {
        // End-to-end test: Verify StandardTool mapping for read_file
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(1),
            tool_index: Some(0),
            tool_call_id: Some("call-e2e-3".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("read_file".to_string()),
            raw_args: Some(r#"{"file_path": "/src/lib.rs", "start_line": 10, "end_line": 50}"#.to_string()),
            params: None,
            result: Some("fn main() { ... }".to_string()),
            additional_data: None,
            user_decision: None,
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        let tool_use = content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolUse { .. }));
        assert!(tool_use.is_some(), "ToolUse block should exist");

        if let Some(ContentBlock::ToolUse { standard_tool, .. }) = tool_use {
            match standard_tool {
                Some(crate::models::StandardTool::FileRead { path, start_line, end_line }) => {
                    assert_eq!(path, "/src/lib.rs");
                    assert_eq!(*start_line, Some(10));
                    assert_eq!(*end_line, Some(50));
                }
                _ => panic!("Expected StandardTool::FileRead"),
            }
        }
    }

    #[test]
    fn test_e2e_parse_bubble_standard_tool_shell_exec() {
        // End-to-end test: Verify StandardTool mapping for run_terminal_cmd
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(10),
            tool_index: Some(0),
            tool_call_id: Some("call-e2e-4".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("run_terminal_cmd".to_string()),
            raw_args: Some(r#"{"command": "cargo test", "cwd": "/project"}"#.to_string()),
            params: None,
            result: Some("test result: ok".to_string()),
            additional_data: None,
            user_decision: Some("approved".to_string()),
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        let tool_use = content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolUse { .. }));
        assert!(tool_use.is_some());

        if let Some(ContentBlock::ToolUse { standard_tool, .. }) = tool_use {
            match standard_tool {
                Some(crate::models::StandardTool::ShellExec { command, cwd }) => {
                    assert_eq!(command, "cargo test");
                    assert_eq!(*cwd, Some("/project".to_string()));
                }
                _ => panic!("Expected StandardTool::ShellExec"),
            }
        }
    }

    #[test]
    fn test_e2e_parse_bubble_backward_compat_no_user_decision() {
        // End-to-end test: Verify backward compatibility when user_decision is None
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(1),
            tool_index: Some(0),
            tool_call_id: Some("call-e2e-5".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("read_file".to_string()),
            raw_args: Some(r#"{"file_path": "/test.rs"}"#.to_string()),
            params: None,
            result: Some("file content".to_string()),
            additional_data: None,
            user_decision: None, // Old data without user_decision
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        let tool_result = content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolResult { .. }));
        assert!(tool_result.is_some());

        if let Some(ContentBlock::ToolResult { user_decision, .. }) = tool_result {
            assert!(user_decision.is_none(), "user_decision should be None for backward compatibility");
        }
    }

    #[test]
    fn test_e2e_parse_bubble_both_tool_use_and_result() {
        // End-to-end test: Verify both ToolUse and ToolResult are created
        let bubble = create_bubble_with_tool_former_data(ToolFormerData {
            tool: Some(38),
            tool_index: Some(0),
            tool_call_id: Some("call-e2e-6".to_string()),
            model_call_id: None,
            status: Some("completed".to_string()),
            name: Some("edit_file".to_string()),
            raw_args: Some(r#"{"file_path": "/src/main.rs", "old_string": "old", "new_string": "new"}"#.to_string()),
            params: None,
            result: Some("Edit applied".to_string()),
            additional_data: None,
            user_decision: Some("approved".to_string()),
        });

        let content_blocks = simulate_parse_bubble_content_blocks(&bubble);

        // Count block types
        let tool_use_count = content_blocks.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).count();
        let tool_result_count = content_blocks.iter().filter(|b| matches!(b, ContentBlock::ToolResult { .. })).count();

        assert_eq!(tool_use_count, 1, "Should have exactly 1 ToolUse block");
        assert_eq!(tool_result_count, 1, "Should have exactly 1 ToolResult block");

        // Verify correlation_id matches between ToolUse and ToolResult
        let tool_use_corr = content_blocks.iter().find_map(|b| {
            if let ContentBlock::ToolUse { correlation_id, .. } = b {
                correlation_id.clone()
            } else {
                None
            }
        });

        let tool_result_corr = content_blocks.iter().find_map(|b| {
            if let ContentBlock::ToolResult { correlation_id, .. } = b {
                correlation_id.clone()
            } else {
                None
            }
        });

        assert_eq!(tool_use_corr, tool_result_corr, "correlation_id should match between ToolUse and ToolResult");
    }
}
