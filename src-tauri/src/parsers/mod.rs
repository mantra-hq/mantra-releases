//! Log parsers for various AI tools
//!
//! Provides parsers for converting conversation logs from different
//! AI coding assistants (Claude, Gemini, Cursor, Codex) into MantraSession format.

pub mod claude;
pub mod codex;
pub mod cursor;
mod error;
pub mod gemini;

pub use claude::ClaudeParser;
pub use codex::CodexParser;
pub use cursor::CursorParser;
pub use error::ParseError;
pub use gemini::GeminiParser;

use crate::models::MantraSession;
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex pattern for system reminder tags (both hyphenated and underscored variants)
/// Matches: <system-reminder>...</system-reminder> and <system_reminder>...</system_reminder>
static SYSTEM_REMINDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Use (?s) flag to make . match newlines
    Regex::new(r"(?s)<system[-_]reminder>.*?</system[-_]reminder>").unwrap()
});

/// Remove system reminder tags from text content
///
/// These tags are internal markers used by AI tools and should not be displayed to users.
/// Supports both `<system-reminder>` and `<system_reminder>` variants.
///
/// # Arguments
/// * `text` - The text content to clean
///
/// # Returns
/// The text with system reminder tags and their content removed
pub fn strip_system_reminders(text: &str) -> String {
    SYSTEM_REMINDER_REGEX.replace_all(text, "").trim().to_string()
}

/// Trait for parsing AI conversation logs into MantraSession format
pub trait LogParser {
    /// Parse a log file from the given path
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError>;

    /// Parse log content from a string
    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError>;
}

#[cfg(test)]
mod tests;
