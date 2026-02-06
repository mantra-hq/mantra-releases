use super::*;

const SIMPLE_CONVERSATION: &str = r#"{
    "id": "conv_123",
    "cwd": "/home/user/project",
    "messages": [
        {
            "role": "user",
            "content": "Hello, please help me with my code."
        },
        {
            "role": "assistant",
            "content": "Of course! I'd be happy to help. What do you need?"
        }
    ]
}"#;

const CONVERSATION_WITH_BLOCKS: &str = r#"{
    "id": "conv_456",
    "cwd": "/tmp/test",
    "model": "claude-3-opus",
    "title": "Code Help Session",
    "messages": [
        {
            "role": "user",
            "content": [
                {"type": "text", "text": "Please read this file"}
            ]
        },
        {
            "role": "assistant",
            "content": [
                {"type": "thinking", "thinking": "The user wants me to read a file..."},
                {"type": "text", "text": "I'll read the file for you."},
                {"type": "tool_use", "id": "tool_1", "name": "read_file", "input": {"path": "main.rs"}}
            ]
        },
        {
            "role": "user",
            "content": [
                {"type": "tool_result", "tool_use_id": "tool_1", "content": "fn main() {}", "is_error": false}
            ]
        }
    ]
}"#;

#[test]
fn test_parse_simple_conversation() {
    let parser = ClaudeParser::new();
    let result = parser.parse_string(SIMPLE_CONVERSATION);
    assert!(result.is_ok());

    let session = result.unwrap();
    assert_eq!(session.id, "conv_123");
    assert_eq!(session.source, sources::CLAUDE);
    assert_eq!(session.cwd, "/home/user/project");
    assert_eq!(session.messages.len(), 2);

    // Check first message
    assert_eq!(session.messages[0].role, Role::User);
    assert_eq!(session.messages[0].content_blocks.len(), 1);

    // Check second message
    assert_eq!(session.messages[1].role, Role::Assistant);
}

#[test]
fn test_parse_conversation_with_blocks() {
    let parser = ClaudeParser::new();
    let result = parser.parse_string(CONVERSATION_WITH_BLOCKS);
    assert!(result.is_ok());

    let session = result.unwrap();
    assert_eq!(session.id, "conv_456");
    assert_eq!(session.metadata.model, Some("claude-3-opus".to_string()));
    assert_eq!(
        session.metadata.title,
        Some("Code Help Session".to_string())
    );

    // 新结构：user + assistant text + assistant tool_action
    // 原始 tool_result user 消息被合并到工具调用消息中
    assert_eq!(session.messages.len(), 3);

    // 消息 0: user
    assert_eq!(session.messages[0].role, Role::User);

    // 消息 1: assistant (包含 thinking + text + tool_use 或单独的消息)
    // 由于消息结构可能变化，检查是否包含预期内容
    let assistant_msgs: Vec<_> = session.messages.iter().filter(|m| m.role == Role::Assistant).collect();
    assert!(assistant_msgs.len() >= 1);
    
    // 检查是否存在 thinking block
    let has_thinking = session.messages.iter().any(|m| {
        m.content_blocks.iter().any(|b| matches!(b, ContentBlock::Thinking { .. }))
    });
    assert!(has_thinking, "Should have thinking block");

    // 检查是否存在 text block
    let has_text = session.messages.iter().any(|m| {
        m.content_blocks.iter().any(|b| matches!(b, ContentBlock::Text { .. }))
    });
    assert!(has_text, "Should have text block");

    // 检查是否存在 tool_use block
    let has_tool_use = session.messages.iter().any(|m| {
        m.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }))
    });
    assert!(has_tool_use, "Should have tool_use block");

    // 检查是否存在 tool_result block
    let has_tool_result = session.messages.iter().any(|m| {
        m.content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolResult { .. }))
    });
    assert!(has_tool_result, "Should have tool_result block");
}

#[test]
fn test_parse_empty_id_fails() {
    let parser = ClaudeParser::new();
    let json = r#"{"id": "", "messages": []}"#;
    let result = parser.parse_string(json);
    assert!(matches!(result, Err(ParseError::MissingField(_))));
}

#[test]
fn test_parse_invalid_json_fails() {
    let parser = ClaudeParser::new();
    let result = parser.parse_string("{ invalid json }");
    assert!(matches!(result, Err(ParseError::InvalidJson(_))));
}

#[test]
fn test_parse_missing_messages_ok() {
    let parser = ClaudeParser::new();
    let json = r#"{"id": "test_123"}"#;
    let result = parser.parse_string(json);
    assert!(result.is_ok());

    let session = result.unwrap();
    assert_eq!(session.id, "test_123");
    assert_eq!(session.messages.len(), 0);
}

#[test]
fn test_unknown_role_skipped() {
    let parser = ClaudeParser::new();
    let json = r#"{
        "id": "test",
        "messages": [
            {"role": "system", "content": "You are an AI assistant"},
            {"role": "user", "content": "Hello"}
        ]
    }"#;
    let result = parser.parse_string(json);
    assert!(result.is_ok());

    let session = result.unwrap();
    // Only user message should be included, system role is skipped
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, Role::User);
}

#[test]
fn test_parse_jsonl_with_summary() {
    // Simulate Claude Code JSONL format with summary record
    let jsonl = r#"{"type":"summary","summary":"Test Session Title","leafUuid":"abc123"}
{"parentUuid":"root","isSidechain":false,"userType":"external","cwd":"/test/project","sessionId":"sess-001","version":"2.0.76","gitBranch":"","message":{"role":"user","content":"Hello"},"type":"user","uuid":"msg-1","timestamp":"2024-01-01T00:00:00Z"}
{"parentUuid":"msg-1","isSidechain":false,"userType":"external","cwd":"/test/project","sessionId":"sess-001","version":"2.0.76","gitBranch":"","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}]},"type":"assistant","uuid":"msg-2","timestamp":"2024-01-01T00:00:01Z"}"#;

    let parser = ClaudeParser::new();
    let result = parser.parse_jsonl(jsonl);
    assert!(result.is_ok());

    let session = result.unwrap();
    assert_eq!(session.id, "sess-001");
    assert_eq!(session.cwd, "/test/project");
    assert_eq!(session.metadata.title, Some("Test Session Title".to_string()));
    assert_eq!(session.messages.len(), 2);
}

#[test]
#[ignore] // 依赖本地真实会话文件，仅用于手动调试
fn test_parse_real_problematic_file() {
    let file_path = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-capsule/4fe9325e-4c69-4633-ac6f-d879ca16d6c5.jsonl";

    let content = std::fs::read_to_string(file_path).expect("Failed to read file");
    println!("\n=== DEBUG: File Info ===");
    println!("Content length: {} bytes", content.len());
    println!("Lines: {}", content.lines().count());

    // 使用 parse_file 而不是 parse_string（这是实际导入流程使用的方法）
    let parser = ClaudeParser::new();
    let result = parser.parse_file(file_path);

    match result {
        Ok(session) => {
            println!("\n=== DEBUG: Parse Result ===");
            println!("Session ID: {}", session.id);
            println!("Messages: {}", session.messages.len());

            for (i, msg) in session.messages.iter().enumerate() {
                let block_types: Vec<&str> = msg.content_blocks.iter().map(|b| {
                    match b {
                        ContentBlock::Text { .. } => "text",
                        ContentBlock::Thinking { .. } => "thinking",
                        ContentBlock::ToolUse { .. } => "tool_use",
                        ContentBlock::ToolResult { .. } => "tool_result",
                        _ => "other",
                    }
                }).collect();
                println!("  Msg {}: {:?} - {:?}", i + 1, msg.role, block_types);
            }

            // 期望 12 条消息
            assert!(session.messages.len() >= 10, "Expected at least 10 messages, got {}", session.messages.len());
        }
        Err(e) => {
            panic!("Parse failed: {:?}", e);
        }
    }
}

#[test]
fn test_parse_empty_session_files() {
    let parser = ClaudeParser::new();

    // Test file with only file-history-snapshot records
    // Story 2.29 V2: Returns empty session (not error) for system-events-only files
    let file1 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/1239d15e-5b17-4607-961f-ba103d232021.jsonl";
    if std::path::Path::new(file1).exists() {
        let result = parser.parse_file(file1);
        println!("\nFile 1 (file-history-snapshot only):");
        println!("  Result: {:?}", result);
        // Story 2.29 V2: Returns empty session instead of error
        assert!(result.is_ok(), "Should return empty session for file-history-snapshot only file");
        if let Ok(session) = result {
            assert!(session.messages.is_empty(), "Session should have no messages");
            assert!(session.is_empty(), "Session should be marked as empty");
        }
    }

    // Test file with only summary record
    // Story 2.29 V2: Returns empty session (not error) for summary-only files
    let file2 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/b7485bbe-3a7d-460c-8452-54ec4ce4a3a5.jsonl";
    if std::path::Path::new(file2).exists() {
        let result = parser.parse_file(file2);
        println!("\nFile 2 (summary only):");
        println!("  Result: {:?}", result);
        // Story 2.29 V2: Returns empty session instead of error
        assert!(result.is_ok(), "Should return empty session for summary only file");
        if let Ok(session) = result {
            assert!(session.messages.is_empty(), "Session should have no messages");
            assert!(session.is_empty(), "Session should be marked as empty");
        }
    }

    // Test file with actual conversation
    let file3 = "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-input-poc/06e56ded-b41d-4904-9760-f83361dd76ae.jsonl";
    if std::path::Path::new(file3).exists() {
        let result = parser.parse_file(file3);
        println!("\nFile 3 (real conversation):");
        println!("  Result: {:?}", result.as_ref().map(|s| format!("Ok({} messages)", s.messages.len())));
        assert!(result.is_ok(), "Should successfully parse file with real conversation");
        if let Ok(session) = result {
            assert!(!session.messages.is_empty(), "Should have messages");
            println!("  Messages: {}", session.messages.len());
        }
    }
}

// ========== Story 8-6: Claude Parser Adaptation Tests ==========

#[test]
fn test_parse_jsonl_message_tree_structure() {
    // Test AC1: Message tree structure (uuid, parentUuid, isSidechain)
    let jsonl = r#"{"type":"user","sessionId":"s1","uuid":"msg-001","parentUuid":"msg-000","isSidechain":true,"cwd":"/test","message":{"role":"user","content":"Hello"},"timestamp":"2024-01-01T00:00:00Z"}
{"type":"assistant","sessionId":"s1","uuid":"msg-002","parentUuid":"msg-001","isSidechain":false,"cwd":"/test","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}]},"timestamp":"2024-01-01T00:00:01Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 2);

    // Check first message (user)
    assert_eq!(session.messages[0].message_id, Some("msg-001".to_string()));
    assert_eq!(session.messages[0].parent_id, Some("msg-000".to_string()));
    assert!(session.messages[0].is_sidechain);

    // Check second message (assistant)
    assert_eq!(session.messages[1].message_id, Some("msg-002".to_string()));
    assert_eq!(session.messages[1].parent_id, Some("msg-001".to_string()));
    assert!(!session.messages[1].is_sidechain);
}

#[test]
fn test_parse_jsonl_message_tree_backward_compatible() {
    // Test AC5: Backward compatibility - missing tree fields default to None/false
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":"Hello"},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].message_id, None);
    assert_eq!(session.messages[0].parent_id, None);
    assert!(!session.messages[0].is_sidechain);
}

#[test]
fn test_parse_jsonl_git_branch() {
    // Test AC2: Git information extraction
    let jsonl = r#"{"type":"user","sessionId":"s1","gitBranch":"feature/test-branch","cwd":"/test","message":{"role":"user","content":"Hi"},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert!(session.metadata.git.is_some());
    let git = session.metadata.git.unwrap();
    assert_eq!(git.branch, Some("feature/test-branch".to_string()));
    assert_eq!(git.commit, None);
    assert_eq!(git.repository_url, None);
}

#[test]
fn test_parse_jsonl_git_branch_empty_ignored() {
    // Test AC5: Empty gitBranch should not create GitInfo
    let jsonl = r#"{"type":"user","sessionId":"s1","gitBranch":"","cwd":"/test","message":{"role":"user","content":"Hi"},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert!(session.metadata.git.is_none());
}

#[test]
fn test_parse_jsonl_standard_tool_read() {
    // Test AC3: StandardTool mapping - Read
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/src/main.rs","offset":10,"limit":50}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
        assert!(standard_tool.is_some());
        if let Some(crate::models::StandardTool::FileRead { path, start_line, end_line }) = standard_tool {
            assert_eq!(path, "/src/main.rs");
            assert_eq!(*start_line, Some(10));
            assert_eq!(*end_line, Some(60)); // offset 10 + limit 50
        } else {
            panic!("Expected StandardTool::FileRead");
        }
    } else {
        panic!("Expected ToolUse content block");
    }
}

#[test]
fn test_parse_jsonl_standard_tool_bash() {
    // Test AC3: StandardTool mapping - Bash
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls -la","cwd":"/tmp"}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
        if let Some(crate::models::StandardTool::ShellExec { command, cwd }) = standard_tool {
            assert_eq!(command, "ls -la");
            assert_eq!(*cwd, Some("/tmp".to_string()));
        } else {
            panic!("Expected StandardTool::ShellExec");
        }
    }
}

#[test]
fn test_parse_jsonl_standard_tool_glob_grep() {
    // Test AC3: StandardTool mapping - Glob and Grep
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Glob","input":{"pattern":"*.rs","path":"/src"}},{"type":"tool_use","id":"t2","name":"Grep","input":{"pattern":"TODO","path":"/project"}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    // Check Glob
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[0] {
        if let Some(crate::models::StandardTool::FileSearch { pattern, path }) = standard_tool {
            assert_eq!(pattern, "*.rs");
            assert_eq!(*path, Some("/src".to_string()));
        } else {
            panic!("Expected StandardTool::FileSearch");
        }
    }

    // Check Grep
    if let ContentBlock::ToolUse { standard_tool, .. } = &session.messages[0].content_blocks[1] {
        if let Some(crate::models::StandardTool::ContentSearch { pattern, path }) = standard_tool {
            assert_eq!(pattern, "TODO");
            assert_eq!(*path, Some("/project".to_string()));
        } else {
            panic!("Expected StandardTool::ContentSearch");
        }
    }
}

#[test]
fn test_parse_tool_use_result_file_read() {
    // Test AC4: toolUseResult parsing - FileRead
    let tool_use_result = serde_json::json!({
        "file": {
            "filePath": "/src/main.rs",
            "startLine": 1,
            "numLines": 50,
            "totalLines": 100
        }
    });

    let result = parse_tool_use_result(&tool_use_result);
    assert!(result.is_some());

    if let Some(ToolResultData::FileRead { file_path, start_line, num_lines, total_lines }) = result {
        assert_eq!(file_path, "/src/main.rs");
        assert_eq!(start_line, Some(1));
        assert_eq!(num_lines, Some(50));
        assert_eq!(total_lines, Some(100));
    } else {
        panic!("Expected ToolResultData::FileRead");
    }
}

#[test]
fn test_parse_tool_use_result_other() {
    // Test AC4: toolUseResult parsing - Other (passthrough)
    let tool_use_result = serde_json::json!({
        "custom": {
            "some_field": "some_value"
        }
    });

    let result = parse_tool_use_result(&tool_use_result);
    assert!(result.is_some());

    if let Some(ToolResultData::Other { data }) = result {
        assert_eq!(data.get("custom").unwrap().get("some_field").unwrap(), "some_value");
    } else {
        panic!("Expected ToolResultData::Other");
    }
}

#[test]
fn test_parse_tool_use_result_empty() {
    // Test AC4: Empty toolUseResult returns None
    let tool_use_result = serde_json::json!({});
    let result = parse_tool_use_result(&tool_use_result);
    assert!(result.is_none());

    let tool_use_result_null = serde_json::Value::Null;
    let result_null = parse_tool_use_result(&tool_use_result_null);
    assert!(result_null.is_none());
}

#[test]
fn test_parse_jsonl_with_tool_use_result() {
    // Test AC4: toolUseResult integration in JSONL parsing
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","toolUseResult":{"file":{"filePath":"/src/lib.rs","startLine":10,"numLines":20,"totalLines":200}},"message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"file content here","is_error":false}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    if let ContentBlock::ToolResult { structured_result, .. } = &session.messages[0].content_blocks[0] {
        assert!(structured_result.is_some());
        if let Some(ToolResultData::FileRead { file_path, start_line, num_lines, total_lines }) = structured_result {
            assert_eq!(file_path, "/src/lib.rs");
            assert_eq!(*start_line, Some(10));
            assert_eq!(*num_lines, Some(20));
            assert_eq!(*total_lines, Some(200));
        } else {
            panic!("Expected ToolResultData::FileRead in structured_result");
        }
    } else {
        panic!("Expected ToolResult content block");
    }
}

#[test]
fn test_convert_block_standard_tool() {
    // Test AC3: StandardTool mapping in legacy JSON format (convert_block)
    let block = ClaudeContentBlock::ToolUse {
        id: "t1".to_string(),
        name: "Write".to_string(),
        input: serde_json::json!({"file_path": "/out.txt", "content": "hello"}),
    };

    let result = convert_block(&block);
    if let Some(ContentBlock::ToolUse { standard_tool, .. }) = result {
        assert!(standard_tool.is_some());
        if let Some(crate::models::StandardTool::FileWrite { path, content }) = standard_tool {
            assert_eq!(path, "/out.txt");
            assert_eq!(content, "hello");
        } else {
            panic!("Expected StandardTool::FileWrite");
        }
    } else {
        panic!("Expected ToolUse content block");
    }
}

// Story 8.12: Tests for strip_line_number_prefix (AC5)
#[test]
fn test_strip_line_number_prefix_pipe_format() {
    // Test pipe format: "   1|content"
    let input = "   1|fn main() {\n   2|    println!(\"Hello\");\n   3|}";
    let expected = "fn main() {\n    println!(\"Hello\");\n}";
    assert_eq!(strip_line_number_prefix(input), expected);
}

#[test]
fn test_strip_line_number_prefix_arrow_format() {
    // Test arrow format: "  42→content"
    let input = "  42→const x = 1;\n  43→const y = 2;";
    let expected = "const x = 1;\nconst y = 2;";
    assert_eq!(strip_line_number_prefix(input), expected);
}

#[test]
fn test_strip_line_number_prefix_unpadded() {
    // Test unpadded numbers
    let input = "1|line one\n2|line two\n10|line ten";
    let expected = "line one\nline two\nline ten";
    assert_eq!(strip_line_number_prefix(input), expected);
}

#[test]
fn test_strip_line_number_prefix_mixed() {
    // Test content without line numbers (should remain unchanged)
    let input = "Hello World\nNo line numbers here";
    assert_eq!(strip_line_number_prefix(input), input);
}

#[test]
fn test_strip_line_number_prefix_empty() {
    // Test empty content
    let input = "";
    assert_eq!(strip_line_number_prefix(input), "");
}

#[test]
fn test_strip_line_number_prefix_preserves_content_with_pipe() {
    // Test that pipes in content are preserved (not line number format)
    let input = "This is a | pipe in text\nAnother line | with pipe";
    // These don't match the pattern (no leading digits), so unchanged
    assert_eq!(strip_line_number_prefix(input), input);
}

#[test]
fn test_strip_line_number_prefix_with_space_after_delimiter() {
    // Test format with space after delimiter: "1| content"
    // The space after the delimiter is preserved (part of code indentation)
    let input = "1| fn main() {";
    let expected = " fn main() {";
    assert_eq!(strip_line_number_prefix(input), expected);
}

// ========== Story 8.15: Parser Resilience Enhancement Tests ==========

#[test]
fn test_parse_jsonl_unknown_content_block_degraded() {
    // Test that unknown content block types are degraded to Text with is_degraded=true
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"future_block_type","data":"some data"},{"type":"text","text":"Normal text"}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content_blocks.len(), 2);

    // First block should be degraded
    if let ContentBlock::Text { text, is_degraded } = &session.messages[0].content_blocks[0] {
        assert!(is_degraded.unwrap_or(false), "Unknown block should be degraded");
        assert!(text.contains("future_block_type"), "Degraded block should contain type name");
    } else {
        panic!("Expected degraded Text block for unknown type");
    }

    // Second block should be normal text
    if let ContentBlock::Text { text, is_degraded } = &session.messages[0].content_blocks[1] {
        assert!(is_degraded.is_none() || !is_degraded.unwrap(), "Normal text should not be degraded");
        assert_eq!(text, "Normal text");
    } else {
        panic!("Expected normal Text block");
    }
}

#[test]
fn test_parse_jsonl_unknown_formats_collected() {
    // Test that unknown formats are collected in session metadata
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"new_feature","value":123},{"type":"another_new","data":"test"}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    // Check unknown_formats is populated
    assert!(session.metadata.unknown_formats.is_some());
    let unknown_formats = session.metadata.unknown_formats.as_ref().unwrap();
    assert_eq!(unknown_formats.len(), 2);

    // Verify first unknown format entry
    assert_eq!(unknown_formats[0].source, "claude");
    assert_eq!(unknown_formats[0].type_name, "new_feature");
    assert!(unknown_formats[0].raw_json.contains("123"));

    // Verify second unknown format entry
    assert_eq!(unknown_formats[1].source, "claude");
    assert_eq!(unknown_formats[1].type_name, "another_new");
}

#[test]
fn test_parse_jsonl_parser_info_included() {
    // Test that parser_info is included in session metadata
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","version":"2.1.0","message":{"role":"user","content":"Hello"},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert!(session.metadata.parser_info.is_some());
    let parser_info = session.metadata.parser_info.as_ref().unwrap();

    assert_eq!(parser_info.parser_version, CLAUDE_PARSER_VERSION);
    assert!(parser_info.supported_formats.contains(&"text".to_string()));
    assert!(parser_info.supported_formats.contains(&"thinking".to_string()));
    assert!(parser_info.supported_formats.contains(&"tool_use".to_string()));
    assert!(parser_info.supported_formats.contains(&"tool_result".to_string()));
    assert_eq!(parser_info.detected_source_version, Some("2.1.0".to_string()));
}

#[test]
fn test_parse_jsonl_no_unknown_formats_when_all_known() {
    // Test that unknown_formats is None when all content blocks are known types
    let jsonl = r#"{"type":"assistant","sessionId":"s1","cwd":"/test","message":{"role":"assistant","content":[{"type":"text","text":"Hello"},{"type":"thinking","thinking":"Let me think..."}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    // unknown_formats should be None when no unknown types
    assert!(session.metadata.unknown_formats.is_none());
}

#[test]
fn test_truncate_raw_json_small() {
    // Test that small JSON is not truncated
    let small_json = serde_json::json!({"type": "test", "value": 123});
    let result = truncate_raw_json(&small_json);
    assert!(!result.contains("truncated"));
    assert!(result.contains("test"));
}

#[test]
fn test_truncate_raw_json_large() {
    // Test that large JSON is truncated
    let large_content = "x".repeat(2000);
    let large_json = serde_json::json!({"type": "test", "data": large_content});
    let result = truncate_raw_json(&large_json);
    assert!(result.contains("truncated"));
    assert!(result.len() <= MAX_RAW_JSON_SIZE + 20); // Allow for "... [truncated]" suffix
}

// ========== Story 8.16: Image Content Block Tests ==========

#[test]
fn test_parse_jsonl_image_content_block() {
    // Test AC2: Parse image type content block from JSONL
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":[{"type":"image","source":{"media_type":"image/png","data":"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content_blocks.len(), 1);

    if let ContentBlock::Image { media_type, data, source_type, alt_text } = &session.messages[0].content_blocks[0] {
        assert_eq!(media_type, "image/png");
        assert!(data.starts_with("iVBORw0KGgo"));
        assert_eq!(*source_type, Some("base64".to_string()));
        assert!(alt_text.is_none());
    } else {
        panic!("Expected Image content block");
    }
}

#[test]
fn test_parse_jsonl_image_with_text() {
    // Test AC2: Parse image alongside text content
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":[{"type":"text","text":"Here is a screenshot:"},{"type":"image","source":{"media_type":"image/jpeg","data":"/9j/4AAQSkZJRg=="}}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content_blocks.len(), 2);

    // First block is text
    if let ContentBlock::Text { text, .. } = &session.messages[0].content_blocks[0] {
        assert_eq!(text, "Here is a screenshot:");
    } else {
        panic!("Expected Text content block first");
    }

    // Second block is image
    if let ContentBlock::Image { media_type, .. } = &session.messages[0].content_blocks[1] {
        assert_eq!(media_type, "image/jpeg");
    } else {
        panic!("Expected Image content block second");
    }
}

#[test]
fn test_parse_jsonl_image_missing_source() {
    // Test graceful handling of image without source field
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":[{"type":"image"},{"type":"text","text":"fallback"}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    // Image without source should be skipped, only text remains
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content_blocks.len(), 1);
    assert!(matches!(session.messages[0].content_blocks[0], ContentBlock::Text { .. }));
}

#[test]
fn test_parse_jsonl_image_missing_media_type() {
    // Test graceful handling of image without media_type
    let jsonl = r#"{"type":"user","sessionId":"s1","cwd":"/test","message":{"role":"user","content":[{"type":"image","source":{"data":"base64data"}},{"type":"text","text":"fallback"}]},"timestamp":"2024-01-01T00:00:00Z"}"#;

    let parser = ClaudeParser::new();
    let session = parser.parse_jsonl(jsonl).unwrap();

    // Image without media_type should be skipped
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content_blocks.len(), 1);
    assert!(matches!(session.messages[0].content_blocks[0], ContentBlock::Text { .. }));
}

#[test]
fn test_convert_block_image() {
    // Test convert_block for ClaudeContentBlock::Image
    let block = ClaudeContentBlock::Image {
        source: types::ClaudeImageSource {
            media_type: "image/webp".to_string(),
            data: "UklGRlYAAABXRUJQ".to_string(),
            source_type: Some("base64".to_string()),
        },
    };

    let result = convert_block(&block);
    assert!(result.is_some());

    if let Some(ContentBlock::Image { media_type, data, source_type, alt_text }) = result {
        assert_eq!(media_type, "image/webp");
        assert_eq!(data, "UklGRlYAAABXRUJQ");
        assert_eq!(source_type, Some("base64".to_string()));
        assert!(alt_text.is_none());
    } else {
        panic!("Expected Image content block");
    }
}

#[test]
fn test_supported_content_types_includes_image() {
    // Verify image is in SUPPORTED_CONTENT_TYPES
    assert!(SUPPORTED_CONTENT_TYPES.contains(&"image"));
}
