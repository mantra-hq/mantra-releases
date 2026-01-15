use super::*;

#[test]
fn test_message_type_should_include() {
    assert!(GeminiMessageType::User.should_include());
    assert!(GeminiMessageType::Gemini.should_include());
    assert!(!GeminiMessageType::Info.should_include());
    assert!(!GeminiMessageType::Error.should_include());
    assert!(!GeminiMessageType::Warning.should_include());
}

#[test]
fn test_message_type_to_role() {
    assert_eq!(
        GeminiMessageType::User.to_mantra_role(),
        Some(crate::models::Role::User)
    );
    assert_eq!(
        GeminiMessageType::Gemini.to_mantra_role(),
        Some(crate::models::Role::Assistant)
    );
    assert_eq!(GeminiMessageType::Info.to_mantra_role(), None);
}

#[test]
fn test_content_text() {
    let content = GeminiContent::Text("Hello".to_string());
    assert_eq!(content.as_text(), "Hello");
    assert!(!content.is_empty());
}

#[test]
fn test_content_default() {
    let content = GeminiContent::default();
    assert!(content.is_empty());
}

#[test]
fn test_content_parts() {
    let content = GeminiContent::Parts(vec![
        GeminiPart {
            text: Some("Hello ".to_string()),
            inline_data: None,
            function_call: None,
            function_response: None,
            unknown_fields: serde_json::Map::new(),
        },
        GeminiPart {
            text: Some("World".to_string()),
            inline_data: None,
            function_call: None,
            function_response: None,
            unknown_fields: serde_json::Map::new(),
        },
    ]);
    assert_eq!(content.as_text(), "Hello World");
}

#[test]
fn test_thought_format() {
    let thought = GeminiThought {
        subject: "Analysis".to_string(),
        description: "Analyzing the code structure".to_string(),
        timestamp: None,
    };
    assert_eq!(
        thought.as_formatted_string(),
        "**Analysis** Analyzing the code structure"
    );
}

#[test]
fn test_tool_response_content() {
    let response = GeminiToolResponse {
        output: Some("Command output".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };
    assert_eq!(response.as_content(), "Command output");

    let error_response = GeminiToolResponse {
        output: None,
        error: Some("Failed".to_string()),
        extra: serde_json::Map::new(),
    };
    assert_eq!(error_response.as_content(), "Error: Failed");
}

#[test]
fn test_deserialize_conversation() {
    let json = r#"{
        "sessionId": "test-123",
        "projectHash": "abc456",
        "startTime": "2025-12-30T20:11:00.000Z",
        "lastUpdated": "2025-12-30T20:15:00.000Z",
        "messages": [],
        "summary": "Test session"
    }"#;

    let conv: GeminiConversation = serde_json::from_str(json).unwrap();
    assert_eq!(conv.session_id, "test-123");
    assert_eq!(conv.project_hash, "abc456");
    assert!(conv.messages.is_empty());
    assert_eq!(conv.summary, Some("Test session".to_string()));
}

#[test]
fn test_deserialize_user_message() {
    let json = r#"{
        "id": "msg-1",
        "timestamp": "2025-12-30T20:11:00.000Z",
        "type": "user",
        "content": "Hello, help me with this code"
    }"#;

    let msg: GeminiMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.id, "msg-1");
    assert_eq!(msg.msg_type, GeminiMessageType::User);
    assert_eq!(msg.content.as_text(), "Hello, help me with this code");
}

#[test]
fn test_deserialize_gemini_message_with_thoughts() {
    let json = r#"{
        "id": "msg-2",
        "timestamp": "2025-12-30T20:13:00.000Z",
        "type": "gemini",
        "content": "I'll help you with that.",
        "thoughts": [
            {
                "subject": "Analysis",
                "description": "User needs help with code",
                "timestamp": "2025-12-30T20:12:58.000Z"
            }
        ],
        "model": "gemini-3-pro-preview"
    }"#;

    let msg: GeminiMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.id, "msg-2");
    assert_eq!(msg.msg_type, GeminiMessageType::Gemini);
    assert!(msg.thoughts.is_some());
    let thoughts = msg.thoughts.unwrap();
    assert_eq!(thoughts.len(), 1);
    assert_eq!(thoughts[0].subject, "Analysis");
    assert_eq!(msg.model, Some("gemini-3-pro-preview".to_string()));
}

#[test]
fn test_deserialize_tool_call() {
    let json = r#"{
        "id": "run_shell_command-123",
        "name": "run_shell_command",
        "args": {"command": "ls -la"},
        "result": [
            {
                "functionResponse": {
                    "id": "run_shell_command-123",
                    "name": "run_shell_command",
                    "response": {
                        "output": "total 0\ndrwxr-xr-x 2 user user 40 Dec 30 20:11 ."
                    }
                }
            }
        ],
        "status": "success",
        "timestamp": "2025-12-30T20:13:20.000Z"
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(tool_call.id, "run_shell_command-123");
    assert_eq!(tool_call.name, "run_shell_command");
    assert_eq!(tool_call.status, "success");
    assert!(tool_call.result.is_some());
    let result = tool_call.result.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].function_response.name,
        "run_shell_command"
    );
}

#[test]
fn test_deserialize_tool_call_with_display_fields() {
    let json = r#"{
        "id": "run_shell_command-123",
        "name": "run_shell_command",
        "args": {"command": "ls -la"},
        "result": [
            {
                "functionResponse": {
                    "id": "run_shell_command-123",
                    "name": "run_shell_command",
                    "response": {
                        "output": "file1.txt\nfile2.txt"
                    }
                }
            }
        ],
        "status": "success",
        "timestamp": "2025-12-30T20:13:20.000Z",
        "displayName": "Shell",
        "description": "Execute shell commands",
        "resultDisplay": "file1.txt\nfile2.txt",
        "renderOutputAsMarkdown": false
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(tool_call.display_name, Some("Shell".to_string()));
    assert_eq!(tool_call.description, Some("Execute shell commands".to_string()));
    assert_eq!(tool_call.result_display, Some("file1.txt\nfile2.txt".to_string()));
    assert_eq!(tool_call.render_output_as_markdown, Some(false));
}

#[test]
fn test_deserialize_message_with_parts_content() {
    let json = r#"{
        "id": "msg-3",
        "timestamp": "2025-12-30T20:14:00.000Z",
        "type": "gemini",
        "content": [
            {"text": "Here is "},
            {"text": "the answer"}
        ]
    }"#;

    let msg: GeminiMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.content.as_text(), "Here is the answer");
}

#[test]
fn test_deserialize_info_message() {
    let json = r#"{
        "id": "msg-info",
        "timestamp": "2025-12-30T20:10:00.000Z",
        "type": "info",
        "content": "Session started"
    }"#;

    let msg: GeminiMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.msg_type, GeminiMessageType::Info);
    assert!(!msg.msg_type.should_include());
}
