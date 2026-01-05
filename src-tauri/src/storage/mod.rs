//! Local storage module for Mantra
//!
//! Provides SQLite-based persistence for projects and sessions,
//! following the Local First architecture principle.

mod database;
mod error;
mod repository;

pub use database::Database;
pub use error::StorageError;
pub use repository::SearchResult;
