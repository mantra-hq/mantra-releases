use super::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// 创建测试用 Git 仓库
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // 配置 Git 用户信息
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to config email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to config name");

    // 创建初始文件
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "initial content").expect("Failed to write file");

    // 提交
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to git add");

    // 第一次提交：使用固定的早期时间戳
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
        .env("GIT_AUTHOR_DATE", "2020-01-01T00:00:00Z")
        .current_dir(repo_path)
        .output()
        .expect("Failed to git commit");

    // 修改文件并再次提交（使用较晚的时间戳）
    fs::write(&test_file, "updated content").expect("Failed to write file");

    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to git add");

    Command::new("git")
        .args(["commit", "-m", "Update test file"])
        .env("GIT_COMMITTER_DATE", "2020-06-01T00:00:00Z")
        .env("GIT_AUTHOR_DATE", "2020-06-01T00:00:00Z")
        .current_dir(repo_path)
        .output()
        .expect("Failed to git commit");

    temp_dir
}

#[test]
fn test_new_with_valid_repo() {
    let temp_dir = create_test_repo();
    let result = GitTimeMachine::new(temp_dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_new_with_invalid_path() {
    let result = GitTimeMachine::new(Path::new("/nonexistent/path"));
    assert!(matches!(result, Err(GitError::NotARepository(_))));
}

#[test]
fn test_find_commit_at_time() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 使用未来时间，应该找到最新的 commit
    let future = Utc::now() + chrono::Duration::hours(1);
    let result = tm.find_commit_at_time(future);
    assert!(result.is_ok());
}

#[test]
fn test_find_commit_at_time_no_match() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 使用很早的时间，应该找不到 commit
    let past = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
    let result = tm.find_commit_at_time(past);
    assert!(matches!(result, Err(GitError::CommitNotFound(_))));
}

#[test]
fn test_get_file_at_commit() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    let future = Utc::now() + chrono::Duration::hours(1);
    let commit_oid = tm.find_commit_at_time(future).expect("Failed to find commit");

    let content = tm
        .get_file_at_commit(commit_oid, "test.txt")
        .expect("Failed to get file content");
    assert_eq!(content, "updated content");
}

#[test]
fn test_get_file_at_commit_not_found() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    let future = Utc::now() + chrono::Duration::hours(1);
    let commit_oid = tm.find_commit_at_time(future).expect("Failed to find commit");

    let result = tm.get_file_at_commit(commit_oid, "nonexistent.txt");
    assert!(matches!(result, Err(GitError::FileNotFound { .. })));
}

#[test]
fn test_get_snapshot_at_time() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    let future = Utc::now() + chrono::Duration::hours(1);
    let snapshot = tm
        .get_snapshot_at_time(future, "test.txt")
        .expect("Failed to get snapshot");

    assert_eq!(snapshot.content, "updated content");
    assert_eq!(snapshot.file_path, "test.txt");
    assert!(!snapshot.commit_hash.is_empty());
    assert!(!snapshot.message.is_empty());
    assert!(!snapshot.author.is_empty());
}

#[test]
fn test_readonly_operations() {
    let temp_dir = create_test_repo();
    let repo_path = temp_dir.path();

    // 记录原始文件 mtime
    let test_file = repo_path.join("test.txt");
    let original_mtime = fs::metadata(&test_file)
        .expect("Failed to get metadata")
        .modified()
        .expect("Failed to get mtime");

    // 执行 GitTimeMachine 操作
    let tm = GitTimeMachine::new(repo_path).expect("Failed to create GitTimeMachine");
    let future = Utc::now() + chrono::Duration::hours(1);
    let _ = tm.get_snapshot_at_time(future, "test.txt");

    // 验证文件未被修改
    let new_mtime = fs::metadata(&test_file)
        .expect("Failed to get metadata")
        .modified()
        .expect("Failed to get mtime");

    assert_eq!(original_mtime, new_mtime, "File should not be modified");
}

// =========================================================================
// Story 2.32: get_commits_in_range 测试
// =========================================================================

#[test]
fn test_get_commits_in_range_finds_commits() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 测试仓库有两个 commit:
    // - 2020-01-01: Initial commit
    // - 2020-06-01: Update test file
    // 使用一个包含这两个时间点的范围
    let start = Utc.with_ymd_and_hms(2019, 12, 1, 0, 0, 0).unwrap().timestamp();
    let end = Utc.with_ymd_and_hms(2020, 12, 31, 23, 59, 59).unwrap().timestamp();

    let commits = tm.get_commits_in_range(start, end)
        .expect("Failed to get commits in range");

    assert_eq!(commits.len(), 2, "Should find 2 commits in range");
    // 验证按时间升序排列（旧的在前）
    assert!(commits[0].message.contains("Initial"), "First commit should be initial");
    assert!(commits[1].message.contains("Update"), "Second commit should be update");
}

#[test]
fn test_get_commits_in_range_partial_range() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 只包含第一个 commit 的范围 (2020-01-01)
    let start = Utc.with_ymd_and_hms(2019, 12, 1, 0, 0, 0).unwrap().timestamp();
    let end = Utc.with_ymd_and_hms(2020, 3, 1, 0, 0, 0).unwrap().timestamp();

    let commits = tm.get_commits_in_range(start, end)
        .expect("Failed to get commits in range");

    assert_eq!(commits.len(), 1, "Should find 1 commit in partial range");
    assert!(commits[0].message.contains("Initial"), "Should be initial commit");
}

#[test]
fn test_get_commits_in_range_empty_range() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 在两个 commit 之间的空白范围 (2020-02 到 2020-04)
    let start = Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap().timestamp();
    let end = Utc.with_ymd_and_hms(2020, 4, 1, 0, 0, 0).unwrap().timestamp();

    let commits = tm.get_commits_in_range(start, end)
        .expect("Failed to get commits in range");

    assert!(commits.is_empty(), "Should find no commits in empty range");
}

#[test]
fn test_get_commits_in_range_before_all_commits() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    // 所有 commit 之前的范围
    let start = Utc.with_ymd_and_hms(2018, 1, 1, 0, 0, 0).unwrap().timestamp();
    let end = Utc.with_ymd_and_hms(2019, 1, 1, 0, 0, 0).unwrap().timestamp();

    let commits = tm.get_commits_in_range(start, end)
        .expect("Failed to get commits in range");

    assert!(commits.is_empty(), "Should find no commits before all commits");
}

#[test]
fn test_get_commits_in_range_commit_info_fields() {
    let temp_dir = create_test_repo();
    let tm = GitTimeMachine::new(temp_dir.path()).expect("Failed to create GitTimeMachine");

    let start = Utc.with_ymd_and_hms(2019, 12, 1, 0, 0, 0).unwrap().timestamp();
    let end = Utc.with_ymd_and_hms(2020, 12, 31, 23, 59, 59).unwrap().timestamp();

    let commits = tm.get_commits_in_range(start, end)
        .expect("Failed to get commits in range");

    assert!(!commits.is_empty(), "Should have at least one commit");

    let commit = &commits[0];
    assert!(!commit.commit_hash.is_empty(), "commit_hash should not be empty");
    assert!(!commit.message.is_empty(), "message should not be empty");
    assert!(!commit.author.is_empty(), "author should not be empty");
    assert!(!commit.author_email.is_empty(), "author_email should not be empty");
}
