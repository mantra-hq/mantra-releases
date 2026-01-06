//! Repository layer for database CRUD operations
//!
//! Provides high-level database operations for projects and sessions.

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::Serialize;
use uuid::Uuid;

use super::database::Database;
use super::error::StorageError;
use crate::models::{
    extract_project_name, normalize_cwd, ContentBlock, ImportResult, MantraSession, Project,
    SessionSource, SessionSummary,
};

/// Search result item
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Unique ID (session_id-message_index)
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// Project ID
    pub project_id: String,
    /// Project name
    pub project_name: String,
    /// Session name (title or formatted ID)
    pub session_name: String,
    /// Message ID (index as string)
    pub message_id: String,
    /// Matched content snippet
    pub content: String,
    /// Match positions [start, end]
    pub match_positions: Vec<(usize, usize)>,
    /// Timestamp
    pub timestamp: i64,
}

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
        self.connection().execute(
            "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, is_empty, raw_data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session.id,
                project_id,
                session.source.to_string(),
                session.cwd,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.messages.len() as i32,
                is_empty,
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
    /// * `cwd` - The working directory path (will be normalized)
    ///
    /// # Returns
    /// A tuple of (Project, bool) where bool indicates if the project was newly created
    pub fn get_or_create_project(&self, cwd: &str) -> Result<(Project, bool), StorageError> {
        // Story 2.25: Normalize cwd for consistent aggregation
        let normalized_cwd = normalize_cwd(cwd);

        // Try to find existing project
        let mut stmt = self
            .connection()
            .prepare("SELECT id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty FROM projects WHERE cwd = ?1")?;

        let project_result = stmt.query_row(params![normalized_cwd], |row| {
            let created_at_str: String = row.get(3)?;
            let last_activity_str: String = row.get(4)?;
            let git_repo_path: Option<String> = row.get(5)?;
            let has_git_repo: i32 = row.get(6)?;
            let git_remote_url: Option<String> = row.get(7)?;
            let is_empty: i32 = row.get(8)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                cwd: row.get(2)?,
                session_count: 0, // Will be filled later
                non_empty_session_count: 0, // Will be filled later
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                git_repo_path,
                has_git_repo: has_git_repo != 0,
                git_remote_url,
                is_empty: is_empty != 0,
            })
        });

        match project_result {
            Ok(mut project) => {
                // Get session count and non-empty session count (Story 2.29 V2)
                let (count, non_empty_count): (i32, i32) = self.connection().query_row(
                    "SELECT COUNT(*), SUM(CASE WHEN is_empty = 0 THEN 1 ELSE 0 END) FROM sessions WHERE project_id = ?1",
                    params![project.id],
                    |row| Ok((row.get(0)?, row.get::<_, Option<i32>>(1)?.unwrap_or(0))),
                )?;
                project.session_count = count as u32;
                project.non_empty_session_count = non_empty_count as u32;
                Ok((project, false))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new project with normalized cwd
                let id = Uuid::new_v4().to_string();
                let name = extract_project_name(&normalized_cwd);
                let now = Utc::now();
                let now_str = now.to_rfc3339();

                self.connection().execute(
                    "INSERT INTO projects (id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![id, name, normalized_cwd, now_str, now_str, Option::<String>::None, 0, Option::<String>::None, 1],
                )?;

                let project = Project {
                    id,
                    name,
                    cwd: normalized_cwd,
                    session_count: 0,
                    non_empty_session_count: 0,
                    created_at: now,
                    last_activity: now,
                    git_repo_path: None,
                    has_git_repo: false,
                    git_remote_url: None,
                    is_empty: true,
                };
                Ok((project, true))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// List all projects ordered by last activity (descending)
    pub fn list_projects(&self) -> Result<Vec<Project>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT p.id, p.name, p.cwd, p.created_at, p.last_activity, p.git_repo_path, p.has_git_repo, p.git_remote_url, p.is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             ORDER BY p.last_activity DESC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;
                let git_repo_path: Option<String> = row.get(5)?;
                let has_git_repo: i32 = row.get(6)?;
                let git_remote_url: Option<String> = row.get(7)?;
                let is_empty: i32 = row.get(8)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cwd: row.get(2)?,
                    session_count: row.get::<_, i32>(9)? as u32,
                    non_empty_session_count: row.get::<_, i32>(10)? as u32,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    git_repo_path,
                    has_git_repo: has_git_repo != 0,
                    git_remote_url,
                    is_empty: is_empty != 0,
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
            "SELECT id, source, created_at, updated_at, message_count, is_empty,
                    json_extract(raw_data, '$.metadata.title') as title
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

    /// Get the project that a session belongs to (Story 1.9)
    ///
    /// This method finds the project by looking up the session's project_id,
    /// which is more reliable than using cwd (cwd might have been updated).
    ///
    /// # Arguments
    /// * `session_id` - The session ID to look up
    ///
    /// # Returns
    /// The project if found, None if session doesn't exist or has no project
    pub fn get_project_by_session_id(&self, session_id: &str) -> Result<Option<Project>, StorageError> {
        let result = self.connection().query_row(
            "SELECT p.id, p.name, p.cwd, p.created_at, p.last_activity, p.git_repo_path, p.has_git_repo, p.git_remote_url, p.is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             INNER JOIN sessions s ON s.project_id = p.id
             WHERE s.id = ?1",
            params![session_id],
            |row| {
                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;
                let git_repo_path: Option<String> = row.get(5)?;
                let has_git_repo: i32 = row.get(6)?;
                let git_remote_url: Option<String> = row.get(7)?;
                let is_empty: i32 = row.get(8)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cwd: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    git_repo_path,
                    has_git_repo: has_git_repo != 0,
                    git_remote_url,
                    session_count: row.get::<_, i32>(9)? as u32,
                    non_empty_session_count: row.get::<_, i32>(10)? as u32,
                    is_empty: is_empty != 0,
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get a session by ID
    ///
    /// # Arguments
    /// * `session_id` - The session ID to retrieve
    ///
    /// # Returns
    /// The full MantraSession if found, None otherwise
    pub fn get_session(&self, session_id: &str) -> Result<Option<MantraSession>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT raw_data FROM sessions WHERE id = ?1",
        )?;

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

    /// Update project's Git repository information
    ///
    /// # Arguments
    /// * `cwd` - The project's working directory (will be normalized)
    /// * `git_repo_path` - The Git repository root path (None if no Git repo)
    pub fn update_project_git_info(
        &self,
        cwd: &str,
        git_repo_path: Option<String>,
    ) -> Result<(), StorageError> {
        // Story 2.25: Normalize cwd for consistent lookup
        let normalized_cwd = normalize_cwd(cwd);
        let has_git_repo = if git_repo_path.is_some() { 1 } else { 0 };
        self.connection().execute(
            "UPDATE projects SET git_repo_path = ?1, has_git_repo = ?2 WHERE cwd = ?3",
            params![git_repo_path, has_git_repo, normalized_cwd],
        )?;
        Ok(())
    }

    // ===== Story 1.9: Enhanced Project Identification =====

    /// Find a project by Git remote URL (Story 1.9)
    ///
    /// # Arguments
    /// * `url` - The Git remote URL (will be normalized)
    pub fn find_by_git_remote(&self, url: &str) -> Result<Option<Project>, StorageError> {
        use crate::git::normalize_git_url;
        let normalized_url = normalize_git_url(url);

        let mut stmt = self.connection().prepare(
            "SELECT id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             WHERE git_remote_url = ?1",
        )?;

        let result = stmt.query_row(params![normalized_url], |row| {
            let created_at_str: String = row.get(3)?;
            let last_activity_str: String = row.get(4)?;
            let git_repo_path: Option<String> = row.get(5)?;
            let has_git_repo: i32 = row.get(6)?;
            let git_remote_url: Option<String> = row.get(7)?;
            let is_empty: i32 = row.get(8)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                cwd: row.get(2)?,
                session_count: row.get::<_, i32>(9)? as u32,
                non_empty_session_count: row.get::<_, i32>(10)? as u32,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                git_repo_path,
                has_git_repo: has_git_repo != 0,
                git_remote_url,
                is_empty: is_empty != 0,
            })
        });

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update a project's Git remote URL (Story 1.9)
    ///
    /// # Arguments
    /// * `project_id` - The project ID
    /// * `git_remote_url` - The new Git remote URL (will be normalized if Some)
    pub fn update_project_git_remote(
        &self,
        project_id: &str,
        git_remote_url: Option<&str>,
    ) -> Result<(), StorageError> {
        use crate::git::normalize_git_url;
        let normalized_url = git_remote_url.map(normalize_git_url);

        self.connection().execute(
            "UPDATE projects SET git_remote_url = ?1 WHERE id = ?2",
            params![normalized_url, project_id],
        )?;
        Ok(())
    }

    /// Update a project's working directory (Story 1.9)
    ///
    /// # Arguments
    /// * `project_id` - The project ID
    /// * `new_cwd` - The new working directory (should be normalized before calling)
    pub fn update_project_cwd(&self, project_id: &str, new_cwd: &str) -> Result<(), StorageError> {
        // Extract new name from cwd
        let new_name = extract_project_name(new_cwd);

        self.connection().execute(
            "UPDATE projects SET cwd = ?1, name = ?2 WHERE id = ?3",
            params![new_cwd, new_name, project_id],
        )?;
        Ok(())
    }

    /// Find or create a project with Git remote URL support (Story 1.9)
    ///
    /// Four-stage matching logic:
    /// 1. Git remote URL match → return existing project
    /// 2. Path match + URL consistency check:
    ///    - Project has no URL, session has URL → update project's URL
    ///    - Both have URL and match → return existing project
    ///    - Both have URL but conflict → create new project (path reuse)
    ///    - Project has URL, session has no URL → return existing project
    ///    - Both have no URL → return existing project
    /// 3. No match → create new project
    ///
    /// # Arguments
    /// * `cwd` - The working directory path (will be normalized)
    /// * `git_remote_url` - Optional Git remote URL
    ///
    /// # Returns
    /// A tuple of (Project, bool) where bool indicates if the project was newly created
    pub fn find_or_create_project(
        &self,
        cwd: &str,
        git_remote_url: Option<&str>,
    ) -> Result<(Project, bool), StorageError> {
        use crate::git::normalize_git_url;

        let normalized_cwd = normalize_cwd(cwd);
        let normalized_url = git_remote_url.map(normalize_git_url);

        // Stage 1: Git remote URL match (highest priority)
        if let Some(ref url) = normalized_url {
            if let Some(project) = self.find_by_git_remote(url)? {
                // Same repo (possibly different path) → aggregate to existing project
                return Ok((project, false));
            }
        }

        // Stage 2: Path match + consistency check
        if let Some(existing_project) = self.get_project_by_cwd(&normalized_cwd)? {
            match (&existing_project.git_remote_url, &normalized_url) {
                // 2a: Project has no URL, session has URL → update project's URL
                (None, Some(url)) => {
                    self.update_project_git_remote(&existing_project.id, Some(url))?;
                    let mut updated = existing_project;
                    updated.git_remote_url = Some(url.clone());
                    return Ok((updated, false));
                }

                // 2b: Both have URL and match → aggregate
                (Some(old_url), Some(new_url)) if old_url == new_url => {
                    return Ok((existing_project, false));
                }

                // 2c: Both have URL but conflict → path reuse! Update to new URL
                (Some(old_url), Some(new_url)) => {
                    // Same path but different repo - directory was reused for a new project
                    // AC8: Log warning about path reuse detection
                    eprintln!(
                        "[Story 1.9 AC8] Path reuse detected: cwd='{}' had git_remote_url='{}', updating to '{}'",
                        normalized_cwd, old_url, new_url
                    );
                    // Update the project's Git URL to the new one (current reality)
                    // Note: This means sessions from old repo will be in the same project,
                    // but this is acceptable as the path is the primary aggregation key
                    self.update_project_git_remote(&existing_project.id, Some(new_url))?;
                    let mut updated = existing_project;
                    updated.git_remote_url = Some(new_url.clone());
                    return Ok((updated, false));
                }

                // 2d: Project has URL, session has no URL → aggregate
                (Some(_), None) => {
                    return Ok((existing_project, false));
                }

                // 2e: Both have no URL → aggregate
                (None, None) => {
                    return Ok((existing_project, false));
                }
            }
        }

        // Stage 3: No match → create new project
        let project = self.create_project_internal(&normalized_cwd, normalized_url.as_deref())?;
        Ok((project, true))
    }

    /// Internal helper to create a new project (Story 1.9)
    fn create_project_internal(
        &self,
        normalized_cwd: &str,
        git_remote_url: Option<&str>,
    ) -> Result<Project, StorageError> {
        let id = Uuid::new_v4().to_string();
        let name = extract_project_name(normalized_cwd);
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        self.connection().execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, name, normalized_cwd, now_str, now_str, Option::<String>::None, 0, git_remote_url, 1],
        )?;

        Ok(Project {
            id,
            name,
            cwd: normalized_cwd.to_string(),
            session_count: 0,
            non_empty_session_count: 0,
            created_at: now,
            last_activity: now,
            git_repo_path: None,
            has_git_repo: false,
            git_remote_url: git_remote_url.map(String::from),
            is_empty: true,
        })
    }

    /// Get a project by ID
    ///
    /// # Arguments
    /// * `project_id` - The project ID to retrieve
    pub fn get_project(&self, project_id: &str) -> Result<Option<Project>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![project_id], |row| {
            let created_at_str: String = row.get(3)?;
            let last_activity_str: String = row.get(4)?;
            let git_repo_path: Option<String> = row.get(5)?;
            let has_git_repo: i32 = row.get(6)?;
            let git_remote_url: Option<String> = row.get(7)?;
            let is_empty: i32 = row.get(8)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                cwd: row.get(2)?,
                session_count: row.get::<_, i32>(9)? as u32,
                non_empty_session_count: row.get::<_, i32>(10)? as u32,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                git_repo_path,
                has_git_repo: has_git_repo != 0,
                git_remote_url,
                is_empty: is_empty != 0,
            })
        });

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get a project by cwd
    ///
    /// # Arguments
    /// * `cwd` - The project's working directory (will be normalized)
    pub fn get_project_by_cwd(&self, cwd: &str) -> Result<Option<Project>, StorageError> {
        // Story 2.25: Normalize cwd for consistent lookup
        let normalized_cwd = normalize_cwd(cwd);

        let mut stmt = self.connection().prepare(
            "SELECT id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             WHERE cwd = ?1",
        )?;

        let result = stmt.query_row(params![normalized_cwd], |row| {
            let created_at_str: String = row.get(3)?;
            let last_activity_str: String = row.get(4)?;
            let git_repo_path: Option<String> = row.get(5)?;
            let has_git_repo: i32 = row.get(6)?;
            let git_remote_url: Option<String> = row.get(7)?;
            let is_empty: i32 = row.get(8)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                cwd: row.get(2)?,
                session_count: row.get::<_, i32>(9)? as u32,
                non_empty_session_count: row.get::<_, i32>(10)? as u32,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                git_repo_path,
                has_git_repo: has_git_repo != 0,
                git_remote_url,
                is_empty: is_empty != 0,
            })
        });

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
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
    pub fn import_sessions(&mut self, sessions: &[MantraSession]) -> Result<ImportResult, StorageError> {
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
                let mut stmt = tx.prepare(
                    "SELECT id FROM projects WHERE cwd = ?1"
                )?;

                match stmt.query_row(params![normalized_cwd], |row: &rusqlite::Row| row.get::<_, String>(0)) {
                    Ok(project_id) => {
                        Ok((project_id, false))
                    },
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        let id = Uuid::new_v4().to_string();
                        let name = extract_project_name(&normalized_cwd);
                        let now = Utc::now().to_rfc3339();

                        tx.execute(
                            "INSERT INTO projects (id, name, cwd, created_at, last_activity, git_repo_path, has_git_repo, git_remote_url, is_empty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![id, name, normalized_cwd, now, now, Option::<String>::None, 0, Option::<String>::None, 1],
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
                        "INSERT INTO sessions (id, project_id, source, cwd, created_at, updated_at, message_count, is_empty, raw_data)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            session.id,
                            project_id,
                            session.source.to_string(),
                            normalized_cwd,
                            session.created_at.to_rfc3339(),
                            session.updated_at.to_rfc3339(),
                            session.messages.len() as i32,
                            if session.is_empty() { 1 } else { 0 },
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

    /// Rename a project
    ///
    /// # Arguments
    /// * `project_id` - The project ID to rename
    /// * `new_name` - The new project name
    pub fn rename_project(&self, project_id: &str, new_name: &str) -> Result<(), StorageError> {
        let rows_affected = self.connection().execute(
            "UPDATE projects SET name = ?1 WHERE id = ?2",
            params![new_name, project_id],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Project with id {} not found",
                project_id
            )));
        }

        Ok(())
    }

    /// Delete a project and all associated sessions
    ///
    /// # Arguments
    /// * `project_id` - The project ID to delete
    pub fn delete_project(&self, project_id: &str) -> Result<(), StorageError> {
        // First, delete all sessions associated with this project
        self.connection().execute(
            "DELETE FROM sessions WHERE project_id = ?1",
            params![project_id],
        )?;

        // Then delete the project
        let rows_affected = self.connection().execute(
            "DELETE FROM projects WHERE id = ?1",
            params![project_id],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Project with id {} not found",
                project_id
            )));
        }

        Ok(())
    }

    /// Get all imported session IDs
    ///
    /// Returns a list of session IDs for all imported sessions.
    /// Used by ImportWizard to identify already-imported files.
    pub fn get_imported_session_ids(&self) -> Result<Vec<String>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id FROM sessions",
        )?;

        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
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
        self.connection().execute(
            "UPDATE sessions SET message_count = ?1, updated_at = ?2, raw_data = ?3 WHERE id = ?4",
            params![
                session.messages.len() as i32,
                session.updated_at.to_rfc3339(),
                raw_data,
                session.id
            ],
        )?;
        Ok(())
    }

    /// Update a project's is_empty status (Story 2.29 V2)
    ///
    /// A project is considered empty if all its sessions are empty.
    ///
    /// # Arguments
    /// * `project_id` - The project ID to update
    pub fn update_project_is_empty(&self, project_id: &str) -> Result<bool, StorageError> {
        // Check if all sessions in this project are empty
        let is_empty: bool = self.connection().query_row(
            "SELECT CASE
                WHEN COUNT(s.id) = 0 THEN 1
                WHEN COUNT(s.id) = SUM(CASE WHEN s.is_empty = 1 THEN 1 ELSE 0 END) THEN 1
                ELSE 0
             END
             FROM projects p
             LEFT JOIN sessions s ON s.project_id = p.id
             WHERE p.id = ?1
             GROUP BY p.id",
            params![project_id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        ).unwrap_or(true); // Default to empty if no sessions

        // Update the project
        self.connection().execute(
            "UPDATE projects SET is_empty = ?1 WHERE id = ?2",
            params![if is_empty { 1 } else { 0 }, project_id],
        )?;

        Ok(is_empty)
    }

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
        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        // Query sessions with project info where raw_data contains the query
        let mut stmt = self.connection().prepare(
            "SELECT s.id, s.project_id, s.raw_data, s.updated_at,
                    p.name as project_name,
                    json_extract(s.raw_data, '$.metadata.title') as session_title
             FROM sessions s
             JOIN projects p ON s.project_id = p.id
             WHERE s.raw_data LIKE ?1
             ORDER BY s.updated_at DESC",
        )?;

        let search_pattern = format!("%{}%", query);
        eprintln!("[search_sessions] SQL pattern: {}", search_pattern);

        let rows = stmt.query_map(params![search_pattern], |row| {
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

            eprintln!("[search_sessions] Processing session: {}", session_id);

            // Parse session JSON
            let session: MantraSession = match serde_json::from_str(&raw_data) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[search_sessions] Failed to parse session {}: {}", session_id, e);
                    continue;
                }
            };

            // Format session name
            let session_name = session_title.unwrap_or_else(|| {
                let parts: Vec<&str> = session_id.split(|c| c == '-' || c == '_').collect();
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

                // Extract text content from content blocks
                for block in &message.content_blocks {
                    let text = match block {
                        ContentBlock::Text { text } => text.clone(),
                        ContentBlock::Thinking { thinking } => thinking.clone(),
                        ContentBlock::ToolResult { content, .. } => content.clone(),
                        _ => continue,
                    };

                    let text_lower = text.to_lowercase();
                    if let Some(start_pos) = text_lower.find(&query_lower) {
                        // Calculate snippet with context (use char indices for UTF-8 safety)
                        let chars: Vec<char> = text.chars().collect();
                        let char_count = chars.len();

                        // Find char index for start_pos (byte position -> char position)
                        let char_start_pos = text[..start_pos].chars().count();
                        let query_char_len = query.chars().count();

                        let snippet_char_start = char_start_pos.saturating_sub(30);
                        let snippet_char_end = (char_start_pos + query_char_len + 70).min(char_count);

                        let snippet: String = chars[snippet_char_start..snippet_char_end].iter().collect();

                        // Adjust match position for snippet
                        let match_start_in_snippet = char_start_pos - snippet_char_start;
                        let match_end_in_snippet = match_start_in_snippet + query_char_len;

                        eprintln!("[search_sessions] Found match in session {} message {}", session_id, msg_idx);

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
                        });

                        // Only one result per message
                        break;
                    }
                }
            }
        }

        eprintln!("[search_sessions] Processed {} sessions, found {} results", session_count, results.len());

        Ok(results)
    }
}

/// Parse session source from string
fn parse_session_source(s: &str) -> SessionSource {
    // SessionSource is now a String type alias, just return the string
    s.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{sources, MantraSession};

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), sources::CLAUDE.to_string(), cwd.to_string())
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

    #[test]
    fn test_project_git_fields_default() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/test").unwrap();

        assert!(!project.has_git_repo);
        assert!(project.git_repo_path.is_none());
    }

    #[test]
    fn test_update_project_git_info() {
        let db = Database::new_in_memory().unwrap();
        db.get_or_create_project("/home/user/test").unwrap();

        // Update Git info
        db.update_project_git_info("/home/user/test", Some("/home/user/test".to_string()))
            .unwrap();

        // Verify update
        let (project, _) = db.get_or_create_project("/home/user/test").unwrap();
        assert!(project.has_git_repo);
        assert_eq!(project.git_repo_path, Some("/home/user/test".to_string()));

        // Clear Git info
        db.update_project_git_info("/home/user/test", None).unwrap();

        let (project, _) = db.get_or_create_project("/home/user/test").unwrap();
        assert!(!project.has_git_repo);
        assert!(project.git_repo_path.is_none());
    }

    #[test]
    fn test_get_project_by_id() {
        let db = Database::new_in_memory().unwrap();
        let (created_project, _) = db.get_or_create_project("/home/user/test").unwrap();

        let project = db.get_project(&created_project.id).unwrap();
        assert!(project.is_some());
        assert_eq!(project.unwrap().id, created_project.id);

        let not_found = db.get_project("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_project_by_cwd() {
        let db = Database::new_in_memory().unwrap();
        db.get_or_create_project("/home/user/test").unwrap();

        let project = db.get_project_by_cwd("/home/user/test").unwrap();
        assert!(project.is_some());
        assert_eq!(project.unwrap().cwd, "/home/user/test");

        let not_found = db.get_project_by_cwd("/nonexistent/path").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_projects_includes_git_fields() {
        let db = Database::new_in_memory().unwrap();
        db.get_or_create_project("/home/user/test").unwrap();
        db.update_project_git_info("/home/user/test", Some("/home/user/test".to_string()))
            .unwrap();

        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert!(projects[0].has_git_repo);
        assert_eq!(projects[0].git_repo_path, Some("/home/user/test".to_string()));
    }

    // Story 2.25: Multi-source aggregation tests
    #[test]
    fn test_multi_source_aggregation_same_cwd() {
        let mut db = Database::new_in_memory().unwrap();

        // Create sessions from different sources with the same cwd
        let claude_session = MantraSession::new(
            "sess_claude_1".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/myproject".to_string(),
        );
        let gemini_session = MantraSession::new(
            "sess_gemini_1".to_string(),
            sources::GEMINI.to_string(),
            "/home/user/myproject".to_string(),
        );
        let cursor_session = MantraSession::new(
            "sess_cursor_1".to_string(),
            sources::CURSOR.to_string(),
            "/home/user/myproject".to_string(),
        );

        // Import all sessions
        let result = db.import_sessions(&[claude_session, gemini_session, cursor_session]).unwrap();

        // All should be imported
        assert_eq!(result.imported_count, 3);
        // Only ONE project should be created (aggregated by cwd)
        assert_eq!(result.new_projects_count, 1);

        // Verify only one project exists
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].session_count, 3);

        // Verify all sessions are under the same project
        let sessions = db.get_project_sessions(&projects[0].id).unwrap();
        assert_eq!(sessions.len(), 3);

        // Verify sources are preserved
        let sources: Vec<&str> = sessions.iter().map(|s| s.source.as_str()).collect();
        assert!(sources.contains(&"claude"));
        assert!(sources.contains(&"gemini"));
        assert!(sources.contains(&"cursor"));
    }

    #[test]
    fn test_path_normalization_aggregation() {
        let mut db = Database::new_in_memory().unwrap();

        // Sessions with different path formats pointing to the same location
        let session1 = MantraSession::new(
            "sess_1".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/project".to_string(),
        );
        let session2 = MantraSession::new(
            "sess_2".to_string(),
            sources::GEMINI.to_string(),
            "/home/user/project/".to_string(), // With trailing slash
        );
        let session3 = MantraSession::new(
            "sess_3".to_string(),
            sources::CURSOR.to_string(),
            "/home/user/project//".to_string(), // Multiple trailing slashes
        );

        let result = db.import_sessions(&[session1, session2, session3]).unwrap();

        assert_eq!(result.imported_count, 3);
        assert_eq!(result.new_projects_count, 1); // All aggregated to one project

        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].cwd, "/home/user/project"); // Normalized
    }

    #[test]
    fn test_first_project_name_preserved() {
        let mut db = Database::new_in_memory().unwrap();

        // First import sets the project name
        let session1 = MantraSession::new(
            "sess_1".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/my-awesome-project".to_string(),
        );
        db.import_sessions(&[session1]).unwrap();

        let projects = db.list_projects().unwrap();
        let original_name = projects[0].name.clone();

        // Second import with same cwd should NOT change the name
        let session2 = MantraSession::new(
            "sess_2".to_string(),
            sources::GEMINI.to_string(),
            "/home/user/my-awesome-project".to_string(),
        );
        db.import_sessions(&[session2]).unwrap();

        let projects = db.list_projects().unwrap();
        assert_eq!(projects[0].name, original_name);
    }

    // ===== Story 1.9: Enhanced Project Identification Tests =====

    #[test]
    fn test_find_by_git_remote_found() {
        let db = Database::new_in_memory().unwrap();

        // Create project with Git remote URL
        let (project, _) = db.find_or_create_project(
            "/home/user/project1",
            Some("https://github.com/user/repo"),
        ).unwrap();

        // Find by Git remote URL
        let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, project.id);
    }

    #[test]
    fn test_find_by_git_remote_not_found() {
        let db = Database::new_in_memory().unwrap();

        // Create project without Git remote URL
        db.find_or_create_project("/home/user/project1", None).unwrap();

        // Should not find any project
        let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_find_by_git_remote_normalizes_url() {
        let db = Database::new_in_memory().unwrap();

        // Create project with SSH format URL
        db.find_or_create_project(
            "/home/user/project1",
            Some("git@github.com:user/repo.git"),
        ).unwrap();

        // Find by HTTPS format (should be normalized to match)
        let found = db.find_by_git_remote("https://github.com/user/repo").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_find_or_create_git_url_priority() {
        let db = Database::new_in_memory().unwrap();

        // Create project with path1 and Git URL
        let (project1, _) = db.find_or_create_project(
            "/home/user/path1",
            Some("https://github.com/user/myrepo"),
        ).unwrap();

        // Create session with path2 but SAME Git URL
        // Should aggregate to existing project (Git URL priority)
        let (project2, is_new) = db.find_or_create_project(
            "/home/user/path2",
            Some("https://github.com/user/myrepo"),
        ).unwrap();

        assert!(!is_new, "Should aggregate to existing project by Git URL");
        assert_eq!(project1.id, project2.id, "Same project ID");
    }

    #[test]
    fn test_find_or_create_path_fallback() {
        let db = Database::new_in_memory().unwrap();

        // Create project without Git URL
        let (project1, _) = db.find_or_create_project(
            "/home/user/project",
            None,
        ).unwrap();

        // Same path, no Git URL → fallback to path match
        let (project2, is_new) = db.find_or_create_project(
            "/home/user/project",
            None,
        ).unwrap();

        assert!(!is_new, "Should find existing project by path");
        assert_eq!(project1.id, project2.id);
    }

    #[test]
    fn test_find_or_create_updates_missing_git_url() {
        let db = Database::new_in_memory().unwrap();

        // Create project without Git URL
        let (project1, _) = db.find_or_create_project(
            "/home/user/project",
            None,
        ).unwrap();
        assert!(project1.git_remote_url.is_none());

        // Same path but now with Git URL → should update
        let (project2, is_new) = db.find_or_create_project(
            "/home/user/project",
            Some("https://github.com/user/repo"),
        ).unwrap();

        assert!(!is_new, "Should aggregate, not create new");
        assert_eq!(project1.id, project2.id);
        assert!(project2.git_remote_url.is_some(), "Git URL should be updated");
        assert_eq!(project2.git_remote_url.unwrap(), "https://github.com/user/repo");
    }

    #[test]
    fn test_find_or_create_path_reuse_conflict() {
        let db = Database::new_in_memory().unwrap();

        // Create project with Git URL A
        let (project1, _) = db.find_or_create_project(
            "/home/user/project",
            Some("https://github.com/user/repoA"),
        ).unwrap();

        // Same path but DIFFERENT Git URL → path reuse!
        // Should update project's Git URL to the new one (not create new project)
        let (project2, is_new) = db.find_or_create_project(
            "/home/user/project",
            Some("https://github.com/user/repoB"),
        ).unwrap();

        assert!(!is_new, "Should update existing project, not create new");
        assert_eq!(project1.id, project2.id, "Same project ID");
        assert_eq!(project2.git_remote_url.unwrap(), "https://github.com/user/repoB", "Git URL should be updated");
    }

    #[test]
    fn test_find_or_create_project_has_url_session_no_url() {
        let db = Database::new_in_memory().unwrap();

        // Create project with Git URL
        let (project1, _) = db.find_or_create_project(
            "/home/user/project",
            Some("https://github.com/user/repo"),
        ).unwrap();

        // Same path, no Git URL → aggregate to existing project
        let (project2, is_new) = db.find_or_create_project(
            "/home/user/project",
            None,
        ).unwrap();

        assert!(!is_new, "Should aggregate by path");
        assert_eq!(project1.id, project2.id);
    }

    #[test]
    fn test_import_session_with_git_url() {
        let db = Database::new_in_memory().unwrap();

        let session = MantraSession::new(
            "sess_1".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/project".to_string(),
        );

        let (imported, is_new, project_id) = db.import_session_with_git_url(
            &session,
            Some("https://github.com/user/repo"),
        ).unwrap();

        assert!(imported);
        assert!(is_new);
        assert!(!project_id.is_empty());

        // Verify Git URL is stored
        let project = db.get_project(&project_id).unwrap().unwrap();
        assert_eq!(project.git_remote_url, Some("https://github.com/user/repo".to_string()));
    }

    #[test]
    fn test_cross_path_aggregation_by_git_url() {
        let db = Database::new_in_memory().unwrap();

        // Session 1: path1 + repo URL
        let session1 = MantraSession::new(
            "sess_1".to_string(),
            sources::CLAUDE.to_string(),
            "/home/user/path1/myrepo".to_string(),
        );
        let (_, _, project_id1) = db.import_session_with_git_url(
            &session1,
            Some("https://github.com/user/myrepo"),
        ).unwrap();

        // Session 2: path2 + SAME repo URL
        let session2 = MantraSession::new(
            "sess_2".to_string(),
            sources::GEMINI.to_string(),
            "/home/user/path2/myrepo".to_string(),
        );
        let (imported, is_new, project_id2) = db.import_session_with_git_url(
            &session2,
            Some("https://github.com/user/myrepo"),
        ).unwrap();

        assert!(imported);
        assert!(!is_new, "Should aggregate by Git URL, not create new project");
        assert_eq!(project_id1, project_id2, "Both sessions in same project");

        // Verify only one project exists
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].session_count, 2);
    }
}
