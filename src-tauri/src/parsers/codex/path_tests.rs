use super::*;

#[test]
fn test_get_codex_dir() {
    let dir = get_codex_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with(".codex"));
}

#[test]
fn test_get_codex_sessions_dir() {
    let dir = get_codex_sessions_dir();
    assert!(dir.is_some());
    let path = dir.unwrap();
    assert!(path.ends_with("sessions"));
}

#[test]
fn test_codex_paths_from_dir() {
    let paths = CodexPaths::from_dir(PathBuf::from("/tmp/test/.codex"));
    assert_eq!(paths.codex_dir, PathBuf::from("/tmp/test/.codex"));
    assert_eq!(paths.sessions_dir, PathBuf::from("/tmp/test/.codex/sessions"));
}

#[test]
fn test_extract_session_id() {
    // Standard format
    let id = extract_session_id("rollout-2026-01-05T19-04-21-019b8dd4-539f-7393-acb6-7a856d6892ca.jsonl");
    assert_eq!(id, Some("019b8dd4-539f-7393-acb6-7a856d6892ca".to_string()));

    // Another format
    let id = extract_session_id("rollout-2025-10-19T09-08-27-0199fa02-be3a-7021-8051-9b891b6d0eb2.jsonl");
    assert_eq!(id, Some("0199fa02-be3a-7021-8051-9b891b6d0eb2".to_string()));
}

#[test]
fn test_extract_date_from_path() {
    let sessions_dir = PathBuf::from("/home/user/.codex/sessions");
    let path = PathBuf::from("/home/user/.codex/sessions/2026/01/05/rollout-test.jsonl");

    let date = extract_date_from_path(&path, &sessions_dir);
    assert_eq!(date, Some("2026-01-05".to_string()));
}

#[test]
fn test_extract_date_from_path_invalid() {
    let sessions_dir = PathBuf::from("/home/user/.codex/sessions");
    let path = PathBuf::from("/home/user/.codex/sessions/invalid/rollout-test.jsonl");

    let date = extract_date_from_path(&path, &sessions_dir);
    assert_eq!(date, None);
}
