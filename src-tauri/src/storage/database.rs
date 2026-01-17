//! SQLite database connection management
//!
//! Provides database initialization and connection management for Mantra.

use std::path::Path;

use rusqlite::Connection;

use super::error::StorageError;

/// Database wrapper for SQLite connection management
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize schema
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    /// A new Database instance with initialized schema
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;

        // Enable foreign key support
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Execute schema migration
        conn.execute_batch(include_str!("schema.sql"))?;

        // Run migrations for existing databases
        Self::run_migrations(&conn)?;

        Ok(Self { conn })
    }

    /// Run database migrations for schema updates
    fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
        // Migration: Add git_repo_path and has_git_repo columns (Story 2.11)
        // SQLite ignores ALTER TABLE if column already exists, but we check to avoid errors
        let has_git_repo_path: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'git_repo_path'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_git_repo_path {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN git_repo_path TEXT;
                 ALTER TABLE projects ADD COLUMN has_git_repo INTEGER NOT NULL DEFAULT 0;",
            )?;
        }

        // Migration: Add deleted_at column (Story 2.19, deprecated - kept for backward compatibility)
        // This column is no longer used after removing soft-delete logic
        let has_deleted_at: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'deleted_at'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_deleted_at {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN deleted_at TEXT;",
            )?;
        }

        // Migration: Add is_empty column (Story 2.29)
        let has_is_empty: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'is_empty'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_is_empty {
            // Add column with default value
            conn.execute_batch(
                "ALTER TABLE sessions ADD COLUMN is_empty INTEGER NOT NULL DEFAULT 0;",
            )?;

            // Backfill: Mark sessions as empty if they have no messages
            // Empty session = no user messages AND no assistant messages (message_count = 0)
            conn.execute_batch(
                "UPDATE sessions SET is_empty = 1 WHERE message_count = 0;",
            )?;
        }

        // Migration: Add git_remote_url column (Story 1.9)
        let has_git_remote_url: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'git_remote_url'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_git_remote_url {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN git_remote_url TEXT;",
            )?;
            // Create index for git_remote_url
            conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_projects_git_remote_url ON projects(git_remote_url);",
            )?;
        }

        // Migration: Add is_empty column to projects table (Story 2.29 V2)
        let has_projects_is_empty: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name = 'is_empty'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !has_projects_is_empty {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN is_empty INTEGER NOT NULL DEFAULT 0;",
            )?;

            // Backfill: Mark projects as empty if all their sessions are empty
            // A project is empty if it has no sessions OR all sessions have is_empty = 1
            conn.execute_batch(
                "UPDATE projects SET is_empty = 1 WHERE id IN (
                    SELECT p.id FROM projects p
                    LEFT JOIN sessions s ON s.project_id = p.id
                    GROUP BY p.id
                    HAVING COUNT(s.id) = 0 OR COUNT(s.id) = SUM(CASE WHEN s.is_empty = 1 THEN 1 ELSE 0 END)
                );",
            )?;
        }

        // Migration: Add interception_records table (Story 3.7)
        Self::run_interception_records_migration(conn)?;

        Ok(())
    }

    /// Migration for interception_records table (Story 3.7)
    fn run_interception_records_migration(conn: &Connection) -> Result<(), StorageError> {
        // Check if table already exists
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='interception_records'",
                [],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if !table_exists {
            conn.execute_batch(
                r#"
                -- 拦截记录表 (Story 3.7)
                CREATE TABLE IF NOT EXISTS interception_records (
                    id TEXT PRIMARY KEY,
                    timestamp TEXT NOT NULL,
                    source_type TEXT NOT NULL,          -- 'pre_upload' | 'claude_code_hook' | 'external_hook'
                    source_context TEXT,                -- JSON: session_id, tool_name 等
                    matches TEXT NOT NULL,              -- JSON: ScanMatch[]
                    user_action TEXT NOT NULL,          -- 'redacted' | 'ignored' | 'cancelled' | 'rule_disabled'
                    original_text_hash TEXT,
                    project_name TEXT,
                    created_at TEXT DEFAULT (datetime('now'))
                );

                -- 时间戳索引 (按时间倒序查询)
                CREATE INDEX IF NOT EXISTS idx_records_timestamp ON interception_records(timestamp DESC);
                -- 来源类型索引 (按来源筛选)
                CREATE INDEX IF NOT EXISTS idx_records_source ON interception_records(source_type);
                -- 项目名索引 (按项目筛选)
                CREATE INDEX IF NOT EXISTS idx_records_project ON interception_records(project_name);
                "#,
            )?;
        }

        Ok(())
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(include_str!("schema.sql"))?;
        Self::run_migrations(&conn)?;
        Ok(Self { conn })
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = Database::new(&db_path);
        assert!(db.is_ok(), "Database creation failed: {:?}", db.err());

        // Verify database file exists
        assert!(db_path.exists());
    }

    #[test]
    fn test_in_memory_database() {
        let db = Database::new_in_memory();
        assert!(db.is_ok(), "In-memory database creation failed: {:?}", db.err());
    }

    #[test]
    fn test_schema_initialization() {
        let db = Database::new_in_memory().unwrap();

        // Verify projects table exists
        let result = db.connection().execute(
            "SELECT 1 FROM projects LIMIT 1",
            [],
        );
        // Table exists but is empty, so query should succeed
        assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

        // Verify sessions table exists
        let result = db.connection().execute(
            "SELECT 1 FROM sessions LIMIT 1",
            [],
        );
        assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let db = Database::new_in_memory().unwrap();

        let fk_enabled: i32 = db
            .connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();

        assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
    }

    #[test]
    fn test_interception_records_table_exists() {
        let db = Database::new_in_memory().unwrap();

        // Verify interception_records table exists
        let result = db.connection().execute(
            "SELECT 1 FROM interception_records LIMIT 1",
            [],
        );
        // Table exists but is empty, so query should succeed
        assert!(result.is_ok() || matches!(result, Err(rusqlite::Error::QueryReturnedNoRows)));

        // Verify columns exist
        let columns: Vec<String> = db
            .connection()
            .prepare("PRAGMA table_info(interception_records)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"timestamp".to_string()));
        assert!(columns.contains(&"source_type".to_string()));
        assert!(columns.contains(&"source_context".to_string()));
        assert!(columns.contains(&"matches".to_string()));
        assert!(columns.contains(&"user_action".to_string()));
        assert!(columns.contains(&"original_text_hash".to_string()));
        assert!(columns.contains(&"project_name".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
    }

    #[test]
    fn test_interception_records_indexes_exist() {
        let db = Database::new_in_memory().unwrap();

        // Verify indexes exist
        let indexes: Vec<String> = db
            .connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='interception_records'")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_records_timestamp".to_string()));
        assert!(indexes.contains(&"idx_records_source".to_string()));
        assert!(indexes.contains(&"idx_records_project".to_string()));
    }
}
