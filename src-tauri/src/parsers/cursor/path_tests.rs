use super::*;

#[test]
fn test_parse_file_uri_linux() {
    let uri = "file:///home/user/project";
    let path = parse_file_uri(uri);
    assert_eq!(path, Some(PathBuf::from("/home/user/project")));
}

#[test]
fn test_parse_file_uri_with_spaces() {
    let uri = "file:///home/user/my%20project";
    let path = parse_file_uri(uri);
    assert_eq!(path, Some(PathBuf::from("/home/user/my project")));
}

#[test]
fn test_parse_file_uri_invalid() {
    let uri = "http://example.com/path";
    let path = parse_file_uri(uri);
    assert_eq!(path, None);
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
    let wsl_path = PathBuf::from("/home/user/project");
    let windows_path = wsl_to_windows_path(&wsl_path);
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
