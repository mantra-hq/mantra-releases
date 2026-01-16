//! Analytics calculation logic
//!
//! Story 2.34: Functions for computing session metrics and project analytics.

use std::collections::{HashMap, HashSet};
use chrono::{NaiveDate, TimeZone, Utc};

use crate::models::{ContentBlock, MantraSession, StandardTool, normalize_tool};

use super::{
    ActivityDataPoint, ProjectAnalytics, SessionMetrics, SessionStatsView,
    TimeRange, ToolCallDetail,
};

/// Extracts the standard tool type name from a ContentBlock
fn get_tool_type_name(block: &ContentBlock) -> Option<String> {
    match block {
        ContentBlock::ToolUse { standard_tool, name, input, .. } => {
            // Prefer standard_tool if available
            if let Some(tool) = standard_tool {
                let json = serde_json::to_value(tool).ok()?;
                json.get("type").and_then(|v| v.as_str()).map(|s| s.to_string())
            } else {
                // Fall back to normalizing the name
                let normalized = normalize_tool(name, input);
                let json = serde_json::to_value(&normalized).ok()?;
                json.get("type").and_then(|v| v.as_str()).map(|s| s.to_string())
            }
        }
        ContentBlock::ToolResult { .. } => None, // Don't count results as calls
        _ => None,
    }
}

/// Checks if a tool result indicates an error
fn is_tool_error(block: &ContentBlock) -> bool {
    match block {
        ContentBlock::ToolResult { is_error, .. } => *is_error,
        _ => false,
    }
}

/// Calculates session metrics from a MantraSession
///
/// This function analyzes a session to extract:
/// - Duration based on first and last message timestamps
/// - Message count
/// - Tool call count and error count
/// - Tool type distribution
///
/// # Arguments
/// * `session` - The session to analyze
///
/// # Returns
/// Computed SessionMetrics for the session
pub fn calculate_session_metrics(session: &MantraSession) -> SessionMetrics {
    let mut metrics = SessionMetrics {
        session_id: session.id.clone(),
        tool_type: session.source.clone(),
        ..Default::default()
    };

    // Calculate time range from message timestamps
    let mut timestamps: Vec<i64> = Vec::new();

    for msg in &session.messages {
        if let Some(ts) = &msg.timestamp {
            timestamps.push(ts.timestamp());
        }
    }

    // Also consider session created_at and updated_at as fallback
    let session_start = session.created_at.timestamp();
    let session_end = session.updated_at.timestamp();

    if timestamps.is_empty() {
        metrics.start_time = session_start;
        metrics.end_time = session_end;
    } else {
        metrics.start_time = *timestamps.iter().min().unwrap_or(&session_start);
        metrics.end_time = *timestamps.iter().max().unwrap_or(&session_end);
    }

    metrics.duration_seconds = (metrics.end_time - metrics.start_time).max(0);

    // Count messages
    metrics.message_count = session.messages.len() as u32;

    // Analyze tool calls
    let mut tool_types_used: HashSet<String> = HashSet::new();
    let mut tool_type_counts: HashMap<String, u32> = HashMap::new();
    let mut tool_call_count: u32 = 0;
    let mut tool_error_count: u32 = 0;

    for msg in &session.messages {
        for block in &msg.content_blocks {
            // Count tool uses
            if let Some(tool_type) = get_tool_type_name(block) {
                tool_call_count += 1;
                tool_types_used.insert(tool_type.clone());
                *tool_type_counts.entry(tool_type).or_insert(0) += 1;
            }

            // Count tool errors
            if is_tool_error(block) {
                tool_error_count += 1;
            }
        }
    }

    metrics.tool_call_count = tool_call_count;
    metrics.tool_error_count = tool_error_count;
    metrics.tool_types_used = tool_types_used.into_iter().collect();
    metrics.tool_type_counts = tool_type_counts;

    metrics
}

/// Calculates project analytics by aggregating session metrics
///
/// # Arguments
/// * `project_id` - The project identifier
/// * `session_metrics` - List of session metrics to aggregate
/// * `time_range` - Time range filter to apply
///
/// # Returns
/// Aggregated ProjectAnalytics
pub fn calculate_project_analytics(
    project_id: &str,
    session_metrics: &[SessionMetrics],
    time_range: TimeRange,
) -> ProjectAnalytics {
    let mut analytics = ProjectAnalytics {
        project_id: project_id.to_string(),
        time_range,
        ..Default::default()
    };

    // Filter sessions by time range
    let now = Utc::now().timestamp();
    let cutoff = time_range.to_seconds().map(|secs| now - secs);

    let filtered_metrics: Vec<&SessionMetrics> = session_metrics
        .iter()
        .filter(|m| {
            match cutoff {
                Some(cutoff_time) => m.end_time >= cutoff_time,
                None => true, // All time, no filter
            }
        })
        .collect();

    if filtered_metrics.is_empty() {
        return analytics;
    }

    // Aggregate session statistics
    analytics.total_sessions = filtered_metrics.len() as u32;

    let mut total_duration: i64 = 0;
    let mut total_messages: u32 = 0;
    let mut total_tool_calls: u32 = 0;
    let mut total_tool_errors: u32 = 0;
    let mut tool_distribution: HashMap<String, u32> = HashMap::new();
    let mut tool_types_distribution: HashMap<String, u32> = HashMap::new();
    let mut active_dates: HashSet<String> = HashSet::new();
    let mut daily_stats: HashMap<String, (u32, u32, i64)> = HashMap::new(); // date -> (sessions, tool_calls, duration)

    for metrics in &filtered_metrics {
        // Duration
        total_duration += metrics.duration_seconds;

        // Messages
        total_messages += metrics.message_count;

        // Tool calls and errors
        total_tool_calls += metrics.tool_call_count;
        total_tool_errors += metrics.tool_error_count;

        // Source/tool distribution (claude, gemini, cursor, codex)
        *tool_distribution.entry(metrics.tool_type.clone()).or_insert(0) += 1;

        // Tool type distribution (file_read, shell_exec, etc.)
        for (tool_type, count) in &metrics.tool_type_counts {
            *tool_types_distribution.entry(tool_type.clone()).or_insert(0) += count;
        }

        // Active days calculation
        let date_str = timestamp_to_date(metrics.start_time);
        active_dates.insert(date_str.clone());

        // Daily stats for activity trend
        let entry = daily_stats.entry(date_str).or_insert((0, 0, 0));
        entry.0 += 1; // session count
        entry.1 += metrics.tool_call_count; // tool calls
        entry.2 += metrics.duration_seconds; // duration
    }

    analytics.total_duration_seconds = total_duration;
    analytics.avg_duration_seconds = if analytics.total_sessions > 0 {
        total_duration / analytics.total_sessions as i64
    } else {
        0
    };
    analytics.active_days = active_dates.len() as u32;
    analytics.tool_distribution = tool_distribution;
    analytics.total_tool_calls = total_tool_calls;
    analytics.total_tool_errors = total_tool_errors;
    analytics.tool_error_rate = if total_tool_calls > 0 {
        total_tool_errors as f64 / total_tool_calls as f64
    } else {
        0.0
    };
    analytics.tool_types_distribution = tool_types_distribution;
    analytics.total_messages = total_messages;

    // Build activity trend (sorted by date)
    let mut dates: Vec<String> = daily_stats.keys().cloned().collect();
    dates.sort();
    analytics.activity_trend = dates
        .into_iter()
        .map(|date| {
            let (session_count, tool_call_count, duration_seconds) = daily_stats[&date];
            ActivityDataPoint {
                date,
                session_count,
                tool_call_count,
                duration_seconds,
            }
        })
        .collect();

    analytics
}

/// Creates a SessionStatsView with detailed tool call information
///
/// # Arguments
/// * `session` - The session to analyze
///
/// # Returns
/// SessionStatsView with metrics and tool call timeline
pub fn create_session_stats_view(session: &MantraSession) -> SessionStatsView {
    let metrics = calculate_session_metrics(session);

    let mut tool_call_timeline: Vec<ToolCallDetail> = Vec::new();
    let mut tool_distribution: HashMap<String, u32> = HashMap::new();

    // Track tool use IDs to match with results
    let mut tool_use_map: HashMap<String, (String, i64)> = HashMap::new(); // id -> (type, timestamp)

    for msg in &session.messages {
        let msg_timestamp = msg.timestamp.map(|ts| ts.timestamp()).unwrap_or(0);

        for block in &msg.content_blocks {
            match block {
                ContentBlock::ToolUse { id, standard_tool, name, input, .. } => {
                    // Determine tool type
                    let tool_type = if let Some(tool) = standard_tool {
                        get_standard_tool_type_name(tool)
                    } else {
                        let normalized = normalize_tool(name, input);
                        get_standard_tool_type_name(&normalized)
                    };

                    // Get description from tool parameters
                    let description = if let Some(tool) = standard_tool {
                        get_tool_description(tool)
                    } else {
                        let normalized = normalize_tool(name, input);
                        get_tool_description(&normalized)
                    };

                    tool_call_timeline.push(ToolCallDetail {
                        tool_type: tool_type.clone(),
                        timestamp: msg_timestamp,
                        is_error: false, // Will be updated when we see the result
                        description,
                    });

                    *tool_distribution.entry(tool_type.clone()).or_insert(0) += 1;

                    // Store for matching with result
                    tool_use_map.insert(id.clone(), (tool_type, msg_timestamp));
                }
                ContentBlock::ToolResult { tool_use_id, is_error, .. } => {
                    // Update the corresponding tool call's error status
                    if let Some((tool_type, ts)) = tool_use_map.get(tool_use_id) {
                        if let Some(detail) = tool_call_timeline.iter_mut().find(|d| {
                            d.tool_type == *tool_type && d.timestamp == *ts
                        }) {
                            detail.is_error = *is_error;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    SessionStatsView {
        metrics,
        tool_call_timeline,
        tool_distribution,
    }
}

/// Gets the type name from a StandardTool
fn get_standard_tool_type_name(tool: &StandardTool) -> String {
    let json = serde_json::to_value(tool).unwrap_or_default();
    json.get("type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Extracts a description from a StandardTool for display
fn get_tool_description(tool: &StandardTool) -> Option<String> {
    match tool {
        StandardTool::FileRead { path, .. } => Some(path.clone()),
        StandardTool::FileWrite { path, .. } => Some(path.clone()),
        StandardTool::FileEdit { path, .. } => Some(path.clone()),
        StandardTool::FileDelete { path } => Some(path.clone()),
        StandardTool::ShellExec { command, .. } => {
            // Truncate long commands (char-boundary safe)
            let truncated = if command.chars().count() > 50 {
                let end: String = command.chars().take(47).collect();
                format!("{}...", end)
            } else {
                command.clone()
            };
            Some(truncated)
        }
        StandardTool::FileSearch { pattern, .. } => Some(pattern.clone()),
        StandardTool::ContentSearch { pattern, .. } => Some(pattern.clone()),
        StandardTool::WebFetch { url, .. } => Some(url.clone()),
        StandardTool::WebSearch { query } => Some(query.clone()),
        StandardTool::KnowledgeQuery { question, .. } => {
            // Truncate long questions (char-boundary safe)
            let truncated = if question.chars().count() > 50 {
                let end: String = question.chars().take(47).collect();
                format!("{}...", end)
            } else {
                question.clone()
            };
            Some(truncated)
        }
        _ => None,
    }
}

/// Converts a Unix timestamp to a date string (YYYY-MM-DD)
fn timestamp_to_date(timestamp: i64) -> String {
    let dt = Utc.timestamp_opt(timestamp, 0).single();
    match dt {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => "unknown".to_string(),
    }
}

/// Parses a date string (YYYY-MM-DD) to a NaiveDate
#[allow(dead_code)]
fn parse_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}
