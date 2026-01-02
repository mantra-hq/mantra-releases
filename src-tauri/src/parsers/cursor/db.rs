//! SQLite database access layer for Cursor data
//!
//! Provides access to Cursor's state.vscdb databases:
//! - workspaceStorage/{hash}/state.vscdb (ItemTable) - conversation index
//! - globalStorage/state.vscdb (cursorDiskKV) - composer data and bubble content

use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;

use crate::parsers::ParseError;

/// Database wrapper for Cursor state.vscdb
pub struct CursorDatabase {
    conn: Connection,
}

/// Composer summary from workspace's allComposers
#[derive(Debug, Clone)]
pub struct ComposerSummary {
    /// Unique composer ID
    pub composer_id: String,
    /// Composer name/title
    pub name: Option<String>,
    /// Creation timestamp (epoch milliseconds)
    pub created_at: Option<i64>,
    /// Mode (e.g., "agent", "chat")
    pub unified_mode: Option<String>,
    /// Total lines added
    pub total_lines_added: Option<i64>,
    /// Total lines removed
    pub total_lines_removed: Option<i64>,
}

impl CursorDatabase {
    /// Open a Cursor state.vscdb database (read-only)
    pub fn open(path: &Path) -> Result<Self, ParseError> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| ParseError::invalid_format(format!("Failed to open database: {}", e)))?;

        Ok(Self { conn })
    }

    /// List all composers from workspaceStorage/state.vscdb (ItemTable)
    /// Key pattern: composer.composerData -> JSON with allComposers array
    pub fn list_composers(&self) -> Result<Vec<ComposerSummary>, ParseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM ItemTable WHERE key = 'composer.composerData'")
            .map_err(|e| ParseError::invalid_format(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| ParseError::invalid_format(format!("Failed to execute query: {}", e)))?;

        if let Some(row) = rows.next().map_err(|e| {
            ParseError::invalid_format(format!("Failed to fetch row: {}", e))
        })? {
            let value: String = row.get(0).map_err(|e| {
                ParseError::invalid_format(format!("Failed to get value: {}", e))
            })?;

            let json: Value = serde_json::from_str(&value)?;
            return parse_all_composers(&json);
        }

        Ok(Vec::new())
    }

    /// Get composer metadata from globalStorage/state.vscdb (cursorDiskKV)
    /// Key pattern: composerData:{composerId}
    pub fn get_composer_data(&self, composer_id: &str) -> Result<Option<Value>, ParseError> {
        let key = format!("composerData:{}", composer_id);

        let mut stmt = self
            .conn
            .prepare("SELECT value FROM cursorDiskKV WHERE key = ?1")
            .map_err(|e| ParseError::invalid_format(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt.query([&key]).map_err(|e| {
            ParseError::invalid_format(format!("Failed to execute query: {}", e))
        })?;

        if let Some(row) = rows.next().map_err(|e| {
            ParseError::invalid_format(format!("Failed to fetch row: {}", e))
        })? {
            let value: String = row.get(0).map_err(|e| {
                ParseError::invalid_format(format!("Failed to get value: {}", e))
            })?;

            let json: Value = serde_json::from_str(&value)?;
            return Ok(Some(json));
        }

        Ok(None)
    }

    /// Get bubble content from globalStorage/state.vscdb (cursorDiskKV)
    /// Key pattern: bubbleId:{composerId}:{bubbleId}
    pub fn get_bubble_content(
        &self,
        composer_id: &str,
        bubble_id: &str,
    ) -> Result<Option<Value>, ParseError> {
        let key = format!("bubbleId:{}:{}", composer_id, bubble_id);

        let mut stmt = self
            .conn
            .prepare("SELECT value FROM cursorDiskKV WHERE key = ?1")
            .map_err(|e| ParseError::invalid_format(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt.query([&key]).map_err(|e| {
            ParseError::invalid_format(format!("Failed to execute query: {}", e))
        })?;

        if let Some(row) = rows.next().map_err(|e| {
            ParseError::invalid_format(format!("Failed to fetch row: {}", e))
        })? {
            let value: String = row.get(0).map_err(|e| {
                ParseError::invalid_format(format!("Failed to get value: {}", e))
            })?;

            let json: Value = serde_json::from_str(&value)?;
            return Ok(Some(json));
        }

        Ok(None)
    }

    /// Get all bubble IDs for a composer from globalStorage/state.vscdb
    /// This queries for keys matching pattern bubbleId:{composerId}:*
    pub fn list_bubble_ids(&self, composer_id: &str) -> Result<Vec<String>, ParseError> {
        let prefix = format!("bubbleId:{}:", composer_id);

        let mut stmt = self
            .conn
            .prepare("SELECT key FROM cursorDiskKV WHERE key LIKE ?1")
            .map_err(|e| ParseError::invalid_format(format!("Failed to prepare query: {}", e)))?;

        let pattern = format!("{}%", prefix);
        let mut rows = stmt.query([&pattern]).map_err(|e| {
            ParseError::invalid_format(format!("Failed to execute query: {}", e))
        })?;

        let mut bubble_ids = Vec::new();
        while let Some(row) = rows.next().map_err(|e| {
            ParseError::invalid_format(format!("Failed to fetch row: {}", e))
        })? {
            let key: String = row.get(0).map_err(|e| {
                ParseError::invalid_format(format!("Failed to get key: {}", e))
            })?;

            // Extract bubble ID from key: bubbleId:{composerId}:{bubbleId}
            if let Some(bubble_id) = key.strip_prefix(&prefix) {
                bubble_ids.push(bubble_id.to_string());
            }
        }

        Ok(bubble_ids)
    }
}

/// Parse allComposers array from composer.composerData JSON
fn parse_all_composers(json: &Value) -> Result<Vec<ComposerSummary>, ParseError> {
    let all_composers = json
        .get("allComposers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ParseError::missing_field("allComposers"))?;

    let mut composers = Vec::new();

    for composer in all_composers {
        let composer_id = composer
            .get("composerId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(id) = composer_id {
            composers.push(ComposerSummary {
                composer_id: id,
                name: composer.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                created_at: composer.get("createdAt").and_then(|v| v.as_i64()),
                unified_mode: composer.get("unifiedMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
                total_lines_added: composer.get("totalLinesAdded").and_then(|v| v.as_i64()),
                total_lines_removed: composer.get("totalLinesRemoved").and_then(|v| v.as_i64()),
            });
        }
    }

    Ok(composers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_all_composers() {
        let json = json!({
            "allComposers": [
                {
                    "composerId": "abc-123",
                    "name": "Test Composer",
                    "createdAt": 1704067200000_i64,
                    "unifiedMode": "agent",
                    "totalLinesAdded": 100,
                    "totalLinesRemoved": 50
                },
                {
                    "composerId": "def-456",
                    "name": null,
                    "createdAt": 1704153600000_i64
                }
            ]
        });

        let composers = parse_all_composers(&json).unwrap();
        assert_eq!(composers.len(), 2);

        assert_eq!(composers[0].composer_id, "abc-123");
        assert_eq!(composers[0].name, Some("Test Composer".to_string()));
        assert_eq!(composers[0].created_at, Some(1704067200000));
        assert_eq!(composers[0].unified_mode, Some("agent".to_string()));
        assert_eq!(composers[0].total_lines_added, Some(100));

        assert_eq!(composers[1].composer_id, "def-456");
        assert_eq!(composers[1].name, None);
    }

    #[test]
    fn test_parse_all_composers_empty() {
        let json = json!({
            "allComposers": []
        });

        let composers = parse_all_composers(&json).unwrap();
        assert!(composers.is_empty());
    }

    #[test]
    fn test_parse_all_composers_missing_field() {
        let json = json!({
            "otherField": "value"
        });

        let result = parse_all_composers(&json);
        assert!(matches!(result, Err(ParseError::MissingField(_))));
    }
}
