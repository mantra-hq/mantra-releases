//! Search operations
//!
//! Provides search functionality for sessions.

use chrono::DateTime;

use super::{ContentType, SearchFilters, SearchResult, StorageError, TimePreset};
use crate::models::{ContentBlock, MantraSession};
use crate::storage::Database;

impl Database {
    /// Search sessions by content
    ///
    /// Searches through all session messages for the given query.
    /// Returns matching results with snippets and highlight positions.
    ///
    /// # Arguments
    /// * `query` - The search query (case-insensitive)
    /// * `limit` - Maximum number of results to return
    pub fn search_sessions(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, StorageError> {
        self.search_sessions_with_filters(query, limit, &SearchFilters::default())
    }

    /// Search sessions by content with filters
    ///
    /// Story 2.33: Enhanced search with filters support
    ///
    /// # Arguments
    /// * `query` - The search query (case-insensitive)
    /// * `limit` - Maximum number of results to return
    /// * `filters` - Search filters (content type, project, time range)
    pub fn search_sessions_with_filters(
        &self,
        query: &str,
        limit: usize,
        filters: &SearchFilters,
    ) -> Result<Vec<SearchResult>, StorageError> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        // Build SQL query with filters
        let mut sql = String::from(
            "SELECT s.id, s.project_id, s.raw_data, s.updated_at,
                    p.name as project_name,
                    json_extract(s.raw_data, '$.metadata.title') as session_title
             FROM sessions s
             JOIN projects p ON s.project_id = p.id
             WHERE s.raw_data LIKE ?1",
        );

        let mut param_index = 2;
        let mut params_vec: Vec<String> = vec![format!("%{}%", query)];

        // AC2: Project filter
        #[allow(unused_assignments)]
        if let Some(ref project_id) = filters.project_id {
            sql.push_str(&format!(" AND s.project_id = ?{}", param_index));
            params_vec.push(project_id.clone());
            param_index += 1; // Keep for future extensibility
        }

        // AC3: Time range filter
        if let Some(time_preset) = filters.time_preset {
            let time_filter = match time_preset {
                TimePreset::All => None,
                TimePreset::Today => Some("datetime('now', 'start of day')"),
                TimePreset::Week => Some("datetime('now', 'weekday 0', '-7 days')"),
                TimePreset::Month => Some("datetime('now', 'start of month')"),
            };
            if let Some(time_sql) = time_filter {
                sql.push_str(&format!(" AND s.updated_at >= {}", time_sql));
            }
        }

        sql.push_str(" ORDER BY s.updated_at DESC");

        eprintln!(
            "[search_sessions_with_filters] SQL: {}, params: {:?}",
            sql, params_vec
        );

        let mut stmt = self.connection().prepare(&sql)?;

        // Convert params to rusqlite format
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,  // session_id
                row.get::<_, String>(1)?,  // project_id
                row.get::<_, String>(2)?,  // raw_data
                row.get::<_, String>(3)?,  // updated_at
                row.get::<_, String>(4)?,  // project_name
                row.get::<_, Option<String>>(5)?, // session_title
            ))
        })?;

        let mut session_count = 0;
        for row_result in rows {
            session_count += 1;
            if results.len() >= limit {
                break;
            }

            let (session_id, project_id, raw_data, updated_at, project_name, session_title) =
                row_result?;

            // Parse session JSON
            let session: MantraSession = match serde_json::from_str(&raw_data) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "[search_sessions_with_filters] Failed to parse session {}: {}",
                        session_id, e
                    );
                    continue;
                }
            };

            // Format session name
            let session_name = session_title.unwrap_or_else(|| {
                let parts: Vec<&str> = session_id.split(['-', '_']).collect();
                if parts.len() > 1 {
                    parts.last().unwrap_or(&"").chars().take(8).collect()
                } else {
                    session_id.chars().take(8).collect()
                }
            });

            // Parse timestamp
            let timestamp = DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);

            // Search through messages
            for (msg_idx, message) in session.messages.iter().enumerate() {
                if results.len() >= limit {
                    break;
                }

                // Extract text content from content blocks with content type detection
                for block in &message.content_blocks {
                    // AC1: Content type filtering
                    let (text, detected_type) = match block {
                        ContentBlock::Text { text, .. } => {
                            // Check if text contains code blocks (markdown fences)
                            let has_code_block = text.contains("```");
                            if has_code_block {
                                // This is mixed content - could contain both code and text
                                (text.clone(), None) // None means "mixed"
                            } else {
                                (text.clone(), Some(ContentType::Conversation))
                            }
                        }
                        ContentBlock::Thinking { thinking, .. } => {
                            (thinking.clone(), Some(ContentType::Conversation))
                        }
                        ContentBlock::ToolResult { content, .. } => {
                            // Tool results might contain code
                            (content.clone(), Some(ContentType::Code))
                        }
                        ContentBlock::CodeSuggestion { code, .. } => {
                            (code.clone(), Some(ContentType::Code))
                        }
                        _ => continue,
                    };

                    // Apply content type filter
                    let passes_filter = match filters.content_type {
                        ContentType::All => true,
                        ContentType::Code => {
                            // For Code filter: must be code content or contain code blocks
                            detected_type == Some(ContentType::Code) || text.contains("```")
                        }
                        ContentType::Conversation => {
                            // For Conversation filter: text without code blocks
                            detected_type == Some(ContentType::Conversation)
                                || (detected_type.is_none() && !text.contains("```"))
                        }
                    };

                    if !passes_filter {
                        continue;
                    }

                    let text_lower = text.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        // Calculate snippet with context (use char indices for UTF-8 safety)
                        let chars: Vec<char> = text.chars().collect();
                        let char_count = chars.len();

                        // Find char index for start_pos (byte position -> char position)
                        let char_start_pos = text[..start_pos].chars().count();
                        let query_char_len = query.chars().count();

                        let snippet_char_start = char_start_pos.saturating_sub(30);
                        let snippet_char_end =
                            (char_start_pos + query_char_len + 70).min(char_count);

                        let snippet: String =
                            chars[snippet_char_start..snippet_char_end].iter().collect();

                        // Adjust match position for snippet
                        let match_start_in_snippet = char_start_pos - snippet_char_start;
                        let match_end_in_snippet = match_start_in_snippet + query_char_len;

                        // Determine final content type for result
                        let result_content_type = if text.contains("```") {
                            Some(ContentType::Code)
                        } else {
                            detected_type
                        };

                        results.push(SearchResult {
                            id: format!("{}-{}", session_id, msg_idx),
                            session_id: session_id.clone(),
                            project_id: project_id.clone(),
                            project_name: project_name.clone(),
                            session_name: session_name.clone(),
                            message_id: msg_idx.to_string(),
                            content: snippet,
                            match_positions: vec![(match_start_in_snippet, match_end_in_snippet)],
                            timestamp,
                            content_type: result_content_type,
                        });

                        // Only one result per message
                        break;
                    }
                }
            }
        }

        eprintln!(
            "[search_sessions_with_filters] Processed {} sessions, found {} results",
            session_count,
            results.len()
        );

        Ok(results)
    }
}
