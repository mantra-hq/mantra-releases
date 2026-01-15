use super::*;

const SIMPLE_SESSION: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-session-123","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/home/user/project","cli_version":"0.77.0","originator":"codex_cli_rs","source":"cli"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Help me with this code"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"I'll help you with that."}]}}"#;

const SESSION_WITH_FUNCTION_CALL: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"func-session","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"List files"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls -la\"]}","call_id":"call_123"}}
{"timestamp":"2025-12-30T20:00:03.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_123","output":"file1.txt\nfile2.txt"}}
{"timestamp":"2025-12-30T20:00:04.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Here are the files."}]}}"#;

const SESSION_WITH_EVENTS: &str = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"event-session","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"event_msg","payload":{"type":"user_message","message":"Hello"}}
{"timestamp":"2025-12-30T20:00:03.000Z","type":"turn_context","payload":{"cwd":"/tmp","model":"gpt-5"}}
{"timestamp":"2025-12-30T20:00:04.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Hi!"}]}}"#;

#[test]
fn test_parse_simple_session() {
    let parser = CodexParser::new();
    let session = parser.parse_string(SIMPLE_SESSION).unwrap();

    assert_eq!(session.id, "test-session-123");
    assert_eq!(session.source, sources::CODEX);
    assert_eq!(session.cwd, "/home/user/project");
    assert_eq!(session.messages.len(), 2);

    // Check user message
    assert_eq!(session.messages[0].role, Role::User);
    match &session.messages[0].content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "Help me with this code"),
        _ => panic!("Expected Text block"),
    }

    // Check assistant message
    assert_eq!(session.messages[1].role, Role::Assistant);
    match &session.messages[1].content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "I'll help you with that."),
        _ => panic!("Expected Text block"),
    }
}

#[test]
fn test_parse_session_with_function_call() {
    let parser = CodexParser::new();
    let session = parser.parse_string(SESSION_WITH_FUNCTION_CALL).unwrap();

    assert_eq!(session.id, "func-session");
    // user + function_call + function_call_output + assistant = 4 messages
    assert_eq!(session.messages.len(), 4);

    // Check function call (ToolUse)
    let tool_use_msg = &session.messages[1];
    assert_eq!(tool_use_msg.role, Role::Assistant);
    match &tool_use_msg.content_blocks[0] {
        ContentBlock::ToolUse { id, name, .. } => {
            assert_eq!(id, "call_123");
            assert_eq!(name, "shell");
        }
        _ => panic!("Expected ToolUse block"),
    }

    // Check function output (ToolResult)
    let tool_result_msg = &session.messages[2];
    match &tool_result_msg.content_blocks[0] {
        ContentBlock::ToolResult { tool_use_id, content, .. } => {
            assert_eq!(tool_use_id, "call_123");
            assert!(content.contains("file1.txt"));
        }
        _ => panic!("Expected ToolResult block"),
    }
}

#[test]
fn test_is_error_detection_rust_panic() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-panic","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"cargo run\"]}","call_id":"call_panic"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_panic","output":"thread 'main' panicked at src/main.rs:10"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_result = session.messages.iter()
        .find_map(|m| match m.content_blocks.first() {
            Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
            _ => None,
        })
        .expect("Should have ToolResult");

    assert!(tool_result, "Rust panic should be detected as error");
}

#[test]
fn test_is_error_detection_rust_compiler() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-compile","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"cargo build\"]}","call_id":"call_compile"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_compile","output":"error[E0308]: mismatched types"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_result = session.messages.iter()
        .find_map(|m| match m.content_blocks.first() {
            Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
            _ => None,
        })
        .expect("Should have ToolResult");

    assert!(tool_result, "Rust compiler error should be detected as error");
}

#[test]
fn test_is_error_detection_git_fatal() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"git push\"]}","call_id":"call_git"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_git","output":"fatal: not a git repository"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_result = session.messages.iter()
        .find_map(|m| match m.content_blocks.first() {
            Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
            _ => None,
        })
        .expect("Should have ToolResult");

    assert!(tool_result, "Git fatal error should be detected as error");
}

#[test]
fn test_is_error_detection_success() {
    // Normal output should NOT be detected as error
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-ok","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls\"]}","call_id":"call_ok"}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_ok","output":"file1.txt\nfile2.txt"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_result = session.messages.iter()
        .find_map(|m| match m.content_blocks.first() {
            Some(ContentBlock::ToolResult { is_error, .. }) => Some(*is_error),
            _ => None,
        })
        .expect("Should have ToolResult");

    assert!(!tool_result, "Normal output should NOT be detected as error");
}

#[test]
fn test_events_filtered() {
    let parser = CodexParser::new();
    let session = parser.parse_string(SESSION_WITH_EVENTS).unwrap();

    // Only user and assistant messages, no event_msg or turn_context
    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, Role::User);
    assert_eq!(session.messages[1].role, Role::Assistant);
}

#[test]
fn test_parse_with_project_path() {
    let parser = CodexParser::with_project_path("/custom/path".to_string());
    let session = parser.parse_string(SIMPLE_SESSION).unwrap();

    assert_eq!(session.cwd, "/custom/path");
}

#[test]
fn test_parse_empty_session_id_fails() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

    let parser = CodexParser::new();
    let result = parser.parse_string(jsonl);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_json_fails() {
    let parser = CodexParser::new();
    let result = parser.parse_string("not valid json");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_session_meta_fails() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let result = parser.parse_string(jsonl);
    assert!(result.is_err());
}

#[test]
fn test_skip_environment_context() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>\n  <cwd>/tmp</cwd>\n</environment_context>"}]}}
{"timestamp":"2025-12-30T20:00:02.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    // Only the "Hello" message should be included
    assert_eq!(session.messages.len(), 1);
    match &session.messages[0].content_blocks[0] {
        ContentBlock::Text { text, .. } => assert_eq!(text, "Hello"),
        _ => panic!("Expected Text block"),
    }
}

#[test]
fn test_parse_timestamp_formats() {
    // RFC 3339 with Z
    let ts = parse_timestamp("2025-12-30T20:11:00.000Z").unwrap();
    assert_eq!(ts.year(), 2025);

    // RFC 3339 with offset
    let ts = parse_timestamp("2025-12-30T20:11:00.000+08:00").unwrap();
    assert_eq!(ts.year(), 2025);
}

use chrono::Datelike;

#[test]
fn test_extract_file_paths_from_shell() {
    let args = serde_json::json!({
        "command": ["bash", "-lc", "cat /etc/passwd ./local/file"]
    });

    let mut files = Vec::new();
    extract_file_paths(&args, &mut files);

    assert!(files.contains(&"/etc/passwd".to_string()));
    assert!(files.contains(&"./local/file".to_string()));
}

// ====== AC1: Git Info Tests ======

#[test]
fn test_parse_git_info() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","git":{"commit_hash":"abc123def456","branch":"main","repository_url":"https://github.com/user/repo"}}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.git.is_some());
    let git = session.metadata.git.unwrap();
    assert_eq!(git.commit, Some("abc123def456".to_string()));
    assert_eq!(git.branch, Some("main".to_string()));
    assert_eq!(git.repository_url, Some("https://github.com/user/repo".to_string()));
}

#[test]
fn test_parse_git_info_partial() {
    // Only branch, no commit or url
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-git-partial","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","git":{"branch":"feature/test"}}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.git.is_some());
    let git = session.metadata.git.unwrap();
    assert_eq!(git.branch, Some("feature/test".to_string()));
    assert!(git.commit.is_none());
    assert!(git.repository_url.is_none());
}

#[test]
fn test_parse_git_info_none() {
    // No git field at all (backward compatibility)
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-git","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.git.is_none());
}

// ====== AC2: Instructions Tests ======

#[test]
fn test_parse_instructions() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-inst","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","instructions":"You are a helpful assistant. Follow coding best practices."}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.metadata.instructions, Some("You are a helpful assistant. Follow coding best practices.".to_string()));
}

#[test]
fn test_parse_instructions_none() {
    // No instructions field (backward compatibility)
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-inst","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.instructions.is_none());
}

// ====== AC3: StandardTool Mapping Tests ======

#[test]
fn test_parse_standard_tool_shell() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-shell","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"shell","arguments":"{\"command\":[\"bash\",\"-lc\",\"ls -la\"]}","call_id":"call_shell_1"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_msg = session.messages.iter()
        .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
        .expect("Should have ToolUse message");

    if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(name, "shell");
        assert!(standard_tool.is_some(), "standard_tool should be set");
        match standard_tool {
            Some(crate::models::StandardTool::ShellExec { command, .. }) => {
                assert_eq!(command, "ls -la");
            }
            _ => panic!("Expected ShellExec, got {:?}", standard_tool),
        }
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_parse_standard_tool_read_file() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-read","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"read_file","arguments":"{\"path\":\"/src/main.rs\"}","call_id":"call_read_1"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_msg = session.messages.iter()
        .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
        .expect("Should have ToolUse message");

    if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(name, "read_file");
        assert!(standard_tool.is_some());
        match standard_tool {
            Some(crate::models::StandardTool::FileRead { path, .. }) => {
                assert_eq!(path, "/src/main.rs");
            }
            _ => panic!("Expected FileRead, got {:?}", standard_tool),
        }
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_parse_standard_tool_apply_diff() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-diff","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"apply_diff","arguments":"{\"path\":\"/src/lib.rs\",\"diff\":\"--- a/lib.rs\\n+++ b/lib.rs\"}","call_id":"call_diff_1"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_msg = session.messages.iter()
        .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
        .expect("Should have ToolUse message");

    if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(name, "apply_diff");
        assert!(standard_tool.is_some());
        match standard_tool {
            Some(crate::models::StandardTool::FileEdit { path, new_string, .. }) => {
                assert_eq!(path, "/src/lib.rs");
                assert_eq!(new_string, &Some("--- a/lib.rs\n+++ b/lib.rs".to_string()));
            }
            _ => panic!("Expected FileEdit, got {:?}", standard_tool),
        }
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_parse_standard_tool_write_file() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-write","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"write_file","arguments":"{\"path\":\"/src/new.rs\",\"content\":\"fn main() {}\"}","call_id":"call_write_1"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_msg = session.messages.iter()
        .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
        .expect("Should have ToolUse message");

    if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(name, "write_file");
        assert!(standard_tool.is_some());
        match standard_tool {
            Some(crate::models::StandardTool::FileWrite { path, content }) => {
                assert_eq!(path, "/src/new.rs");
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected FileWrite, got {:?}", standard_tool),
        }
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_parse_standard_tool_update_plan() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-plan","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"function_call","name":"update_plan","arguments":"{\"plan\":\"Step 1: Read code\"}","call_id":"call_plan_1"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    let tool_msg = session.messages.iter()
        .find(|m| matches!(m.content_blocks.first(), Some(ContentBlock::ToolUse { .. })))
        .expect("Should have ToolUse message");

    if let ContentBlock::ToolUse { name, standard_tool, .. } = &tool_msg.content_blocks[0] {
        assert_eq!(name, "update_plan");
        assert!(standard_tool.is_some());
        match standard_tool {
            Some(crate::models::StandardTool::Unknown { name: tool_name, .. }) => {
                assert_eq!(tool_name, "update_plan");
            }
            _ => panic!("Expected Unknown, got {:?}", standard_tool),
        }
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_preprocess_codex_tool_input_shell() {
    let input = serde_json::json!({
        "command": ["bash", "-lc", "cargo build"]
    });

    let result = preprocess_codex_tool_input("shell", &input);

    assert_eq!(result.get("command").and_then(|v| v.as_str()), Some("cargo build"));
}

#[test]
fn test_preprocess_codex_tool_input_shell_with_cwd() {
    let input = serde_json::json!({
        "command": ["bash", "-lc", "npm install"],
        "cwd": "/project"
    });

    let result = preprocess_codex_tool_input("shell", &input);

    assert_eq!(result.get("command").and_then(|v| v.as_str()), Some("npm install"));
    assert_eq!(result.get("cwd").and_then(|v| v.as_str()), Some("/project"));
}

#[test]
fn test_preprocess_codex_tool_input_passthrough() {
    let input = serde_json::json!({
        "path": "/src/main.rs"
    });

    let result = preprocess_codex_tool_input("read_file", &input);

    assert_eq!(result, input);
}

#[test]
fn test_preprocess_codex_tool_input_shell_empty_array() {
    let input = serde_json::json!({
        "command": []
    });

    let result = preprocess_codex_tool_input("shell", &input);

    assert_eq!(result.get("command").and_then(|v| v.as_str()), Some(""));
}

#[test]
fn test_preprocess_codex_tool_input_shell_short_array() {
    let input = serde_json::json!({
        "command": ["bash", "-lc"]
    });

    let result = preprocess_codex_tool_input("shell", &input);

    assert_eq!(result.get("command").and_then(|v| v.as_str()), Some(""));
}

#[test]
fn test_preprocess_codex_tool_input_shell_not_array() {
    let input = serde_json::json!({
        "command": "ls -la"
    });

    let result = preprocess_codex_tool_input("shell", &input);

    assert_eq!(result, input);
}

// ====== AC4: source_metadata Tests ======

#[test]
fn test_parse_source_metadata_full() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-sm","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","cli_version":"0.77.0","originator":"codex_cli_rs","source":"cli"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.source_metadata.is_some());
    let sm = session.metadata.source_metadata.unwrap();
    assert_eq!(sm.get("cli_version").and_then(|v| v.as_str()), Some("0.77.0"));
    assert_eq!(sm.get("originator").and_then(|v| v.as_str()), Some("codex_cli_rs"));
    assert_eq!(sm.get("source").and_then(|v| v.as_str()), Some("cli"));
}

#[test]
fn test_parse_source_metadata_partial() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-sm-partial","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","cli_version":"0.75.0"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.source_metadata.is_some());
    let sm = session.metadata.source_metadata.unwrap();
    assert_eq!(sm.get("cli_version").and_then(|v| v.as_str()), Some("0.75.0"));
    assert!(sm.get("originator").is_none());
    assert!(sm.get("source").is_none());
}

#[test]
fn test_parse_source_metadata_none() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-no-sm","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.source_metadata.is_none());
}

// ====== AC5: Backward Compatibility Test ======

#[test]
fn test_backward_compatibility_no_new_fields() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"old-session-123","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/home/user/project"}}
{"timestamp":"2025-12-30T20:00:01.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.git.is_none());
    assert!(session.metadata.instructions.is_none());
    assert!(session.metadata.source_metadata.is_none());

    assert_eq!(session.id, "old-session-123");
    assert_eq!(session.cwd, "/home/user/project");
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, Role::User);
}

// ====== New ResponseItem Types Tests ======

#[test]
fn test_parse_reasoning_type() {
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-reasoning","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"**Thinking about the problem**"}],"content":null,"encrypted_content":"gAAAAABo4kai..."}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);

    if let ContentBlock::Thinking { thinking, .. } = &session.messages[0].content_blocks[0] {
        assert!(thinking.contains("Thinking about the problem"));
    } else {
        panic!("Expected Thinking block");
    }
}

#[test]
fn test_parse_reasoning_with_raw_content() {
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-reasoning-raw","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"reasoning","summary":[{"type":"summary_text","text":"Step 1"}],"content":[{"type":"reasoning_text","text":"First, analyze the code..."},{"type":"text","text":"Then implement the fix..."}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);

    if let ContentBlock::Thinking { thinking, .. } = &session.messages[0].content_blocks[0] {
        assert!(thinking.contains("Step 1"));
        assert!(thinking.contains("First, analyze the code..."));
        assert!(thinking.contains("Then implement the fix..."));
    } else {
        panic!("Expected Thinking block");
    }
}

#[test]
fn test_parse_web_search_call() {
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-websearch","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"web_search_call","status":"completed","action":{"type":"search","query":"Rust async programming"}}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.messages.len(), 1);

    if let ContentBlock::ToolUse { name, input, .. } = &session.messages[0].content_blocks[0] {
        assert_eq!(name, "web_search");
        assert_eq!(input.get("query").and_then(|v| v.as_str()), Some("Rust async programming"));
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_parse_local_shell_call() {
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-localshell","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"local_shell_call","call_id":"shell_123","status":"completed","action":{"type":"exec","command":["bash","-c","ls -la"],"cwd":"/tmp","exit_code":0,"output":"file1.txt\nfile2.txt"}}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.messages.len(), 2);

    if let ContentBlock::ToolUse { name, .. } = &session.messages[0].content_blocks[0] {
        assert_eq!(name, "shell");
    } else {
        panic!("Expected ToolUse block");
    }

    if let ContentBlock::ToolResult { content, is_error, .. } = &session.messages[1].content_blocks[0] {
        assert!(content.contains("file1.txt"));
        assert!(!is_error);
    } else {
        panic!("Expected ToolResult block");
    }
}

#[test]
fn test_parse_custom_tool_call() {
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-customtool","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"custom_tool_call","call_id":"custom_123","name":"my_custom_tool","input":"{\"param\":\"value\"}"}}
{"timestamp":"2025-10-05T10:21:24.326Z","type":"response_item","payload":{"type":"custom_tool_call_output","call_id":"custom_123","output":"Custom tool result"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert_eq!(session.messages.len(), 2);

    if let ContentBlock::ToolUse { name, .. } = &session.messages[0].content_blocks[0] {
        assert_eq!(name, "my_custom_tool");
    } else {
        panic!("Expected ToolUse block");
    }

    if let ContentBlock::ToolResult { content, .. } = &session.messages[1].content_blocks[0] {
        assert!(content.contains("Custom tool result"));
    } else {
        panic!("Expected ToolResult block");
    }
}

#[test]
fn test_parse_unknown_response_item_type() {
    // Unknown types should be silently skipped
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-unknown","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"future_new_type","some_field":"value"}}
{"timestamp":"2025-10-05T10:21:24.326Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    // Should only have the user message, unknown type is skipped
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, Role::User);
}

#[test]
fn test_parse_compaction_skipped() {
    // Compaction types should be skipped (internal to Codex)
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-compaction","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"compaction","encrypted_content":"ENCRYPTED_CONTENT"}}
{"timestamp":"2025-10-05T10:21:24.326Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    // Should only have the user message, compaction is skipped
    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, Role::User);
}

// ===== Story 8.15: Parser 弹性增强测试 =====

#[test]
fn test_parser_version_constant() {
    // Verify parser version is defined
    assert!(!CODEX_PARSER_VERSION.is_empty());
    assert!(CODEX_PARSER_VERSION.starts_with("1."));
}

#[test]
fn test_supported_formats_defined() {
    // Verify supported formats list is populated
    assert!(!SUPPORTED_RESPONSE_TYPES.is_empty());
    assert!(SUPPORTED_RESPONSE_TYPES.contains(&"message"));
    assert!(SUPPORTED_RESPONSE_TYPES.contains(&"function_call"));
    assert!(SUPPORTED_RESPONSE_TYPES.contains(&"reasoning"));
}

#[test]
fn test_truncate_raw_json_short() {
    let json = serde_json::json!({"type": "test", "value": 123});
    let result = truncate_raw_json(&json);
    assert!(!result.contains("[truncated]"));
    assert!(result.contains("test"));
}

#[test]
fn test_truncate_raw_json_long() {
    let long_content = "x".repeat(2000);
    let json = serde_json::json!({"type": "test", "content": long_content});
    let result = truncate_raw_json(&json);
    assert!(result.contains("[truncated]"));
    assert!(result.len() <= MAX_RAW_JSON_SIZE + 20);
}

#[test]
fn test_truncate_raw_json_str_short() {
    let json_str = r#"{"type": "test"}"#;
    let result = truncate_raw_json_str(json_str);
    assert_eq!(result, json_str);
    assert!(!result.contains("[truncated]"));
}

#[test]
fn test_truncate_raw_json_str_long() {
    let long_str = format!(r#"{{"content": "{}"}}"#, "x".repeat(2000));
    let result = truncate_raw_json_str(&long_str);
    assert!(result.contains("[truncated]"));
    assert!(result.len() <= MAX_RAW_JSON_SIZE + 20);
}

#[test]
fn test_parser_info_populated() {
    let parser = CodexParser::new();
    let session = parser.parse_string(SIMPLE_SESSION).unwrap();

    assert!(session.metadata.parser_info.is_some());
    let parser_info = session.metadata.parser_info.unwrap();
    assert_eq!(parser_info.parser_version, CODEX_PARSER_VERSION);
    assert!(!parser_info.supported_formats.is_empty());
}

#[test]
fn test_unknown_response_type_collected() {
    // Session with unknown response_item type
    let jsonl = r#"{"timestamp":"2025-10-05T10:21:15.988Z","type":"session_meta","payload":{"id":"test-unknown-collected","timestamp":"2025-10-05T10:21:15.983Z","cwd":"/tmp"}}
{"timestamp":"2025-10-05T10:21:23.326Z","type":"response_item","payload":{"type":"future_new_type","some_field":"value"}}
{"timestamp":"2025-10-05T10:21:24.326Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello"}]}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    // Unknown format should be collected
    assert!(session.metadata.unknown_formats.is_some());
    let unknown = session.metadata.unknown_formats.unwrap();
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].source, "codex");
    assert_eq!(unknown[0].type_name, "unknown_response_type");
}

#[test]
fn test_known_response_types_no_unknown_formats() {
    // Session with only known response types should have no unknown_formats
    let parser = CodexParser::new();
    let session = parser.parse_string(SIMPLE_SESSION).unwrap();

    // No unknown formats for known types
    assert!(session.metadata.unknown_formats.is_none());
}

#[test]
fn test_detected_source_version_from_cli_version() {
    let jsonl = r#"{"timestamp":"2025-12-30T20:00:00.000Z","type":"session_meta","payload":{"id":"test-version","timestamp":"2025-12-30T20:00:00.000Z","cwd":"/tmp","cli_version":"0.77.0"}}"#;

    let parser = CodexParser::new();
    let session = parser.parse_string(jsonl).unwrap();

    assert!(session.metadata.parser_info.is_some());
    let parser_info = session.metadata.parser_info.unwrap();
    assert_eq!(parser_info.detected_source_version, Some("0.77.0".to_string()));
}
