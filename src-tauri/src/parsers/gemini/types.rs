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

use serde::Deserialize;

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

    /// Pre-formatted result for display (cleaner than raw output)
    #[serde(default)]
    pub result_display: Option<String>,

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
