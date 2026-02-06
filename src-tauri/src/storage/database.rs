//! SQLite database connection management
//!
//! Provides database initialization and connection management for Mantra.

use std::path::Path;

use rusqlite::Connection;

use super::error::StorageError;
use super::migrations;

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

        // Enable WAL mode for better concurrent read/write support
        // This allows readers to see committed changes immediately
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        // Enable foreign key support
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Execute schema migration
        conn.execute_batch(include_str!("schema.sql"))?;

        // Run migrations for existing databases
        migrations::run_all(&conn)?;

        Ok(Self { conn })
    }

    /// Open an existing database for queries (lightweight version)
    ///
    /// This is a lightweight connection method that does NOT run schema or migrations.
    /// Use this for background services that only need to query existing data.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    /// A new Database instance for querying
    ///
    /// # Note
    /// This method assumes the database already exists and has the correct schema.
    /// Using it on an uninitialized database may cause query errors.
    pub fn open_for_query(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent read/write support
        // WAL mode allows readers to see committed changes immediately
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        // Enable foreign key support for query consistency
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        Ok(Self { conn })
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(include_str!("schema.sql"))?;
        migrations::run_all(&conn)?;
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
mod tests;
