//! Project scanner for aggregating sessions by working directory
//!
//! Provides the core logic for scanning and importing MantraSession data,
//! grouping sessions by their working directory (cwd) into projects.

use crate::git::detect_git_repo_sync;
use crate::models::{ImportResult, MantraSession};
use crate::storage::{Database, StorageError};

/// Project scanner for importing and aggregating sessions
pub struct ProjectScanner<'a> {
    db: &'a mut Database,
}

impl<'a> ProjectScanner<'a> {
    /// Create a new ProjectScanner with the given database
    pub fn new(db: &'a mut Database) -> Self {
        Self { db }
    }

    /// Scan and import sessions, aggregating by project
    ///
    /// # Arguments
    /// * `sessions` - The sessions to import
    ///
    /// # Returns
    /// Import result with statistics
    pub fn scan_and_import(&mut self, sessions: Vec<MantraSession>) -> Result<ImportResult, StorageError> {
        let result = self.db.import_sessions(&sessions)?;

        // For new projects, detect Git repositories
        if result.new_projects_count > 0 {
            for session in &sessions {
                // Check if this session's project was newly created
                // by checking if the project exists and has no Git info yet
                if let Ok(Some(project)) = self.db.get_project_by_cwd(&session.cwd) {
                    if !project.has_git_repo && project.git_repo_path.is_none() {
                        let git_repo_path = detect_git_repo_sync(&session.cwd);
                        if git_repo_path.is_some() {
                            let _ = self.db.update_project_git_info(&session.cwd, git_repo_path);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Import a single session
    ///
    /// # Arguments
    /// * `session` - The session to import
    ///
    /// # Returns
    /// A tuple of (was_imported, was_new_project)
    pub fn import_session(&self, session: &MantraSession) -> Result<(bool, bool), StorageError> {
        let (imported, is_new_project) = self.db.import_session(session)?;

        // Detect Git repository for new projects
        if is_new_project {
            let git_repo_path = detect_git_repo_sync(&session.cwd);
            self.db.update_project_git_info(&session.cwd, git_repo_path)?;
        }

        Ok((imported, is_new_project))
    }

    /// Check if a session already exists
    ///
    /// # Arguments
    /// * `session_id` - The session ID to check
    pub fn session_exists(&self, session_id: &str) -> Result<bool, StorageError> {
        self.db.session_exists(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sources;

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), sources::CLAUDE.to_string(), cwd.to_string())
    }

    #[test]
    fn test_scan_and_import() {
        let mut db = Database::new_in_memory().unwrap();
        let mut scanner = ProjectScanner::new(&mut db);

        let sessions = vec![
            create_test_session("sess_1", "/home/user/project1"),
            create_test_session("sess_2", "/home/user/project1"),
            create_test_session("sess_3", "/home/user/project2"),
        ];

        let result = scanner.scan_and_import(sessions).unwrap();

        assert_eq!(result.imported_count, 3);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.new_projects_count, 2);
    }

    #[test]
    fn test_import_deduplication() {
        let mut db = Database::new_in_memory().unwrap();
        let scanner = ProjectScanner::new(&mut db);

        let session = create_test_session("sess_1", "/home/user/test");

        // First import
        let (imported, new_project) = scanner.import_session(&session).unwrap();
        assert!(imported);
        assert!(new_project);

        // Second import should be skipped
        let (imported, _) = scanner.import_session(&session).unwrap();
        assert!(!imported);
    }

    #[test]
    fn test_session_exists() {
        let mut db = Database::new_in_memory().unwrap();
        let scanner = ProjectScanner::new(&mut db);

        assert!(!scanner.session_exists("sess_1").unwrap());

        let session = create_test_session("sess_1", "/home/user/test");
        scanner.import_session(&session).unwrap();

        assert!(scanner.session_exists("sess_1").unwrap());
    }

    #[test]
    fn test_cwd_aggregation() {
        let mut db = Database::new_in_memory().unwrap();
        let mut scanner = ProjectScanner::new(&mut db);

        // Import sessions with same cwd
        let sessions = vec![
            create_test_session("sess_1", "/home/user/myproject"),
            create_test_session("sess_2", "/home/user/myproject"),
            create_test_session("sess_3", "/home/user/myproject"),
        ];

        let result = scanner.scan_and_import(sessions).unwrap();

        // Should create only one project
        assert_eq!(result.new_projects_count, 1);
        assert_eq!(result.imported_count, 3);

        // Verify single project with 3 sessions
        drop(scanner); // Release mutable borrow
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "myproject");
        assert_eq!(projects[0].session_count, 3);
    }

    #[test]
    fn test_import_session_detects_git_repo() {
        let mut db = Database::new_in_memory().unwrap();
        let scanner = ProjectScanner::new(&mut db);

        // Use current project directory which has .git
        let cwd = env!("CARGO_MANIFEST_DIR");
        let session = create_test_session("sess_git_1", cwd);

        let (imported, is_new_project) = scanner.import_session(&session).unwrap();
        assert!(imported);
        assert!(is_new_project);

        // Check that Git repo was detected
        drop(scanner);
        let project = db.get_project_by_cwd(cwd).unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert!(project.has_git_repo);
        assert!(project.git_repo_path.is_some());
    }

    #[test]
    fn test_import_session_no_git_repo() {
        let mut db = Database::new_in_memory().unwrap();
        let scanner = ProjectScanner::new(&mut db);

        // Use /tmp which typically doesn't have .git
        let session = create_test_session("sess_no_git_1", "/tmp/no_git_project");

        let (imported, is_new_project) = scanner.import_session(&session).unwrap();
        assert!(imported);
        assert!(is_new_project);

        // Check that no Git repo was detected
        drop(scanner);
        let project = db.get_project_by_cwd("/tmp/no_git_project").unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert!(!project.has_git_repo);
        assert!(project.git_repo_path.is_none());
    }

    #[test]
    fn test_scan_and_import_detects_git_for_new_projects() {
        let mut db = Database::new_in_memory().unwrap();
        let mut scanner = ProjectScanner::new(&mut db);

        let cwd = env!("CARGO_MANIFEST_DIR");
        let sessions = vec![
            create_test_session("sess_batch_1", cwd),
            create_test_session("sess_batch_2", cwd),
        ];

        let result = scanner.scan_and_import(sessions).unwrap();
        assert_eq!(result.new_projects_count, 1);
        assert_eq!(result.imported_count, 2);

        // Verify Git repo was detected
        drop(scanner);
        let project = db.get_project_by_cwd(cwd).unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert!(project.has_git_repo);
    }
}
