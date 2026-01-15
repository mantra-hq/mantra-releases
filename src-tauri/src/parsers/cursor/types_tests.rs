use super::*;

#[test]
fn test_cursor_role_from_bubble_type() {
    assert_eq!(CursorRole::from(1), CursorRole::User);
    assert_eq!(CursorRole::from(2), CursorRole::Assistant);
    assert_eq!(CursorRole::from(0), CursorRole::Unknown);
    assert_eq!(CursorRole::from(99), CursorRole::Unknown);
}

#[test]
fn test_cursor_role_to_mantra_role() {
    assert_eq!(
        CursorRole::User.to_mantra_role(),
        Some(crate::models::Role::User)
    );
    assert_eq!(
        CursorRole::Assistant.to_mantra_role(),
        Some(crate::models::Role::Assistant)
    );
    assert_eq!(CursorRole::Unknown.to_mantra_role(), None);
}

#[test]
fn test_deserialize_bubble_header() {
    let json = r#"{"bubbleId": "abc-123", "type": 1}"#;
    let header: BubbleHeader = serde_json::from_str(json).unwrap();
    assert_eq!(header.bubble_id, "abc-123");
    assert_eq!(header.bubble_type, 1);
}

#[test]
fn test_deserialize_cursor_bubble() {
    let json = r#"{
        "_v": 3,
        "bubbleId": "bubble-123",
        "type": 2,
        "text": "Here is the code you requested.",
        "isAgentic": true,
        "toolResults": [],
        "suggestedCodeBlocks": []
    }"#;

    let bubble: CursorBubble = serde_json::from_str(json).unwrap();
    assert_eq!(bubble.version, Some(3));
    assert_eq!(bubble.bubble_id, Some("bubble-123".to_string()));
    assert_eq!(bubble.bubble_type, 2);
    assert_eq!(bubble.text, Some("Here is the code you requested.".to_string()));
    assert!(bubble.is_agentic);
}

#[test]
fn test_deserialize_cursor_composer() {
    let json = r#"{
        "_v": 2,
        "composerId": "comp-456",
        "fullConversationHeadersOnly": [
            {"bubbleId": "b1", "type": 1},
            {"bubbleId": "b2", "type": 2}
        ],
        "createdAt": 1704067200000,
        "unifiedMode": "agent"
    }"#;

    let composer: CursorComposer = serde_json::from_str(json).unwrap();
    assert_eq!(composer.version, Some(2));
    assert_eq!(composer.composer_id, Some("comp-456".to_string()));
    assert_eq!(composer.full_conversation_headers_only.len(), 2);
    assert_eq!(composer.created_at, Some(1704067200000));
    assert_eq!(composer.unified_mode, Some("agent".to_string()));
}
