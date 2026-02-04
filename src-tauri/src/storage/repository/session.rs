//! Session CRUD and import operations
//!
//! Provides session management methods for the Database.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::{parse_session_source, StorageError};
use crate::models::{
    extract_project_name, normalize_cwd, ImportResult, MantraSession, SessionSummary,
};
use crate::storage::Database;

impl Database {
    /// Insert a session into the database
    ///
    /// # Arguments
    /// * `session` - The MantraSession to insert
    /// * `project_id` - The project ID this session belongs to
    pub fn insert_session(
        &self,
        session: &MantraSession,
        project_id: &str,
    ) -> Result<(), StorageError> {
        let raw_data = serde_json::to_string(session)?;
        // Story 2.29: Calculate is_empty based on session content
        let is_empty = if session.is_empty() { 1 } else { 0 };
        // Story 1.12: Store original_cwd at import time
        let original_cwd = normalize_cwd(&session.cwd);

        // Story 1.12: Build source_context from session metadata
        let source_context = Self::build_source_context(session);

        self.connection().execute(
            "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, is_empty, original_cwd, source_context, raw_data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session.id,
                project_id,
                session.source.to_string(),
                session.cwd,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.messages.len() as i32,
                is_empty,
                original_cwd,
                source_context,
                raw_data
            ],
        )?;
        Ok(())
    }

    /// Build source_context JSON from session metadata (Story 1.12)
    ///
    /// Extracts source-specific context information from the original file path:
    /// - Claude: project_path_encoded from file path
    /// - Gemini: project_hash and session_filename
    /// - Cursor: workspace_id from path
    /// - Codex: file_path
    fn build_source_context(session: &MantraSession) -> String {
        use crate::models::sources;
        use serde_json::json;

        let file_path = session.metadata.original_path.as_deref().unwrap_or("");

        if file_path.is_empty() {
            return "{}".to_string();
        }

        let context = match session.source.as_str() {
            sources::CLAUDE => {
                // Extract project_path_encoded from path like ~/.claude/projects/-mnt-disk0-project-foo/abc.jsonl
                let project_path_encoded = std::path::Path::new(file_path)
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                json!({
                    "file_path": file_path,
                    "project_path_encoded": project_path_encoded
                })
            }
            sources::GEMINI => {
                // Extract from path like ~/.gemini/history/abc123/session-xxx.json
                let session_filename = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                let project_hash = std::path::Path::new(file_path)
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                json!({
                    "file_path": file_path,
                    "project_hash": project_hash,
                    "session_filename": session_filename
                })
            }
            sources::CURSOR => {
                // Extract workspace_id from path like ~/.config/Cursor/User/workspaceStorage/a1b2c3d4/...
                let workspace_id = file_path
                    .split("workspaceStorage/")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .unwrap_or("");
                json!({
                    "workspace_id": workspace_id,
                    "workspace_path": file_path
                })
            }
            sources::CODEX => {
                json!({
                    "file_path": file_path
                })
            }
            _ => {
                json!({
                    "file_path": file_path
                })
            }
        };

        context.to_string()
    }

    /// Get all sessions for a project
    ///
    /// # Arguments
    /// * `project_id` - The project ID to get sessions for
    pub fn get_project_sessions(
        &self,
        project_id: &str,
    ) -> Result<Vec<SessionSummary>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id, source, created_at, updated_at, message_count, is_empty,
                    json_extract(raw_data, '$.metadata.title') as title, original_cwd
             FROM sessions
             WHERE project_id = ?1
             ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                let source_str: String = row.get(1)?;
                let created_at_str: String = row.get(2)?;
                let updated_at_str: String = row.get(3)?;
                let is_empty_int: i32 = row.get(5)?;
                let title: Option<String> = row.get(6)?;
                let original_cwd: Option<String> = row.get(7)?;

                Ok(SessionSummary {
                    id: row.get(0)?,
                    source: parse_session_source(&source_str),
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    message_count: row.get::<_, i32>(4)? as u32,
                    is_empty: is_empty_int != 0,
                    title,
                    original_cwd,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Check if a session with the given ID exists
    ///
    /// # Arguments
    /// * `session_id` - The session ID to check
    pub fn session_exists(&self, session_id: &str) -> Result<bool, StorageError> {
        let count: i32 = self.connection().query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get a session by ID
    ///
    /// # Arguments
    /// * `session_id` - The session ID to retrieve
    ///
    /// # Returns
    /// The full MantraSession if found, None otherwise
    pub fn get_session(&self, session_id: &str) -> Result<Option<MantraSession>, StorageError> {
        let mut stmt = self
            .connection()
            .prepare("SELECT raw_data FROM sessions WHERE id = ?1")?;

        let result = stmt.query_row(params![session_id], |row| {
            let raw_data: String = row.get(0)?;
            Ok(raw_data)
        });

        match result {
            Ok(raw_data) => {
                let session: MantraSession = serde_json::from_str(&raw_data)?;
                Ok(Some(session))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get session message count for a specific session
    ///
    /// # Arguments
    /// * `session_id` - The session ID
    pub fn get_session_message_count(&self, session_id: &str) -> Result<u32, StorageError> {
        let count: i32 = self.connection().query_row(
            "SELECT message_count FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Update session message count and raw data
    ///
    /// # Arguments
    /// * `session` - The updated session
    pub fn update_session(&self, session: &MantraSession) -> Result<(), StorageError> {
        let raw_data = serde_json::to_string(session)?;
        let is_empty = if session.is_empty() { 1 } else { 0 };
        self.connection().execute(
            "UPDATE sessions SET message_count = ?1, updated_at = ?2, is_empty = ?3, raw_data = ?4 WHERE id = ?5",
            params![
                session.messages.len() as i32,
                session.updated_at.to_rfc3339(),
                is_empty,
                raw_data,
                session.id
            ],
        )?;
        Ok(())
    }

    /// Get all full sessions for a project (Story 2.34: Analytics)
    ///
    /// Returns full MantraSession objects for analytics calculations.
    ///
    /// # Arguments
    /// * `project_id` - The project ID to get sessions for
    pub fn get_sessions_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<MantraSession>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT raw_data FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                let raw_data: String = row.get(0)?;
                Ok(raw_data)
            })?
            .filter_map(|result| match result {
                Ok(raw_data) => match serde_json::from_str::<MantraSession>(&raw_data) {
                    Ok(session) => Some(Ok(session)),
                    Err(e) => {
                        eprintln!("[get_sessions_by_project] Failed to parse session: {}", e);
                        None
                    }
                },
                Err(e) => Some(Err(StorageError::from(e))),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get all imported session IDs
    ///
    /// Returns a list of session IDs for all imported sessions.
    /// Used by ImportWizard to identify already-imported files.
    pub fn get_imported_session_ids(&self) -> Result<Vec<String>, StorageError> {
        let mut stmt = self.connection().prepare("SELECT id FROM sessions")?;

        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }

    /// Import a single session, handling project creation and deduplication
    ///
    /// # Arguments
    /// * `session` - The session to import
    ///
    /// # Returns
    /// A tuple of (was_imported, was_new_project)
    pub fn import_session(&self, session: &MantraSession) -> Result<(bool, bool), StorageError> {
        // Check for duplicate
        if self.session_exists(&session.id)? {
            return Ok((false, false));
        }

        // Get or create project
        let (project, is_new_project) = self.get_or_create_project(&session.cwd)?;

        // Insert session
        self.insert_session(session, &project.id)?;

        // Update project last activity if this session is newer
        if session.updated_at > project.last_activity {
            self.update_project_last_activity(&project.id, session.updated_at)?;
        }

        Ok((true, is_new_project))
    }

    /// Import a single session with Git remote URL support (Story 1.9)
    ///
    /// Uses the enhanced four-stage project matching logic.
    ///
    /// # Arguments
    /// * `session` - The session to import
    /// * `git_remote_url` - Optional Git remote URL for enhanced project matching
    ///
    /// # Returns
    /// A tuple of (was_imported, was_new_project, project_id)
    pub fn import_session_with_git_url(
        &self,
        session: &MantraSession,
        git_remote_url: Option<&str>,
    ) -> Result<(bool, bool, String), StorageError> {
        // Check for duplicate
        if self.session_exists(&session.id)? {
            return Ok((false, false, String::new()));
        }

        // Use enhanced find_or_create_project with Git URL support
        let (project, is_new_project) = self.find_or_create_project(&session.cwd, git_remote_url)?;

        // Insert session
        self.insert_session(session, &project.id)?;

        // Update project last activity if this session is newer
        if session.updated_at > project.last_activity {
            self.update_project_last_activity(&project.id, session.updated_at)?;
        }

        Ok((true, is_new_project, project.id))
    }

    /// Import multiple sessions with transaction support
    ///
    /// All successful imports are committed together. Individual session
    /// errors are collected and reported, but don't prevent other sessions
    /// from being imported.
    ///
    /// # Arguments
    /// * `sessions` - The sessions to import
    pub fn import_sessions(
        &mut self,
        sessions: &[MantraSession],
    ) -> Result<ImportResult, StorageError> {
        let mut result = ImportResult::default();

        // Use a transaction for atomicity
        let tx = self.connection_mut().transaction()?;

        for session in sessions {
            // Story 2.25: Normalize cwd for consistent aggregation
            let normalized_cwd = normalize_cwd(&session.cwd);

            // Check for duplicate
            let exists: i32 = tx.query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = ?1",
                params![session.id],
                |row: &rusqlite::Row| row.get(0),
            )?;

            if exists > 0 {
                result.skipped_count += 1;
                continue;
            }

            // Get or create project within transaction
            let project_result: Result<(String, bool), StorageError> = {
                let mut stmt = tx.prepare("SELECT id FROM projects WHERE cwd = ?1")?;

                match stmt.query_row(params![normalized_cwd], |row: &rusqlite::Row| {
                    row.get::<_, String>(0)
                }) {
                    Ok(project_id) => Ok((project_id, false)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        let id = Uuid::new_v4().to_string();
                        let name = extract_project_name(&normalized_cwd);
                        let now = Utc::now().to_rfc3339();

                        tx.execute(
                            "INSERT INTO projects (id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![id, name, normalized_cwd, now, now, Option::<String>::None, 0, Option::<String>::None, 1],
                        )?;

                        // Story 1.12: Also add to project_paths table
                        let path_id = Uuid::new_v4().to_string();
                        tx.execute(
                            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
                            params![path_id, id, normalized_cwd, now],
                        )?;

                        Ok((id, true))
                    }
                    Err(e) => Err(StorageError::from(e)),
                }
            };

            match project_result {
                Ok((project_id, is_new_project)) => {
                    // Insert session
                    let raw_data = match serde_json::to_string(session) {
                        Ok(data) => data,
                        Err(e) => {
                            result.errors.push(format!("Session {}: {}", session.id, e));
                            continue;
                        }
                    };

                    // Story 1.12: Build source_context from session metadata
                    let source_context = Self::build_source_context(session);

                    match tx.execute(
                        "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, is_empty, original_cwd, source_context, raw_data)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            session.id,
                            project_id,
                            session.source.to_string(),
                            normalized_cwd,
                            session.created_at.to_rfc3339(),
                            session.updated_at.to_rfc3339(),
                            session.messages.len() as i32,
                            if session.is_empty() { 1 } else { 0 },
                            normalized_cwd, // Story 1.12: original_cwd = cwd at import time
                            source_context, // Story 1.12: source_context from metadata
                            raw_data
                        ],
                    ) {
                        Ok(_) => {
                            result.imported_count += 1;
                            if is_new_project {
                                result.new_projects_count += 1;
                            }

                            // Update project last_activity if this session is newer
                            let _ = tx.execute(
                                "UPDATE projects SET last_activity = ?1 WHERE id = ?2 AND last_activity < ?1",
                                params![session.updated_at.to_rfc3339(), project_id],
                            );
                        }
                        Err(e) => {
                            result.errors.push(format!("Session {}: {}", session.id, e));
                        }
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Session {}: {}", session.id, e));
                }
            }
        }

        // Commit the transaction
        tx.commit()?;

        Ok(result)
    }
}
