//! Project scanner for aggregating sessions by working directory
//!
//! Provides the core logic for scanning and importing MantraSession data,
//! grouping sessions by their working directory (cwd) into projects.

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
        self.db.import_sessions(&sessions)
    }

    /// Import a single session
    ///
    /// # Arguments
    /// * `session` - The session to import
    ///
    /// # Returns
    /// A tuple of (was_imported, was_new_project)
    pub fn import_session(&self, session: &MantraSession) -> Result<(bool, bool), StorageError> {
        self.db.import_session(session)
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
    use crate::models::SessionSource;

    fn create_test_session(id: &str, cwd: &str) -> MantraSession {
        MantraSession::new(id.to_string(), SessionSource::Claude, cwd.to_string())
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
}
