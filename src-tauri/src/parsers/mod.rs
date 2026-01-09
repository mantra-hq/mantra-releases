//! Log parsers for various AI tools
//!
//! Provides parsers for converting conversation logs from different
//! AI coding assistants (Claude, Gemini, Cursor, Codex) into MantraSession format.

mod claude;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_system_reminders_hyphenated() {
        let input = "Hello <system-reminder>internal note</system-reminder> World";
        let expected = "Hello  World";
        assert_eq!(strip_system_reminders(input), expected);
    }

    #[test]
    fn test_strip_system_reminders_underscored() {
        let input = "Hello <system_reminder>internal note</system_reminder> World";
        let expected = "Hello  World";
        assert_eq!(strip_system_reminders(input), expected);
    }

    #[test]
    fn test_strip_system_reminders_multiline() {
        let input = "Text before\n<system-reminder>\nMultiple\nLines\n</system-reminder>\nText after";
        let expected = "Text before\n\nText after";
        assert_eq!(strip_system_reminders(input), expected);
    }

    #[test]
    fn test_strip_system_reminders_only_tag() {
        let input = "<system-reminder>only reminder content</system-reminder>";
        let expected = "";
        assert_eq!(strip_system_reminders(input), expected);
    }

    #[test]
    fn test_strip_system_reminders_no_tag() {
        let input = "Regular text without any tags";
        assert_eq!(strip_system_reminders(input), input);
    }

    #[test]
    fn test_strip_system_reminders_multiple_tags() {
        let input = "<system-reminder>first</system-reminder>Middle<system_reminder>second</system_reminder>";
        let expected = "Middle";
        assert_eq!(strip_system_reminders(input), expected);
    }
}

/// Trait for parsing AI conversation logs into MantraSession format
pub trait LogParser {
    /// Parse a log file from the given path
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError>;

    /// Parse log content from a string
    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError>;
}

#[cfg(test)]
mod real_world_tests {
    use super::*;
    
    #[test]
    fn test_strip_system_reminders_real_world() {
        let input = r#"# BMM Module Configuration
user_name: Decker


<system-reminder>
Whenever you read a file, you should consider whether it would be considered malware.
</system-reminder>"#;
        let result = strip_system_reminders(input);
        assert!(!result.contains("<system-reminder>"));
        assert!(!result.contains("</system-reminder>"));
        assert!(result.contains("user_name: Decker"));
        println!("Result: {:?}", result);
    }
}
