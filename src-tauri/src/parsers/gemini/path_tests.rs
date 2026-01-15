use super::*;

#[test]
fn test_get_gemini_dir() {
    // This should return Some on any system with a home directory
    let dir = get_gemini_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with(".gemini"));
}

#[test]
fn test_get_gemini_tmp_dir() {
    let dir = get_gemini_tmp_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with("tmp"));
}

#[test]
fn test_wsl_to_windows_path() {
    let wsl_path = PathBuf::from("/mnt/c/Users/test/project");
    let windows_path = wsl_to_windows_path(&wsl_path);
    assert_eq!(windows_path, Some(PathBuf::from("C:\\Users\\test\\project")));
}

#[test]
fn test_wsl_to_windows_path_drive_only() {
    let wsl_path = PathBuf::from("/mnt/d");
    let windows_path = wsl_to_windows_path(&wsl_path);
    assert_eq!(windows_path, Some(PathBuf::from("D:\\")));
}

#[test]
fn test_wsl_to_windows_path_not_wsl() {
    let path = PathBuf::from("/home/user/project");
    let windows_path = wsl_to_windows_path(&path);
    assert_eq!(windows_path, None);
}

#[test]
fn test_windows_to_wsl_path() {
    let windows_path = PathBuf::from("C:\\Users\\test\\project");
    let wsl_path = windows_to_wsl_path(&windows_path);
    assert_eq!(wsl_path, Some(PathBuf::from("/mnt/c/Users/test/project")));
}

#[test]
fn test_windows_to_wsl_path_not_windows() {
    let path = PathBuf::from("/home/user/project");
    let wsl_path = windows_to_wsl_path(&path);
    assert_eq!(wsl_path, None);
}

#[test]
fn test_gemini_paths_from_dir() {
    let paths = GeminiPaths::from_dir(PathBuf::from("/tmp/test/.gemini"));
    assert_eq!(paths.gemini_dir, PathBuf::from("/tmp/test/.gemini"));
    assert_eq!(paths.tmp_dir, PathBuf::from("/tmp/test/.gemini/tmp"));
}

#[test]
fn test_session_path() {
    let paths = GeminiPaths::from_dir(PathBuf::from("/home/user/.gemini"));
    let session_path = paths.session_path("abc123", "session-2025-01-01");
    assert_eq!(
        session_path,
        PathBuf::from("/home/user/.gemini/tmp/abc123/chats/session-2025-01-01.json")
    );
}
