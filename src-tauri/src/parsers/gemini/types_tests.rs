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
    // result_display is now ToolResultDisplay enum
    let result_display = tool_call.result_display.unwrap();
    assert_eq!(result_display.as_display_string(), "file1.txt\nfile2.txt");
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

// ========== Shell Result Format Parsing Tests ==========

#[test]
fn test_is_shell_result_format_positive() {
    let response = GeminiToolResponse {
        output: Some("Command: ls -la\nDirectory: /home/user\nOutput: file1.txt\nfile2.txt\nError: (none)\nExit Code: 0\nSignal: (none)\nBackground PIDs: (none)\nProcess Group PGID: 12345".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };
    assert!(response.is_shell_result_format());
}

#[test]
fn test_is_shell_result_format_negative() {
    // Simple output without shell format
    let response = GeminiToolResponse {
        output: Some("file1.txt\nfile2.txt".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };
    assert!(!response.is_shell_result_format());
}

#[test]
fn test_parse_shell_result_basic() {
    let response = GeminiToolResponse {
        output: Some("Command: ls -la\nDirectory: /home/user\nOutput: file1.txt\nfile2.txt\nError: (none)\nExit Code: 0\nSignal: (none)\nBackground PIDs: (none)\nProcess Group PGID: 12345".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    let parsed = response.parse_shell_result().unwrap();
    assert_eq!(parsed.command, Some("ls -la".to_string()));
    assert_eq!(parsed.directory, Some("/home/user".to_string()));
    assert_eq!(parsed.output, Some("file1.txt\nfile2.txt".to_string()));
    assert_eq!(parsed.error, None); // (none) should be parsed as None
    assert_eq!(parsed.exit_code, Some(0));
    assert_eq!(parsed.signal, None); // (none) should be parsed as None
}

#[test]
fn test_parse_shell_result_empty_output() {
    let response = GeminiToolResponse {
        output: Some("Command: grep -r \"test\" /src\nDirectory: (root)\nOutput: (empty)\nError: (none)\nExit Code: 1\nSignal: (none)\nBackground PIDs: (none)\nProcess Group PGID: 54321".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    let parsed = response.parse_shell_result().unwrap();
    assert_eq!(parsed.command, Some("grep -r \"test\" /src".to_string()));
    assert_eq!(parsed.directory, None); // (root) should be parsed as None
    assert_eq!(parsed.output, None); // (empty) should be parsed as None
    assert_eq!(parsed.exit_code, Some(1));
}

#[test]
fn test_parse_shell_result_with_error() {
    let response = GeminiToolResponse {
        output: Some("Command: cat nonexistent.txt\nDirectory: /tmp\nOutput: (empty)\nError: No such file or directory\nExit Code: 1\nSignal: (none)\nBackground PIDs: (none)\nProcess Group PGID: 99999".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    let parsed = response.parse_shell_result().unwrap();
    assert_eq!(parsed.command, Some("cat nonexistent.txt".to_string()));
    assert_eq!(parsed.error, Some("No such file or directory".to_string()));
    assert_eq!(parsed.exit_code, Some(1));
}

#[test]
fn test_parse_shell_result_multiline_output() {
    // Test multi-line output that spans multiple lines before the next field
    let response = GeminiToolResponse {
        output: Some("Command: ls -la\nDirectory: /home\nOutput: drwxr-xr-x  2 user user  4096 Jan  1 00:00 .\ndrwxr-xr-x 10 user user  4096 Jan  1 00:00 ..\n-rw-r--r--  1 user user   100 Jan  1 00:00 test.txt\nError: (none)\nExit Code: 0\nSignal: (none)\nBackground PIDs: (none)\nProcess Group PGID: 11111".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    let parsed = response.parse_shell_result().unwrap();
    assert_eq!(parsed.command, Some("ls -la".to_string()));
    // The output should contain all three lines
    let output = parsed.output.unwrap();
    assert!(output.contains("drwxr-xr-x  2 user user"));
    assert!(output.contains("drwxr-xr-x 10 user user"));
    assert!(output.contains("-rw-r--r--  1 user user"));
    assert_eq!(parsed.exit_code, Some(0));
}

#[test]
fn test_parse_shell_result_returns_none_for_non_shell_format() {
    // Should return None for content that doesn't match shell format
    let response = GeminiToolResponse {
        output: Some("Just some regular output\nwithout shell format markers".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    assert!(response.parse_shell_result().is_none());
}

#[test]
fn test_parse_shell_result_signal_terminated() {
    let response = GeminiToolResponse {
        output: Some("Command: sleep 100\nDirectory: /tmp\nOutput: (empty)\nError: (none)\nExit Code: (none)\nSignal: 9\nBackground PIDs: (none)\nProcess Group PGID: 77777".to_string()),
        error: None,
        extra: serde_json::Map::new(),
    };

    let parsed = response.parse_shell_result().unwrap();
    assert_eq!(parsed.exit_code, None); // (none) should be None
    assert_eq!(parsed.signal, Some(9));
}

// ========== ToolResultDisplay Tests ==========

#[test]
fn test_tool_result_display_string() {
    let json = r#""file1.txt\nfile2.txt""#;
    let display: ToolResultDisplay = serde_json::from_str(json).unwrap();
    match &display {
        ToolResultDisplay::String(s) => assert_eq!(s, "file1.txt\nfile2.txt"),
        _ => panic!("Expected String variant"),
    }
    assert_eq!(display.as_display_string(), "file1.txt\nfile2.txt");
}

#[test]
fn test_tool_result_display_file_diff() {
    let json = r#"{
        "fileDiff": "--- a/test.ts\n+++ b/test.ts\n@@ -1,3 +1,4 @@\n+const x = 1;",
        "fileName": "test.ts",
        "originalContent": "// old content",
        "newContent": "const x = 1;\n// old content",
        "diffStat": {
            "model_added_lines": 1,
            "model_removed_lines": 0,
            "model_added_chars": 12,
            "model_removed_chars": 0,
            "user_added_lines": 0,
            "user_removed_lines": 0,
            "user_added_chars": 0,
            "user_removed_chars": 0
        }
    }"#;
    
    let display: ToolResultDisplay = serde_json::from_str(json).unwrap();
    assert!(display.is_file_diff());
    
    match &display {
        ToolResultDisplay::FileDiff(diff) => {
            assert_eq!(diff.file_name, "test.ts");
            assert!(diff.file_diff.contains("const x = 1;"));
            assert_eq!(diff.original_content, Some("// old content".to_string()));
            assert!(diff.diff_stat.is_some());
            let stat = diff.diff_stat.as_ref().unwrap();
            assert_eq!(stat.model_added_lines, 1);
        }
        _ => panic!("Expected FileDiff variant"),
    }
}

#[test]
fn test_tool_result_display_file_diff_new_file() {
    // New file has null originalContent
    let json = r#"{
        "fileDiff": "--- /dev/null\n+++ b/new_file.ts\n@@ -0,0 +1,3 @@\n+const x = 1;\n+const y = 2;",
        "fileName": "new_file.ts",
        "originalContent": null,
        "newContent": "const x = 1;\nconst y = 2;"
    }"#;
    
    let display: ToolResultDisplay = serde_json::from_str(json).unwrap();
    
    if let Some(diff) = display.as_file_diff() {
        assert_eq!(diff.file_name, "new_file.ts");
        assert!(diff.original_content.is_none());
        assert!(diff.diff_stat.is_none());
    } else {
        panic!("Expected FileDiff variant");
    }
}

#[test]
fn test_tool_result_display_ansi_output() {
    // Use escaped hex codes to avoid Rust raw string issues with #
    let json = r##"[
        [
            {"text": "Hello ", "bold": true, "italic": false, "underline": false, "dim": false, "inverse": false, "fg": "#00ff00", "bg": ""},
            {"text": "World", "bold": false, "italic": false, "underline": true, "dim": false, "inverse": false, "fg": "", "bg": ""}
        ],
        [
            {"text": "Line 2", "bold": false, "italic": false, "underline": false, "dim": false, "inverse": false, "fg": "", "bg": ""}
        ]
    ]"##;
    
    let display: ToolResultDisplay = serde_json::from_str(json).unwrap();
    
    match &display {
        ToolResultDisplay::AnsiOutput(output) => {
            assert_eq!(output.len(), 2);
            assert_eq!(output[0].len(), 2);
            assert_eq!(output[0][0].text, "Hello ");
            assert!(output[0][0].bold);
            assert_eq!(output[0][0].fg, "#00ff00");
            assert_eq!(output[0][1].text, "World");
            assert!(output[0][1].underline);
            assert_eq!(output[1][0].text, "Line 2");
        }
        _ => panic!("Expected AnsiOutput variant"),
    }
    
    // Check display string conversion
    assert_eq!(display.as_display_string(), "Hello World\nLine 2");
}

#[test]
fn test_tool_result_display_todo_list() {
    let json = r#"{
        "todos": [
            {"description": "Implement feature A", "status": "completed"},
            {"description": "Write tests", "status": "in_progress"},
            {"description": "Review code", "status": "pending"},
            {"description": "Old task", "status": "cancelled"}
        ]
    }"#;
    
    let display: ToolResultDisplay = serde_json::from_str(json).unwrap();
    
    match display {
        ToolResultDisplay::TodoList(list) => {
            assert_eq!(list.todos.len(), 4);
            assert_eq!(list.todos[0].description, "Implement feature A");
            assert_eq!(list.todos[0].status, TodoStatus::Completed);
            assert_eq!(list.todos[1].status, TodoStatus::InProgress);
            assert_eq!(list.todos[2].status, TodoStatus::Pending);
            assert_eq!(list.todos[3].status, TodoStatus::Cancelled);
        }
        _ => panic!("Expected TodoList variant"),
    }
}

#[test]
fn test_deserialize_tool_call_with_file_diff_result_display() {
    let json = r#"{
        "id": "edit-123",
        "name": "edit_file",
        "args": {"filePath": "/src/main.rs", "newContent": "fn main() {}"},
        "result": [
            {
                "functionResponse": {
                    "id": "edit-123",
                    "name": "edit_file",
                    "response": {"output": "File edited successfully"}
                }
            }
        ],
        "status": "success",
        "timestamp": "2025-12-30T20:13:20.000Z",
        "displayName": "Edit File",
        "resultDisplay": {
            "fileDiff": "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-fn main() { println!(\"old\"); }\n+fn main() {}",
            "fileName": "src/main.rs",
            "originalContent": "fn main() { println!(\"old\"); }",
            "newContent": "fn main() {}"
        }
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    assert_eq!(tool_call.id, "edit-123");
    assert_eq!(tool_call.display_name, Some("Edit File".to_string()));
    
    let result_display = tool_call.result_display.unwrap();
    assert!(result_display.is_file_diff());
    
    let diff = result_display.as_file_diff().unwrap();
    assert_eq!(diff.file_name, "src/main.rs");
    assert!(diff.file_diff.contains("-fn main() { println!(\"old\"); }"));
    assert!(diff.file_diff.contains("+fn main() {}"));
}

#[test]
fn test_deserialize_tool_call_with_ansi_result_display() {
    // Use r##"..."## to allow # in the JSON string
    let json = r##"{
        "id": "shell-123",
        "name": "run_shell_command",
        "args": {"command": "ls --color"},
        "result": [
            {
                "functionResponse": {
                    "id": "shell-123",
                    "name": "run_shell_command",
                    "response": {"output": "file1.txt file2.txt"}
                }
            }
        ],
        "status": "success",
        "resultDisplay": [
            [
                {"text": "file1.txt", "bold": false, "italic": false, "underline": false, "dim": false, "inverse": false, "fg": "#0000ff", "bg": ""},
                {"text": " ", "bold": false, "italic": false, "underline": false, "dim": false, "inverse": false, "fg": "", "bg": ""},
                {"text": "file2.txt", "bold": false, "italic": false, "underline": false, "dim": false, "inverse": false, "fg": "#00ff00", "bg": ""}
            ]
        ]
    }"##;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    
    let result_display = tool_call.result_display.unwrap();
    match result_display {
        ToolResultDisplay::AnsiOutput(output) => {
            assert_eq!(output.len(), 1);
            assert_eq!(output[0].len(), 3);
            assert_eq!(output[0][0].text, "file1.txt");
            assert_eq!(output[0][0].fg, "#0000ff");
        }
        _ => panic!("Expected AnsiOutput variant"),
    }
}

#[test]
fn test_deserialize_tool_call_with_todo_result_display() {
    let json = r#"{
        "id": "todo-123",
        "name": "todo_write",
        "args": {},
        "status": "success",
        "resultDisplay": {
            "todos": [
                {"description": "Task 1", "status": "pending"},
                {"description": "Task 2", "status": "completed"}
            ]
        }
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    
    let result_display = tool_call.result_display.unwrap();
    match result_display {
        ToolResultDisplay::TodoList(list) => {
            assert_eq!(list.todos.len(), 2);
            assert_eq!(list.todos[0].description, "Task 1");
            assert_eq!(list.todos[0].status, TodoStatus::Pending);
        }
        _ => panic!("Expected TodoList variant"),
    }
}

#[test]
fn test_deserialize_tool_call_with_null_result_display() {
    let json = r#"{
        "id": "test-123",
        "name": "read_file",
        "args": {},
        "status": "success",
        "resultDisplay": null
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    assert!(tool_call.result_display.is_none());
}

#[test]
fn test_deserialize_tool_call_without_result_display() {
    let json = r#"{
        "id": "test-123",
        "name": "read_file",
        "args": {},
        "status": "success"
    }"#;

    let tool_call: GeminiToolCall = serde_json::from_str(json).unwrap();
    assert!(tool_call.result_display.is_none());
}
