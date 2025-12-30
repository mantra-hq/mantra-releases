//! Repository layer for database CRUD operations
//!
//! Provides high-level database operations for projects and sessions.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::database::Database;
use super::error::StorageError;
use crate::models::{
    extract_project_name, ImportResult, MantraSession, Project, SessionSource, SessionSummary,
};

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
        self.connection().execute(
            "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, raw_data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.id,
                project_id,
                session.source.to_string(),
                session.cwd,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.messages.len() as i32,
                raw_data
            ],
        )?;
        Ok(())
    }

    /// Get or create a project by cwd
    ///
    /// If a project with the given cwd exists, returns it.
    /// Otherwise, creates a new project and returns it.
    ///
    /// # Arguments
    /// * `cwd` - The working directory path
    ///
    /// # Returns
    /// A tuple of (Project, bool) where bool indicates if the project was newly created
    pub fn get_or_create_project(&self, cwd: &str) -> Result<(Project, bool), StorageError> {
        // Try to find existing project
        let mut stmt = self
            .connection()
            .prepare("SELECT id, name, cwd, created_at, last_activity FROM projects WHERE cwd = ?1")?;

        let project_result = stmt.query_row(params![cwd], |row| {
            let created_at_str: String = row.get(3)?;
            let last_activity_str: String = row.get(4)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                cwd: row.get(2)?,
                session_count: 0, // Will be filled later
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        });

        match project_result {
            Ok(mut project) => {
                // Get session count
                let count: i32 = self.connection().query_row(
                    "SELECT COUNT(*) FROM sessions WHERE project_id = ?1",
                    params![project.id],
                    |row| row.get(0),
                )?;
                project.session_count = count as u32;
                Ok((project, false))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new project
                let id = Uuid::new_v4().to_string();
                let name = extract_project_name(cwd);
                let now = Utc::now();
                let now_str = now.to_rfc3339();

                self.connection().execute(
                    "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, name, cwd, now_str, now_str],
                )?;

                let project = Project {
                    id,
                    name,
                    cwd: cwd.to_string(),
                    session_count: 0,
                    created_at: now,
                    last_activity: now,
                };
                Ok((project, true))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// List all projects ordered by last activity (descending)
    pub fn list_projects(&self) -> Result<Vec<Project>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT p.id, p.name, p.cwd, p.created_at, p.last_activity,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count
             FROM projects p
             ORDER BY p.last_activity DESC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cwd: row.get(2)?,
                    session_count: row.get::<_, i32>(5)? as u32,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Get all sessions for a project
    ///
    /// # Arguments
    /// * `project_id` - The project ID to get sessions for
    pub fn get_project_sessions(&self, project_id: &str) -> Result<Vec<SessionSummary>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id, source, created_at, updated_at, message_count
             FROM sessions
             WHERE project_id = ?1
             ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                let source_str: String = row.get(1)?;
                let created_at_str: String = row.get(2)?;
                let updated_at_str: String = row.get(3)?;

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

    /// Update project's last activity time
    ///
    /// # Arguments
    /// * `project_id` - The project ID to update
    /// * `last_activity` - The new last activity time
    pub fn update_project_last_activity(
        &self,
        project_id: &str,
        last_activity: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        self.connection().execute(
            "UPDATE projects SET last_activity = ?1 WHERE id = ?2",
            params![last_activity.to_rfc3339(), project_id],
        )?;
        Ok(())
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

    /// Import multiple sessions with transaction support
    ///
    /// All successful imports are committed together. Individual session
    /// errors are collected and reported, but don't prevent other sessions
    /// from being imported.
    ///
    /// # Arguments
    /// * `sessions` - The sessions to import
    pub fn import_sessions(&mut self, sessions: &[MantraSession]) -> Result<ImportResult, StorageError> {
        let mut result = ImportResult::default();

        // Use a transaction for atomicity
        let tx = self.connection_mut().transaction()?;

        for session in sessions {
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
                let mut stmt = tx.prepare(
                    "SELECT id FROM projects WHERE cwd = ?1"
                )?;

                match stmt.query_row(params![session.cwd], |row: &rusqlite::Row| row.get::<_, String>(0)) {
                    Ok(project_id) => Ok((project_id, false)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        let id = Uuid::new_v4().to_string();
                        let name = extract_project_name(&session.cwd);
                        let now = Utc::now().to_rfc3339();

                        tx.execute(
                            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![id, name, session.cwd, now, now],
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

                    match tx.execute(
                        "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, raw_data)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![
                            session.id,
                            project_id,
                            session.source.to_string(),
                            session.cwd,
                            session.created_at.to_rfc3339(),
                            session.updated_at.to_rfc3339(),
                            session.messages.len() as i32,
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

/// Parse session source from string
fn parse_session_source(s: &str) -> SessionSource {
    match s.to_lowercase().as_str() {
        "claude" => SessionSource::Claude,
        "gemini" => SessionSource::Gemini,
        "cursor" => SessionSource::Cursor,
        "unknown" => SessionSource::Unknown,
        _ => SessionSource::Unknown, // Explicit unknown for unrecognized sources
    }
}

/// Implement Display for SessionSource for database storage
impl std::fmt::Display for SessionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionSource::Claude => write!(f, "claude"),
            SessionSource::Gemini => write!(f, "gemini"),
            SessionSource::Cursor => write!(f, "cursor"),
            SessionSource::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MantraSession;

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), SessionSource::Claude, cwd.to_string())
    }

    #[test]
    fn test_get_or_create_project_creates_new() {
        let db = Database::new_in_memory().unwrap();
        let (project, is_new) = db.get_or_create_project("/home/user/test").unwrap();

        assert!(is_new);
        assert_eq!(project.name, "test");
        assert_eq!(project.cwd, "/home/user/test");
        assert_eq!(project.session_count, 0);
    }

    #[test]
    fn test_get_or_create_project_returns_existing() {
        let db = Database::new_in_memory().unwrap();

        // Create first
        let (project1, is_new1) = db.get_or_create_project("/home/user/test").unwrap();
        assert!(is_new1);

        // Get existing
        let (project2, is_new2) = db.get_or_create_project("/home/user/test").unwrap();
        assert!(!is_new2);
        assert_eq!(project1.id, project2.id);
    }

    #[test]
    fn test_insert_and_list_sessions() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

        let session = create_test_session("sess_1", "/home/user/test");
        db.insert_session(&session, &project.id).unwrap();

        let sessions = db.get_project_sessions(&project.id).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "sess_1");
    }

    #[test]
    fn test_session_exists() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

        assert!(!db.session_exists("sess_1").unwrap());

        let session = create_test_session("sess_1", "/home/user/test");
        db.insert_session(&session, &project.id).unwrap();

        assert!(db.session_exists("sess_1").unwrap());
    }

    #[test]
    fn test_list_projects_ordered_by_activity() {
        let db = Database::new_in_memory().unwrap();

        // Create projects
        db.get_or_create_project("/home/user/project1").unwrap();
        db.get_or_create_project("/home/user/project2").unwrap();

        // Update project1 to be more recent
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let future_time = Utc::now() + chrono::Duration::hours(1);
        db.update_project_last_activity(&project1.id, future_time)
            .unwrap();

        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "project1"); // Most recent first
    }

    #[test]
    fn test_import_session_deduplication() {
        let db = Database::new_in_memory().unwrap();

        let session = create_test_session("sess_1", "/home/user/test");

        // First import
        let (imported1, new_project1) = db.import_session(&session).unwrap();
        assert!(imported1);
        assert!(new_project1);

        // Second import (should be skipped)
        let (imported2, new_project2) = db.import_session(&session).unwrap();
        assert!(!imported2);
        assert!(!new_project2);
    }

    #[test]
    fn test_import_sessions_batch() {
        let mut db = Database::new_in_memory().unwrap();

        let sessions = vec![
            create_test_session("sess_1", "/home/user/project1"),
            create_test_session("sess_2", "/home/user/project1"),
            create_test_session("sess_3", "/home/user/project2"),
            create_test_session("sess_1", "/home/user/project1"), // Duplicate
        ];

        let result = db.import_sessions(&sessions).unwrap();
        assert_eq!(result.imported_count, 3);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.new_projects_count, 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_project_session_count() {
        let mut db = Database::new_in_memory().unwrap();

        let sessions = vec![
            create_test_session("sess_1", "/home/user/test"),
            create_test_session("sess_2", "/home/user/test"),
        ];

        db.import_sessions(&sessions).unwrap();

        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].session_count, 2);
    }
}
