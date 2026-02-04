//! Repository layer for database CRUD operations
//!
//! Provides high-level database operations for projects and sessions.

mod binding;
mod project;
mod search;
mod session;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use super::error::StorageError;
use crate::models::SessionSource;

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

/// Parse session source from string
pub(super) fn parse_session_source(s: &str) -> SessionSource {
    // SessionSource is now a String type alias, just return the string
    s.to_lowercase()
}

#[cfg(test)]
mod tests;
