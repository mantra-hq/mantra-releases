//! Analytics Tauri IPC commands
//!
//! Story 2.34: Provides commands for fetching project and session analytics.

use tauri::State;

use crate::analytics::{
    calculator::{calculate_project_analytics, calculate_session_metrics, create_session_stats_view},
    ProjectAnalytics, SessionMetrics, SessionStatsView, TimeRange,
};
use crate::error::AppError;

use super::AppState;

/// Get analytics for a project
///
/// Aggregates session metrics for a project within the specified time range.
///
/// # Arguments
/// * `project_id` - The project identifier
/// * `time_range` - Time range filter ("days7", "days30", "all")
///
/// # Returns
/// Aggregated ProjectAnalytics or error
#[tauri::command]
pub async fn get_project_analytics(
    project_id: String,
    time_range: TimeRange,
    state: State<'_, AppState>,
) -> Result<ProjectAnalytics, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // Get all sessions for the project
    let sessions = db.get_sessions_by_project(&project_id)?;

    // Calculate metrics for each session
    let session_metrics: Vec<SessionMetrics> = sessions
        .iter()
        .map(calculate_session_metrics)
        .collect();

    // Calculate project analytics
    let analytics = calculate_project_analytics(&project_id, &session_metrics, time_range);

    Ok(analytics)
}

/// Get metrics for a specific session
///
/// # Arguments
/// * `session_id` - The session identifier
///
/// # Returns
/// SessionMetrics for the session or error if not found
#[tauri::command]
pub async fn get_session_metrics(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<SessionMetrics, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // Get the session
    let session = db.get_session(&session_id)?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    // Calculate metrics
    let metrics = calculate_session_metrics(&session);

    Ok(metrics)
}

/// Get detailed session statistics view
///
/// Includes tool call timeline and distribution for session-level statistics display.
///
/// # Arguments
/// * `session_id` - The session identifier
///
/// # Returns
/// SessionStatsView with metrics and tool call details, or error if not found
#[tauri::command]
pub async fn get_session_stats_view(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<SessionStatsView, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // Get the session
    let session = db.get_session(&session_id)?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    // Create stats view
    let view = create_session_stats_view(&session);

    Ok(view)
}
