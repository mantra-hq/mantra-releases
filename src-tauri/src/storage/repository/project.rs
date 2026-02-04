//! Project CRUD operations
//!
//! Provides project management methods for the Database.

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::StorageError;
use crate::models::{classify_path_type, extract_project_name, normalize_cwd, PathType, Project};
use crate::storage::Database;

impl Database {
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
    pub(super) fn create_project_internal(
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
}
