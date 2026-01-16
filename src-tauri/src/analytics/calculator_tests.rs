//! Unit tests for analytics calculator
//!
//! Story 2.34: Tests for calculate_session_metrics and calculate_project_analytics.

use super::calculator::*;
use super::*;
use crate::models::{ContentBlock, MantraSession, Message, Role, StandardTool};
use chrono::{TimeZone, Utc};

// ===== Helper Functions =====

fn create_test_session(id: &str, source: &str, message_count: usize, tool_calls: usize) -> MantraSession {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

    let mut messages = Vec::new();

    // Add user messages
    for i in 0..message_count {
        let role = if i % 2 == 0 { Role::User } else { Role::Assistant };
        let mut content_blocks = vec![ContentBlock::Text {
            text: format!("Message {}", i),
            is_degraded: None,
        }];

        // Add tool calls for assistant messages
        if role == Role::Assistant && i / 2 < tool_calls {
            let tool_id = format!("tool-{}", i);
            content_blocks.push(ContentBlock::ToolUse {
                id: tool_id.clone(),
                name: "Read".to_string(),
                input: serde_json::json!({"path": format!("/src/file{}.rs", i)}),
                correlation_id: None,
                standard_tool: Some(StandardTool::FileRead {
                    path: format!("/src/file{}.rs", i),
                    start_line: None,
                    end_line: None,
                }),
                display_name: None,
                description: None,
            });
            content_blocks.push(ContentBlock::ToolResult {
                tool_use_id: tool_id,
                content: "file content".to_string(),
                is_error: false,
                correlation_id: None,
                structured_result: None,
                display_content: None,
                render_as_markdown: None,
                user_decision: None,
            });
        }

        messages.push(Message {
            role,
            content_blocks,
            timestamp: Some(base_time + chrono::Duration::minutes(i as i64 * 5)),
            mentioned_files: vec![],
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        });
    }

    MantraSession {
        id: id.to_string(),
        source: source.to_string(),
        cwd: "/test/project".to_string(),
        created_at: base_time,
        updated_at: base_time + chrono::Duration::hours(1),
        messages,
        metadata: Default::default(),
    }
}

fn create_session_with_errors(id: &str, error_count: usize) -> MantraSession {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

    let mut messages = Vec::new();

    for i in 0..error_count {
        let tool_id = format!("tool-err-{}", i);
        messages.push(Message {
            role: Role::Assistant,
            content_blocks: vec![
                ContentBlock::ToolUse {
                    id: tool_id.clone(),
                    name: "Bash".to_string(),
                    input: serde_json::json!({"command": "failing_command"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::ShellExec {
                        command: "failing_command".to_string(),
                        cwd: None,
                    }),
                    display_name: None,
                    description: None,
                },
                ContentBlock::ToolResult {
                    tool_use_id: tool_id,
                    content: "Error: command failed".to_string(),
                    is_error: true,
                    correlation_id: None,
                    structured_result: None,
                    display_content: None,
                    render_as_markdown: None,
                    user_decision: None,
                },
            ],
            timestamp: Some(base_time + chrono::Duration::minutes(i as i64 * 2)),
            mentioned_files: vec![],
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        });
    }

    MantraSession {
        id: id.to_string(),
        source: "claude".to_string(),
        cwd: "/test/project".to_string(),
        created_at: base_time,
        updated_at: base_time + chrono::Duration::minutes(error_count as i64 * 2),
        messages,
        metadata: Default::default(),
    }
}

// ===== calculate_session_metrics Tests =====

#[test]
fn test_calculate_session_metrics_basic() {
    let session = create_test_session("test-1", "claude", 10, 3);
    let metrics = calculate_session_metrics(&session);

    assert_eq!(metrics.session_id, "test-1");
    assert_eq!(metrics.tool_type, "claude");
    assert_eq!(metrics.message_count, 10);
    assert_eq!(metrics.tool_call_count, 3);
    assert_eq!(metrics.tool_error_count, 0);
    assert!(metrics.tool_types_used.contains(&"file_read".to_string()));
}

#[test]
fn test_calculate_session_metrics_with_errors() {
    let session = create_session_with_errors("error-session", 5);
    let metrics = calculate_session_metrics(&session);

    assert_eq!(metrics.session_id, "error-session");
    assert_eq!(metrics.tool_call_count, 5);
    assert_eq!(metrics.tool_error_count, 5);
    assert!(metrics.tool_types_used.contains(&"shell_exec".to_string()));
}

#[test]
fn test_calculate_session_metrics_duration() {
    let session = create_test_session("duration-test", "gemini", 6, 0);
    let metrics = calculate_session_metrics(&session);

    // 6 messages at 5 minute intervals = 25 minutes = 1500 seconds
    assert_eq!(metrics.duration_seconds, 25 * 60);
    assert!(metrics.start_time < metrics.end_time);
}

#[test]
fn test_calculate_session_metrics_empty_session() {
    let session = MantraSession::new("empty".to_string(), "cursor".to_string(), "/test".to_string());
    let metrics = calculate_session_metrics(&session);

    assert_eq!(metrics.session_id, "empty");
    assert_eq!(metrics.tool_type, "cursor");
    assert_eq!(metrics.message_count, 0);
    assert_eq!(metrics.tool_call_count, 0);
    assert_eq!(metrics.tool_error_count, 0);
    assert!(metrics.tool_types_used.is_empty());
}

#[test]
fn test_calculate_session_metrics_tool_type_counts() {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

    let messages = vec![
        Message {
            role: Role::Assistant,
            content_blocks: vec![
                ContentBlock::ToolUse {
                    id: "t1".to_string(),
                    name: "Read".to_string(),
                    input: serde_json::json!({"path": "/a.rs"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::FileRead {
                        path: "/a.rs".to_string(),
                        start_line: None,
                        end_line: None,
                    }),
                    display_name: None,
                    description: None,
                },
                ContentBlock::ToolUse {
                    id: "t2".to_string(),
                    name: "Read".to_string(),
                    input: serde_json::json!({"path": "/b.rs"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::FileRead {
                        path: "/b.rs".to_string(),
                        start_line: None,
                        end_line: None,
                    }),
                    display_name: None,
                    description: None,
                },
                ContentBlock::ToolUse {
                    id: "t3".to_string(),
                    name: "Bash".to_string(),
                    input: serde_json::json!({"command": "ls"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::ShellExec {
                        command: "ls".to_string(),
                        cwd: None,
                    }),
                    display_name: None,
                    description: None,
                },
            ],
            timestamp: Some(base_time),
            mentioned_files: vec![],
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        },
    ];

    let session = MantraSession {
        id: "counts-test".to_string(),
        source: "claude".to_string(),
        cwd: "/test".to_string(),
        created_at: base_time,
        updated_at: base_time,
        messages,
        metadata: Default::default(),
    };

    let metrics = calculate_session_metrics(&session);

    assert_eq!(metrics.tool_call_count, 3);
    assert_eq!(metrics.tool_type_counts.get("file_read"), Some(&2));
    assert_eq!(metrics.tool_type_counts.get("shell_exec"), Some(&1));
}

// ===== calculate_project_analytics Tests =====

#[test]
fn test_calculate_project_analytics_basic() {
    let metrics = vec![
        SessionMetrics {
            session_id: "s1".to_string(),
            tool_type: "claude".to_string(),
            start_time: Utc::now().timestamp() - 3600,
            end_time: Utc::now().timestamp(),
            duration_seconds: 3600,
            message_count: 20,
            tool_call_count: 10,
            tool_error_count: 1,
            tool_types_used: vec!["file_read".to_string()],
            tool_type_counts: [("file_read".to_string(), 10)].into_iter().collect(),
        },
        SessionMetrics {
            session_id: "s2".to_string(),
            tool_type: "gemini".to_string(),
            start_time: Utc::now().timestamp() - 7200,
            end_time: Utc::now().timestamp() - 3600,
            duration_seconds: 3600,
            message_count: 15,
            tool_call_count: 5,
            tool_error_count: 0,
            tool_types_used: vec!["shell_exec".to_string()],
            tool_type_counts: [("shell_exec".to_string(), 5)].into_iter().collect(),
        },
    ];

    let analytics = calculate_project_analytics("project-1", &metrics, TimeRange::All);

    assert_eq!(analytics.project_id, "project-1");
    assert_eq!(analytics.total_sessions, 2);
    assert_eq!(analytics.total_duration_seconds, 7200);
    assert_eq!(analytics.avg_duration_seconds, 3600);
    assert_eq!(analytics.total_tool_calls, 15);
    assert_eq!(analytics.total_tool_errors, 1);
    assert!((analytics.tool_error_rate - 1.0 / 15.0).abs() < 0.001);
    assert_eq!(analytics.tool_distribution.get("claude"), Some(&1));
    assert_eq!(analytics.tool_distribution.get("gemini"), Some(&1));
    assert_eq!(analytics.tool_types_distribution.get("file_read"), Some(&10));
    assert_eq!(analytics.tool_types_distribution.get("shell_exec"), Some(&5));
    assert_eq!(analytics.total_messages, 35);
}

#[test]
fn test_calculate_project_analytics_time_filter_days7() {
    let now = Utc::now().timestamp();
    let days_7 = 7 * 24 * 60 * 60;

    let metrics = vec![
        // Within 7 days
        SessionMetrics {
            session_id: "recent".to_string(),
            tool_type: "claude".to_string(),
            start_time: now - 3600,
            end_time: now,
            duration_seconds: 3600,
            message_count: 10,
            tool_call_count: 5,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
        // Outside 7 days
        SessionMetrics {
            session_id: "old".to_string(),
            tool_type: "claude".to_string(),
            start_time: now - days_7 - 86400,
            end_time: now - days_7 - 82800,
            duration_seconds: 3600,
            message_count: 20,
            tool_call_count: 10,
            tool_error_count: 2,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
    ];

    let analytics = calculate_project_analytics("project-2", &metrics, TimeRange::Days7);

    assert_eq!(analytics.total_sessions, 1);
    assert_eq!(analytics.total_messages, 10);
    assert_eq!(analytics.total_tool_calls, 5);
}

#[test]
fn test_calculate_project_analytics_empty() {
    let analytics = calculate_project_analytics("empty-project", &[], TimeRange::All);

    assert_eq!(analytics.project_id, "empty-project");
    assert_eq!(analytics.total_sessions, 0);
    assert_eq!(analytics.total_duration_seconds, 0);
    assert_eq!(analytics.active_days, 0);
    assert!(analytics.tool_distribution.is_empty());
    assert!(analytics.activity_trend.is_empty());
}

#[test]
fn test_calculate_project_analytics_active_days() {
    let day1 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap().timestamp();
    let day2 = Utc.with_ymd_and_hms(2024, 1, 16, 14, 0, 0).unwrap().timestamp();
    let day2_again = Utc.with_ymd_and_hms(2024, 1, 16, 18, 0, 0).unwrap().timestamp();

    let metrics = vec![
        SessionMetrics {
            session_id: "s1".to_string(),
            tool_type: "claude".to_string(),
            start_time: day1,
            end_time: day1 + 3600,
            duration_seconds: 3600,
            message_count: 5,
            tool_call_count: 2,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
        SessionMetrics {
            session_id: "s2".to_string(),
            tool_type: "claude".to_string(),
            start_time: day2,
            end_time: day2 + 3600,
            duration_seconds: 3600,
            message_count: 5,
            tool_call_count: 2,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
        SessionMetrics {
            session_id: "s3".to_string(),
            tool_type: "claude".to_string(),
            start_time: day2_again,
            end_time: day2_again + 1800,
            duration_seconds: 1800,
            message_count: 3,
            tool_call_count: 1,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
    ];

    let analytics = calculate_project_analytics("project-3", &metrics, TimeRange::All);

    assert_eq!(analytics.total_sessions, 3);
    assert_eq!(analytics.active_days, 2); // Only 2 unique days
}

#[test]
fn test_calculate_project_analytics_activity_trend() {
    let day1 = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap().timestamp();
    let day2 = Utc.with_ymd_and_hms(2024, 1, 16, 10, 0, 0).unwrap().timestamp();

    let metrics = vec![
        SessionMetrics {
            session_id: "s1".to_string(),
            tool_type: "claude".to_string(),
            start_time: day1,
            end_time: day1 + 3600,
            duration_seconds: 3600,
            message_count: 10,
            tool_call_count: 5,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
        SessionMetrics {
            session_id: "s2".to_string(),
            tool_type: "claude".to_string(),
            start_time: day1 + 7200,
            end_time: day1 + 10800,
            duration_seconds: 3600,
            message_count: 8,
            tool_call_count: 3,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
        SessionMetrics {
            session_id: "s3".to_string(),
            tool_type: "gemini".to_string(),
            start_time: day2,
            end_time: day2 + 1800,
            duration_seconds: 1800,
            message_count: 5,
            tool_call_count: 2,
            tool_error_count: 0,
            tool_types_used: vec![],
            tool_type_counts: Default::default(),
        },
    ];

    let analytics = calculate_project_analytics("project-4", &metrics, TimeRange::All);

    assert_eq!(analytics.activity_trend.len(), 2);

    // Day 1 should have 2 sessions, 8 tool calls, 7200 seconds
    let day1_data = analytics.activity_trend.iter().find(|d| d.date == "2024-01-15").unwrap();
    assert_eq!(day1_data.session_count, 2);
    assert_eq!(day1_data.tool_call_count, 8);
    assert_eq!(day1_data.duration_seconds, 7200);

    // Day 2 should have 1 session, 2 tool calls, 1800 seconds
    let day2_data = analytics.activity_trend.iter().find(|d| d.date == "2024-01-16").unwrap();
    assert_eq!(day2_data.session_count, 1);
    assert_eq!(day2_data.tool_call_count, 2);
    assert_eq!(day2_data.duration_seconds, 1800);
}

// ===== create_session_stats_view Tests =====

#[test]
fn test_create_session_stats_view_basic() {
    let session = create_test_session("view-test", "cursor", 8, 3);
    let view = create_session_stats_view(&session);

    assert_eq!(view.metrics.session_id, "view-test");
    assert_eq!(view.metrics.tool_type, "cursor");
    assert_eq!(view.tool_call_timeline.len(), 3);
    assert!(!view.tool_distribution.is_empty());
}

#[test]
fn test_create_session_stats_view_timeline_order() {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();

    let messages = vec![
        Message {
            role: Role::Assistant,
            content_blocks: vec![
                ContentBlock::ToolUse {
                    id: "t1".to_string(),
                    name: "Read".to_string(),
                    input: serde_json::json!({"path": "/first.rs"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::FileRead {
                        path: "/first.rs".to_string(),
                        start_line: None,
                        end_line: None,
                    }),
                    display_name: None,
                    description: None,
                },
            ],
            timestamp: Some(base_time),
            mentioned_files: vec![],
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        },
        Message {
            role: Role::Assistant,
            content_blocks: vec![
                ContentBlock::ToolUse {
                    id: "t2".to_string(),
                    name: "Bash".to_string(),
                    input: serde_json::json!({"command": "cargo build"}),
                    correlation_id: None,
                    standard_tool: Some(StandardTool::ShellExec {
                        command: "cargo build".to_string(),
                        cwd: None,
                    }),
                    display_name: None,
                    description: None,
                },
            ],
            timestamp: Some(base_time + chrono::Duration::minutes(5)),
            mentioned_files: vec![],
            message_id: None,
            parent_id: None,
            is_sidechain: false,
            source_metadata: None,
        },
    ];

    let session = MantraSession {
        id: "timeline-test".to_string(),
        source: "claude".to_string(),
        cwd: "/test".to_string(),
        created_at: base_time,
        updated_at: base_time + chrono::Duration::minutes(10),
        messages,
        metadata: Default::default(),
    };

    let view = create_session_stats_view(&session);

    assert_eq!(view.tool_call_timeline.len(), 2);
    assert_eq!(view.tool_call_timeline[0].tool_type, "file_read");
    assert_eq!(view.tool_call_timeline[0].description, Some("/first.rs".to_string()));
    assert_eq!(view.tool_call_timeline[1].tool_type, "shell_exec");
    assert_eq!(view.tool_call_timeline[1].description, Some("cargo build".to_string()));
}

#[test]
fn test_create_session_stats_view_error_tracking() {
    let session = create_session_with_errors("error-view", 3);
    let view = create_session_stats_view(&session);

    assert_eq!(view.tool_call_timeline.len(), 3);
    // All should be marked as errors
    for detail in &view.tool_call_timeline {
        assert!(detail.is_error);
    }
}
