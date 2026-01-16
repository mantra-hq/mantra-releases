//! Unit tests for analytics types
//!
//! Story 2.34: Tests for SessionMetrics, ProjectAnalytics, and TimeRange.

use super::*;

// ===== TimeRange Tests =====

#[test]
fn test_time_range_default() {
    let range = TimeRange::default();
    assert_eq!(range, TimeRange::Days30);
}

#[test]
fn test_time_range_to_seconds() {
    assert_eq!(TimeRange::Days7.to_seconds(), Some(7 * 24 * 60 * 60));
    assert_eq!(TimeRange::Days30.to_seconds(), Some(30 * 24 * 60 * 60));
    assert_eq!(TimeRange::All.to_seconds(), None);
}

#[test]
fn test_time_range_serialization() {
    let days7_json = serde_json::to_string(&TimeRange::Days7).unwrap();
    assert_eq!(days7_json, r#""days7""#);

    let days30_json = serde_json::to_string(&TimeRange::Days30).unwrap();
    assert_eq!(days30_json, r#""days30""#);

    let all_json = serde_json::to_string(&TimeRange::All).unwrap();
    assert_eq!(all_json, r#""all""#);
}

#[test]
fn test_time_range_deserialization() {
    let days7: TimeRange = serde_json::from_str(r#""days7""#).unwrap();
    assert_eq!(days7, TimeRange::Days7);

    let days30: TimeRange = serde_json::from_str(r#""days30""#).unwrap();
    assert_eq!(days30, TimeRange::Days30);

    let all: TimeRange = serde_json::from_str(r#""all""#).unwrap();
    assert_eq!(all, TimeRange::All);
}

// ===== SessionMetrics Tests =====

#[test]
fn test_session_metrics_default() {
    let metrics = SessionMetrics::default();
    assert!(metrics.session_id.is_empty());
    assert!(metrics.tool_type.is_empty());
    assert_eq!(metrics.start_time, 0);
    assert_eq!(metrics.end_time, 0);
    assert_eq!(metrics.duration_seconds, 0);
    assert_eq!(metrics.message_count, 0);
    assert_eq!(metrics.tool_call_count, 0);
    assert_eq!(metrics.tool_error_count, 0);
    assert!(metrics.tool_types_used.is_empty());
    assert!(metrics.tool_type_counts.is_empty());
}

#[test]
fn test_session_metrics_serialization() {
    let mut tool_counts = std::collections::HashMap::new();
    tool_counts.insert("file_read".to_string(), 10);
    tool_counts.insert("shell_exec".to_string(), 5);

    let metrics = SessionMetrics {
        session_id: "test-session-123".to_string(),
        tool_type: "claude".to_string(),
        start_time: 1700000000,
        end_time: 1700003600,
        duration_seconds: 3600,
        message_count: 20,
        tool_call_count: 15,
        tool_error_count: 2,
        tool_types_used: vec!["file_read".to_string(), "shell_exec".to_string()],
        tool_type_counts: tool_counts,
    };

    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains(r#""session_id":"test-session-123""#));
    assert!(json.contains(r#""tool_type":"claude""#));
    assert!(json.contains(r#""duration_seconds":3600"#));
    assert!(json.contains(r#""message_count":20"#));
    assert!(json.contains(r#""tool_call_count":15"#));
    assert!(json.contains(r#""tool_error_count":2"#));

    // Roundtrip test
    let deserialized: SessionMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, metrics);
}

#[test]
fn test_session_metrics_partial_deserialization() {
    // Test backward compatibility with minimal JSON
    let json = r#"{
        "session_id": "minimal-session",
        "tool_type": "gemini",
        "start_time": 1700000000,
        "end_time": 1700001000,
        "duration_seconds": 1000,
        "message_count": 5,
        "tool_call_count": 3,
        "tool_error_count": 0,
        "tool_types_used": []
    }"#;

    let metrics: SessionMetrics = serde_json::from_str(json).unwrap();
    assert_eq!(metrics.session_id, "minimal-session");
    assert_eq!(metrics.tool_type, "gemini");
    assert_eq!(metrics.duration_seconds, 1000);
    assert!(metrics.tool_type_counts.is_empty()); // Default empty
}

// ===== ProjectAnalytics Tests =====

#[test]
fn test_project_analytics_default() {
    let analytics = ProjectAnalytics::default();
    assert!(analytics.project_id.is_empty());
    assert_eq!(analytics.time_range, TimeRange::Days30);
    assert_eq!(analytics.total_sessions, 0);
    assert_eq!(analytics.total_duration_seconds, 0);
    assert_eq!(analytics.avg_duration_seconds, 0);
    assert_eq!(analytics.active_days, 0);
    assert!(analytics.tool_distribution.is_empty());
    assert_eq!(analytics.total_tool_calls, 0);
    assert_eq!(analytics.total_tool_errors, 0);
    assert_eq!(analytics.tool_error_rate, 0.0);
    assert!(analytics.tool_types_distribution.is_empty());
    assert!(analytics.activity_trend.is_empty());
    assert_eq!(analytics.total_messages, 0);
}

#[test]
fn test_project_analytics_serialization() {
    let mut tool_dist = std::collections::HashMap::new();
    tool_dist.insert("claude".to_string(), 10);
    tool_dist.insert("gemini".to_string(), 5);

    let mut tool_types_dist = std::collections::HashMap::new();
    tool_types_dist.insert("file_read".to_string(), 50);
    tool_types_dist.insert("shell_exec".to_string(), 30);

    let analytics = ProjectAnalytics {
        project_id: "project-123".to_string(),
        time_range: TimeRange::Days7,
        total_sessions: 15,
        total_duration_seconds: 36000,
        avg_duration_seconds: 2400,
        active_days: 5,
        tool_distribution: tool_dist,
        total_tool_calls: 80,
        total_tool_errors: 4,
        tool_error_rate: 0.05,
        tool_types_distribution: tool_types_dist,
        activity_trend: vec![
            ActivityDataPoint {
                date: "2024-01-15".to_string(),
                session_count: 3,
                tool_call_count: 20,
                duration_seconds: 7200,
            },
        ],
        total_messages: 150,
    };

    let json = serde_json::to_string(&analytics).unwrap();
    assert!(json.contains(r#""project_id":"project-123""#));
    assert!(json.contains(r#""time_range":"days7""#));
    assert!(json.contains(r#""total_sessions":15"#));
    assert!(json.contains(r#""active_days":5"#));
    assert!(json.contains(r#""tool_error_rate":0.05"#));
    assert!(json.contains(r#""total_messages":150"#));

    // Roundtrip test
    let deserialized: ProjectAnalytics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, analytics);
}

// ===== ActivityDataPoint Tests =====

#[test]
fn test_activity_data_point_serialization() {
    let point = ActivityDataPoint {
        date: "2024-01-15".to_string(),
        session_count: 5,
        tool_call_count: 30,
        duration_seconds: 9000,
    };

    let json = serde_json::to_string(&point).unwrap();
    assert!(json.contains(r#""date":"2024-01-15""#));
    assert!(json.contains(r#""session_count":5"#));
    assert!(json.contains(r#""tool_call_count":30"#));
    assert!(json.contains(r#""duration_seconds":9000"#));

    let deserialized: ActivityDataPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, point);
}

// ===== ToolCallDetail Tests =====

#[test]
fn test_tool_call_detail_serialization() {
    let detail = ToolCallDetail {
        tool_type: "file_read".to_string(),
        timestamp: 1700000100,
        is_error: false,
        description: Some("/src/main.rs".to_string()),
    };

    let json = serde_json::to_string(&detail).unwrap();
    assert!(json.contains(r#""tool_type":"file_read""#));
    assert!(json.contains(r#""timestamp":1700000100"#));
    assert!(json.contains(r#""is_error":false"#));
    assert!(json.contains(r#""description":"/src/main.rs""#));

    let deserialized: ToolCallDetail = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, detail);
}

#[test]
fn test_tool_call_detail_skip_none_description() {
    let detail = ToolCallDetail {
        tool_type: "shell_exec".to_string(),
        timestamp: 1700000200,
        is_error: true,
        description: None,
    };

    let json = serde_json::to_string(&detail).unwrap();
    assert!(json.contains(r#""tool_type":"shell_exec""#));
    assert!(json.contains(r#""is_error":true"#));
    assert!(!json.contains("description"));
}

// ===== SessionStatsView Tests =====

#[test]
fn test_session_stats_view_serialization() {
    let mut tool_dist = std::collections::HashMap::new();
    tool_dist.insert("file_read".to_string(), 5);

    let view = SessionStatsView {
        metrics: SessionMetrics {
            session_id: "session-456".to_string(),
            tool_type: "cursor".to_string(),
            start_time: 1700000000,
            end_time: 1700001800,
            duration_seconds: 1800,
            message_count: 10,
            tool_call_count: 5,
            tool_error_count: 0,
            tool_types_used: vec!["file_read".to_string()],
            tool_type_counts: tool_dist.clone(),
        },
        tool_call_timeline: vec![
            ToolCallDetail {
                tool_type: "file_read".to_string(),
                timestamp: 1700000100,
                is_error: false,
                description: Some("/src/lib.rs".to_string()),
            },
        ],
        tool_distribution: tool_dist,
    };

    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains(r#""session_id":"session-456""#));
    assert!(json.contains("tool_call_timeline"));
    assert!(json.contains("tool_distribution"));

    let deserialized: SessionStatsView = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, view);
}
