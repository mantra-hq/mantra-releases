use super::*;
use crate::models::Role;
use chrono::Datelike;

const SIMPLE_CONVERSATION: &str = r#"{
    "sessionId": "test-session-123",
    "projectHash": "abc456def789",
    "startTime": "2025-12-30T20:11:00.000Z",
    "lastUpdated": "2025-12-30T20:15:00.000Z",
    "messages": [
        {
            "id": "msg-1",
            "timestamp": "2025-12-30T20:11:18.207Z",
            "type": "user",
            "content": "Help me with this code"
        },
        {
            "id": "msg-2",
            "timestamp": "2025-12-30T20:13:18.207Z",
            "type": "gemini",
            "content": "I'll help you with that.",
            "model": "gemini-3-pro-preview"
        }
    ]
}"#;

const CONVERSATION_WITH_THOUGHTS: &str = r#"{
    "sessionId": "thought-session",
    "projectHash": "project123",
    "startTime": "2025-12-30T20:00:00.000Z",
    "lastUpdated": "2025-12-30T20:05:00.000Z",
    "messages": [
        {
            "id": "msg-1",
            "timestamp": "2025-12-30T20:00:10.000Z",
            "type": "user",
            "content": "Explain this problem"
        },
        {
            "id": "msg-2",
            "timestamp": "2025-12-30T20:01:00.000Z",
            "type": "gemini",
            "content": "Let me analyze this.",
            "thoughts": [
                {
                    "subject": "Problem Analysis",
                    "description": "The user is asking about understanding a problem.",
                    "timestamp": "2025-12-30T20:00:55.000Z"
                }
            ]
        }
    ]
}"#;

const CONVERSATION_WITH_TOOL_CALLS: &str = r#"{
    "sessionId": "tool-session",
    "projectHash": "tools123",
    "startTime": "2025-12-30T21:00:00.000Z",
    "lastUpdated": "2025-12-30T21:05:00.000Z",
    "messages": [
        {
            "id": "msg-1",
            "timestamp": "2025-12-30T21:00:10.000Z",
            "type": "user",
            "content": "List files in current directory"
        },
        {
            "id": "msg-2",
            "timestamp": "2025-12-30T21:01:00.000Z",
            "type": "gemini",
            "content": "I'll list the files for you.",
            "toolCalls": [
                {
                    "id": "run_shell_command-123",
                    "name": "run_shell_command",
                    "args": {"command": "ls -la"},
                    "result": [
                        {
                            "functionResponse": {
                                "id": "run_shell_command-123",
                                "name": "run_shell_command",
                                "response": {
                                    "output": "total 0\ndrwxr-xr-x 2 user user 40 Dec 30 21:00 ."
                                }
                            }
                        }
                    ],
                    "status": "success",
                    "timestamp": "2025-12-30T21:01:05.000Z"
                }
            ]
        }
    ]
}"#;

const CONVERSATION_WITH_SYSTEM_MESSAGES: &str = r#"{
    "sessionId": "system-session",
    "projectHash": "sys123",
    "startTime": "2025-12-30T22:00:00.000Z",
    "lastUpdated": "2025-12-30T22:05:00.000Z",
    "messages": [
        {
            "id": "info-1",
            "timestamp": "2025-12-30T22:00:00.000Z",
            "type": "info",
            "content": "Session started"
        },
        {
            "id": "msg-1",
            "timestamp": "2025-12-30T22:00:10.000Z",
            "type": "user",
            "content": "Hello"
        },
        {
            "id": "error-1",
            "timestamp": "2025-12-30T22:00:20.000Z",
            "type": "error",
            "content": "API rate limit exceeded"
        },
        {
            "id": "msg-2",
            "timestamp": "2025-12-30T22:01:00.000Z",
            "type": "gemini",
            "content": "Hello! How can I help?"
        },
        {
            "id": "warning-1",
            "timestamp": "2025-12-30T22:02:00.000Z",
            "type": "warning",
            "content": "Token limit approaching"
        }
    ]
}"#;

#[test]
fn test_parse_simple_conversation() {
    let parser = GeminiParser::new();
    let session = parser.parse_string(SIMPLE_CONVERSATION).unwrap();

    assert_eq!(session.id, "test-session-123");
    assert_eq!(session.source, sources::GEMINI);
    assert!(session.cwd.contains("abc456def789"));
    assert_eq!(session.messages.len(), 2);

    // Check user message
    assert_eq!(session.messages[0].role, Role::User);
    assert_eq!(session.messages[0].content_blocks.len(), 1);
    match &session.messages[0].content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "Help me with this code"),
        _ => panic!("Expected Text block"),
    }

    // Check gemini message
    assert_eq!(session.messages[1].role, Role::Assistant);
    assert_eq!(session.metadata.model, Some("gemini-3-pro-preview".to_string()));
}

#[test]
fn test_parse_conversation_with_thoughts() {
    let parser = GeminiParser::new();
    let session = parser.parse_string(CONVERSATION_WITH_THOUGHTS).unwrap();

    assert_eq!(session.messages.len(), 2);

    // Check gemini message with thoughts
    let gemini_msg = &session.messages[1];
    assert_eq!(gemini_msg.role, Role::Assistant);
    assert!(gemini_msg.content_blocks.len() >= 2);

    // First block should be thinking
    match &gemini_msg.content_blocks[0] {
        ContentBlock::Thinking { thinking, subject, timestamp } => {
            assert!(thinking.contains("Problem Analysis"));
            assert!(thinking.contains("understanding a problem"));
            // Verify new fields are populated from Gemini thoughts
            assert_eq!(subject, &Some("Problem Analysis".to_string()));
            assert!(timestamp.is_some());
        }
        _ => panic!("Expected Thinking block"),
    }

    // Second block should be text
    match &gemini_msg.content_blocks[1] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "Let me analyze this."),
        _ => panic!("Expected Text block"),
    }
}

#[test]
fn test_parse_conversation_with_tool_calls() {
    let parser = GeminiParser::new();
    let session = parser.parse_string(CONVERSATION_WITH_TOOL_CALLS).unwrap();

    // 新结构：user + assistant text + assistant tool_action
    assert_eq!(session.messages.len(), 3);

    // 消息 0: user
    assert_eq!(session.messages[0].role, Role::User);

    // 消息 1: assistant 文本消息
    let text_msg = &session.messages[1];
    assert_eq!(text_msg.role, Role::Assistant);
    assert_eq!(text_msg.content_blocks.len(), 1);
    match &text_msg.content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "I'll list the files for you."),
        _ => panic!("Expected Text block"),
    }

    // 消息 2: assistant 工具调用消息 (tool_use + tool_result)
    let tool_msg = &session.messages[2];
    assert_eq!(tool_msg.role, Role::Assistant);
    assert_eq!(tool_msg.content_blocks.len(), 2); // tool_use + tool_result

    // Check ToolUse block
    match &tool_msg.content_blocks[0] {
        ContentBlock::ToolUse { id, name, input, .. } => {
            assert_eq!(id, "run_shell_command-123");
            assert_eq!(name, "run_shell_command");
            assert_eq!(input["command"], "ls -la");
        }
        _ => panic!("Expected ToolUse block"),
    }

    // Check ToolResult block
    match &tool_msg.content_blocks[1] {
        ContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
            ..
        } => {
            assert_eq!(tool_use_id, "run_shell_command-123");
            assert!(content.contains("drwxr-xr-x"));
            assert!(!is_error);
        }
        _ => panic!("Expected ToolResult block"),
    }
}

#[test]
fn test_system_messages_filtered() {
    let parser = GeminiParser::new();
    let session = parser.parse_string(CONVERSATION_WITH_SYSTEM_MESSAGES).unwrap();

    // Only user and gemini messages should be included
    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, Role::User);
    assert_eq!(session.messages[1].role, Role::Assistant);
}

#[test]
fn test_parse_with_project_path() {
    let parser = GeminiParser::with_project_path("/home/user/my-project".to_string());
    let session = parser.parse_string(SIMPLE_CONVERSATION).unwrap();

    assert_eq!(session.cwd, "/home/user/my-project");
}

#[test]
fn test_parse_empty_session_id_fails() {
    let json = r#"{
        "sessionId": "",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": []
    }"#;

    let parser = GeminiParser::new();
    let result = parser.parse_string(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_json_fails() {
    let parser = GeminiParser::new();
    let result = parser.parse_string("not valid json");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_messages_ok() {
    let json = r#"{
        "sessionId": "test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z"
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();
    assert!(session.messages.is_empty());
}

#[test]
fn test_parse_empty_content_message() {
    let json = r#"{
        "sessionId": "test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "gemini",
                "content": "",
                "toolCalls": [
                    {
                        "id": "tool-1",
                        "name": "read_file",
                        "args": {"path": "/tmp/test"},
                        "status": "success"
                    }
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();
    // Message should exist with just the tool call
    assert_eq!(session.messages.len(), 1);
    // Should have ToolUse but no Text block
    let msg = &session.messages[0];
    assert!(msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. })));
    assert!(!msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::Text { .. })));
}

#[test]
fn test_parse_timestamp_formats() {
    // RFC 3339 with timezone
    let ts = parse_timestamp("2025-12-30T20:11:00.000Z").unwrap();
    assert_eq!(ts.year(), 2025);

    // RFC 3339 with offset
    let ts = parse_timestamp("2025-12-30T20:11:00.000+08:00").unwrap();
    assert_eq!(ts.year(), 2025);
}

#[test]
fn test_parse_error_tool_call() {
    let json = r#"{
        "sessionId": "error-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "gemini",
                "content": "Let me try that.",
                "toolCalls": [
                    {
                        "id": "tool-1",
                        "name": "run_command",
                        "args": {"command": "rm -rf /"},
                        "result": [
                            {
                                "functionResponse": {
                                    "id": "tool-1",
                                    "name": "run_command",
                                    "response": {
                                        "error": "Permission denied"
                                    }
                                }
                            }
                        ],
                        "status": "error"
                    }
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // 新结构：消息 0 是文本，消息 1 是工具调用
    assert_eq!(session.messages.len(), 2);

    let tool_msg = &session.messages[1];
    let tool_result = tool_msg.content_blocks.iter().find(|b| matches!(b, ContentBlock::ToolResult { .. }));
    assert!(tool_result.is_some());

    match tool_result.unwrap() {
        ContentBlock::ToolResult {
            is_error, content, ..
        } => {
            assert!(is_error);
            assert!(content.contains("Permission denied"));
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_summary_becomes_title() {
    let json = r#"{
        "sessionId": "summary-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [],
        "summary": "Discussion about Rust parsing"
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();
    assert_eq!(
        session.metadata.title,
        Some("Discussion about Rust parsing".to_string())
    );
}

#[test]
fn test_content_with_parts_array() {
    let json = r#"{
        "sessionId": "parts-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "gemini",
                "content": [
                    {"text": "First part. "},
                    {"text": "Second part."}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    match &session.messages[0].content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "First part. Second part."),
        _ => panic!("Expected Text block"),
    }
}

#[test]
fn test_mentioned_files_extracted_from_tool_calls() {
    let json = r#"{
        "sessionId": "files-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "gemini",
                "content": "I'll read the file.",
                "toolCalls": [
                    {
                        "id": "read-1",
                        "name": "read_file",
                        "args": {"path": "/src/main.rs"},
                        "status": "success"
                    },
                    {
                        "id": "write-1",
                        "name": "write_file",
                        "args": {"target_file": "/src/lib.rs", "content": "test"},
                        "status": "success"
                    }
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // 新结构：消息 0 是文本，消息 1 和 2 是工具调用
    assert_eq!(session.messages.len(), 3);

    // 工具调用消息各自包含自己的 mentioned_files
    let tool_msg_1 = &session.messages[1];
    assert!(tool_msg_1.mentioned_files.contains(&"/src/main.rs".to_string()));

    let tool_msg_2 = &session.messages[2];
    assert!(tool_msg_2.mentioned_files.contains(&"/src/lib.rs".to_string()));
}

#[test]
fn test_total_tokens_aggregated() {
    let json = r#"{
        "sessionId": "tokens-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "user",
                "content": "Hello"
            },
            {
                "id": "msg-2",
                "timestamp": "2025-12-30T20:01:00.000Z",
                "type": "gemini",
                "content": "Hi there!",
                "tokens": {"input": 100, "output": 50}
            },
            {
                "id": "msg-3",
                "timestamp": "2025-12-30T20:02:00.000Z",
                "type": "gemini",
                "content": "Here's more info.",
                "tokens": {"input": 200, "output": 150}
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // Should aggregate: 100 + 50 + 200 + 150 = 500
    assert_eq!(session.metadata.total_tokens, Some(500));
}

#[test]
fn test_parse_real_world_tool_call_format() {
    // Based on actual Gemini CLI output format
    let json = r#"{
        "sessionId": "8c9a7d96-6b65-4e36-9540-0484bc3e3eb2",
        "projectHash": "3f39cb10e8c4f4196f80e8daa74c4d17f88708c17dcf1ae0d56da82a11a9f941",
        "startTime": "2025-12-30T20:11:51.773Z",
        "lastUpdated": "2025-12-30T20:23:26.927Z",
        "messages": [
            {
                "id": "61a5e2cb-2840-4249-93cb-3406903fa0e1",
                "timestamp": "2025-12-30T20:11:51.774Z",
                "type": "user",
                "content": "请帮我分析这个问题"
            },
            {
                "id": "b20dd04d-74a5-47dd-913c-8232f30231b9",
                "timestamp": "2025-12-30T20:12:02.838Z",
                "type": "gemini",
                "content": "我来分析一下",
                "thoughts": [
                    {
                        "subject": "Examining System",
                        "description": "Analyzing the problem",
                        "timestamp": "2025-12-30T20:11:55.073Z"
                    }
                ],
                "tokens": {
                    "input": 6926,
                    "output": 246,
                    "cached": 0,
                    "thoughts": 651,
                    "tool": 0,
                    "total": 7823
                },
                "model": "gemini-3-pro-preview",
                "toolCalls": [
                    {
                        "id": "run_shell_command-1767125522547",
                        "name": "run_shell_command",
                        "args": {
                            "command": "hostnamectl"
                        },
                        "result": [
                            {
                                "functionResponse": {
                                    "id": "run_shell_command-1767125522547",
                                    "name": "run_shell_command",
                                    "response": {
                                        "output": "Static hostname: test-machine"
                                    }
                                }
                            }
                        ],
                        "status": "success",
                        "timestamp": "2025-12-30T20:12:41.239Z",
                        "displayName": "Shell",
                        "description": "Execute shell commands",
                        "resultDisplay": "Static hostname: test-machine",
                        "renderOutputAsMarkdown": false
                    }
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert_eq!(session.id, "8c9a7d96-6b65-4e36-9540-0484bc3e3eb2");
    // 新结构：user + assistant text/thinking + assistant tool_action
    assert_eq!(session.messages.len(), 3);

    // 消息 1: 文本消息，包含 thinking
    let text_msg = &session.messages[1];
    let has_thinking = text_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::Thinking { .. }));
    assert!(has_thinking, "Should have thinking block");

    // 消息 2: 工具调用消息
    let tool_msg = &session.messages[2];
    let has_tool_use = tool_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }));
    let has_tool_result = tool_msg.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolResult { .. }));
    assert!(has_tool_use, "Should have tool use block");
    assert!(has_tool_result, "Should have tool result block");

    // Check tokens
    assert_eq!(session.metadata.total_tokens, Some(7823));
}

// === Story 8-7 New Tests ===

#[test]
fn test_parse_tokens_breakdown() {
    // AC1: Token 细分解析测试
    let json = r#"{
        "sessionId": "tokens-breakdown-test",
        "projectHash": "abc123",
        "startTime": "2026-01-09T10:00:00Z",
        "lastUpdated": "2026-01-09T10:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2026-01-09T10:01:00Z",
                "type": "gemini",
                "content": "Test",
                "tokens": {
                    "input": 100,
                    "output": 50,
                    "cached": 10,
                    "thoughts": 20,
                    "tool": 5,
                    "total": 185
                }
            },
            {
                "id": "m2",
                "timestamp": "2026-01-09T10:02:00Z",
                "type": "gemini",
                "content": "More",
                "tokens": {
                    "input": 200,
                    "output": 100,
                    "cached": 20,
                    "thoughts": 40,
                    "tool": 10,
                    "total": 370
                }
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // Verify tokens_breakdown is populated with accumulated values
    let tb = session.metadata.tokens_breakdown.as_ref().expect("tokens_breakdown should exist");
    assert_eq!(tb.input, Some(300));   // 100 + 200
    assert_eq!(tb.output, Some(150));  // 50 + 100
    assert_eq!(tb.cached, Some(30));   // 10 + 20
    assert_eq!(tb.thoughts, Some(60)); // 20 + 40
    assert_eq!(tb.tool, Some(15));     // 5 + 10
}

#[test]
fn test_parse_display_metadata() {
    // AC2: 显示元数据映射测试
    let json = r#"{
        "sessionId": "display-metadata-test",
        "projectHash": "abc",
        "startTime": "2026-01-09T10:00:00Z",
        "lastUpdated": "2026-01-09T10:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2026-01-09T10:01:00Z",
                "type": "gemini",
                "content": "",
                "toolCalls": [{
                    "id": "t1",
                    "name": "run_shell_command",
                    "args": {"command": "ls"},
                    "displayName": "Shell Command",
                    "description": "Execute shell commands in terminal",
                    "resultDisplay": "file1.txt\nfile2.txt",
                    "renderOutputAsMarkdown": false,
                    "status": "success",
                    "result": [{
                        "functionResponse": {
                            "id": "t1",
                            "name": "run_shell_command",
                            "response": {"output": "file1.txt\nfile2.txt"}
                        }
                    }]
                }]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    let tool_msg = &session.messages[0];
    
    // Check ToolUse display fields
    if let ContentBlock::ToolUse { display_name, description, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(display_name, &Some("Shell Command".to_string()));
        assert_eq!(description, &Some("Execute shell commands in terminal".to_string()));
    } else {
        panic!("Expected ToolUse block");
    }

    // Check ToolResult display fields
    if let ContentBlock::ToolResult { display_content, render_as_markdown, .. } = &tool_msg.content_blocks[1] {
        assert_eq!(display_content, &Some("file1.txt\nfile2.txt".to_string()));
        assert_eq!(render_as_markdown, &Some(false));
    } else {
        panic!("Expected ToolResult block");
    }
}

#[test]
fn test_standard_tool_mapping() {
    // AC3: StandardTool 映射测试
    use crate::models::StandardTool;
    
    let json = r#"{
        "sessionId": "standard-tool-test",
        "projectHash": "abc",
        "startTime": "2026-01-09T10:00:00Z",
        "lastUpdated": "2026-01-09T10:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2026-01-09T10:01:00Z",
                "type": "gemini",
                "content": "",
                "toolCalls": [
                    {
                        "id": "t1",
                        "name": "read_file",
                        "args": {"path": "/tmp/test.rs", "start_line": 1, "end_line": 100},
                        "status": "success"
                    },
                    {
                        "id": "t2",
                        "name": "write_file",
                        "args": {"path": "/tmp/out.rs", "content": "fn main() {}"},
                        "status": "success"
                    },
                    {
                        "id": "t3",
                        "name": "run_shell_command",
                        "args": {"command": "cargo build"},
                        "status": "success"
                    },
                    {
                        "id": "t4",
                        "name": "grep",
                        "args": {"pattern": "TODO", "path": "/src"},
                        "status": "success"
                    }
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // 4 tool calls = 4 messages
    assert_eq!(session.messages.len(), 4);

    // Check read_file -> FileRead
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
        match standard_tool.as_ref().unwrap() {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert_eq!(start_line, &Some(1));
                assert_eq!(end_line, &Some(100));
            }
            _ => panic!("Expected FileRead"),
        }
    }

    // Check write_file -> FileWrite
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[1].content_blocks[0] {
        match standard_tool.as_ref().unwrap() {
            StandardTool::FileWrite { path, content } => {
                assert_eq!(path, "/tmp/out.rs");
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected FileWrite"),
        }
    }

    // Check run_shell_command -> ShellExec
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[2].content_blocks[0] {
        match standard_tool.as_ref().unwrap() {
            StandardTool::ShellExec { command, .. } => {
                assert_eq!(command, "cargo build");
            }
            _ => panic!("Expected ShellExec"),
        }
    }

    // Check grep -> ContentSearch
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[3].content_blocks[0] {
        match standard_tool.as_ref().unwrap() {
            StandardTool::ContentSearch { pattern, path } => {
                assert_eq!(pattern, "TODO");
                assert_eq!(path, &Some("/src".to_string()));
            }
            _ => panic!("Expected ContentSearch"),
        }
    }
}

#[test]
fn test_source_metadata_passthrough() {
    // AC4: source_metadata 透传测试
    let json = r#"{
        "sessionId": "source-metadata-test",
        "projectHash": "abc123def456",
        "startTime": "2026-01-09T10:00:00Z",
        "lastUpdated": "2026-01-09T10:05:00Z",
        "messages": []
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    let source_meta = session.metadata.source_metadata.as_ref().expect("source_metadata should exist");
    assert_eq!(source_meta["project_hash"], "abc123def456");
}

#[test]
fn test_backward_compatibility_no_new_fields() {
    // AC5: 向后兼容测试 - 旧日志不包含新字段
    let json = r#"{
        "sessionId": "old-format-test",
        "projectHash": "",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": "Hello"
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // All new fields should be None
    assert!(session.metadata.tokens_breakdown.is_none());
    assert!(session.metadata.source_metadata.is_none());
    
    // Existing functionality should work
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.id, "old-format-test");
}

#[test]
fn test_backward_compatibility_partial_tokens() {
    // AC5: 向后兼容测试 - 部分 tokens 字段
    let json = r#"{
        "sessionId": "partial-tokens-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": "Hello",
                "tokens": {"input": 100, "output": 50}
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    let tb = session.metadata.tokens_breakdown.as_ref().expect("tokens_breakdown should exist");
    assert_eq!(tb.input, Some(100));
    assert_eq!(tb.output, Some(50));
    assert!(tb.cached.is_none());
    assert!(tb.thoughts.is_none());
    assert!(tb.tool.is_none());
}

// ========== Story 8.15: Parser Resilience Enhancement Tests ==========

#[test]
fn test_parser_info_included() {
    let json = r#"{
        "sessionId": "parser-info-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": "Hello",
                "model": "gemini-2.0-flash"
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert!(session.metadata.parser_info.is_some());
    let parser_info = session.metadata.parser_info.as_ref().unwrap();

    assert_eq!(parser_info.parser_version, GEMINI_PARSER_VERSION);
    assert!(parser_info.supported_formats.contains(&"text".to_string()));
    assert!(parser_info.supported_formats.contains(&"inline_data".to_string()));
    assert!(parser_info.supported_formats.contains(&"function_call".to_string()));
    assert!(parser_info.supported_formats.contains(&"function_response".to_string()));
    assert_eq!(parser_info.detected_source_version, Some("gemini-2.0-flash".to_string()));
}

#[test]
fn test_unknown_part_fields_detected() {
    // Test that unknown fields in content parts are detected and recorded
    let json = r#"{
        "sessionId": "unknown-fields-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": [
                    {"text": "Normal text"},
                    {"newFeature": "some data", "anotherNew": 123}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // Should have unknown_formats recorded
    assert!(session.metadata.unknown_formats.is_some());
    let unknown_formats = session.metadata.unknown_formats.as_ref().unwrap();

    // Should have 2 unknown fields: newFeature and anotherNew
    assert_eq!(unknown_formats.len(), 2);
    assert!(unknown_formats.iter().any(|e| e.type_name == "newFeature" && e.source == "gemini"));
    assert!(unknown_formats.iter().any(|e| e.type_name == "anotherNew" && e.source == "gemini"));
}

#[test]
fn test_no_unknown_formats_when_all_known() {
    let json = r#"{
        "sessionId": "known-fields-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": [
                    {"text": "Only known text field"}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // unknown_formats should be None when no unknown fields
    assert!(session.metadata.unknown_formats.is_none());
}

#[test]
fn test_degraded_content_blocks_created() {
    let json = r#"{
        "sessionId": "degraded-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00Z",
        "lastUpdated": "2025-12-30T20:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2025-12-30T20:01:00Z",
                "type": "gemini",
                "content": [
                    {"unknownType": "data"}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    // Should have a message with a degraded content block
    assert_eq!(session.messages.len(), 1);
    assert!(!session.messages[0].content_blocks.is_empty());

    // Find the degraded block
    let degraded_block = session.messages[0].content_blocks.iter()
        .find(|b| matches!(b, ContentBlock::Text { is_degraded: Some(true), .. }));

    assert!(degraded_block.is_some(), "Should have a degraded text block");
    if let Some(ContentBlock::Text { text, .. }) = degraded_block {
        assert!(text.contains("unknownType"), "Degraded block should mention unknown field name");
    }
}

// ========== Story 8.16: Gemini inline_data Image Parsing Tests ==========

#[test]
fn test_parse_inline_data_image() {
    // Test that inline_data in GeminiPart is correctly parsed as ContentBlock::Image
    let json = r#"{
        "sessionId": "image-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "user",
                "content": [
                    {"text": "Here is an image: "},
                    {"inlineData": {"mimeType": "image/png", "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="}}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert_eq!(session.messages.len(), 1);
    let msg = &session.messages[0];

    // Should have 2 content blocks: Text and Image
    assert_eq!(msg.content_blocks.len(), 2);

    // Check Text block
    match &msg.content_blocks[0] {
        ContentBlock::Image { media_type, data, source_type, alt_text } => {
            assert_eq!(media_type, "image/png");
            assert!(data.contains("iVBORw0KGgo"));
            assert_eq!(source_type, &Some("base64".to_string()));
            assert_eq!(alt_text, &None);
        }
        _ => panic!("Expected Image block first (inline_data processed before text)"),
    }

    // Check that text is also present
    match &msg.content_blocks[1] {
        ContentBlock::Text { text, .. } => {
            assert!(text.contains("Here is an image"));
        }
        _ => panic!("Expected Text block second"),
    }
}

#[test]
fn test_parse_inline_data_non_image_ignored() {
    // Test that non-image inline_data is NOT parsed as Image block
    let json = r#"{
        "sessionId": "non-image-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "user",
                "content": [
                    {"text": "Here is a file: "},
                    {"inlineData": {"mimeType": "application/pdf", "data": "JVBERi0xLjQ="}}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert_eq!(session.messages.len(), 1);
    let msg = &session.messages[0];

    // Should have only 1 content block (Text) since non-image is ignored
    assert_eq!(msg.content_blocks.len(), 1);

    match &msg.content_blocks[0] {
        ContentBlock::Text { text, .. } => {
            assert!(text.contains("Here is a file"));
        }
        _ => panic!("Expected only Text block for non-image inline_data"),
    }
}

#[test]
fn test_parse_inline_data_missing_fields() {
    // Test that incomplete inline_data (missing fields) is gracefully handled
    let json = r#"{
        "sessionId": "incomplete-test",
        "projectHash": "abc",
        "startTime": "2025-12-30T20:00:00.000Z",
        "lastUpdated": "2025-12-30T20:00:00.000Z",
        "messages": [
            {
                "id": "msg-1",
                "timestamp": "2025-12-30T20:00:10.000Z",
                "type": "user",
                "content": [
                    {"text": "Incomplete image: "},
                    {"inlineData": {"mimeType": "image/png"}}
                ]
            }
        ]
    }"#;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert_eq!(session.messages.len(), 1);
    let msg = &session.messages[0];

    // Should have only 1 content block (Text) since image is incomplete
    assert_eq!(msg.content_blocks.len(), 1);

    match &msg.content_blocks[0] {
        ContentBlock::Text { text, .. } => {
            assert!(text.contains("Incomplete image"));
        }
        _ => panic!("Expected only Text block for incomplete inline_data"),
    }
}

#[test]
fn test_read_file_with_empty_result_display() {
    // Test that when resultDisplay is empty string, content still comes from response.output
    // This is the real-world Gemini CLI behavior for read_file
    let json = r##"{
        "sessionId": "readfile-test",
        "projectHash": "abc",
        "startTime": "2026-01-23T10:00:00Z",
        "lastUpdated": "2026-01-23T10:05:00Z",
        "messages": [
            {
                "id": "m1",
                "timestamp": "2026-01-23T10:01:00Z",
                "type": "gemini",
                "content": "",
                "toolCalls": [{
                    "id": "read_file-1",
                    "name": "read_file",
                    "args": {"file_path": "test.txt"},
                    "status": "success",
                    "resultDisplay": "",
                    "result": [{
                        "functionResponse": {
                            "id": "read_file-1",
                            "name": "read_file",
                            "response": {"output": "#!/bin/bash\necho 'hello world'\nexit 0"}
                        }
                    }]
                }]
            }
        ]
    }"##;

    let parser = GeminiParser::new();
    let session = parser.parse_string(json).unwrap();

    assert_eq!(session.messages.len(), 1);
    let tool_msg = &session.messages[0];
    
    // Should have ToolUse and ToolResult
    assert_eq!(tool_msg.content_blocks.len(), 2);

    // Check ToolResult content is from response.output, NOT from empty resultDisplay
    if let ContentBlock::ToolResult { content, display_content, .. } = &tool_msg.content_blocks[1] {
        // Content should have the actual file content
        assert!(content.contains("#!/bin/bash"), "Content should contain file content, got: {}", content);
        assert!(content.contains("echo 'hello world'"), "Content should contain file content");
        // display_content should be the empty string from resultDisplay
        assert_eq!(display_content, &Some("".to_string()));
    } else {
        panic!("Expected ToolResult block");
    }
}
