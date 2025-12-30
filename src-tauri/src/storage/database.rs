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

        Ok(Self { conn })
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(include_str!("schema.sql"))?;
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
}
