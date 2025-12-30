//! Log parsers for various AI tools
//!
//! Provides parsers for converting conversation logs from different
//! AI coding assistants (Claude, Gemini, Cursor) into MantraSession format.

mod claude;
mod error;

pub use claude::ClaudeParser;
pub use error::ParseError;

use crate::models::MantraSession;

/// Trait for parsing AI conversation logs into MantraSession format
pub trait LogParser {
    /// Parse a log file from the given path
    fn parse_file(&self, path: &str) -> Result<MantraSession, ParseError>;

    /// Parse log content from a string
    fn parse_string(&self, content: &str) -> Result<MantraSession, ParseError>;
}
