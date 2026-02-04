//! Path binding and aggregation operations (Story 1.12)
//!
//! Provides project path management and session binding methods.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::{parse_session_source, LogicalProjectStats, StorageError};
use crate::git::detect_git_repo_sync;
use crate::models::{
    check_path_exists, classify_path_type, extract_project_name, normalize_cwd, PathType, Project,
    ProjectPath, SessionBinding, SessionSummary,
};
use crate::storage::Database;

impl Database {
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
    ) -> Result<ProjectPath, StorageError> {
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
            return Ok(ProjectPath {
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

        Ok(ProjectPath {
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
        let rows_affected = self
            .connection()
            .execute("DELETE FROM project_paths WHERE id = ?1", params![path_id])?;

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
    pub fn get_project_paths(&self, project_id: &str) -> Result<Vec<ProjectPath>, StorageError> {
        let mut stmt = self.connection().prepare(
            "SELECT id, project_id, path, is_primary, created_at FROM project_paths WHERE project_id = ?1 ORDER BY is_primary DESC, created_at ASC",
        )?;

        let paths = stmt
            .query_map(params![project_id], |row| {
                let created_at_str: String = row.get(4)?;
                let is_primary_int: i32 = row.get(3)?;

                Ok(ProjectPath {
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

    /// Set a project's primary path (Story 1.12 - AC1)
    ///
    /// Updates the project's cwd and sets the specified path as primary.
    ///
    /// # Arguments
    /// * `project_id` - The project to update
    /// * `path` - The new primary path (will be normalized)
    pub fn set_project_primary_path(
        &self,
        project_id: &str,
        path: &str,
    ) -> Result<(), StorageError> {
        let normalized_path = normalize_cwd(path);
        let now = Utc::now();

        // First, check if path already exists for this project
        let existing_path_id: Option<String> = self
            .connection()
            .query_row(
                "SELECT id FROM project_paths WHERE project_id = ?1 AND path = ?2",
                params![project_id, normalized_path],
                |row| row.get(0),
            )
            .ok();

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
    ) -> Result<SessionBinding, StorageError> {
        let now = Utc::now();

        // Use INSERT OR REPLACE to handle re-binding
        self.connection().execute(
            "INSERT OR REPLACE INTO session_project_bindings (session_id, project_id, bound_at) VALUES (?1, ?2, ?3)",
            params![session_id, project_id, now.to_rfc3339()],
        )?;

        Ok(SessionBinding {
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
    pub fn get_session_binding(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionBinding>, StorageError> {
        let result = self.connection().query_row(
            "SELECT session_id, project_id, bound_at FROM session_project_bindings WHERE session_id = ?1",
            params![session_id],
            |row| {
                let bound_at_str: String = row.get(2)?;
                Ok(SessionBinding {
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
    pub fn get_project_sessions_aggregated(
        &self,
        project_id: &str,
    ) -> Result<Vec<SessionSummary>, StorageError> {
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

    /// Get the project a session belongs to, considering binding priority (Story 1.12 - AC4)
    ///
    /// Priority order:
    /// 1. Manual binding (session_project_bindings)
    /// 2. Path match (project_paths via original_cwd)
    /// 3. Direct project_id (legacy fallback)
    ///
    /// # Arguments
    /// * `session_id` - The session to look up
    pub fn get_session_project_aggregated(
        &self,
        session_id: &str,
    ) -> Result<Option<Project>, StorageError> {
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

    /// Get logical project statistics grouped by physical path (Story 1.12 - AC9)
    ///
    /// Returns aggregated statistics for each unique physical path across all projects.
    /// This enables the view layer to display "logical projects" that combine sessions
    /// from different import sources (Claude, Gemini, Cursor, etc.) that share the same path.
    ///
    /// Story 1.13: Now includes custom names from logical_project_names table.
    pub fn get_logical_project_stats(&self) -> Result<Vec<LogicalProjectStats>, StorageError> {
        // Task 9.1: Remove virtual path exclusion - include ALL paths including virtual ones
        // Story 1.13: LEFT JOIN logical_project_names to get custom names
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
                MAX(p.has_git_repo) as has_git_repo,
                lpn.custom_name
            FROM project_paths pp
            JOIN projects p ON pp.project_id = p.id
            LEFT JOIN logical_project_names lpn ON lpn.physical_path = pp.path
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
                let custom_name: Option<String> = row.get(6)?;

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

                // Story 1.13: Use custom_name if set, otherwise extract from path
                let display_name =
                    custom_name.unwrap_or_else(|| extract_project_name(&physical_path));

                // Task 17: Aggregate has_git_repo from DB + real-time check
                let has_git_repo =
                    has_git_repo_db > 0 || (path_exists && Self::check_git_repo_exists(&physical_path));

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

    /// Check if a path is inside a Git repository (Task 17: AC15)
    /// Bug Fix: Use detect_git_repo_sync to search upward for .git directory
    /// This correctly handles cases where the associated path is a subdirectory of a git repo
    fn check_git_repo_exists(path: &str) -> bool {
        detect_git_repo_sync(path).is_some()
    }

    // ============================================================================
    // Story 1.13: Logical Project Rename Methods
    // ============================================================================

    /// Get the custom name for a logical project (Story 1.13 - AC2)
    ///
    /// # Arguments
    /// * `physical_path` - The physical path of the logical project
    ///
    /// # Returns
    /// The custom name if set, None otherwise
    pub fn get_logical_project_name(
        &self,
        physical_path: &str,
    ) -> Result<Option<String>, StorageError> {
        let normalized_path = normalize_cwd(physical_path);

        let result = self.connection().query_row(
            "SELECT custom_name FROM logical_project_names WHERE physical_path = ?1",
            params![normalized_path],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(name) => Ok(Some(name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// Set or update the custom name for a logical project (Story 1.13 - AC2)
    ///
    /// Uses INSERT OR REPLACE to handle both new and existing names.
    ///
    /// # Arguments
    /// * `physical_path` - The physical path of the logical project
    /// * `custom_name` - The custom name to set
    ///
    /// # Returns
    /// Success or error
    pub fn set_logical_project_name(
        &self,
        physical_path: &str,
        custom_name: &str,
    ) -> Result<(), StorageError> {
        let normalized_path = normalize_cwd(physical_path);
        let now = Utc::now().to_rfc3339();

        // Use INSERT OR REPLACE for upsert behavior
        self.connection().execute(
            r#"
            INSERT INTO logical_project_names (physical_path, custom_name, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?3)
            ON CONFLICT(physical_path) DO UPDATE SET
                custom_name = excluded.custom_name,
                updated_at = excluded.updated_at
            "#,
            params![normalized_path, custom_name, now],
        )?;

        Ok(())
    }

    /// Delete the custom name for a logical project (Story 1.13 - AC4)
    ///
    /// After deletion, the display name will revert to the default extracted from path.
    ///
    /// # Arguments
    /// * `physical_path` - The physical path of the logical project
    ///
    /// # Returns
    /// Success or error (NotFound if no custom name exists)
    pub fn delete_logical_project_name(&self, physical_path: &str) -> Result<(), StorageError> {
        let normalized_path = normalize_cwd(physical_path);

        let rows_affected = self.connection().execute(
            "DELETE FROM logical_project_names WHERE physical_path = ?1",
            params![normalized_path],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!(
                "No custom name found for path: {}",
                physical_path
            )));
        }

        Ok(())
    }
}
