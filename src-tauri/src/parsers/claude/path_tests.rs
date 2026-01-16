use super::*;

#[test]
fn test_get_claude_dir() {
    // This should return Some on any system with a home directory
    let dir = get_claude_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with(".claude"));
}

#[test]
fn test_get_claude_projects_dir() {
    let dir = get_claude_projects_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with("projects"));
}

#[test]
fn test_decode_claude_path_encoded() {
    let encoded = "-mnt-disk0-project-foo";
    let decoded = decode_claude_path(encoded);
    assert_eq!(decoded, "/mnt/disk0/project/foo");
}

#[test]
fn test_decode_claude_path_not_encoded() {
    let path = "/home/user/project";
    let decoded = decode_claude_path(path);
    assert_eq!(decoded, path);
}

#[test]
fn test_claude_paths_from_dir() {
    let paths = ClaudePaths::from_dir(PathBuf::from("/tmp/test/.claude"));
    assert_eq!(paths.claude_dir, PathBuf::from("/tmp/test/.claude"));
    assert_eq!(paths.projects_dir, PathBuf::from("/tmp/test/.claude/projects"));
}

#[test]
fn test_session_path() {
    let paths = ClaudePaths::from_dir(PathBuf::from("/home/user/.claude"));
    let session_path = paths.session_path("-mnt-project", "abc123");
    assert_eq!(
        session_path,
        PathBuf::from("/home/user/.claude/projects/-mnt-project/abc123.jsonl")
    );
}

#[test]
fn test_decode_claude_path_with_hyphens() {
    // Known limitation: project names with hyphens will be incorrectly decoded
    // e.g., "mantra-landing" becomes "mantra/landing"
    // This is why we added extract_cwd_from_sibling_sessions as a fallback
    let encoded = "-mnt-disk0-project-newx-mantra-landing";
    let decoded = decode_claude_path(encoded);
    // This is the INCORRECT result, documenting the limitation
    assert_eq!(decoded, "/mnt/disk0/project/newx/mantra/landing");
    // The correct result should be "/mnt/disk0/project/newx/mantra-landing"
    // but decode_claude_path cannot distinguish path separators from hyphens in project names
}

#[test]
fn test_extract_cwd_from_file_content_with_cwd() {
    use std::io::Write;
    
    // Create a temp file with cwd in content
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_claude_cwd.jsonl");
    
    let content = r#"{"type":"file-history-snapshot","messageId":"abc"}
{"type":"user","cwd":"/mnt/disk0/project/newx/mantra-landing","sessionId":"s1","message":{"role":"user","content":"hello"}}
"#;
    
    let mut file = std::fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    
    let result = extract_cwd_from_file_content(test_file.to_str().unwrap());
    assert_eq!(result, Some("/mnt/disk0/project/newx/mantra-landing".to_string()));
    
    // Cleanup
    std::fs::remove_file(&test_file).ok();
}

#[test]
fn test_extract_cwd_from_file_content_no_cwd() {
    use std::io::Write;
    
    // Create a temp file without cwd (only system events)
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_claude_no_cwd.jsonl");
    
    let content = r#"{"type":"file-history-snapshot","messageId":"abc"}
{"type":"file-history-snapshot","messageId":"def"}
"#;
    
    let mut file = std::fs::File::create(&test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    
    let result = extract_cwd_from_file_content(test_file.to_str().unwrap());
    assert_eq!(result, None);
    
    // Cleanup
    std::fs::remove_file(&test_file).ok();
}

#[test]
fn test_extract_cwd_from_sibling_sessions() {
    use std::io::Write;
    
    // Create a temp directory with two session files
    let temp_dir = std::env::temp_dir().join("test_claude_siblings");
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    // First file: has cwd
    let file1_path = temp_dir.join("session1.jsonl");
    let content1 = r#"{"type":"user","cwd":"/mnt/disk0/project/newx/mantra-landing","sessionId":"s1","message":{"role":"user","content":"hello"}}
"#;
    let mut file1 = std::fs::File::create(&file1_path).unwrap();
    file1.write_all(content1.as_bytes()).unwrap();
    
    // Second file: no cwd (only system events)
    let file2_path = temp_dir.join("session2.jsonl");
    let content2 = r#"{"type":"file-history-snapshot","messageId":"abc"}
{"type":"file-history-snapshot","messageId":"def"}
"#;
    let mut file2 = std::fs::File::create(&file2_path).unwrap();
    file2.write_all(content2.as_bytes()).unwrap();
    
    // Test: file2 should get cwd from file1
    let result = extract_cwd_from_sibling_sessions(file2_path.to_str().unwrap());
    assert_eq!(result, Some("/mnt/disk0/project/newx/mantra-landing".to_string()));
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
