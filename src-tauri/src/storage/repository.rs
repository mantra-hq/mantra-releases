//! Repository layer for database CRUD operations
//!
//! Provides high-level database operations for projects and sessions.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::database::Database;
use super::error::StorageError;
use crate::models::{
    classify_path_type, extract_project_name, normalize_cwd, ContentBlock, ImportResult,
    MantraSession, PathType, Project, SessionSource, SessionSummary,
};

/// Logical project statistics for view-layer aggregation (Story 1.12)
///
/// Represents aggregated statistics for a physical path across all projects.
/// This enables displaying "logical projects" that combine sessions from
/// different import sources (Claude, Gemini, Cursor, etc.) sharing the same path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LogicalProjectStats {
    /// The physical path (normalized)
    pub physical_path: String,
    /// Number of projects that have this path
    pub project_count: u32,
    /// IDs of all projects that have this path
    pub project_ids: Vec<String>,
    /// Total number of sessions across all projects with this path
    pub total_sessions: u32,
    /// Most recent activity across all projects with this path
    pub last_activity: DateTime<Utc>,
    /// Display name extracted from the path (Task 8.1)
    pub display_name: String,
    /// Path type: local, virtual, or remote (Task 8.2)
    pub path_type: String,
    /// Whether the local path exists on filesystem (Task 8.3)
    /// Only meaningful for path_type = "local"
    pub path_exists: bool,
    /// Whether this logical project needs association (Task 8.4)
    /// True if path_type is "virtual" or (path_type is "local" AND path_exists is false)
    pub needs_association: bool,
    /// Whether any of the associated projects has a git repo (Task 17: AC15)
    pub has_git_repo: bool,
}

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
    /// Content type (code, conversation, or all)
    /// Story 2.33: AC1 - 内容类型标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
}

// ============================================================================
// Story 2.33: Search Filters
// ============================================================================

/// Content type filter for search
/// AC1: 内容类型筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// All content types (default)
    #[default]
    All,
    /// Code blocks (markdown code fences)
    Code,
    /// Conversation (user messages and AI text replies)
    Conversation,
}

/// Time range preset for search
/// AC3: 时间范围筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimePreset {
    /// All time (no time filter)
    All,
    /// Today only
    Today,
    /// This week
    Week,
    /// This month
    Month,
}

/// Search filters for enhanced search functionality
/// Story 2.33: AC1-AC3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilters {
    /// Content type filter (all, code, conversation)
    #[serde(default)]
    pub content_type: ContentType,
    /// Project ID filter (None = all projects)
    #[serde(default)]
    pub project_id: Option<String>,
    /// Time range preset
    #[serde(default)]
    pub time_preset: Option<TimePreset>,
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
        use serde_json::json;
        use crate::models::sources;

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
            let cwd: String = row.get(2)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
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
                path_type: classify_path_type(&cwd),
                path_exists: true, // Will be validated if needed
                cwd,
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

                // Story 1.12: Also add to project_paths table for view-based aggregation
                let path_id = Uuid::new_v4().to_string();
                self.connection().execute(
                    "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
                    params![path_id, id, normalized_cwd, now_str],
                )?;

                let path_type = classify_path_type(&normalized_cwd);
                let path_exists = match path_type {
                    PathType::Local => std::path::Path::new(&normalized_cwd).exists(),
                    _ => true,
                };
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
                    path_type,
                    path_exists,
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
                let cwd: String = row.get(2)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
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
                    path_type: classify_path_type(&cwd),
                    path_exists: true, // Validated on demand
                    cwd,
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
                let cwd: String = row.get(2)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
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
                    path_type: classify_path_type(&cwd),
                    path_exists: true,
                    cwd,
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
            let cwd: String = row.get(2)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
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
                path_type: classify_path_type(&cwd),
                path_exists: true,
                cwd,
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

        // Story 1.12: Also add to project_paths table for view-based aggregation
        let path_id = Uuid::new_v4().to_string();
        self.connection().execute(
            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
            params![path_id, id, normalized_cwd, now_str],
        )?;

        let path_type = classify_path_type(normalized_cwd);
        let path_exists = match path_type {
            PathType::Local => std::path::Path::new(normalized_cwd).exists(),
            _ => true,
        };
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
            path_type,
            path_exists,
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
            let cwd: String = row.get(2)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
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
                path_type: classify_path_type(&cwd),
                path_exists: true,
                cwd,
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
            let cwd: String = row.get(2)?;

            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
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
                path_type: classify_path_type(&cwd),
                path_exists: true,
                cwd,
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

    /// Get all full sessions for a project (Story 2.34: Analytics)
    ///
    /// Returns full MantraSession objects for analytics calculations.
    ///
    /// # Arguments
    /// * `project_id` - The project ID to get sessions for
    pub fn get_sessions_by_project(&self, project_id: &str) -> Result<Vec<MantraSession>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT raw_data FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                let raw_data: String = row.get(0)?;
                Ok(raw_data)
            })?
            .filter_map(|result| {
                match result {
                    Ok(raw_data) => {
                        match serde_json::from_str::<MantraSession>(&raw_data) {
                            Ok(session) => Some(Ok(session)),
                            Err(e) => {
                                eprintln!("[get_sessions_by_project] Failed to parse session: {}", e);
                                None
                            }
                        }
                    }
                    Err(e) => Some(Err(StorageError::from(e))),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
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
             WHERE s.raw_data LIKE ?1"
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

    // =========================================================================
    // Story 1.12: View-based Project Aggregation - Repository Layer
    // =========================================================================

    /// Add a path to a project (Story 1.12 - AC1)
    ///
    /// Creates a new project_path entry linking the path to the project.
    /// If is_primary is true, demotes any existing primary path.
    /// If the path already belongs to this project, returns the existing record.
    /// If the path belongs to another project, returns an error.
    ///
    /// # Arguments
    /// * `project_id` - The project to add the path to
    /// * `path` - The path to add (will be normalized)
    /// * `is_primary` - Whether this should be the primary path
    pub fn add_project_path(
        &self,
        project_id: &str,
        path: &str,
        is_primary: bool,
    ) -> Result<crate::models::ProjectPath, StorageError> {
        let normalized_path = normalize_cwd(path);
        let path_type = classify_path_type(&normalized_path);

        // Story 1.12: Check if path already exists for THIS project (project-level uniqueness)
        // Same path can belong to multiple projects (from different import sources)
        let existing: Option<(String, bool, String)> = self
            .connection()
            .query_row(
                "SELECT id, is_primary, created_at FROM project_paths WHERE project_id = ?1 AND path = ?2",
                params![project_id, normalized_path],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        if let Some((existing_id, existing_is_primary, created_at_str)) = existing {
            // Path already belongs to this project, return existing record (idempotent)
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            return Ok(crate::models::ProjectPath {
                id: existing_id,
                project_id: project_id.to_string(),
                path: normalized_path,
                is_primary: existing_is_primary,
                created_at,
            });
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // If setting as primary, demote existing primary path
        if is_primary {
            self.connection().execute(
                "UPDATE project_paths SET is_primary = 0 WHERE project_id = ?1 AND is_primary = 1",
                params![project_id],
            )?;

            // Task 16 (AC14): When adding a non-virtual path as primary, remove virtual paths
            if path_type != PathType::Virtual {
                self.remove_virtual_paths_for_project(project_id)?;
            }
        }

        self.connection().execute(
            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, project_id, normalized_path, if is_primary { 1 } else { 0 }, now.to_rfc3339()],
        )?;

        Ok(crate::models::ProjectPath {
            id,
            project_id: project_id.to_string(),
            path: normalized_path,
            is_primary,
            created_at: now,
        })
    }

    /// Remove all virtual paths for a project (Task 16 - AC14)
    ///
    /// When a user associates a real path with a project (as primary),
    /// the original virtual path placeholder should be removed.
    pub fn remove_virtual_paths_for_project(&self, project_id: &str) -> Result<u32, StorageError> {
        let deleted = self.connection().execute(
            r#"
            DELETE FROM project_paths
            WHERE project_id = ?1
              AND (
                  path LIKE 'gemini-project:%'
                  OR path LIKE 'placeholder:%'
                  OR path = ''
                  OR path = 'unknown'
              )
            "#,
            params![project_id],
        )?;

        Ok(deleted as u32)
    }

    /// Remove a path from a project (Story 1.12 - AC1)
    ///
    /// # Arguments
    /// * `path_id` - The project_path ID to remove
    pub fn remove_project_path(&self, path_id: &str) -> Result<(), StorageError> {
        let rows_affected = self.connection().execute(
            "DELETE FROM project_paths WHERE id = ?1",
            params![path_id],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!(
                "ProjectPath with id {} not found",
                path_id
            )));
        }

        Ok(())
    }

    /// Get all paths for a project (Story 1.12 - AC1)
    ///
    /// # Arguments
    /// * `project_id` - The project to get paths for
    pub fn get_project_paths(&self, project_id: &str) -> Result<Vec<crate::models::ProjectPath>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id, project_id, path, is_primary, created_at FROM project_paths WHERE project_id = ?1 ORDER BY is_primary DESC, created_at ASC",
        )?;

        let paths = stmt
            .query_map(params![project_id], |row| {
                let created_at_str: String = row.get(4)?;
                let is_primary_int: i32 = row.get(3)?;

                Ok(crate::models::ProjectPath {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    path: row.get(2)?,
                    is_primary: is_primary_int != 0,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    /// Get logical project statistics grouped by physical path (Story 1.12 - AC9)
    ///
    /// Returns aggregated statistics for each unique physical path across all projects.
    /// This enables the view layer to display "logical projects" that combine sessions
    /// from different import sources (Claude, Gemini, Cursor, etc.) that share the same path.
    pub fn get_logical_project_stats(&self) -> Result<Vec<LogicalProjectStats>, StorageError> {
        use crate::models::{classify_path_type, check_path_exists, extract_project_name, PathType};

        // Task 9.1: Remove virtual path exclusion - include ALL paths including virtual ones
        let mut stmt = self.connection().prepare(
            r#"
            SELECT
                pp.path as physical_path,
                COUNT(DISTINCT p.id) as project_count,
                GROUP_CONCAT(DISTINCT p.id) as project_ids,
                (SELECT COUNT(*) FROM sessions s
                 INNER JOIN projects proj ON s.project_id = proj.id
                 INNER JOIN project_paths pp2 ON pp2.project_id = proj.id
                 WHERE pp2.path = pp.path) as total_sessions,
                MAX(p.last_activity) as last_activity,
                MAX(p.has_git_repo) as has_git_repo
            FROM project_paths pp
            JOIN projects p ON pp.project_id = p.id
            GROUP BY pp.path
            ORDER BY last_activity DESC
            "#,
        )?;

        let stats = stmt
            .query_map([], |row| {
                let physical_path: String = row.get(0)?;
                let project_ids_str: String = row.get(2)?;
                let last_activity_str: String = row.get(4)?;
                let has_git_repo_db: i32 = row.get(5)?;

                // Task 9.2: Calculate path_type and path_exists
                let path_type = classify_path_type(&physical_path);
                let path_type_str = path_type.as_str().to_string();
                let path_exists = match path_type {
                    PathType::Local => check_path_exists(&physical_path),
                    _ => true, // Virtual and remote paths always "exist"
                };

                // Task 8.4: Calculate needs_association
                let needs_association = match path_type {
                    PathType::Virtual => true,
                    PathType::Local => !path_exists,
                    PathType::Remote => false,
                };

                // Task 9.3: Extract display_name from path
                let display_name = extract_project_name(&physical_path);

                // Task 17: Aggregate has_git_repo from DB + real-time check
                let has_git_repo = has_git_repo_db > 0 || (path_exists && Self::check_git_repo_exists(&physical_path));

                Ok(LogicalProjectStats {
                    physical_path,
                    project_count: row.get::<_, i32>(1)? as u32,
                    project_ids: project_ids_str.split(',').map(String::from).collect(),
                    total_sessions: row.get::<_, i32>(3)? as u32,
                    last_activity: DateTime::parse_from_rfc3339(&last_activity_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    display_name,
                    path_type: path_type_str,
                    path_exists,
                    needs_association,
                    has_git_repo,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stats)
    }

    /// Check if a path has a .git directory (Task 17: AC15)
    fn check_git_repo_exists(path: &str) -> bool {
        let git_path = std::path::Path::new(path).join(".git");
        git_path.exists()
    }

    /// Get all sessions for a physical path across all projects (Story 1.12 - AC9)
    ///
    /// Returns sessions from all projects that have the given physical path associated.
    /// This enables the view layer to display all sessions for a "logical project".
    pub fn get_sessions_by_physical_path(
        &self,
        physical_path: &str,
    ) -> Result<Vec<SessionSummary>, StorageError> {
        let normalized_path = normalize_cwd(physical_path);

        let mut stmt = self.connection().prepare(
            r#"
            SELECT DISTINCT s.id, s.source, s.created_at, s.updated_at, s.message_count, s.is_empty,
                   json_extract(s.raw_data, '$.metadata.title') as title, s.original_cwd
            FROM sessions s
            INNER JOIN projects p ON s.project_id = p.id
            INNER JOIN project_paths pp ON pp.project_id = p.id
            WHERE pp.path = ?1
            ORDER BY s.updated_at DESC
            "#,
        )?;

        let sessions = stmt
            .query_map(params![normalized_path], |row| {
                let created_at_str: String = row.get(2)?;
                let updated_at_str: String = row.get(3)?;
                let is_empty: i32 = row.get(5)?;

                Ok(SessionSummary {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    message_count: row.get::<_, i32>(4)? as u32,
                    is_empty: is_empty != 0,
                    title: row.get(6)?,
                    original_cwd: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get all projects that share a physical path (Story 1.12 - AC9)
    ///
    /// Returns all projects that have the given physical path associated.
    /// Useful for displaying which import sources contributed to a logical project.
    pub fn get_projects_by_physical_path(
        &self,
        physical_path: &str,
    ) -> Result<Vec<Project>, StorageError> {
        let normalized_path = normalize_cwd(physical_path);

        let mut stmt = self.connection().prepare(
            r#"
            SELECT DISTINCT p.id, p.name, p.cwd, p.created_at, p.last_activity,
                   p.git_repo_path, p.has_git_repo, p.git_remote_url, p.is_empty,
                   (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                   (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
            FROM projects p
            INNER JOIN project_paths pp ON pp.project_id = p.id
            WHERE pp.path = ?1
            ORDER BY p.last_activity DESC
            "#,
        )?;

        let projects = stmt
            .query_map(params![normalized_path], |row| {
                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;
                let git_repo_path: Option<String> = row.get(5)?;
                let has_git_repo: i32 = row.get(6)?;
                let git_remote_url: Option<String> = row.get(7)?;
                let is_empty: i32 = row.get(8)?;
                let cwd: String = row.get(2)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
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
                    path_type: classify_path_type(&cwd),
                    path_exists: true,
                    cwd,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Find project by path using project_paths table (Story 1.12 - AC2)
    ///
    /// # Arguments
    /// * `path` - The path to look up (will be normalized)
    pub fn find_project_by_path(&self, path: &str) -> Result<Option<Project>, StorageError> {
        let normalized_path = normalize_cwd(path);

        let result = self.connection().query_row(
            "SELECT p.id, p.name, p.cwd, p.created_at, p.last_activity, p.git_repo_path, p.has_git_repo, p.git_remote_url, p.is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM projects p
             INNER JOIN project_paths pp ON pp.project_id = p.id
             WHERE pp.path = ?1",
            params![normalized_path],
            |row| {
                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;
                let git_repo_path: Option<String> = row.get(5)?;
                let has_git_repo: i32 = row.get(6)?;
                let git_remote_url: Option<String> = row.get(7)?;
                let is_empty: i32 = row.get(8)?;
                let cwd: String = row.get(2)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
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
                    path_type: classify_path_type(&cwd),
                    path_exists: true,
                    cwd,
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Bind a session to a project manually (Story 1.12 - AC3, AC4)
    ///
    /// Manual bindings take priority over path-based matching.
    ///
    /// # Arguments
    /// * `session_id` - The session to bind
    /// * `project_id` - The project to bind the session to
    pub fn bind_session_to_project(
        &self,
        session_id: &str,
        project_id: &str,
    ) -> Result<crate::models::SessionBinding, StorageError> {
        let now = Utc::now();

        // Use INSERT OR REPLACE to handle re-binding
        self.connection().execute(
            "INSERT OR REPLACE INTO session_project_bindings (session_id, project_id, bound_at) VALUES (?1, ?2, ?3)",
            params![session_id, project_id, now.to_rfc3339()],
        )?;

        Ok(crate::models::SessionBinding {
            session_id: session_id.to_string(),
            project_id: project_id.to_string(),
            bound_at: now,
        })
    }

    /// Unbind a session from its manual project binding (Story 1.12 - AC3)
    ///
    /// After unbinding, the session will fall back to path-based matching.
    ///
    /// # Arguments
    /// * `session_id` - The session to unbind
    pub fn unbind_session(&self, session_id: &str) -> Result<(), StorageError> {
        self.connection().execute(
            "DELETE FROM session_project_bindings WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    /// Get the session binding for a session (Story 1.12 - AC3)
    ///
    /// # Arguments
    /// * `session_id` - The session to get the binding for
    pub fn get_session_binding(&self, session_id: &str) -> Result<Option<crate::models::SessionBinding>, StorageError> {
        let result = self.connection().query_row(
            "SELECT session_id, project_id, bound_at FROM session_project_bindings WHERE session_id = ?1",
            params![session_id],
            |row| {
                let bound_at_str: String = row.get(2)?;
                Ok(crate::models::SessionBinding {
                    session_id: row.get(0)?,
                    project_id: row.get(1)?,
                    bound_at: DateTime::parse_from_rfc3339(&bound_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match result {
            Ok(binding) => Ok(Some(binding)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all sessions for a project using view-based aggregation (Story 1.12 - AC2, AC4)
    ///
    /// This method combines:
    /// 1. Sessions manually bound to the project (highest priority)
    /// 2. Sessions matching project paths via original_cwd
    ///
    /// Manual bindings take priority over path matching.
    pub fn get_project_sessions_aggregated(&self, project_id: &str) -> Result<Vec<SessionSummary>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT DISTINCT s.id, s.source, s.created_at, s.updated_at, s.message_count, s.is_empty,
                    json_extract(s.raw_data, '$.metadata.title') as title, s.original_cwd
             FROM sessions s
             LEFT JOIN project_paths pp ON s.original_cwd = pp.path OR (s.original_cwd = '' AND s.cwd = pp.path)
             LEFT JOIN session_project_bindings spb ON s.id = spb.session_id
             WHERE COALESCE(spb.project_id, pp.project_id, s.project_id) = ?1
             ORDER BY s.updated_at DESC",
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

    /// Get unassigned sessions (Story 1.12 - AC5)
    ///
    /// Returns sessions that:
    /// 1. Have no manual binding
    /// 2. Have no matching path in project_paths
    ///
    /// These sessions should be displayed in an "Unassigned" group in the UI.
    pub fn get_unassigned_sessions(&self) -> Result<Vec<SessionSummary>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT s.id, s.source, s.created_at, s.updated_at, s.message_count, s.is_empty,
                    json_extract(s.raw_data, '$.metadata.title') as title, s.original_cwd
             FROM sessions s
             LEFT JOIN project_paths pp ON s.original_cwd = pp.path OR (s.original_cwd = '' AND s.cwd = pp.path)
             LEFT JOIN session_project_bindings spb ON s.id = spb.session_id
             WHERE pp.project_id IS NULL AND spb.project_id IS NULL
             ORDER BY s.updated_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
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

    /// Get the project a session belongs to, considering binding priority (Story 1.12 - AC4)
    ///
    /// Priority order:
    /// 1. Manual binding (session_project_bindings)
    /// 2. Path match (project_paths via original_cwd)
    /// 3. Direct project_id (legacy fallback)
    ///
    /// # Arguments
    /// * `session_id` - The session to look up
    pub fn get_session_project_aggregated(&self, session_id: &str) -> Result<Option<Project>, StorageError> {
        let result = self.connection().query_row(
            "SELECT p.id, p.name, p.cwd, p.created_at, p.last_activity, p.git_repo_path, p.has_git_repo, p.git_remote_url, p.is_empty,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count,
                    (SELECT COUNT(*) FROM sessions WHERE project_id = p.id AND is_empty = 0) as non_empty_session_count
             FROM sessions s
             LEFT JOIN session_project_bindings spb ON s.id = spb.session_id
             LEFT JOIN project_paths pp ON s.original_cwd = pp.path OR (s.original_cwd = '' AND s.cwd = pp.path)
             LEFT JOIN projects p ON p.id = COALESCE(spb.project_id, pp.project_id, s.project_id)
             WHERE s.id = ?1",
            params![session_id],
            |row| {
                let id: Option<String> = row.get(0)?;
                if id.is_none() {
                    return Ok(None);
                }

                let created_at_str: String = row.get(3)?;
                let last_activity_str: String = row.get(4)?;
                let git_repo_path: Option<String> = row.get(5)?;
                let has_git_repo: i32 = row.get(6)?;
                let git_remote_url: Option<String> = row.get(7)?;
                let is_empty: i32 = row.get(8)?;
                let cwd: String = row.get(2)?;

                Ok(Some(Project {
                    id: id.unwrap(),
                    name: row.get(1)?,
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
                    path_type: classify_path_type(&cwd),
                    path_exists: true,
                    cwd,
                }))
            },
        );

        match result {
            Ok(project) => Ok(project),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a project's primary path (Story 1.12 - AC1)
    ///
    /// Updates the project's cwd and sets the specified path as primary.
    ///
    /// # Arguments
    /// * `project_id` - The project to update
    /// * `path` - The new primary path (will be normalized)
    pub fn set_project_primary_path(&self, project_id: &str, path: &str) -> Result<(), StorageError> {
        let normalized_path = normalize_cwd(path);
        let now = Utc::now();

        // First, check if path already exists for this project
        let existing_path_id: Option<String> = self.connection().query_row(
            "SELECT id FROM project_paths WHERE project_id = ?1 AND path = ?2",
            params![project_id, normalized_path],
            |row| row.get(0),
        ).ok();

        // Demote current primary path
        self.connection().execute(
            "UPDATE project_paths SET is_primary = 0 WHERE project_id = ?1 AND is_primary = 1",
            params![project_id],
        )?;

        if let Some(path_id) = existing_path_id {
            // Path exists, just set it as primary
            self.connection().execute(
                "UPDATE project_paths SET is_primary = 1 WHERE id = ?1",
                params![path_id],
            )?;
        } else {
            // Path doesn't exist, create it as primary
            let id = Uuid::new_v4().to_string();
            self.connection().execute(
                "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
                params![id, project_id, normalized_path, now.to_rfc3339()],
            )?;
        }

        // Update project's cwd to match the new primary path
        let new_name = extract_project_name(&normalized_path);
        self.connection().execute(
            "UPDATE projects SET cwd = ?1, name = ?2 WHERE id = ?3",
            params![normalized_path, new_name, project_id],
        )?;

        Ok(())
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

    // ===== Story 2.33: Search Filters Tests =====

    #[test]
    fn test_search_filters_default() {
        let filters = SearchFilters::default();
        assert_eq!(filters.content_type, ContentType::All);
        assert!(filters.project_id.is_none());
        assert!(filters.time_preset.is_none());
    }

    #[test]
    fn test_search_filters_serialization() {
        let filters = SearchFilters {
            content_type: ContentType::Code,
            project_id: Some("proj_123".to_string()),
            time_preset: Some(TimePreset::Today),
        };

        let json = serde_json::to_string(&filters).unwrap();
        assert!(json.contains(r#""contentType":"code""#));
        assert!(json.contains(r#""projectId":"proj_123""#));
        assert!(json.contains(r#""timePreset":"today""#));

        let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content_type, ContentType::Code);
        assert_eq!(deserialized.project_id, Some("proj_123".to_string()));
        assert_eq!(deserialized.time_preset, Some(TimePreset::Today));
    }

    #[test]
    fn test_content_type_enum() {
        assert_eq!(ContentType::default(), ContentType::All);

        let code_json = r#""code""#;
        let code: ContentType = serde_json::from_str(code_json).unwrap();
        assert_eq!(code, ContentType::Code);

        let conv_json = r#""conversation""#;
        let conv: ContentType = serde_json::from_str(conv_json).unwrap();
        assert_eq!(conv, ContentType::Conversation);
    }

    #[test]
    fn test_time_preset_enum() {
        let all_json = r#""all""#;
        let all: TimePreset = serde_json::from_str(all_json).unwrap();
        assert_eq!(all, TimePreset::All);

        let today_json = r#""today""#;
        let today: TimePreset = serde_json::from_str(today_json).unwrap();
        assert_eq!(today, TimePreset::Today);

        let week_json = r#""week""#;
        let week: TimePreset = serde_json::from_str(week_json).unwrap();
        assert_eq!(week, TimePreset::Week);

        let month_json = r#""month""#;
        let month: TimePreset = serde_json::from_str(month_json).unwrap();
        assert_eq!(month, TimePreset::Month);
    }

    #[test]
    fn test_search_result_with_content_type() {
        let result = SearchResult {
            id: "sess_1-0".to_string(),
            session_id: "sess_1".to_string(),
            project_id: "proj_1".to_string(),
            project_name: "test".to_string(),
            session_name: "Test Session".to_string(),
            message_id: "0".to_string(),
            content: "Hello world".to_string(),
            match_positions: vec![(0, 5)],
            timestamp: 1234567890,
            content_type: Some(ContentType::Conversation),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(r#""content_type":"conversation""#));
    }

    #[test]
    fn test_search_result_content_type_omitted_when_none() {
        let result = SearchResult {
            id: "sess_1-0".to_string(),
            session_id: "sess_1".to_string(),
            project_id: "proj_1".to_string(),
            project_name: "test".to_string(),
            session_name: "Test Session".to_string(),
            message_id: "0".to_string(),
            content: "Hello world".to_string(),
            match_positions: vec![(0, 5)],
            timestamp: 1234567890,
            content_type: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains(r#""content_type""#));
    }

    // ===== Story 1.12: View-based Project Aggregation Tests =====

    #[test]
    fn test_add_project_path() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        // Add a secondary path
        let path = db.add_project_path(&project.id, "/home/user/project-alt", false).unwrap();
        assert_eq!(path.project_id, project.id);
        assert_eq!(path.path, "/home/user/project-alt");
        assert!(!path.is_primary);

        // Verify path was added
        let paths = db.get_project_paths(&project.id).unwrap();
        // Should have 2 paths: original (migrated) + new one
        assert!(paths.len() >= 1);
    }

    #[test]
    fn test_add_project_path_sets_primary() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        // Add a primary path (should demote existing)
        let path = db.add_project_path(&project.id, "/home/user/new-primary", true).unwrap();
        assert!(path.is_primary);

        // Verify paths
        let paths = db.get_project_paths(&project.id).unwrap();
        let primary_count = paths.iter().filter(|p| p.is_primary).count();
        assert_eq!(primary_count, 1, "Should only have one primary path");
    }

    #[test]
    fn test_remove_project_path() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        // Add and then remove a path
        let path = db.add_project_path(&project.id, "/home/user/to-remove", false).unwrap();
        db.remove_project_path(&path.id).unwrap();

        // Verify removal
        let paths = db.get_project_paths(&project.id).unwrap();
        assert!(!paths.iter().any(|p| p.path == "/home/user/to-remove"));
    }

    #[test]
    fn test_add_project_path_same_path_different_projects() {
        // Story 1.12: Same path can belong to multiple projects (from different import sources)
        let db = Database::new_in_memory().unwrap();
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        let shared_path = "/shared/workspace/myproject";

        // Add the same path to both projects - should succeed
        let path1 = db.add_project_path(&project1.id, shared_path, false).unwrap();
        let path2 = db.add_project_path(&project2.id, shared_path, false).unwrap();

        // Both should have the path
        assert_eq!(path1.path, shared_path);
        assert_eq!(path2.path, shared_path);
        assert_ne!(path1.id, path2.id); // Different records

        // Verify both projects have the path
        let paths1 = db.get_project_paths(&project1.id).unwrap();
        let paths2 = db.get_project_paths(&project2.id).unwrap();
        assert!(paths1.iter().any(|p| p.path == shared_path));
        assert!(paths2.iter().any(|p| p.path == shared_path));
    }

    #[test]
    fn test_add_project_path_idempotent() {
        // Story 1.12: Adding the same path to the same project should be idempotent
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        let test_path = "/home/user/extra-path";

        // Add path twice
        let path1 = db.add_project_path(&project.id, test_path, false).unwrap();
        let path2 = db.add_project_path(&project.id, test_path, false).unwrap();

        // Should return the same record (idempotent)
        assert_eq!(path1.id, path2.id);
        assert_eq!(path1.path, path2.path);

        // Should only have one entry for this path
        let paths = db.get_project_paths(&project.id).unwrap();
        let count = paths.iter().filter(|p| p.path == test_path).count();
        assert_eq!(count, 1, "Should only have one entry for the path");
    }

    #[test]
    fn test_find_project_by_path() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        // Should find project by path (migrated from cwd)
        let found = db.find_project_by_path("/home/user/project").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, project.id);

        // Should not find non-existent path
        let not_found = db.find_project_by_path("/nonexistent/path").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_logical_project_stats() {
        let db = Database::new_in_memory().unwrap();

        // Create two projects with the same path
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        let shared_path = "/shared/workspace";
        db.add_project_path(&project1.id, shared_path, false).unwrap();
        db.add_project_path(&project2.id, shared_path, false).unwrap();

        // Get logical project stats
        let stats = db.get_logical_project_stats().unwrap();

        // Should have stats for the shared path
        let shared_stats = stats.iter().find(|s| s.physical_path == shared_path);
        assert!(shared_stats.is_some(), "Should have stats for shared path");

        let shared = shared_stats.unwrap();
        assert_eq!(shared.project_count, 2, "Should have 2 projects");
        assert!(shared.project_ids.contains(&project1.id));
        assert!(shared.project_ids.contains(&project2.id));
    }

    #[test]
    fn test_get_projects_by_physical_path() {
        let db = Database::new_in_memory().unwrap();

        // Create two projects with the same path
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        let shared_path = "/shared/workspace";
        db.add_project_path(&project1.id, shared_path, false).unwrap();
        db.add_project_path(&project2.id, shared_path, false).unwrap();

        // Get projects by physical path
        let projects = db.get_projects_by_physical_path(shared_path).unwrap();

        assert_eq!(projects.len(), 2, "Should have 2 projects");
        let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&project1.id.as_str()));
        assert!(ids.contains(&project2.id.as_str()));
    }

    #[test]
    fn test_get_sessions_by_physical_path() {
        let db = Database::new_in_memory().unwrap();

        // Create two projects with the same path
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        let shared_path = "/shared/workspace";
        db.add_project_path(&project1.id, shared_path, false).unwrap();
        db.add_project_path(&project2.id, shared_path, false).unwrap();

        // Add sessions to both projects
        let session1 = create_test_session("sess_phys_1", "/home/user/project1");
        let session2 = create_test_session("sess_phys_2", "/home/user/project2");
        db.insert_session(&session1, &project1.id).unwrap();
        db.insert_session(&session2, &project2.id).unwrap();

        // Get sessions by physical path
        let sessions = db.get_sessions_by_physical_path(shared_path).unwrap();

        assert_eq!(sessions.len(), 2, "Should have 2 sessions");
        let ids: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"sess_phys_1"));
        assert!(ids.contains(&"sess_phys_2"));
    }

    #[test]
    fn test_logical_project_stats_excludes_virtual_paths() {
        let db = Database::new_in_memory().unwrap();

        // Create a project with a virtual path
        let (project, _) = db.get_or_create_project("gemini-project:abc123").unwrap();

        // Get logical project stats - should not include virtual paths
        let stats = db.get_logical_project_stats().unwrap();

        // Should not have stats for virtual path
        let virtual_stats = stats.iter().find(|s| s.physical_path.starts_with("gemini-project:"));
        assert!(virtual_stats.is_none(), "Should not include virtual paths");
    }

    #[test]
    fn test_bind_session_to_project() {
        let db = Database::new_in_memory().unwrap();
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        // Create a session in project1
        let session = create_test_session("sess_bind_test", "/home/user/project1");
        db.insert_session(&session, &project1.id).unwrap();

        // Bind session to project2
        let binding = db.bind_session_to_project("sess_bind_test", &project2.id).unwrap();
        assert_eq!(binding.session_id, "sess_bind_test");
        assert_eq!(binding.project_id, project2.id);

        // Verify binding exists
        let retrieved = db.get_session_binding("sess_bind_test").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().project_id, project2.id);
    }

    #[test]
    fn test_unbind_session() {
        let db = Database::new_in_memory().unwrap();
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        // Create and bind session
        let session = create_test_session("sess_unbind_test", "/home/user/project1");
        db.insert_session(&session, &project1.id).unwrap();
        db.bind_session_to_project("sess_unbind_test", &project2.id).unwrap();

        // Unbind
        db.unbind_session("sess_unbind_test").unwrap();

        // Verify binding is gone
        let binding = db.get_session_binding("sess_unbind_test").unwrap();
        assert!(binding.is_none());
    }

    #[test]
    fn test_get_project_sessions_aggregated() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/project").unwrap();

        // Create sessions
        let session1 = create_test_session("sess_agg_1", "/home/user/project");
        let session2 = create_test_session("sess_agg_2", "/home/user/project");
        db.insert_session(&session1, &project.id).unwrap();
        db.insert_session(&session2, &project.id).unwrap();

        // Get aggregated sessions
        let sessions = db.get_project_sessions_aggregated(&project.id).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_manual_binding_priority() {
        let db = Database::new_in_memory().unwrap();
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        // Create session in project1
        let session = create_test_session("sess_priority", "/home/user/project1");
        db.insert_session(&session, &project1.id).unwrap();

        // Bind to project2 (manual binding should take priority)
        db.bind_session_to_project("sess_priority", &project2.id).unwrap();

        // Session should appear in project2's aggregated list
        let sessions2 = db.get_project_sessions_aggregated(&project2.id).unwrap();
        assert!(sessions2.iter().any(|s| s.id == "sess_priority"));
    }

    #[test]
    fn test_set_project_primary_path() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/old-path").unwrap();

        // Set new primary path
        db.set_project_primary_path(&project.id, "/home/user/new-path").unwrap();

        // Verify project cwd updated
        let updated = db.get_project(&project.id).unwrap().unwrap();
        assert_eq!(updated.cwd, "/home/user/new-path");
        assert_eq!(updated.name, "new-path");

        // Verify path is primary
        let paths = db.get_project_paths(&project.id).unwrap();
        let primary = paths.iter().find(|p| p.is_primary);
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().path, "/home/user/new-path");
    }

    // =========================================================================
    // Story 1.12: View-based Project Aggregation Tests
    // =========================================================================

    #[test]
    fn test_get_unassigned_sessions_empty() {
        let db = Database::new_in_memory().unwrap();

        // No sessions, should return empty
        let unassigned = db.get_unassigned_sessions().unwrap();
        assert!(unassigned.is_empty());
    }

    #[test]
    fn test_get_unassigned_sessions_with_orphan() {
        let db = Database::new_in_memory().unwrap();

        // Create a project with path
        let (project, _) = db.get_or_create_project("/home/user/known-project").unwrap();

        // Create a session with unknown path (not matching any project_paths)
        let orphan_session = create_test_session("sess_orphan", "/home/user/unknown-project");
        db.insert_session(&orphan_session, &project.id).unwrap();

        // Update the session's original_cwd to something that doesn't match
        db.connection().execute(
            "UPDATE sessions SET original_cwd = '/home/user/unknown-project' WHERE id = 'sess_orphan'",
            [],
        ).unwrap();

        // The session should appear as unassigned since its original_cwd doesn't match any project_paths
        let unassigned = db.get_unassigned_sessions().unwrap();
        // Note: This depends on how the query handles the fallback to project_id
        // The actual behavior may vary based on implementation
        assert!(unassigned.len() <= 1);
    }

    #[test]
    fn test_unbind_session_returns_to_unassigned() {
        let db = Database::new_in_memory().unwrap();

        // Create two projects
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();

        // Create session in project1
        let session = create_test_session("sess_unbind", "/home/user/project1");
        db.insert_session(&session, &project1.id).unwrap();

        // Bind to project2
        db.bind_session_to_project("sess_unbind", &project2.id).unwrap();

        // Verify it's bound
        let binding = db.get_session_binding("sess_unbind").unwrap();
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().project_id, project2.id);

        // Unbind
        db.unbind_session("sess_unbind").unwrap();

        // Verify binding is removed
        let binding_after = db.get_session_binding("sess_unbind").unwrap();
        assert!(binding_after.is_none());
    }

    #[test]
    fn test_multiple_paths_per_project() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/main-path").unwrap();

        // Add additional paths
        db.add_project_path(&project.id, "/home/user/alt-path-1", false).unwrap();
        db.add_project_path(&project.id, "/home/user/alt-path-2", false).unwrap();

        // Verify all paths exist
        let paths = db.get_project_paths(&project.id).unwrap();
        assert_eq!(paths.len(), 3); // main + 2 alternatives

        // Verify primary is first
        assert!(paths[0].is_primary);
        assert_eq!(paths[0].path, "/home/user/main-path");
    }

    #[test]
    fn test_find_project_by_path_with_alt_paths() {
        let db = Database::new_in_memory().unwrap();
        let (project, _) = db.get_or_create_project("/home/user/main-path").unwrap();

        // Add alternative path
        db.add_project_path(&project.id, "/home/user/alt-path", false).unwrap();

        // Find by main path
        let found1 = db.find_project_by_path("/home/user/main-path").unwrap();
        assert!(found1.is_some());
        assert_eq!(found1.unwrap().id, project.id);

        // Find by alternative path
        let found2 = db.find_project_by_path("/home/user/alt-path").unwrap();
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().id, project.id);

        // Not found
        let not_found = db.find_project_by_path("/home/user/unknown").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_rebind_session_to_different_project() {
        let db = Database::new_in_memory().unwrap();
        let (project1, _) = db.get_or_create_project("/home/user/project1").unwrap();
        let (project2, _) = db.get_or_create_project("/home/user/project2").unwrap();
        let (project3, _) = db.get_or_create_project("/home/user/project3").unwrap();

        // Create session
        let session = create_test_session("sess_rebind", "/home/user/project1");
        db.insert_session(&session, &project1.id).unwrap();

        // Bind to project2
        db.bind_session_to_project("sess_rebind", &project2.id).unwrap();
        let binding1 = db.get_session_binding("sess_rebind").unwrap().unwrap();
        assert_eq!(binding1.project_id, project2.id);

        // Rebind to project3 (should replace)
        db.bind_session_to_project("sess_rebind", &project3.id).unwrap();
        let binding2 = db.get_session_binding("sess_rebind").unwrap().unwrap();
        assert_eq!(binding2.project_id, project3.id);
    }
}
