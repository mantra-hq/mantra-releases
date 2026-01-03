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

use crate::models::{sources, ContentBlock, MantraSession, Message, SessionMetadata};
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
        };

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

        // Add main text content
        if let Some(text) = &bubble.text {
            if !text.is_empty() {
                content_blocks.push(ContentBlock::Text { text: text.clone() });
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

                // Add ToolUse block
                content_blocks.push(ContentBlock::ToolUse {
                    id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                    name: name.clone(),
                    input,
                    correlation_id: correlation_id.clone(),
                });

                // Add ToolResult if result exists
                if let Some(result_str) = &tfd.result {
                    content_blocks.push(ContentBlock::ToolResult {
                        tool_use_id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                        content: result_str.clone(),
                        is_error: tfd.status.as_deref() == Some("failed"),
                        correlation_id,
                    });
                }
            }
        }

        // Fallback: parse legacy toolResults (usually empty, but kept for backwards compatibility)
        if bubble.tool_former_data.is_none() {
            for tool_result in &bubble.tool_results {
                if let (Some(id), Some(name)) = (&tool_result.id, &tool_result.name) {
                    let correlation_id = Some(id.clone());

                    content_blocks.push(ContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: serde_json::json!({}),
                        correlation_id: correlation_id.clone(),
                    });

                    if let Some(result) = &tool_result.result {
                        content_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: result.to_string(),
                            is_error: tool_result.is_error,
                            correlation_id,
                        });
                    }
                }
            }
        }

        // Add code blocks from suggestions
        for code_block in &bubble.suggested_code_blocks {
            if let Some(code) = &code_block.code {
                let file_info = code_block.file_path.as_deref().unwrap_or("unknown");
                let lang = code_block.language.as_deref().unwrap_or("");
                let formatted = format!("```{}\n// File: {}\n{}\n```", lang, file_info, code);
                content_blocks.push(ContentBlock::Text { text: formatted });
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
}
