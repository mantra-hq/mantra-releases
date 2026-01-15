use super::*;

#[test]
fn test_line_type_should_process() {
    assert!(CodexLineType::SessionMeta.should_process());
    assert!(CodexLineType::ResponseItem.should_process());
    assert!(!CodexLineType::EventMsg.should_process());
    assert!(!CodexLineType::TurnContext.should_process());
}

#[test]
fn test_role_to_mantra_role() {
    assert_eq!(CodexRole::User.to_mantra_role(), crate::models::Role::User);
    assert_eq!(CodexRole::Assistant.to_mantra_role(), crate::models::Role::Assistant);
}

#[test]
fn test_content_item_text() {
    let input = CodexContentItem::InputText { text: "hello".to_string() };
    assert_eq!(input.text(), "hello");

    let output = CodexContentItem::OutputText { text: "world".to_string() };
    assert_eq!(output.text(), "world");
}

#[test]
fn test_deserialize_session_meta() {
    let json = r#"{
        "id": "test-123",
        "timestamp": "2025-12-30T20:00:00.000Z",
        "cwd": "/home/user/project",
        "cli_version": "0.77.0",
        "originator": "codex_cli_rs",
        "source": "cli"
    }"#;

    let meta: CodexSessionMeta = serde_json::from_str(json).unwrap();
    assert_eq!(meta.id, "test-123");
    assert_eq!(meta.cwd, "/home/user/project");
    assert_eq!(meta.cli_version, Some("0.77.0".to_string()));
}

#[test]
fn test_deserialize_rollout_line() {
    let json = r#"{
        "timestamp": "2025-12-30T20:00:00.000Z",
        "type": "session_meta",
        "payload": {"id": "test-123", "timestamp": "2025-12-30T20:00:00.000Z", "cwd": "/tmp"}
    }"#;

    let line: CodexRolloutLine = serde_json::from_str(json).unwrap();
    assert_eq!(line.line_type, CodexLineType::SessionMeta);
}

#[test]
fn test_deserialize_user_message() {
    let json = r#"{
        "type": "message",
        "role": "user",
        "content": [{"type": "input_text", "text": "Hello"}]
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::Message { role, content } => {
            assert_eq!(role, CodexRole::User);
            assert_eq!(content.len(), 1);
            assert_eq!(content[0].text(), "Hello");
        }
        _ => panic!("Expected Message"),
    }
}

#[test]
fn test_deserialize_assistant_message() {
    let json = r#"{
        "type": "message",
        "role": "assistant",
        "content": [{"type": "output_text", "text": "I'll help you."}]
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::Message { role, content } => {
            assert_eq!(role, CodexRole::Assistant);
            assert_eq!(content.len(), 1);
            assert_eq!(content[0].text(), "I'll help you.");
        }
        _ => panic!("Expected Message"),
    }
}

#[test]
fn test_deserialize_function_call() {
    let json = r#"{
        "type": "function_call",
        "name": "shell",
        "arguments": "{\"command\": \"ls\"}",
        "call_id": "call_123"
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::FunctionCall { name, arguments, call_id } => {
            assert_eq!(name, "shell");
            assert_eq!(call_id, "call_123");
            assert!(arguments.contains("ls"));
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_deserialize_function_call_output() {
    let json = r#"{
        "type": "function_call_output",
        "call_id": "call_123",
        "output": "file1.txt\nfile2.txt"
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "call_123");
            assert!(output.get_output().contains("file1.txt"));
        }
        _ => panic!("Expected FunctionCallOutput"),
    }
}

#[test]
fn test_deserialize_function_call_output_with_success() {
    let json = r#"{
        "type": "function_call_output",
        "call_id": "call_456",
        "output": "Success!",
        "success": true
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "call_456");
            assert_eq!(output.get_output(), "Success!");
            assert_eq!(output.success, Some(true));
        }
        _ => panic!("Expected FunctionCallOutput"),
    }
}

#[test]
fn test_deserialize_reasoning() {
    let json = r#"{
        "type": "reasoning",
        "summary": [{"type": "summary_text", "text": "**Thinking about the problem**"}],
        "content": null,
        "encrypted_content": "gAAAAABo4kai..."
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::Reasoning { summary, content, encrypted_content } => {
            assert_eq!(summary.len(), 1);
            assert_eq!(summary[0].text(), "**Thinking about the problem**");
            assert!(content.is_none());
            assert!(encrypted_content.is_some());
        }
        _ => panic!("Expected Reasoning"),
    }
}

#[test]
fn test_deserialize_reasoning_with_content() {
    let json = r#"{
        "type": "reasoning",
        "summary": [{"type": "summary_text", "text": "Step 1"}],
        "content": [
            {"type": "reasoning_text", "text": "First, let me analyze..."},
            {"type": "text", "text": "Then, I'll implement..."}
        ]
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::Reasoning { summary, content, .. } => {
            assert_eq!(summary.len(), 1);
            assert!(content.is_some());
            let contents = content.unwrap();
            assert_eq!(contents.len(), 2);
            assert_eq!(contents[0].text(), "First, let me analyze...");
            assert_eq!(contents[1].text(), "Then, I'll implement...");
        }
        _ => panic!("Expected Reasoning"),
    }
}

#[test]
fn test_deserialize_web_search_call() {
    let json = r#"{
        "type": "web_search_call",
        "status": "completed",
        "action": {"type": "search", "query": "Rust async programming"}
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::WebSearchCall { action, status } => {
            assert_eq!(status, Some("completed".to_string()));
            if let WebSearchAction::Search { query } = action {
                assert_eq!(query, Some("Rust async programming".to_string()));
            } else {
                panic!("Expected Search action");
            }
        }
        _ => panic!("Expected WebSearchCall"),
    }
}

#[test]
fn test_deserialize_custom_tool_call() {
    let json = r#"{
        "type": "custom_tool_call",
        "call_id": "custom_123",
        "name": "my_tool",
        "input": "{\"param\": \"value\"}"
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::CustomToolCall { call_id, name, input, .. } => {
            assert_eq!(call_id, "custom_123");
            assert_eq!(name, "my_tool");
            assert!(input.contains("param"));
        }
        _ => panic!("Expected CustomToolCall"),
    }
}

#[test]
fn test_deserialize_compaction() {
    let json = r#"{
        "type": "compaction",
        "encrypted_content": "ENCRYPTED_SUMMARY_CONTENT"
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::Compaction { encrypted_content } => {
            assert_eq!(encrypted_content, "ENCRYPTED_SUMMARY_CONTENT");
        }
        _ => panic!("Expected Compaction"),
    }
}

#[test]
fn test_deserialize_unknown_type() {
    let json = r#"{
        "type": "future_new_type",
        "some_field": "value"
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    assert!(matches!(item, CodexResponseItem::Other));
}

#[test]
fn test_deserialize_local_shell_call() {
    let json = r#"{
        "type": "local_shell_call",
        "call_id": "shell_123",
        "status": "completed",
        "action": {
            "type": "exec",
            "command": ["bash", "-c", "ls -la"],
            "cwd": "/tmp",
            "exit_code": 0,
            "output": "file1.txt\nfile2.txt"
        }
    }"#;

    let item: CodexResponseItem = serde_json::from_str(json).unwrap();
    match item {
        CodexResponseItem::LocalShellCall { call_id, status, action } => {
            assert_eq!(call_id, Some("shell_123".to_string()));
            assert!(matches!(status, Some(LocalShellStatus::Completed)));
            if let LocalShellAction::Exec { command, cwd, exit_code, output } = action {
                assert_eq!(command, Some(vec!["bash".to_string(), "-c".to_string(), "ls -la".to_string()]));
                assert_eq!(cwd, Some("/tmp".to_string()));
                assert_eq!(exit_code, Some(0));
                assert!(output.unwrap().contains("file1.txt"));
            } else {
                panic!("Expected Exec action");
            }
        }
        _ => panic!("Expected LocalShellCall"),
    }
}

#[test]
fn test_deserialize_git_info() {
    let json = r#"{
        "id": "test-123",
        "timestamp": "2025-12-30T20:00:00.000Z",
        "cwd": "/home/user/project",
        "git": {
            "commit_hash": "abc123",
            "branch": "main",
            "repository_url": "https://github.com/user/repo"
        }
    }"#;

    let meta: CodexSessionMeta = serde_json::from_str(json).unwrap();
    assert!(meta.git.is_some());
    let git = meta.git.unwrap();
    assert_eq!(git.commit_hash, Some("abc123".to_string()));
    assert_eq!(git.branch, Some("main".to_string()));
}

#[test]
fn test_function_call_output_extracts_json_output() {
    // JSON format: {"metadata": {...}, "output": "actual content"}
    let payload = FunctionCallOutputPayload {
        output: Some(r#"{"metadata":{"exit_code":0,"duration_seconds":0.1},"output":"file1.txt\nfile2.txt"}"#.to_string()),
        success: Some(true),
    };
    assert_eq!(payload.get_output(), "file1.txt\nfile2.txt");
}

#[test]
fn test_function_call_output_returns_raw_for_plain_text() {
    // Plain text format: Exit code: 0\nWall time: ...\nOutput:\nactual content
    let payload = FunctionCallOutputPayload {
        output: Some("Exit code: 0\nWall time: 0.1 seconds\nOutput:\nfile1.txt\nfile2.txt".to_string()),
        success: Some(true),
    };
    assert_eq!(payload.get_output(), "Exit code: 0\nWall time: 0.1 seconds\nOutput:\nfile1.txt\nfile2.txt");
}

#[test]
fn test_function_call_output_returns_raw_for_non_shell_json() {
    // JSON without "output" field should return as-is
    let payload = FunctionCallOutputPayload {
        output: Some(r#"{"status":"ok","data":"some value"}"#.to_string()),
        success: Some(true),
    };
    assert_eq!(payload.get_output(), r#"{"status":"ok","data":"some value"}"#);
}

#[test]
fn test_function_call_output_handles_empty() {
    let payload = FunctionCallOutputPayload {
        output: None,
        success: None,
    };
    assert_eq!(payload.get_output(), "");
}
