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
mod tests {
    use super::*;
    use crate::sanitizer::{ScanMatch, SensitiveType, Severity};

    fn create_test_record(
        source: InterceptionSource,
        action: UserAction,
        project: Option<&str>,
    ) -> InterceptionRecord {
        InterceptionRecord::new(
            source,
            vec![ScanMatch {
                rule_id: "test_rule".to_string(),
                sensitive_type: SensitiveType::ApiKey,
                severity: Severity::Critical,
                line: 1,
                column: 1,
                matched_text: "sk-test123456789".to_string(),
                masked_text: "sk-****".to_string(),
                context: "API key".to_string(),
            }],
            action,
            "hash123".to_string(),
            project.map(|s| s.to_string()),
        )
    }

    #[test]
    fn test_save_and_get_record() {
        let db = Database::new_in_memory().unwrap();

        let record = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-123".to_string(),
            },
            UserAction::Redacted,
            Some("test-project"),
        );

        // Save
        let result = db.save_interception_record(&record);
        assert!(result.is_ok(), "Save failed: {:?}", result.err());

        // Get
        let paginated = db.get_interception_records(1, 10, None).unwrap();
        assert_eq!(paginated.total, 1);
        assert_eq!(paginated.records.len(), 1);
        assert_eq!(paginated.records[0].id, record.id);
        assert_eq!(paginated.records[0].user_action, UserAction::Redacted);
        assert_eq!(
            paginated.records[0].project_name,
            Some("test-project".to_string())
        );
    }

    #[test]
    fn test_pagination() {
        let db = Database::new_in_memory().unwrap();

        // Insert 5 records
        for i in 0..5 {
            let record = create_test_record(
                InterceptionSource::ClaudeCodeHook {
                    session_id: Some(format!("sess-{}", i)),
                },
                UserAction::Ignored,
                None,
            );
            db.save_interception_record(&record).unwrap();
        }

        // Page 1 with 2 items
        let page1 = db.get_interception_records(1, 2, None).unwrap();
        assert_eq!(page1.total, 5);
        assert_eq!(page1.records.len(), 2);
        assert_eq!(page1.page, 1);
        assert_eq!(page1.per_page, 2);

        // Page 3 with 2 items (should have 1 record)
        let page3 = db.get_interception_records(3, 2, None).unwrap();
        assert_eq!(page3.total, 5);
        assert_eq!(page3.records.len(), 1);
    }

    #[test]
    fn test_source_filter() {
        let db = Database::new_in_memory().unwrap();

        // Insert records with different sources
        let pre_upload = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-1".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        let claude_hook = create_test_record(
            InterceptionSource::ClaudeCodeHook {
                session_id: Some("sess-2".to_string()),
            },
            UserAction::Ignored,
            None,
        );
        let external = create_test_record(
            InterceptionSource::ExternalHook {
                tool_name: "copilot".to_string(),
            },
            UserAction::Cancelled,
            None,
        );

        db.save_interception_record(&pre_upload).unwrap();
        db.save_interception_record(&claude_hook).unwrap();
        db.save_interception_record(&external).unwrap();

        // Filter by pre_upload
        let filtered = db
            .get_interception_records(1, 10, Some("pre_upload"))
            .unwrap();
        assert_eq!(filtered.total, 1);
        assert_eq!(filtered.records[0].source.source_type(), "pre_upload");

        // Filter by claude_code_hook
        let filtered = db
            .get_interception_records(1, 10, Some("claude_code_hook"))
            .unwrap();
        assert_eq!(filtered.total, 1);
    }

    #[test]
    fn test_delete_records() {
        let db = Database::new_in_memory().unwrap();

        let record1 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-1".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        let record2 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-2".to_string(),
            },
            UserAction::Ignored,
            None,
        );
        let record3 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-3".to_string(),
            },
            UserAction::Cancelled,
            None,
        );

        db.save_interception_record(&record1).unwrap();
        db.save_interception_record(&record2).unwrap();
        db.save_interception_record(&record3).unwrap();

        // Delete 2 records
        let deleted = db
            .delete_interception_records(&[record1.id.clone(), record3.id.clone()])
            .unwrap();
        assert_eq!(deleted, 2);

        // Verify only 1 remains
        let remaining = db.get_interception_records(1, 10, None).unwrap();
        assert_eq!(remaining.total, 1);
        assert_eq!(remaining.records[0].id, record2.id);
    }

    #[test]
    fn test_delete_empty_list() {
        let db = Database::new_in_memory().unwrap();
        let deleted = db.delete_interception_records(&[]).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_delete_batch_limit() {
        let db = Database::new_in_memory().unwrap();

        // Create a list of IDs that exceeds MAX_DELETE_BATCH
        let too_many_ids: Vec<String> = (0..1001).map(|i| format!("id-{}", i)).collect();

        // Should return error for oversized batch
        let result = db.delete_interception_records(&too_many_ids);
        assert!(result.is_err());

        if let Err(StorageError::InvalidInput(msg)) = result {
            assert!(msg.contains("1001"));
            assert!(msg.contains("1000"));
        } else {
            panic!("Expected InvalidInput error");
        }

        // Exactly at limit should work
        let exact_limit: Vec<String> = (0..1000).map(|i| format!("id-{}", i)).collect();
        let result = db.delete_interception_records(&exact_limit);
        assert!(result.is_ok()); // Will return 0 since no records exist with these IDs
    }

    #[test]
    fn test_get_stats() {
        let db = Database::new_in_memory().unwrap();

        // Insert records with different actions
        let record1 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-1".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        let record2 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-2".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        let record3 = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-3".to_string(),
            },
            UserAction::Ignored,
            None,
        );

        db.save_interception_record(&record1).unwrap();
        db.save_interception_record(&record2).unwrap();
        db.save_interception_record(&record3).unwrap();

        let stats = db.get_interception_stats().unwrap();

        assert_eq!(stats.total_interceptions, 3);
        assert_eq!(*stats.by_action.get("redacted").unwrap_or(&0), 2);
        assert_eq!(*stats.by_action.get("ignored").unwrap_or(&0), 1);
        assert!(stats.recent_7_days >= 3); // All records should be within 7 days
    }

    #[test]
    fn test_stats_by_type() {
        let db = Database::new_in_memory().unwrap();

        // Create record with ApiKey type (from the helper)
        let record = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-1".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        db.save_interception_record(&record).unwrap();

        let stats = db.get_interception_stats().unwrap();

        // Should have count for api_key type
        assert!(stats.by_type.get("api_key").is_some() || stats.by_type.get("API_KEY").is_some());
    }

    #[test]
    fn test_stats_by_severity() {
        let db = Database::new_in_memory().unwrap();

        // Create record with Critical severity (from the helper)
        let record = create_test_record(
            InterceptionSource::PreUpload {
                session_id: "sess-1".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        db.save_interception_record(&record).unwrap();

        let stats = db.get_interception_stats().unwrap();

        // Should have count for critical severity
        assert!(
            stats.by_severity.get("critical").is_some()
                || stats.by_severity.get("Critical").is_some()
        );
    }

    #[test]
    fn test_empty_stats() {
        let db = Database::new_in_memory().unwrap();

        let stats = db.get_interception_stats().unwrap();

        assert_eq!(stats.total_interceptions, 0);
        assert!(stats.by_type.is_empty());
        assert!(stats.by_severity.is_empty());
        assert!(stats.by_action.is_empty());
        assert_eq!(stats.recent_7_days, 0);
    }

    #[test]
    fn test_source_context_serialization() {
        let db = Database::new_in_memory().unwrap();

        // Test with external hook
        let record = create_test_record(
            InterceptionSource::ExternalHook {
                tool_name: "github-copilot".to_string(),
            },
            UserAction::Redacted,
            None,
        );
        db.save_interception_record(&record).unwrap();

        let paginated = db.get_interception_records(1, 10, None).unwrap();
        if let InterceptionSource::ExternalHook { tool_name } = &paginated.records[0].source {
            assert_eq!(tool_name, "github-copilot");
        } else {
            panic!("Expected ExternalHook source");
        }
    }
}
