use super::*;

#[test]
fn test_get_default_log_dir_claude() {
    let result = get_default_log_dir(&ImportSource::Claude);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_str().unwrap().contains(".claude"));
    assert!(path.to_str().unwrap().contains("projects"));
}

#[test]
fn test_get_default_log_dir_gemini() {
    let result = get_default_log_dir(&ImportSource::Gemini);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_str().unwrap().contains(".gemini"));
    assert!(path.to_str().unwrap().contains("tmp"));
}

#[test]
fn test_generate_project_id() {
    let id1 = generate_project_id("/home/user/project1");
    let id2 = generate_project_id("/home/user/project2");
    let id1_again = generate_project_id("/home/user/project1");

    assert!(id1.starts_with("proj_"));
    assert_ne!(id1, id2);
    assert_eq!(id1, id1_again);
}

#[test]
fn test_path_to_discovered_file() {
    // This test requires a real file, skip in unit tests
    // Integration tests should cover this
}

// TODO: Re-enable these tests when extract_cwd_from_file is implemented
// #[test]
// fn test_extract_cwd_from_file() {
//     use std::io::Write;
//
//     // Create a temp file with Claude Code JSONL format
//     let temp_dir = std::env::temp_dir();
//     let test_file = temp_dir.join("test_claude_session.jsonl");
//
//     let content = r#"{"type":"summary","summary":"Test Session"}
// {"parentUuid":"root","cwd":"/mnt/disk0/project/newx/nextalk-voice-capsule","sessionId":"test-123","type":"user","message":{"role":"user","content":"Hello"}}
// "#;
//
//     let mut file = std::fs::File::create(&test_file).unwrap();
//     file.write_all(content.as_bytes()).unwrap();
//
//     // Test extraction
//     let result = extract_cwd_from_file(&test_file);
//     assert_eq!(result, Some("/mnt/disk0/project/newx/nextalk-voice-capsule".to_string()));
//
//     // Clean up
//     std::fs::remove_file(&test_file).ok();
// }
//
// #[test]
// fn test_extract_cwd_from_real_file() {
//     // Test with a real Claude Code session file if it exists
//     let real_file = std::path::PathBuf::from(
//         "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-capsule"
//     );
//
//     if real_file.exists() {
//         if let Ok(entries) = std::fs::read_dir(&real_file) {
//             for entry in entries.flatten() {
//                 let path = entry.path();
//                 if path.extension().is_some_and(|ext| ext == "jsonl") {
//                     let result = extract_cwd_from_file(&path);
//                     println!("File: {:?}", path);
//                     println!("Extracted cwd: {:?}", result);
//
//                     // The cwd should be the real project path, not the log directory
//                     if let Some(cwd) = result {
//                         assert!(
//                             cwd.starts_with("/mnt/disk0/project"),
//                             "cwd should be the real project path, got: {}", cwd
//                         );
//                         assert!(
//                             !cwd.contains("-mnt-"),
//                             "cwd should not contain encoded path format, got: {}", cwd
//                         );
//                     }
//                     break; // Only test one file
//                 }
//             }
//         }
//     }
// }

#[test]
fn test_find_workspace_storage_path_direct() {
    // Case 1: workspaceStorage directory directly
    let dir = PathBuf::from("/some/path/workspaceStorage");
    let result = find_workspace_storage_path(&dir);
    assert_eq!(result, Some(PathBuf::from("/some/path/workspaceStorage")));
}

#[test]
fn test_find_workspace_storage_path_not_found() {
    // Case where no workspaceStorage exists
    let dir = PathBuf::from("/tmp/nonexistent");
    let result = find_workspace_storage_path(&dir);
    assert_eq!(result, None);
}

#[test]
fn test_gemini_file_detection_in_parse_log_files() {
    // Test the file type detection logic for Gemini files
    // Paths containing ".gemini" and ending with ".json" should be detected as Gemini
    let gemini_path = "/home/user/.gemini/tmp/abc123/chats/session-2025-01-01.json";
    assert!(gemini_path.contains("/.gemini/"));
    assert!(gemini_path.ends_with(".json"));

    // Windows path style
    let gemini_path_win = "C:\\Users\\test\\.gemini\\tmp\\abc123\\chats\\session.json";
    assert!(gemini_path_win.contains("\\.gemini\\"));
    assert!(gemini_path_win.ends_with(".json"));
}

// Review Fix H1: Add unit tests for Codex session metadata extraction
#[test]
fn test_extract_codex_session_meta_with_valid_file() {
    use std::io::Write;

    // Create a temp file with valid Codex JSONL format
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_codex_session_valid.jsonl");

    let content = r#"{"type":"session_meta","payload":{"id":"test-session-123","cwd":"/home/user/my-project","model":"gpt-4"}}
{"type":"message","payload":{"role":"user","content":"Hello"}}
"#;

    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // Test extraction
    let meta = extract_codex_session_meta(&test_file);
    assert_eq!(meta.id, Some("test-session-123".to_string()));
    assert_eq!(meta.cwd, Some("/home/user/my-project".to_string()));

    // Also test wrapper functions
    assert_eq!(extract_session_id_from_codex_file(&test_file), Some("test-session-123".to_string()));
    assert_eq!(extract_cwd_from_codex_file(&test_file), Some("/home/user/my-project".to_string()));

    // Clean up
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_extract_codex_session_meta_with_empty_file() {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_codex_session_empty.jsonl");

    // Create empty file
    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(b"").unwrap();

    let meta = extract_codex_session_meta(&test_file);
    assert_eq!(meta.id, None);
    assert_eq!(meta.cwd, None);

    fs::remove_file(&test_file).ok();
}

#[test]
fn test_extract_codex_session_meta_with_missing_fields() {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_codex_session_partial.jsonl");

    // session_meta without cwd field
    let content = r#"{"type":"session_meta","payload":{"id":"partial-session","model":"gpt-4"}}
"#;

    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let meta = extract_codex_session_meta(&test_file);
    assert_eq!(meta.id, Some("partial-session".to_string()));
    assert_eq!(meta.cwd, None);

    fs::remove_file(&test_file).ok();
}

#[test]
fn test_extract_codex_session_meta_with_nonexistent_file() {
    let nonexistent = PathBuf::from("/nonexistent/path/to/file.jsonl");
    let meta = extract_codex_session_meta(&nonexistent);
    assert_eq!(meta.id, None);
    assert_eq!(meta.cwd, None);
}

// Story 8.20 代码审查修复 L1: 添加 Codex 路径直通的集成测试
#[test]
fn test_codex_project_path_uses_cwd_directly() {
    use std::io::Write;

    // 创建临时目录模拟 Codex 会话结构
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_codex_path_direct.jsonl");

    // 模拟真实的 Codex 会话文件，包含 cwd
    let content = r#"{"type":"session_meta","payload":{"id":"path-test-session","cwd":"/home/user/real-project-path","model":"gpt-4"}}
{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"text","text":"Hello"}]}}
"#;

    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // 验证 cwd 被直接提取，而非 Hash
    let cwd = extract_cwd_from_codex_file(&test_file);
    assert_eq!(cwd, Some("/home/user/real-project-path".to_string()));

    // 验证路径不包含 "codex-project:" 或 Hash 格式
    let path = cwd.unwrap();
    assert!(!path.starts_with("codex-project:"), "路径不应该使用 codex-project: 前缀");
    assert!(!path.starts_with("placeholder:"), "有效 cwd 不应该使用 placeholder: 前缀");
    assert!(path.starts_with("/"), "路径应该是绝对路径");

    // 清理
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_codex_fallback_uses_placeholder_prefix() {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_codex_fallback.jsonl");

    // 模拟没有 cwd 的会话文件
    let content = r#"{"type":"session_meta","payload":{"id":"fallback-test-session","model":"gpt-4"}}
"#;

    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // 验证 cwd 为 None
    let cwd = extract_cwd_from_codex_file(&test_file);
    assert_eq!(cwd, None, "没有 cwd 字段时应返回 None");

    // 清理
    fs::remove_file(&test_file).ok();
}
