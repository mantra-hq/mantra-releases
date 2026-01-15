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

use crate::models::{sources, normalize_tool, ContentBlock, MantraSession, Message, ParserInfo, SessionMetadata, StandardTool, ToolResultData, UnknownFormatEntry};
use crate::parsers::ParseError;

/// Cursor Parser version for compatibility tracking (Story 8.15)
pub const CURSOR_PARSER_VERSION: &str = "1.2.0";

/// Supported bubble types in Cursor format
pub const SUPPORTED_BUBBLE_TYPES: &[&str] = &["user", "assistant", "1", "2"];

/// Supported content fields in Cursor bubbles
pub const SUPPORTED_CONTENT_TYPES: &[&str] = &["text", "tool_former_data", "tool_results", "suggested_code_blocks", "images", "all_thinking_blocks"];

/// Maximum raw JSON size to store in UnknownFormatEntry (1KB)
const MAX_RAW_JSON_SIZE: usize = 1024;

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

        // Set metadata with parser_info (Story 8.15)
        session.metadata = SessionMetadata {
            title: summary.name.clone(),
            model: composer
                .model
                .as_ref()
                .and_then(|m| m.model_name.clone().or(m.model_id.clone())),
            total_tokens: None,
            original_path: None,
            parser_info: Some(ParserInfo {
                parser_version: CURSOR_PARSER_VERSION.to_string(),
                supported_formats: SUPPORTED_CONTENT_TYPES.iter().map(|s| s.to_string()).collect(),
                detected_source_version: None, // Cursor doesn't expose version info
            }),
            unknown_formats: None, // Will be populated if unknown formats are encountered
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

        // Parse messages from bubble headers (Story 8.15: collect unknown formats)
        let mut messages = Vec::new();
        let mut all_unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

        for header in &composer.full_conversation_headers_only {
            match self.parse_bubble(global_db, &summary.composer_id, header) {
                Ok((Some(msg), unknown_formats)) => {
                    messages.push(msg);
                    all_unknown_formats.extend(unknown_formats);
                }
                Ok((None, unknown_formats)) => {
                    // Message was skipped (e.g., empty), but still collect unknown formats
                    all_unknown_formats.extend(unknown_formats);
                }
                Err(_) => {
                    // Skip parsing errors silently (existing behavior)
                }
            }
        }

        session.messages = messages;

        // Story 8.15: Set unknown_formats in metadata if any were collected
        if !all_unknown_formats.is_empty() {
            session.metadata.unknown_formats = Some(all_unknown_formats);
        }

        // Update last timestamp from messages
        if let Some(last_msg) = session.messages.last() {
            if let Some(ts) = last_msg.timestamp {
                session.updated_at = ts;
            }
        }

        Ok(session)
    }

    /// Parse a single bubble to Message
    /// Story 8.15: Now returns (Option<Message>, Vec<UnknownFormatEntry>) to collect unknown formats
    fn parse_bubble(
        &self,
        global_db: &CursorDatabase,
        composer_id: &str,
        header: &BubbleHeader,
    ) -> Result<(Option<Message>, Vec<UnknownFormatEntry>), ParseError> {
        let mut unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

        // Get bubble content
        let bubble_data = global_db.get_bubble_content(composer_id, &header.bubble_id)?;

        let bubble_data = match bubble_data {
            Some(data) => data,
            None => return Ok((None, unknown_formats)),
        };

        // Story 8.15: Graceful degradation for bubble parsing
        let bubble: CursorBubble = match serde_json::from_value(bubble_data.clone()) {
            Ok(b) => b,
            Err(_) => {
                // Record unknown format and create degraded message
                unknown_formats.push(UnknownFormatEntry {
                    source: "cursor".to_string(),
                    type_name: "bubble_parse_error".to_string(),
                    raw_json: truncate_raw_json(&bubble_data),
                    timestamp: Utc::now().to_rfc3339(),
                });

                // Create degraded text block with original content
                let degraded_text = format!(
                    "[无法解析的 Bubble]\n{}",
                    truncate_raw_json(&bubble_data)
                );
                let message = Message {
                    role: crate::models::Role::Assistant, // Default to assistant for degraded content
                    content_blocks: vec![ContentBlock::Text {
                        text: degraded_text,
                        is_degraded: Some(true),
                    }],
                    timestamp: None,
                    mentioned_files: vec![],
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                };
                return Ok((Some(message), unknown_formats));
            }
        };

        // Map role - Story 8.15: Handle unknown bubble types gracefully
        let role = match CursorRole::from(bubble.bubble_type).to_mantra_role() {
            Some(r) => r,
            None => {
                // Record unknown bubble type
                unknown_formats.push(UnknownFormatEntry {
                    source: "cursor".to_string(),
                    type_name: format!("unknown_bubble_type_{}", bubble.bubble_type),
                    raw_json: truncate_raw_json(&bubble_data),
                    timestamp: Utc::now().to_rfc3339(),
                });

                // Create degraded message with original content
                let degraded_text = format!(
                    "[未知消息类型: {}]\n{}",
                    bubble.bubble_type,
                    bubble.text.as_deref().unwrap_or("")
                );
                let message = Message {
                    role: crate::models::Role::Assistant, // Default to assistant for unknown types
                    content_blocks: vec![ContentBlock::Text {
                        text: degraded_text,
                        is_degraded: Some(true),
                    }],
                    timestamp: bubble.timestamp.map(epoch_ms_to_datetime),
                    mentioned_files: vec![],
                    message_id: None,
                    parent_id: None,
                    is_sidechain: false,
                    source_metadata: None,
                };
                return Ok((Some(message), unknown_formats));
            }
        };

        // Build content blocks
        let mut content_blocks = Vec::new();

        // Add main text content (strip system reminder tags)
        if let Some(text) = &bubble.text {
            let cleaned = crate::parsers::strip_system_reminders(text);
            if !cleaned.is_empty() {
                content_blocks.push(ContentBlock::Text { text: cleaned, is_degraded: None });
            }
        }

        // Story 8.17: Parse allThinkingBlocks (AC1)
        for thinking_block in &bubble.all_thinking_blocks {
            if let Some(thinking_text) = thinking_block.get_text() {
                if !thinking_text.is_empty() {
                    // Convert timestamp from epoch_ms to ISO 8601 string if available
                    let timestamp_str = thinking_block.get_timestamp()
                        .map(|ms| epoch_ms_to_datetime(ms).to_rfc3339());

                    content_blocks.push(ContentBlock::Thinking {
                        thinking: thinking_text.to_string(),
                        subject: thinking_block.get_subject().map(|s| s.to_string()),
                        timestamp: timestamp_str,
                    });
                }
            }
        }

        // Parse toolFormerData (PRIMARY: this is where Cursor stores tool call data)
        // Story 8.15: Enhanced with unknown format collection
        if let Some(tfd) = &bubble.tool_former_data {
            let (blocks, tfd_unknown) = self.process_tool_former_data(tfd);
            content_blocks.extend(blocks);
            unknown_formats.extend(tfd_unknown);
        }

        // Fallback: parse legacy toolResults (usually empty, but kept for backwards compatibility)
        if bubble.tool_former_data.is_none() {
            for tool_result in &bubble.tool_results {
                if let (Some(id), Some(name)) = (&tool_result.id, &tool_result.name) {
                    let correlation_id = Some(id.clone());
                    let input = serde_json::json!({});
                    let standard_tool = normalize_tool(name, &input);

                    content_blocks.push(ContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                        correlation_id: correlation_id.clone(),
                        standard_tool: Some(standard_tool),
                        display_name: None,
                        description: None,
                    });

                    if let Some(result) = &tool_result.result {
                        // Fix: Handle serde_json::Value correctly
                        // - If String: use the string content directly
                        // - If Object/other: serialize to JSON string
                        let result_str = match result {
                            serde_json::Value::String(s) => s.clone(),
                            _ => serde_json::to_string(result).unwrap_or_default(),
                        };

                        let (cleaned_result, display_content, structured_result) =
                            process_tool_result_content(name, &result_str, &input);

                        content_blocks.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: cleaned_result,
                            is_error: tool_result.is_error,
                            correlation_id,
                            structured_result,
                            display_content,
                            render_as_markdown: None,
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

        // Story 8.16: Parse images array (AC4)
        for image in &bubble.images {
            // Handle base64 data images
            if let (Some(mime_type), Some(data)) = (&image.mime_type, &image.data) {
                if mime_type.starts_with("image/") && !data.is_empty() {
                    content_blocks.push(ContentBlock::Image {
                        media_type: mime_type.clone(),
                        data: data.clone(),
                        source_type: Some("base64".to_string()),
                        alt_text: image.alt.clone(),
                    });
                }
            }
            // Handle URL-based images
            else if let Some(url) = &image.url {
                if !url.is_empty() {
                    // For URL images, we store the URL in data field with source_type "url"
                    content_blocks.push(ContentBlock::Image {
                        media_type: image.mime_type.clone().unwrap_or_else(|| "image/unknown".to_string()),
                        data: url.clone(),
                        source_type: Some("url".to_string()),
                        alt_text: image.alt.clone(),
                    });
                }
            }
        }

        // Skip empty messages
        if content_blocks.is_empty() {
            return Ok((None, unknown_formats));
        }

        // Extract mentioned files from bubble context
        let mentioned_files = extract_mentioned_files_from_bubble(&bubble.context);

        // Build message
        let timestamp = bubble.timestamp.map(epoch_ms_to_datetime);

        Ok((Some(Message {
            role,
            content_blocks,
            timestamp,
            mentioned_files,
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        }), unknown_formats))
    }

    /// Process toolFormerData into ContentBlocks with unknown format collection
    /// Story 8.15: Returns (content_blocks, unknown_formats)
    fn process_tool_former_data(&self, tfd: &ToolFormerData) -> (Vec<ContentBlock>, Vec<UnknownFormatEntry>) {
        let mut content_blocks = Vec::new();
        let unknown_formats: Vec<UnknownFormatEntry> = Vec::new();

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
            let standard_tool = normalize_tool(name, &input);

            // Process tool result using unified helper function
            let (cleaned_result, display_content, structured_result) =
                if let Some(result_str) = &tfd.result {
                    let (cleaned, display, structured) = process_tool_result_content(name, result_str, &input);
                    (Some(cleaned), display, structured)
                } else {
                    (None, None, None)
                };

            // Add ToolUse block
            content_blocks.push(ContentBlock::ToolUse {
                id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                name: name.clone(),
                input,
                correlation_id: correlation_id.clone(),
                standard_tool: Some(standard_tool),
                display_name: None,
                description: None,
            });

            // Add ToolResult if result exists
            if let Some(content) = cleaned_result {
                content_blocks.push(ContentBlock::ToolResult {
                    tool_use_id: tfd.tool_call_id.clone().unwrap_or_else(|| format!("{}-{}", name, tfd.tool_index.unwrap_or(0))),
                    content,
                    is_error: tfd.status.as_deref() == Some("failed"),
                    correlation_id,
                    structured_result,
                    display_content,
                    render_as_markdown: None,
                    user_decision: tfd.user_decision.clone(),
                });
            }
        }

        (content_blocks, unknown_formats)
    }
}

/// Parse Cursor tool result into structured ToolResultData (Story 8.19)
///
/// Converts StandardTool and result string into ToolResultData for frontend display.
/// This enables displaying structured summaries like "读取 main.rs L10-L50" instead of raw JSON.
///
/// # Arguments
/// * `standard_tool` - The normalized StandardTool enum
/// * `result` - The raw result string from toolFormerData.result
///
/// # Returns
/// * `Some(ToolResultData)` - Structured result data for known tool types
/// * `None` - For unknown tools or parsing failures (backward compatible)
fn parse_cursor_tool_result(standard_tool: &StandardTool, result: &str) -> Option<ToolResultData> {
    match standard_tool {
        // AC1: FileRead 结构化结果
        StandardTool::FileRead { path, start_line, end_line: _ } => {
            // Calculate num_lines from result content
            let num_lines = if !result.is_empty() {
                Some(result.lines().count() as u32)
            } else {
                None
            };

            Some(ToolResultData::FileRead {
                file_path: path.clone(),
                start_line: start_line.map(|v| v),
                num_lines,
                total_lines: None, // Cursor doesn't provide total_lines in result
            })
        }

        // AC2: FileWrite 结构化结果
        StandardTool::FileWrite { path, .. } => {
            Some(ToolResultData::FileWrite {
                file_path: path.clone(),
            })
        }

        // AC3: FileEdit 结构化结果
        StandardTool::FileEdit { path, old_string, new_string } => {
            Some(ToolResultData::FileEdit {
                file_path: path.clone(),
                old_string: old_string.clone(),
                new_string: new_string.clone(),
            })
        }

        // AC4: ShellExec 结构化结果
        StandardTool::ShellExec { .. } => {
            // Try to extract exit code from result if present
            // Common patterns: "exit code: 0", "Exit code: 1", etc.
            let exit_code = extract_exit_code_from_result(result);

            Some(ToolResultData::ShellExec {
                exit_code,
                stdout: if !result.is_empty() { Some(result.to_string()) } else { None },
                stderr: None, // Cursor doesn't separate stdout/stderr
            })
        }

        // AC5: 向后兼容 - Unknown and other tools return None
        _ => None,
    }
}

/// Extract exit code from shell command result string
///
/// Tries to find patterns like "exit code: 0", "Exit Code: 1", "exited with 0", etc.
fn extract_exit_code_from_result(result: &str) -> Option<i32> {
    // Common patterns for exit code in terminal output
    let patterns = [
        r"(?i)exit\s*code[:\s]+(\d+)",
        r"(?i)exited\s+with\s+(\d+)",
        r"(?i)returned\s+(\d+)",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(result) {
                if let Some(code_str) = caps.get(1) {
                    if let Ok(code) = code_str.as_str().parse::<i32>() {
                        return Some(code);
                    }
                }
            }
        }
    }

    None
}

/// Extract display content from Cursor JSON result (Story 8.19)
///
/// Cursor's read_file may return JSON in various formats:
/// - {"contents": "file content...", "numCharactersInRequestedRange": 5134, ...}
/// - {"content": "file content..."}
/// - {"text": "file content..."}
/// - {"value": "file content..."}
///
/// This function tries multiple common field names to extract the actual content.
fn extract_display_content_from_result(result: &str) -> Option<String> {
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(result) {
        // Try multiple common field names for content extraction
        // Order: most specific to most generic
        let content_fields = ["contents", "content", "text", "value", "output", "data"];
        
        for field in content_fields {
            if let Some(content) = json.get(field).and_then(|v| v.as_str()) {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    // Not JSON or no extractable content - return None (use original content)
    None
}

/// Process tool result content - unified logic for both toolFormerData and legacy paths
///
/// Extracts display content from JSON and generates structured result data.
/// This function consolidates the common processing logic to avoid duplication.
///
/// # Arguments
/// * `tool_name` - The tool name (e.g., "read_file", "edit_file")
/// * `result` - The raw result string
/// * `input` - The tool input parameters (used for StandardTool normalization)
///
/// # Returns
/// Tuple of (cleaned_content, display_content, structured_result)
fn process_tool_result_content(
    tool_name: &str,
    result: &str,
    input: &serde_json::Value,
) -> (String, Option<String>, Option<ToolResultData>) {
    // Strip system reminder tags
    let cleaned_content = crate::parsers::strip_system_reminders(result);

    // Extract display content from JSON
    let display_content = extract_display_content_from_result(&cleaned_content);

    // Normalize tool and parse structured result
    let standard_tool = normalize_tool(tool_name, input);
    let content_for_parsing = display_content.as_deref().unwrap_or(&cleaned_content);
    let structured_result = parse_cursor_tool_result(&standard_tool, content_for_parsing);

    (cleaned_content, display_content, structured_result)
}

/// Convert epoch milliseconds to DateTime<Utc>
fn epoch_ms_to_datetime(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Truncate raw JSON to maximum size for storage (Story 8.15)
fn truncate_raw_json(json: &serde_json::Value) -> String {
    let raw = serde_json::to_string(json).unwrap_or_default();
    if raw.len() <= MAX_RAW_JSON_SIZE {
        raw
    } else {
        format!("{}... [truncated]", &raw[..MAX_RAW_JSON_SIZE])
    }
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
mod tests;
