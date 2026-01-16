//! Analytics type definitions
//!
//! Contains data structures for session metrics and project analytics.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Time range filter for project analytics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    /// Last 7 days
    Days7,
    /// Last 30 days (default)
    Days30,
    /// All time (no filter)
    All,
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange::Days30
    }
}

impl TimeRange {
    /// Returns the number of seconds for this time range
    /// Returns None for All (no time limit)
    pub fn to_seconds(&self) -> Option<i64> {
        match self {
            TimeRange::Days7 => Some(7 * 24 * 60 * 60),
            TimeRange::Days30 => Some(30 * 24 * 60 * 60),
            TimeRange::All => None,
        }
    }
}

/// Session-level metrics
///
/// Pre-computed during session import and stored for quick retrieval.
/// Contains efficiency and usage statistics for a single session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionMetrics {
    /// Session identifier
    pub session_id: String,

    /// Tool/source type (claude, gemini, cursor, codex)
    pub tool_type: String,

    /// Session start time (Unix timestamp in seconds)
    pub start_time: i64,

    /// Session end time (Unix timestamp in seconds)
    pub end_time: i64,

    /// Session duration in seconds
    pub duration_seconds: i64,

    /// Total message count (user + assistant)
    pub message_count: u32,

    /// Total tool call count
    pub tool_call_count: u32,

    /// Tool call error count
    pub tool_error_count: u32,

    /// List of tool types used (e.g., ["file_read", "shell_exec", "content_search"])
    pub tool_types_used: Vec<String>,

    /// Tool type distribution (tool_type -> count)
    #[serde(default)]
    pub tool_type_counts: HashMap<String, u32>,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            tool_type: String::new(),
            start_time: 0,
            end_time: 0,
            duration_seconds: 0,
            message_count: 0,
            tool_call_count: 0,
            tool_error_count: 0,
            tool_types_used: Vec::new(),
            tool_type_counts: HashMap::new(),
        }
    }
}

/// Activity data point for trend charts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ActivityDataPoint {
    /// Date string (YYYY-MM-DD)
    pub date: String,

    /// Number of sessions on this day
    pub session_count: u32,

    /// Total tool calls on this day
    pub tool_call_count: u32,

    /// Total duration in seconds on this day
    pub duration_seconds: i64,
}

/// Project-level analytics
///
/// Aggregated from session metrics for a specific project and time range.
/// Used to display project statistics dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectAnalytics {
    /// Project identifier
    pub project_id: String,

    /// Time range used for this analytics calculation
    pub time_range: TimeRange,

    // === Session Statistics ===

    /// Total number of sessions
    pub total_sessions: u32,

    /// Total duration across all sessions (in seconds)
    pub total_duration_seconds: i64,

    /// Average session duration (in seconds)
    pub avg_duration_seconds: i64,

    /// Number of distinct active days
    pub active_days: u32,

    /// Tool/source distribution (claude/gemini/cursor/codex -> count)
    pub tool_distribution: HashMap<String, u32>,

    // === Efficiency Metrics ===

    /// Total tool calls across all sessions
    pub total_tool_calls: u32,

    /// Total tool errors across all sessions
    pub total_tool_errors: u32,

    /// Tool error rate (errors / calls, 0.0 - 1.0)
    pub tool_error_rate: f64,

    /// Tool type distribution (file_read/shell_exec/etc -> count)
    pub tool_types_distribution: HashMap<String, u32>,

    // === Activity Trend ===

    /// Daily activity data for trend charts
    #[serde(default)]
    pub activity_trend: Vec<ActivityDataPoint>,

    // === Message Statistics ===

    /// Total messages across all sessions
    #[serde(default)]
    pub total_messages: u32,
}

impl Default for ProjectAnalytics {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            time_range: TimeRange::default(),
            total_sessions: 0,
            total_duration_seconds: 0,
            avg_duration_seconds: 0,
            active_days: 0,
            tool_distribution: HashMap::new(),
            total_tool_calls: 0,
            total_tool_errors: 0,
            tool_error_rate: 0.0,
            tool_types_distribution: HashMap::new(),
            activity_trend: Vec::new(),
            total_messages: 0,
        }
    }
}

/// Tool call detail for session-level view
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolCallDetail {
    /// Tool type (e.g., "file_read", "shell_exec")
    pub tool_type: String,

    /// Timestamp of the tool call (Unix timestamp in seconds)
    pub timestamp: i64,

    /// Whether the call resulted in an error
    pub is_error: bool,

    /// Brief description or path (for display)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Extended session metrics for session-level statistics view
///
/// Includes tool call timeline and detailed breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SessionStatsView {
    /// Basic session metrics
    pub metrics: SessionMetrics,

    /// Tool call timeline (chronological list of tool calls)
    pub tool_call_timeline: Vec<ToolCallDetail>,

    /// Tool distribution for pie chart (tool_type -> count)
    pub tool_distribution: HashMap<String, u32>,
}
