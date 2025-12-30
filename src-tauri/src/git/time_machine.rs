//! Git Time Machine 核心实现
//!
//! 提供历史代码快照功能：
//! - 根据时间戳查找最近的 Commit
//! - 读取历史文件内容（不执行 checkout）
//! - 获取完整快照（内容 + 元数据）

use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use git2::{Oid, Repository, Sort};
use serde::{Deserialize, Serialize};

use super::error::GitError;

/// 历史快照数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 文件内容
    pub content: String,
    /// Commit SHA
    pub commit_hash: String,
    /// Commit 消息
    pub message: String,
    /// 作者名
    pub author: String,
    /// 作者邮箱
    pub author_email: String,
    /// Commit 时间
    pub committed_at: DateTime<Utc>,
    /// 文件路径
    pub file_path: String,
}

/// Commit 信息（不含文件内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Commit SHA
    pub commit_hash: String,
    /// Commit 消息
    pub message: String,
    /// 作者名
    pub author: String,
    /// 作者邮箱
    pub author_email: String,
    /// Commit 时间
    pub committed_at: DateTime<Utc>,
}

/// Git Time Machine 服务
///
/// 提供根据时间戳查看历史代码状态的功能。
/// 所有操作都是只读的，不修改工作目录。
pub struct GitTimeMachine {
    repo: Repository,
}

impl GitTimeMachine {
    /// 创建新的 GitTimeMachine 实例
    ///
    /// # Arguments
    /// * `repo_path` - Git 仓库路径
    ///
    /// # Errors
    /// 如果路径不是有效的 Git 仓库，返回 NotARepository 错误
    pub fn new(repo_path: &Path) -> Result<Self, GitError> {
        let repo = Repository::open(repo_path).map_err(|e| {
            if e.code() == git2::ErrorCode::NotFound {
                GitError::NotARepository(repo_path.display().to_string())
            } else {
                GitError::RepositoryError(e)
            }
        })?;
        Ok(Self { repo })
    }

    /// 查找指定时间戳之前最近的 Commit
    ///
    /// 使用反向 Commit 遍历，找到第一个匹配后立即返回。
    ///
    /// # Arguments
    /// * `timestamp` - 目标时间戳
    ///
    /// # Returns
    /// 返回该时间点之前最近的 Commit OID
    pub fn find_commit_at_time(&self, timestamp: DateTime<Utc>) -> Result<Oid, GitError> {
        let target_secs = timestamp.timestamp();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME)?;

        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            let commit_time = commit.time().seconds();

            if commit_time <= target_secs {
                return Ok(oid);
            }
        }

        Err(GitError::CommitNotFound(format!(
            "在 {} 之前没有找到任何 Commit",
            timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )))
    }

    /// 获取指定时间戳之前最近的 Commit 完整信息
    ///
    /// 结合 find_commit_at_time 和 commit 元数据提取。
    ///
    /// # Arguments
    /// * `timestamp` - 目标时间戳
    ///
    /// # Returns
    /// 返回包含完整元数据的 CommitInfo
    pub fn get_commit_info(&self, timestamp: DateTime<Utc>) -> Result<CommitInfo, GitError> {
        let commit_oid = self.find_commit_at_time(timestamp)?;
        let commit = self.repo.find_commit(commit_oid)?;
        let author = commit.author();
        let commit_time = commit.time();
        let committed_at = Utc
            .timestamp_opt(commit_time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(CommitInfo {
            commit_hash: commit_oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            committed_at,
        })
    }

    /// 获取指定 Commit 中文件的内容
    ///
    /// 直接读取 Git ODB，不执行 checkout。
    ///
    /// # Arguments
    /// * `commit_oid` - Commit OID
    /// * `file_path` - 相对于仓库根目录的文件路径
    ///
    /// # Returns
    /// 返回文件内容字符串
    pub fn get_file_at_commit(&self, commit_oid: Oid, file_path: &str) -> Result<String, GitError> {
        let commit = self.repo.find_commit(commit_oid)?;
        let tree = commit.tree()?;

        let entry = tree.get_path(Path::new(file_path)).map_err(|_| {
            GitError::FileNotFound {
                commit: commit_oid.to_string(),
                path: file_path.to_string(),
            }
        })?;

        let blob = self.repo.find_blob(entry.id())?;
        let content = std::str::from_utf8(blob.content())
            .map_err(|_| GitError::InvalidUtf8)?;

        Ok(content.to_string())
    }

    /// 获取指定时间戳的文件快照
    ///
    /// 组合 find_commit_at_time 和 get_file_at_commit，
    /// 返回完整的快照数据（内容 + 元数据）。
    ///
    /// # Arguments
    /// * `timestamp` - 目标时间戳
    /// * `file_path` - 相对于仓库根目录的文件路径
    ///
    /// # Returns
    /// 返回包含内容和元数据的 Snapshot
    pub fn get_snapshot_at_time(
        &self,
        timestamp: DateTime<Utc>,
        file_path: &str,
    ) -> Result<Snapshot, GitError> {
        let commit_oid = self.find_commit_at_time(timestamp)?;
        let content = self.get_file_at_commit(commit_oid, file_path)?;

        let commit = self.repo.find_commit(commit_oid)?;
        let author = commit.author();
        let commit_time = commit.time();
        let committed_at = Utc
            .timestamp_opt(commit_time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(Snapshot {
            content,
            commit_hash: commit_oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            committed_at,
            file_path: file_path.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
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
}
