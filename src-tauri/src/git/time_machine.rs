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

/// 快照来源类型 (Story 2.30)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotSource {
    /// 来自 Git 历史 Commit
    Git,
    /// 来自工作目录 (未提交)
    Workdir,
    /// 来自会话日志 (tool_use Write 操作)
    Session,
}

/// 历史快照数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 文件内容
    pub content: String,
    /// Commit SHA (工作目录或会话来源时为空)
    pub commit_hash: String,
    /// Commit 消息 (工作目录或会话来源时为空)
    pub message: String,
    /// 作者名 (工作目录或会话来源时为空)
    pub author: String,
    /// 作者邮箱 (工作目录或会话来源时为空)
    pub author_email: String,
    /// Commit 时间 (工作目录或会话来源时为请求时间)
    pub committed_at: DateTime<Utc>,
    /// 文件路径
    pub file_path: String,
    /// 快照来源 (Story 2.30)
    #[serde(default = "default_source")]
    pub source: SnapshotSource,
}

/// 默认来源为 Git (保持向后兼容)
fn default_source() -> SnapshotSource {
    SnapshotSource::Git
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

    /// 获取指定时间范围内的所有 Commits
    ///
    /// 遍历 Git 历史，返回在 [start_timestamp, end_timestamp] 范围内的所有 Commit。
    /// 结果按时间升序排列（从旧到新）。
    ///
    /// # Arguments
    /// * `start_timestamp` - 开始时间 (Unix seconds)
    /// * `end_timestamp` - 结束时间 (Unix seconds)
    ///
    /// # Returns
    /// 返回符合时间范围的 CommitInfo 列表，如果没有匹配返回空 Vec
    pub fn get_commits_in_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<CommitInfo>, GitError> {
        let mut commits = Vec::new();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME)?;

        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            let commit_time = commit.time().seconds();

            // 跳过比 end_timestamp 更新的 commits
            if commit_time > end_timestamp {
                continue;
            }

            // 如果 commit 时间早于 start_timestamp，停止遍历（因为是按时间降序遍历）
            if commit_time < start_timestamp {
                break;
            }

            // commit 在范围内，收集信息
            let author = commit.author();
            let committed_at = Utc
                .timestamp_opt(commit_time, 0)
                .single()
                .unwrap_or_else(Utc::now);

            commits.push(CommitInfo {
                commit_hash: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: author.name().unwrap_or("Unknown").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                committed_at,
            });
        }

        // 反转为时间升序（从旧到新）
        commits.reverse();
        Ok(commits)
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
            source: SnapshotSource::Git,
        })
    }

    /// 获取文件快照，支持分层回退 (Story 2.30)
    ///
    /// 回退策略:
    /// 1. 先尝试 Git Commit 历史
    /// 2. 失败时 → 检查工作目录是否存在该文件
    /// 3. 全部失败 → 返回 FileNotFound 错误
    ///
    /// # Arguments
    /// * `timestamp` - 目标时间戳
    /// * `file_path` - 文件路径 (可以是相对路径或绝对路径)
    ///
    /// # Returns
    /// 返回包含内容、元数据和来源的 Snapshot
    pub fn get_file_with_fallback(
        &self,
        timestamp: DateTime<Utc>,
        file_path: &str,
    ) -> Result<Snapshot, GitError> {
        // 策略 1: 尝试 Git 历史
        match self.get_snapshot_at_time(timestamp, file_path) {
            Ok(snapshot) => return Ok(snapshot),
            Err(GitError::FileNotFound { .. }) => {
                // 文件在 Git 历史中不存在，继续回退
            }
            Err(e) => return Err(e), // 其他错误直接返回
        }

        // 策略 2: 尝试工作目录
        let file_path_buf = std::path::PathBuf::from(file_path);
        let full_path = if file_path_buf.is_absolute() {
            // 绝对路径，直接使用
            file_path_buf
        } else {
            // 相对路径，与工作目录拼接
            let repo_workdir = self.repo.workdir().ok_or_else(|| {
                GitError::NotARepository("Bare repository has no workdir".to_string())
            })?;
            repo_workdir.join(file_path)
        };

        if full_path.exists() && full_path.is_file() {
            let content = std::fs::read_to_string(&full_path).map_err(|_| GitError::InvalidUtf8)?;
            return Ok(Snapshot {
                content,
                commit_hash: String::new(),
                message: String::new(),
                author: String::new(),
                author_email: String::new(),
                committed_at: Utc::now(),
                file_path: file_path.to_string(),
                source: SnapshotSource::Workdir,
            });
        }

        // 策略 3: 全部失败，返回错误
        Err(GitError::FileNotFound {
            commit: "workdir".to_string(),
            path: file_path.to_string(),
        })
    }
}

#[cfg(test)]
mod tests;
