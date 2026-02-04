use super::*;
use crate::sanitizer::{ScanMatch, SensitiveType, Severity};

fn create_test_record(
    source: InterceptionSource,
    action: UserAction,
    project: Option<&str>,
) -> InterceptionRecord {
    InterceptionRecord::new(
        source,
        vec![ScanMatch {
            rule_id: "test_rule".to_string(),
            sensitive_type: SensitiveType::ApiKey,
            severity: Severity::Critical,
            line: 1,
            column: 1,
            matched_text: "sk-test123456789".to_string(),
            masked_text: "sk-****".to_string(),
            context: "API key".to_string(),
        }],
        action,
        "hash123".to_string(),
        project.map(|s| s.to_string()),
    )
}

#[test]
fn test_save_and_get_record() {
    let db = Database::new_in_memory().unwrap();

    let record = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-123".to_string(),
        },
        UserAction::Redacted,
        Some("test-project"),
    );

    // Save
    let result = db.save_interception_record(&record);
    assert!(result.is_ok(), "Save failed: {:?}", result.err());

    // Get
    let paginated = db.get_interception_records(1, 10, None).unwrap();
    assert_eq!(paginated.total, 1);
    assert_eq!(paginated.records.len(), 1);
    assert_eq!(paginated.records[0].id, record.id);
    assert_eq!(paginated.records[0].user_action, UserAction::Redacted);
    assert_eq!(
        paginated.records[0].project_name,
        Some("test-project".to_string())
    );
}

#[test]
fn test_pagination() {
    let db = Database::new_in_memory().unwrap();

    // Insert 5 records
    for i in 0..5 {
        let record = create_test_record(
            InterceptionSource::ClaudeCodeHook {
                session_id: Some(format!("sess-{}", i)),
            },
            UserAction::Ignored,
            None,
        );
        db.save_interception_record(&record).unwrap();
    }

    // Page 1 with 2 items
    let page1 = db.get_interception_records(1, 2, None).unwrap();
    assert_eq!(page1.total, 5);
    assert_eq!(page1.records.len(), 2);
    assert_eq!(page1.page, 1);
    assert_eq!(page1.per_page, 2);

    // Page 3 with 2 items (should have 1 record)
    let page3 = db.get_interception_records(3, 2, None).unwrap();
    assert_eq!(page3.total, 5);
    assert_eq!(page3.records.len(), 1);
}

#[test]
fn test_source_filter() {
    let db = Database::new_in_memory().unwrap();

    // Insert records with different sources
    let pre_upload = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-1".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    let claude_hook = create_test_record(
        InterceptionSource::ClaudeCodeHook {
            session_id: Some("sess-2".to_string()),
        },
        UserAction::Ignored,
        None,
    );
    let external = create_test_record(
        InterceptionSource::ExternalHook {
            tool_name: "copilot".to_string(),
        },
        UserAction::Cancelled,
        None,
    );

    db.save_interception_record(&pre_upload).unwrap();
    db.save_interception_record(&claude_hook).unwrap();
    db.save_interception_record(&external).unwrap();

    // Filter by pre_upload
    let filtered = db
        .get_interception_records(1, 10, Some("pre_upload"))
        .unwrap();
    assert_eq!(filtered.total, 1);
    assert_eq!(filtered.records[0].source.source_type(), "pre_upload");

    // Filter by claude_code_hook
    let filtered = db
        .get_interception_records(1, 10, Some("claude_code_hook"))
        .unwrap();
    assert_eq!(filtered.total, 1);
}

#[test]
fn test_delete_records() {
    let db = Database::new_in_memory().unwrap();

    let record1 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-1".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    let record2 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-2".to_string(),
        },
        UserAction::Ignored,
        None,
    );
    let record3 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-3".to_string(),
        },
        UserAction::Cancelled,
        None,
    );

    db.save_interception_record(&record1).unwrap();
    db.save_interception_record(&record2).unwrap();
    db.save_interception_record(&record3).unwrap();

    // Delete 2 records
    let deleted = db
        .delete_interception_records(&[record1.id.clone(), record3.id.clone()])
        .unwrap();
    assert_eq!(deleted, 2);

    // Verify only 1 remains
    let remaining = db.get_interception_records(1, 10, None).unwrap();
    assert_eq!(remaining.total, 1);
    assert_eq!(remaining.records[0].id, record2.id);
}

#[test]
fn test_delete_empty_list() {
    let db = Database::new_in_memory().unwrap();
    let deleted = db.delete_interception_records(&[]).unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn test_delete_batch_limit() {
    let db = Database::new_in_memory().unwrap();

    // Create a list of IDs that exceeds MAX_DELETE_BATCH
    let too_many_ids: Vec<String> = (0..1001).map(|i| format!("id-{}", i)).collect();

    // Should return error for oversized batch
    let result = db.delete_interception_records(&too_many_ids);
    assert!(result.is_err());

    if let Err(StorageError::InvalidInput(msg)) = result {
        assert!(msg.contains("1001"));
        assert!(msg.contains("1000"));
    } else {
        panic!("Expected InvalidInput error");
    }

    // Exactly at limit should work
    let exact_limit: Vec<String> = (0..1000).map(|i| format!("id-{}", i)).collect();
    let result = db.delete_interception_records(&exact_limit);
    assert!(result.is_ok()); // Will return 0 since no records exist with these IDs
}

#[test]
fn test_get_stats() {
    let db = Database::new_in_memory().unwrap();

    // Insert records with different actions
    let record1 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-1".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    let record2 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-2".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    let record3 = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-3".to_string(),
        },
        UserAction::Ignored,
        None,
    );

    db.save_interception_record(&record1).unwrap();
    db.save_interception_record(&record2).unwrap();
    db.save_interception_record(&record3).unwrap();

    let stats = db.get_interception_stats().unwrap();

    assert_eq!(stats.total_interceptions, 3);
    assert_eq!(*stats.by_action.get("redacted").unwrap_or(&0), 2);
    assert_eq!(*stats.by_action.get("ignored").unwrap_or(&0), 1);
    assert!(stats.recent_7_days >= 3); // All records should be within 7 days
}

#[test]
fn test_stats_by_type() {
    let db = Database::new_in_memory().unwrap();

    // Create record with ApiKey type (from the helper)
    let record = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-1".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    db.save_interception_record(&record).unwrap();

    let stats = db.get_interception_stats().unwrap();

    // Should have count for api_key type
    assert!(stats.by_type.get("api_key").is_some() || stats.by_type.get("API_KEY").is_some());
}

#[test]
fn test_stats_by_severity() {
    let db = Database::new_in_memory().unwrap();

    // Create record with Critical severity (from the helper)
    let record = create_test_record(
        InterceptionSource::PreUpload {
            session_id: "sess-1".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    db.save_interception_record(&record).unwrap();

    let stats = db.get_interception_stats().unwrap();

    // Should have count for critical severity
    assert!(
        stats.by_severity.get("critical").is_some()
            || stats.by_severity.get("Critical").is_some()
    );
}

#[test]
fn test_empty_stats() {
    let db = Database::new_in_memory().unwrap();

    let stats = db.get_interception_stats().unwrap();

    assert_eq!(stats.total_interceptions, 0);
    assert!(stats.by_type.is_empty());
    assert!(stats.by_severity.is_empty());
    assert!(stats.by_action.is_empty());
    assert_eq!(stats.recent_7_days, 0);
}

#[test]
fn test_source_context_serialization() {
    let db = Database::new_in_memory().unwrap();

    // Test with external hook
    let record = create_test_record(
        InterceptionSource::ExternalHook {
            tool_name: "github-copilot".to_string(),
        },
        UserAction::Redacted,
        None,
    );
    db.save_interception_record(&record).unwrap();

    let paginated = db.get_interception_records(1, 10, None).unwrap();
    if let InterceptionSource::ExternalHook { tool_name } = &paginated.records[0].source {
        assert_eq!(tool_name, "github-copilot");
    } else {
        panic!("Expected ExternalHook source");
    }
}
