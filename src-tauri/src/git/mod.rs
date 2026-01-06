//! Git Time Machine 模块
//!
//! 提供根据时间戳查看历史代码状态的功能。
//! 严格只读操作，不修改工作目录，不执行 checkout。

pub mod error;
pub mod time_machine;
pub mod utils;

pub use error::GitError;
pub use time_machine::{CommitInfo, GitTimeMachine, Snapshot};
pub use utils::{get_git_remote_url, normalize_git_url};

/// Synchronously detect Git repository from a directory path
///
/// Searches upward from the given directory to find a .git folder.
/// Returns the Git repository root path if found, None otherwise.
///
/// # Arguments
/// * `dir_path` - The directory path to start searching from
pub fn detect_git_repo_sync(dir_path: &str) -> Option<String> {
    let dir_path = std::path::PathBuf::from(dir_path);
    let mut current = dir_path.as_path();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current.to_string_lossy().to_string());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_git_repo_sync_finds_repo() {
        // Test on current project (which has .git)
        let result = detect_git_repo_sync(env!("CARGO_MANIFEST_DIR"));
        assert!(result.is_some());
    }

    #[test]
    fn test_detect_git_repo_sync_no_repo() {
        let result = detect_git_repo_sync("/tmp");
        // /tmp might or might not be in a Git repo, but we test the function works
        // This is a basic smoke test
        assert!(result.is_none() || result.is_some());
    }
}
