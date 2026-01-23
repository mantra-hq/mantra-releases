//! Gemini CLI data types and structures
//!
//! Defines types for parsing Gemini CLI's conversation JSON files.
//!
//! ## Session File Structure
//!
//! ```text
//! ~/.gemini/tmp/{projectHash}/chats/session-{date}-{uuid}.json
//! ```
//!
//! ## File Format
//!
//! ```json
//! {
//!   "sessionId": "uuid",
//!   "projectHash": "sha256hex",
//!   "startTime": "ISO8601",
//!   "lastUpdated": "ISO8601",
//!   "messages": [...],
//!   "summary": "optional summary"
//! }
//! ```
//!
//! ## ToolResultDisplay Types
//!
//! Gemini CLI's `ToolResultDisplay` is a union type:
//! ```typescript
//! type ToolResultDisplay = string | FileDiff | AnsiOutput | TodoList;
//! ```
//!
//! This module provides complete support for all variants.

use serde::{Deserialize, Deserializer, Serialize};

// ===== ToolResultDisplay Types (matching Gemini CLI official definitions) =====

/// Diff statistics for file edits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiffStat {
    pub model_added_lines: i32,
    pub model_removed_lines: i32,
    pub model_added_chars: i32,
    pub model_removed_chars: i32,
    pub user_added_lines: i32,
    pub user_removed_lines: i32,
    pub user_added_chars: i32,
    pub user_removed_chars: i32,
}

/// File diff display (from Gemini CLI write-file/edit tools)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    /// The unified diff content
    pub file_diff: String,
    /// The file name/path
    pub file_name: String,
    /// Original file content (null for new files)
    pub original_content: Option<String>,
    /// New file content after edit
    pub new_content: String,
    /// Optional diff statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_stat: Option<DiffStat>,
}

/// ANSI token for terminal output styling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnsiToken {
    pub text: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub dim: bool,
    #[serde(default)]
    pub inverse: bool,
    #[serde(default)]
    pub fg: String,
    #[serde(default)]
    pub bg: String,
}

/// ANSI line (array of tokens)
pub type AnsiLine = Vec<AnsiToken>;

/// ANSI output (array of lines) - terminal output with styling
pub type AnsiOutput = Vec<AnsiLine>;

/// Todo item status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

/// Todo item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub description: String,
    pub status: TodoStatus,
}

/// Todo list display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoList {
    pub todos: Vec<TodoItem>,
}

/// Gemini CLI's ToolResultDisplay union type
///
/// Supports all official variants:
/// - String: Simple text output
/// - FileDiff: File edit with diff display
/// - AnsiOutput: Terminal output with ANSI styling
/// - TodoList: Todo list display
#[derive(Debug, Clone, PartialEq)]
pub enum ToolResultDisplay {
    /// Simple string content
    String(String),
    /// File diff display
    FileDiff(FileDiff),
    /// ANSI terminal output
    AnsiOutput(AnsiOutput),
    /// Todo list display
    TodoList(TodoList),
}

impl ToolResultDisplay {
    /// Get display content as string (for backward compatibility)
    pub fn as_display_string(&self) -> String {
        match self {
            ToolResultDisplay::String(s) => s.clone(),
            ToolResultDisplay::FileDiff(diff) => diff.file_diff.clone(),
            ToolResultDisplay::AnsiOutput(output) => {
                // Convert ANSI output to plain text
                output
                    .iter()
                    .map(|line| {
                        line.iter().map(|token| token.text.as_str()).collect::<Vec<_>>().join("")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            ToolResultDisplay::TodoList(list) => {
                list.todos
                    .iter()
                    .map(|todo| format!("[{:?}] {}", todo.status, todo.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    /// Check if this is a FileDiff variant
    pub fn is_file_diff(&self) -> bool {
        matches!(self, ToolResultDisplay::FileDiff(_))
    }

    /// Get FileDiff if this is a FileDiff variant
    pub fn as_file_diff(&self) -> Option<&FileDiff> {
        match self {
            ToolResultDisplay::FileDiff(diff) => Some(diff),
            _ => None,
        }
    }
}

/// Custom deserializer for ToolResultDisplay
///
/// Handles the union type: string | FileDiff | AnsiOutput | TodoList
impl<'de> Deserialize<'de> for ToolResultDisplay {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        match value {
            serde_json::Value::String(s) => Ok(ToolResultDisplay::String(s)),
            serde_json::Value::Array(arr) => {
                // AnsiOutput is Array<Array<AnsiToken>>
                // Try to deserialize as AnsiOutput
                match serde_json::from_value::<AnsiOutput>(serde_json::Value::Array(arr.clone())) {
                    Ok(output) => Ok(ToolResultDisplay::AnsiOutput(output)),
                    Err(_) => {
                        // Fallback: convert to string representation
                        Ok(ToolResultDisplay::String(serde_json::to_string(&arr).unwrap_or_default()))
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                // Could be FileDiff or TodoList
                // Check for fileDiff field (FileDiff)
                if obj.contains_key("fileDiff") {
                    match serde_json::from_value::<FileDiff>(serde_json::Value::Object(obj.clone())) {
                        Ok(diff) => return Ok(ToolResultDisplay::FileDiff(diff)),
                        Err(_) => {}
                    }
                }
                // Check for todos field (TodoList)
                if obj.contains_key("todos") {
                    match serde_json::from_value::<TodoList>(serde_json::Value::Object(obj.clone())) {
                        Ok(list) => return Ok(ToolResultDisplay::TodoList(list)),
                        Err(_) => {}
                    }
                }
                // Fallback: serialize object as JSON string
                Ok(ToolResultDisplay::String(serde_json::to_string(&obj).unwrap_or_default()))
            }
            other => {
                // Other types: convert to string
                Ok(ToolResultDisplay::String(other.to_string()))
            }
        }
    }
}

impl Serialize for ToolResultDisplay {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ToolResultDisplay::String(s) => serializer.serialize_str(s),
            ToolResultDisplay::FileDiff(diff) => diff.serialize(serializer),
            ToolResultDisplay::AnsiOutput(output) => output.serialize(serializer),
            ToolResultDisplay::TodoList(list) => list.serialize(serializer),
        }
    }
}

/// Custom deserializer for Option<ToolResultDisplay>
fn deserialize_result_display<'de, D>(deserializer: D) -> Result<Option<ToolResultDisplay>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(v) => {
            // Use the ToolResultDisplay deserializer
            match serde_json::from_value::<ToolResultDisplay>(v) {
                Ok(display) => Ok(Some(display)),
                Err(_) => Ok(None),
            }
        }
    }
}

/// Gemini CLI conversation record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiConversation {
    /// Session unique ID
    pub session_id: String,

    /// Project hash (SHA256 of project root path)
    pub project_hash: String,

    /// Session start time (ISO 8601)
    pub start_time: String,

    /// Last update time (ISO 8601)
    pub last_updated: String,

    /// Messages in the conversation
    #[serde(default)]
    pub messages: Vec<GeminiMessage>,

    /// Optional summary
    pub summary: Option<String>,
}

/// Gemini CLI message record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMessage {
    /// Message unique ID
    pub id: String,

    /// Message timestamp (ISO 8601)
    pub timestamp: String,

    /// Message content (string or array of parts)
    #[serde(default)]
    pub content: GeminiContent,

    /// Message type: user, gemini, info, error, warning
    #[serde(rename = "type")]
    pub msg_type: GeminiMessageType,

    /// Tool calls made by Gemini (only for gemini type)
    #[serde(default)]
    pub tool_calls: Option<Vec<GeminiToolCall>>,

    /// Gemini's thoughts/reasoning (only for gemini type)
    #[serde(default)]
    pub thoughts: Option<Vec<GeminiThought>>,

    /// Token usage (only for gemini type)
    #[serde(default)]
    pub tokens: Option<GeminiTokens>,

    /// Model used (only for gemini type)
    #[serde(default)]
    pub model: Option<String>,
}

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeminiMessageType {
    /// User message
    User,
    /// Gemini assistant message
    Gemini,
    /// System info message (skip)
    Info,
    /// Error message (skip)
    Error,
    /// Warning message (skip)
    Warning,
}

impl GeminiMessageType {
    /// Check if this message type should be included in the session
    pub fn should_include(self) -> bool {
        matches!(self, GeminiMessageType::User | GeminiMessageType::Gemini)
    }

    /// Convert to MantraSession Role
    pub fn to_mantra_role(self) -> Option<crate::models::Role> {
        match self {
            GeminiMessageType::User => Some(crate::models::Role::User),
            GeminiMessageType::Gemini => Some(crate::models::Role::Assistant),
            _ => None,
        }
    }
}

/// Content can be either a simple string or an array of parts (PartListUnion)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum GeminiContent {
    /// Simple text content
    Text(String),
    /// Array of content parts
    Parts(Vec<GeminiPart>),
}

impl Default for GeminiContent {
    fn default() -> Self {
        GeminiContent::Text(String::new())
    }
}

impl GeminiContent {
    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            GeminiContent::Text(s) => s.is_empty(),
            GeminiContent::Parts(parts) => parts.is_empty(),
        }
    }

    /// Get text content as string
    pub fn as_text(&self) -> String {
        match self {
            GeminiContent::Text(s) => s.clone(),
            GeminiContent::Parts(parts) => {
                parts
                    .iter()
                    .filter_map(|p| p.text.as_ref())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }
}

/// Content part (from @google/genai Part type)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPart {
    /// Text content
    pub text: Option<String>,

    /// Inline data (e.g., images)
    pub inline_data: Option<GeminiInlineData>,

    /// Function call
    pub function_call: Option<GeminiFunctionCall>,

    /// Function response
    pub function_response: Option<GeminiFunctionResponse>,

    /// Story 8.15: Capture unknown fields for degradation monitoring
    #[serde(flatten)]
    pub unknown_fields: serde_json::Map<String, serde_json::Value>,
}

impl GeminiPart {
    /// Check if this part has any unknown/unrecognized fields
    pub fn has_unknown_fields(&self) -> bool {
        !self.unknown_fields.is_empty()
    }

    /// Get the list of unknown field names
    pub fn unknown_field_names(&self) -> Vec<String> {
        self.unknown_fields.keys().cloned().collect()
    }
}

/// Inline data (e.g., base64 encoded images)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineData {
    /// MIME type
    pub mime_type: Option<String>,
    /// Base64 encoded data
    pub data: Option<String>,
}

/// Function call in content
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments
    #[serde(default)]
    pub args: serde_json::Value,
}

/// Function response in content
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionResponse {
    /// Function name
    pub name: String,
    /// Response content
    #[serde(default)]
    pub response: serde_json::Value,
}

/// Tool call record
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolCall {
    /// Tool call ID
    pub id: String,

    /// Tool/function name
    pub name: String,

    /// Arguments passed to the tool
    #[serde(default)]
    pub args: serde_json::Value,

    /// Tool results
    #[serde(default)]
    pub result: Option<Vec<GeminiToolResultWrapper>>,

    /// Execution status
    #[serde(default)]
    pub status: String,

    /// Execution timestamp
    pub timestamp: Option<String>,

    /// Human-readable tool name for display (e.g., "Shell", "Edit File")
    #[serde(default)]
    pub display_name: Option<String>,

    /// Tool description for UI display
    #[serde(default)]
    pub description: Option<String>,

    /// Pre-formatted result for display (supports multiple types: string | FileDiff | AnsiOutput | TodoList)
    #[serde(default, deserialize_with = "deserialize_result_display")]
    pub result_display: Option<ToolResultDisplay>,

    /// Whether to render output as markdown
    #[serde(default)]
    pub render_output_as_markdown: Option<bool>,
}

/// Wrapper for tool result containing functionResponse
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolResultWrapper {
    /// Function response object
    pub function_response: GeminiFunctionResponseResult,
}

/// Function response result
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionResponseResult {
    /// Function call ID
    pub id: String,

    /// Function name
    pub name: String,

    /// Response content
    #[serde(default)]
    pub response: GeminiToolResponse,
}

/// Tool response content
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolResponse {
    /// Output text
    pub output: Option<String>,

    /// Error message if failed
    pub error: Option<String>,

    /// Additional fields stored as raw Value
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Parsed shell result from Gemini CLI's multi-line format
#[derive(Debug, Clone, Default)]
pub struct ParsedShellResult {
    /// The actual command output (from "Output:" or "Stdout:" field)
    pub output: Option<String>,
    /// Error output (from "Stderr:" field)
    pub stderr: Option<String>,
    /// Error message (from "Error:" field)
    pub error: Option<String>,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Signal number
    pub signal: Option<i32>,
    /// The executed command
    pub command: Option<String>,
    /// Working directory
    pub directory: Option<String>,
}

impl GeminiToolResponse {
    /// Get the output or error as a string
    pub fn as_content(&self) -> String {
        if let Some(output) = &self.output {
            output.clone()
        } else if let Some(error) = &self.error {
            format!("Error: {}", error)
        } else {
            String::new()
        }
    }

    /// Check if the output is in Gemini CLI's multi-line shell result format
    ///
    /// Gemini CLI stores shell results in this format:
    /// ```text
    /// Command: <command>
    /// Directory: <dir or (root)>
    /// Output: <output or (empty)>
    /// Error: <error or (none)>
    /// Exit Code: <code or (none)>
    /// Signal: <signal or (none)>
    /// Background PIDs: <pids or (none)>
    /// Process Group PGID: <pgid or (none)>
    /// ```
    pub fn is_shell_result_format(&self) -> bool {
        if let Some(output) = &self.output {
            // Check for characteristic shell result patterns
            output.starts_with("Command:") && output.contains("\nOutput:")
        } else {
            false
        }
    }

    /// Parse Gemini CLI's multi-line shell result format
    ///
    /// Returns parsed components if the output matches the shell result format,
    /// otherwise returns None.
    pub fn parse_shell_result(&self) -> Option<ParsedShellResult> {
        let output = self.output.as_ref()?;

        // Quick check: must have "Command:" and "Output:" markers
        if !output.starts_with("Command:") || !output.contains("\nOutput:") {
            return None;
        }

        let mut result = ParsedShellResult::default();
        let mut current_field: Option<&str> = None;
        let mut current_value = String::new();

        // Helper to save the current field value
        let mut save_field = |field: Option<&str>, value: &str| {
            if let Some(f) = field {
                let trimmed = value.trim();
                let is_empty = trimmed.is_empty() || trimmed == "(empty)" || trimmed == "(none)";

                match f {
                    "Command" => result.command = if is_empty { None } else { Some(trimmed.to_string()) },
                    "Directory" => result.directory = if trimmed == "(root)" || is_empty { None } else { Some(trimmed.to_string()) },
                    "Output" | "Stdout" => result.output = if is_empty { None } else { Some(trimmed.to_string()) },
                    "Stderr" => result.stderr = if is_empty { None } else { Some(trimmed.to_string()) },
                    "Error" => result.error = if is_empty { None } else { Some(trimmed.to_string()) },
                    "Exit Code" => result.exit_code = trimmed.parse().ok(),
                    "Signal" => result.signal = trimmed.parse().ok(),
                    _ => {} // Ignore other fields like "Background PIDs", "Process Group PGID"
                }
            }
        };

        // Parse line by line
        for line in output.lines() {
            // Check if this line starts a new field
            let field_markers = [
                "Command:", "Directory:", "Output:", "Stdout:", "Stderr:",
                "Error:", "Exit Code:", "Signal:", "Background PIDs:", "Process Group PGID:",
            ];

            let mut found_new_field = false;
            for marker in field_markers {
                if line.starts_with(marker) {
                    // Save previous field
                    save_field(current_field, &current_value);

                    // Start new field
                    let field_name = marker.trim_end_matches(':');
                    current_field = Some(field_name);
                    current_value = line[marker.len()..].to_string();
                    found_new_field = true;
                    break;
                }
            }

            // If not a new field, append to current value (multi-line output)
            if !found_new_field && current_field.is_some() {
                current_value.push('\n');
                current_value.push_str(line);
            }
        }

        // Save the last field
        save_field(current_field, &current_value);

        Some(result)
    }
}

/// Thought/reasoning summary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiThought {
    /// Thought subject/title
    pub subject: String,

    /// Thought description/content
    pub description: String,

    /// Thought timestamp
    pub timestamp: Option<String>,
}

impl GeminiThought {
    /// Format thought as a string for ContentBlock::Thinking
    pub fn as_formatted_string(&self) -> String {
        format!("**{}** {}", self.subject, self.description)
    }
}

/// Token usage summary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTokens {
    /// Input/prompt tokens
    pub input: Option<i64>,

    /// Output/completion tokens
    pub output: Option<i64>,

    /// Cached tokens (from Gemini CLI: cachedContentTokenCount)
    pub cached: Option<i64>,

    /// Thoughts tokens (optional)
    pub thoughts: Option<i64>,

    /// Tool use tokens (optional)
    pub tool: Option<i64>,

    /// Total tokens (the authoritative sum)
    pub total: Option<i64>,

    /// Cache read tokens (legacy field)
    pub cache_read: Option<i64>,

    /// Cache write tokens (legacy field)
    pub cache_write: Option<i64>,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
