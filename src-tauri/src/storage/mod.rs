//! Local storage module for Mantra
//!
//! Provides SQLite-based persistence for projects and sessions,
//! following the Local First architecture principle.

mod database;
mod error;
mod gateway;
mod interception;
mod mcp_service;
mod repository;

pub use database::Database;
pub use error::StorageError;
pub use gateway::{GatewayConfigRecord, GatewayConfigUpdate};
pub use repository::{ContentType, LogicalProjectStats, SearchFilters, SearchResult, TimePreset};
