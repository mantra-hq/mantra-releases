use super::*;
use serde_json::json;

#[test]
fn test_parse_all_composers() {
    let json = json!({
        "allComposers": [
            {
                "composerId": "abc-123",
                "name": "Test Composer",
                "createdAt": 1704067200000_i64,
                "unifiedMode": "agent",
                "totalLinesAdded": 100,
                "totalLinesRemoved": 50
            },
            {
                "composerId": "def-456",
                "name": null,
                "createdAt": 1704153600000_i64
            }
        ]
    });

    let composers = parse_all_composers(&json).unwrap();
    assert_eq!(composers.len(), 2);

    assert_eq!(composers[0].composer_id, "abc-123");
    assert_eq!(composers[0].name, Some("Test Composer".to_string()));
    assert_eq!(composers[0].created_at, Some(1704067200000));
    assert_eq!(composers[0].unified_mode, Some("agent".to_string()));
    assert_eq!(composers[0].total_lines_added, Some(100));

    assert_eq!(composers[1].composer_id, "def-456");
    assert_eq!(composers[1].name, None);
}

#[test]
fn test_parse_all_composers_empty() {
    let json = json!({
        "allComposers": []
    });

    let composers = parse_all_composers(&json).unwrap();
    assert!(composers.is_empty());
}

#[test]
fn test_parse_all_composers_missing_field() {
    let json = json!({
        "otherField": "value"
    });

    let result = parse_all_composers(&json);
    assert!(matches!(result, Err(ParseError::MissingField(_))));
}
