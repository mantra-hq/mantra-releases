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
