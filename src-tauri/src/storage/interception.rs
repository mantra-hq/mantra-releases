//! Interception records storage operations (Story 3.7)
//!
//! Provides CRUD operations for privacy scan interception records.

use chrono::{Duration, Utc};
use rusqlite::params;
use std::collections::HashMap;

use super::database::Database;
use super::error::StorageError;
use crate::sanitizer::{
    InterceptionRecord, InterceptionSource, InterceptionStats, PaginatedRecords, UserAction,
};

impl Database {
    /// Save an interception record to the database
    ///
    /// # Arguments
    /// * `record` - The InterceptionRecord to save
    ///
    /// # Returns
    /// Ok(()) on success, or StorageError on failure
    pub fn save_interception_record(&self, record: &InterceptionRecord) -> Result<(), StorageError> {
        let matches_json = serde_json::to_string(&record.matches)?;
        let source_context = record.source.context_json();

        self.connection().execute(
            r#"
            INSERT INTO interception_records
                (id, timestamp, source_type, source_context, matches, user_action, original_text_hash, project_name)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                record.id,
                record.timestamp.to_rfc3339(),
                record.source.source_type(),
                source_context,
                matches_json,
                record.user_action.as_str(),
                record.original_text_hash,
                record.project_name,
            ],
        )?;

        Ok(())
    }

    /// Get interception records with pagination
    ///
    /// # Arguments
    /// * `page` - Page number (1-based)
    /// * `per_page` - Records per page
    /// * `source_filter` - Optional source type filter ('pre_upload', 'claude_code_hook', 'external_hook')
    ///
    /// # Returns
    /// PaginatedRecords containing the records and pagination info
    pub fn get_interception_records(
        &self,
        page: u32,
        per_page: u32,
        source_filter: Option<&str>,
    ) -> Result<PaginatedRecords, StorageError> {
        let conn = self.connection();

        // Ensure valid pagination values
        let page = page.max(1);
        let per_page = per_page.max(1).min(100);
        let offset = (page - 1) * per_page;

        // Get total count
        let total: u64 = if let Some(source) = source_filter {
            conn.query_row(
                "SELECT COUNT(*) FROM interception_records WHERE source_type = ?1",
                params![source],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM interception_records",
                [],
                |row| row.get(0),
            )?
        };

        // Build query
        let query = if source_filter.is_some() {
            r#"
            SELECT id, timestamp, source_type, source_context, matches, user_action, original_text_hash, project_name
            FROM interception_records
            WHERE source_type = ?1
            ORDER BY timestamp DESC
            LIMIT ?2 OFFSET ?3
            "#
        } else {
            r#"
            SELECT id, timestamp, source_type, source_context, matches, user_action, original_text_hash, project_name
            FROM interception_records
            ORDER BY timestamp DESC
            LIMIT ?1 OFFSET ?2
            "#
        };

        let records: Vec<InterceptionRecord> = if let Some(source) = source_filter {
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(params![source, per_page, offset], |row| {
                Self::row_to_interception_record(row)
            })?;
            rows.filter_map(|r| r.ok()).collect()
        } else {
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(params![per_page, offset], |row| {
                Self::row_to_interception_record(row)
            })?;
            rows.filter_map(|r| r.ok()).collect()
        };

        Ok(PaginatedRecords {
            records,
            total,
            page,
            per_page,
        })
    }

    /// Maximum number of records that can be deleted in a single batch
    const MAX_DELETE_BATCH: usize = 1000;

    /// Delete interception records by IDs
    ///
    /// # Arguments
    /// * `ids` - List of record IDs to delete (max 1000)
    ///
    /// # Returns
    /// Number of deleted records
    ///
    /// # Errors
    /// Returns error if batch size exceeds MAX_DELETE_BATCH
    pub fn delete_interception_records(&self, ids: &[String]) -> Result<usize, StorageError> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Issue 4 fix: Limit batch size to prevent oversized SQL statements
        if ids.len() > Self::MAX_DELETE_BATCH {
            return Err(StorageError::InvalidInput(format!(
                "Delete batch size {} exceeds maximum allowed {}",
                ids.len(),
                Self::MAX_DELETE_BATCH
            )));
        }

        let conn = self.connection();

        // Build IN clause with placeholders
        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let in_clause = placeholders.join(", ");

        let query = format!(
            "DELETE FROM interception_records WHERE id IN ({})",
            in_clause
        );

        let mut stmt = conn.prepare(&query)?;

        // Bind parameters
        let params: Vec<&dyn rusqlite::ToSql> = ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let deleted = stmt.execute(params.as_slice())?;

        Ok(deleted)
    }

    /// Get interception statistics
    ///
    /// # Returns
    /// InterceptionStats with aggregated statistics
    ///
    /// # Performance
    /// Optimized to use single query for by_type and by_severity aggregation
    pub fn get_interception_stats(&self) -> Result<InterceptionStats, StorageError> {
        let conn = self.connection();

        // Total count
        let total_interceptions: u64 = conn.query_row(
            "SELECT COUNT(*) FROM interception_records",
            [],
            |row| row.get(0),
        )?;

        // Issue 1 fix: Single query for by_type and by_severity (merged from separate functions)
        let (by_type, by_severity) = self.get_stats_by_type_and_severity()?;

        // By action (efficient GROUP BY query)
        let by_action = self.get_stats_by_action()?;

        // Recent 7 days
        let seven_days_ago = Utc::now() - Duration::days(7);
        let recent_7_days: u64 = conn.query_row(
            "SELECT COUNT(*) FROM interception_records WHERE datetime(timestamp) >= datetime(?1)",
            params![seven_days_ago.to_rfc3339()],
            |row| row.get(0),
        )?;

        Ok(InterceptionStats {
            total_interceptions,
            by_type,
            by_severity,
            by_action,
            recent_7_days,
        })
    }

    /// Get statistics grouped by sensitive type AND severity in a single pass
    ///
    /// Issue 1 fix: Merged get_stats_by_type() and get_stats_by_severity() into single query
    fn get_stats_by_type_and_severity(&self) -> Result<(HashMap<String, u64>, HashMap<String, u64>), StorageError> {
        let conn = self.connection();
        let mut by_type = HashMap::new();
        let mut by_severity = HashMap::new();
        let mut parse_error_count = 0u64;

        // Single query for all matches
        let mut stmt = conn.prepare("SELECT matches FROM interception_records")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for matches_json in rows.filter_map(|r| r.ok()) {
            match serde_json::from_str::<Vec<serde_json::Value>>(&matches_json) {
                Ok(matches) => {
                    for m in matches {
                        // Aggregate by type
                        if let Some(sensitive_type) = m.get("sensitive_type").and_then(|v| v.as_str()) {
                            *by_type.entry(sensitive_type.to_string()).or_insert(0) += 1;
                        }
                        // Aggregate by severity
                        if let Some(severity) = m.get("severity").and_then(|v| v.as_str()) {
                            *by_severity.entry(severity.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                Err(e) => {
                    // Issue 2 fix: Log JSON parse errors instead of silently ignoring
                    parse_error_count += 1;
                    eprintln!(
                        "[WARN] Invalid matches JSON in interception record: {}",
                        e
                    );
                }
            }
        }

        if parse_error_count > 0 {
            eprintln!(
                "[WARN] {} interception records had invalid JSON matches",
                parse_error_count
            );
        }

        Ok((by_type, by_severity))
    }

    /// Get statistics grouped by user action
    fn get_stats_by_action(&self) -> Result<HashMap<String, u64>, StorageError> {
        let conn = self.connection();
        let mut stats = HashMap::new();

        let mut stmt =
            conn.prepare("SELECT user_action, COUNT(*) FROM interception_records GROUP BY user_action")?;
        let rows = stmt.query_map([], |row| {
            let action: String = row.get(0)?;
            let count: u64 = row.get(1)?;
            Ok((action, count))
        })?;

        for result in rows.filter_map(|r| r.ok()) {
            stats.insert(result.0, result.1);
        }

        Ok(stats)
    }

    /// Helper function to convert database row to InterceptionRecord
    fn row_to_interception_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<InterceptionRecord> {
        let id: String = row.get(0)?;
        let timestamp_str: String = row.get(1)?;
        let source_type: String = row.get(2)?;
        let source_context: Option<String> = row.get(3)?;
        let matches_json: String = row.get(4)?;
        let user_action_str: String = row.get(5)?;
        let original_text_hash: String = row.get(6)?;
        let project_name: Option<String> = row.get(7)?;

        // Parse timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Parse source
        let source = InterceptionSource::from_db(&source_type, source_context.as_deref())
            .unwrap_or(InterceptionSource::PreUpload {
                session_id: String::new(),
            });

        // Parse matches
        let matches = serde_json::from_str(&matches_json).unwrap_or_default();

        // Parse user action
        let user_action = UserAction::from_str(&user_action_str).unwrap_or(UserAction::Ignored);

        Ok(InterceptionRecord {
            id,
            timestamp,
            source,
            matches,
            user_action,
            original_text_hash,
            project_name,
        })
    }
}

#[cfg(test)]
mod tests;
