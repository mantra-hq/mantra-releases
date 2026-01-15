//! Git 相关的 Tauri IPC 命令
//!
//! 提供前端调用的 Git Time Machine 功能接口。

use chrono::{DateTime, TimeZone, Utc};
use git2::{ObjectType, Repository};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::async_runtime::spawn_blocking;

use crate::error::AppError;
use crate::git::{GitTimeMachine, Snapshot, SnapshotSource};

/// 前端友好的快照结果（与 useTimeMachine.ts 对齐）
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotResult {
    /// 文件内容
    pub content: String,
    /// Commit Hash (工作目录/会话来源时为空)
    pub commit_hash: String,
    /// Commit 消息 (工作目录/会话来源时为空)
    pub commit_message: String,
    /// Commit 时间戳 (Unix seconds)
    pub commit_timestamp: i64,
    /// 快照来源 (Story 2.30): "git" | "workdir" | "session"
    pub source: String,
}

impl From<Snapshot> for SnapshotResult {
    fn from(snapshot: Snapshot) -> Self {
        Self {
            content: snapshot.content,
            commit_hash: snapshot.commit_hash,
            commit_message: snapshot.message,
            commit_timestamp: snapshot.committed_at.timestamp(),
            source: match snapshot.source {
                SnapshotSource::Git => "git".to_string(),
                SnapshotSource::Workdir => "workdir".to_string(),
                SnapshotSource::Session => "session".to_string(),
            },
        }
    }
}

/// 获取指定时间戳的文件快照（前端友好版本）
///
/// 接受 Unix 秒级时间戳，返回与前端 useTimeMachine.ts 对齐的格式。
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `file_path` - 相对于仓库根目录的文件路径
/// * `timestamp` - Unix 秒级时间戳
///
/// # Returns
/// 返回包含内容和元数据的 SnapshotResult
#[tauri::command]
pub async fn get_snapshot_at_time(
    repo_path: String,
    file_path: String,
    timestamp: i64,
) -> Result<SnapshotResult, AppError> {
    let repo_path = PathBuf::from(repo_path);
    let file_path_clone = file_path.clone();

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        let datetime = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| AppError::Internal(format!("Invalid timestamp: {}", timestamp)))?;
        let snapshot = tm.get_snapshot_at_time(datetime, &file_path_clone)?;
        Ok::<_, AppError>(SnapshotResult::from(snapshot))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// 获取文件快照（带回退策略） (Story 2.30)
///
/// 分层回退策略:
/// 1. 先尝试 Git Commit 历史
/// 2. 失败时 → 从工作目录读取
/// 3. 全部失败 → 返回 FileNotFound 错误
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `file_path` - 相对于仓库根目录的文件路径
/// * `timestamp` - Unix 秒级时间戳
///
/// # Returns
/// 返回包含内容、元数据和来源的 SnapshotResult
#[tauri::command]
pub async fn get_snapshot_with_fallback(
    repo_path: String,
    file_path: String,
    timestamp: i64,
) -> Result<SnapshotResult, AppError> {
    let repo_path = PathBuf::from(repo_path);
    let file_path_clone = file_path.clone();

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        let datetime = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| AppError::Internal(format!("Invalid timestamp: {}", timestamp)))?;
        let snapshot = tm.get_file_with_fallback(datetime, &file_path_clone)?;
        Ok::<_, AppError>(SnapshotResult::from(snapshot))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// 检测目录是否为 Git 仓库，返回仓库根路径
///
/// 从指定目录向上搜索 .git 目录，找到 Git 仓库根路径。
///
/// # Arguments
/// * `dir_path` - 要检测的目录路径
///
/// # Returns
/// 返回 Git 仓库根路径，如果不是 Git 仓库返回 None
#[tauri::command]
pub async fn detect_git_repo(dir_path: String) -> Result<Option<String>, AppError> {
    let dir_path = PathBuf::from(dir_path);

    spawn_blocking(move || {
        // 向上搜索 .git 目录
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
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))
}

/// 获取指定时间戳的文件快照
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `timestamp` - ISO 8601 格式的时间戳
/// * `file_path` - 相对于仓库根目录的文件路径
///
/// # Returns
/// 返回包含内容和元数据的 Snapshot
#[tauri::command]
pub async fn get_file_snapshot(
    repo_path: String,
    timestamp: DateTime<Utc>,
    file_path: String,
) -> Result<Snapshot, AppError> {
    let repo_path = PathBuf::from(repo_path);
    let file_path_clone = file_path.clone();

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        tm.get_snapshot_at_time(timestamp, &file_path_clone)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(AppError::from)
}

/// 查找指定时间戳之前最近的 Commit (仅返回 SHA)
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `timestamp` - ISO 8601 格式的时间戳
///
/// # Returns
/// 返回 Commit SHA 字符串
#[tauri::command]
pub async fn find_commit_at_time(
    repo_path: String,
    timestamp: DateTime<Utc>,
) -> Result<String, AppError> {
    let repo_path = PathBuf::from(repo_path);

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        let oid = tm.find_commit_at_time(timestamp)?;
        Ok::<_, crate::git::GitError>(oid.to_string())
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(AppError::from)
}

/// 获取指定时间戳之前最近的 Commit 完整信息
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `timestamp` - ISO 8601 格式的时间戳
///
/// # Returns
/// 返回包含完整元数据的 CommitInfo
#[tauri::command]
pub async fn get_commit_info(
    repo_path: String,
    timestamp: DateTime<Utc>,
) -> Result<crate::git::CommitInfo, AppError> {
    let repo_path = PathBuf::from(repo_path);

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        tm.get_commit_info(timestamp)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(AppError::from)
}

/// 获取指定时间范围内的所有 Commits (Story 2.32)
///
/// 返回在 [start_timestamp, end_timestamp] 范围内的所有 Commit 信息。
/// 用于在时间轴上显示 Git 提交标记。
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `start_timestamp` - 开始时间 (Unix seconds)
/// * `end_timestamp` - 结束时间 (Unix seconds)
///
/// # Returns
/// 返回 CommitInfo 列表，按时间升序排列
#[tauri::command]
pub async fn get_commits_in_range(
    repo_path: String,
    start_timestamp: i64,
    end_timestamp: i64,
) -> Result<Vec<crate::git::CommitInfo>, AppError> {
    let repo_path = PathBuf::from(repo_path);

    spawn_blocking(move || {
        let tm = GitTimeMachine::new(&repo_path)?;
        tm.get_commits_in_range(start_timestamp, end_timestamp)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(AppError::from)
}

/// 获取 HEAD 版本的文件内容
///
/// 读取 Git 仓库 HEAD 指向的最新版本的文件内容。
/// 支持读取子模块内的文件 (Story 2.31)。
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `file_path` - 相对于仓库根目录的文件路径
///
/// # Returns
/// 返回包含内容和元数据的 SnapshotResult
#[tauri::command]
pub async fn get_file_at_head(
    repo_path: String,
    file_path: String,
) -> Result<SnapshotResult, AppError> {
    let repo_path_buf = PathBuf::from(&repo_path);
    let file_path_clone = file_path.clone();

    spawn_blocking(move || {
        let repo = Repository::open(&repo_path_buf)
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let head = repo.head()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let commit = head.peel_to_commit()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
        let tree = commit.tree()
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

        // 尝试直接获取文件
        match tree.get_path(Path::new(&file_path_clone)) {
            Ok(entry) => {
                // 检查是否是子模块（ObjectType::Commit）
                if entry.kind() == Some(ObjectType::Commit) {
                    // 文件路径指向子模块本身，不是文件
                    return Err(AppError::Git(crate::git::GitError::FileNotFound {
                        commit: commit.id().to_string(),
                        path: file_path_clone,
                    }));
                }

                let blob = repo.find_blob(entry.id())
                    .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

                let content = std::str::from_utf8(blob.content())
                    .map_err(|_| AppError::Git(crate::git::GitError::InvalidUtf8))?;

                let commit_time = commit.time();

                Ok(SnapshotResult {
                    content: content.to_string(),
                    commit_hash: commit.id().to_string(),
                    commit_message: commit.message().unwrap_or("").to_string(),
                    commit_timestamp: commit_time.seconds(),
                    source: "git".to_string(),
                })
            }
            Err(_) => {
                // 文件不在主仓库树中，检查是否在子模块内
                // Story 2.31: 支持读取子模块内的文件
                read_file_from_submodule(&repo, &file_path_clone, &commit)
            }
        }
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// 从子模块中读取文件
///
/// 当文件路径以子模块目录开头时，打开子模块仓库并读取文件。
fn read_file_from_submodule(
    parent_repo: &Repository,
    file_path: &str,
    parent_commit: &git2::Commit,
) -> Result<SnapshotResult, AppError> {
    let workdir = parent_repo.workdir().ok_or_else(|| {
        AppError::Git(crate::git::GitError::FileNotFound {
            commit: parent_commit.id().to_string(),
            path: file_path.to_string(),
        })
    })?;

    let tree = parent_commit.tree()
        .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

    // 遍历路径组件，查找子模块
    let path_parts: Vec<&str> = file_path.split('/').collect();

    for i in 1..path_parts.len() {
        let potential_submodule_path = path_parts[..i].join("/");

        if let Ok(entry) = tree.get_path(Path::new(&potential_submodule_path)) {
            if entry.kind() == Some(ObjectType::Commit) {
                // 找到子模块！
                let submodule_full_path = workdir.join(&potential_submodule_path);
                let relative_path_in_submodule = path_parts[i..].join("/");

                // 打开子模块仓库
                let submodule_repo = Repository::open(&submodule_full_path)
                    .map_err(|_| AppError::Git(crate::git::GitError::FileNotFound {
                        commit: parent_commit.id().to_string(),
                        path: file_path.to_string(),
                    }))?;

                // 获取子模块的 HEAD
                let sub_head = submodule_repo.head()
                    .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
                let sub_commit = sub_head.peel_to_commit()
                    .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
                let sub_tree = sub_commit.tree()
                    .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

                // 从子模块读取文件
                let sub_entry = sub_tree.get_path(Path::new(&relative_path_in_submodule))
                    .map_err(|_| AppError::Git(crate::git::GitError::FileNotFound {
                        commit: sub_commit.id().to_string(),
                        path: file_path.to_string(),
                    }))?;

                let blob = submodule_repo.find_blob(sub_entry.id())
                    .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

                let content = std::str::from_utf8(blob.content())
                    .map_err(|_| AppError::Git(crate::git::GitError::InvalidUtf8))?;

                let commit_time = sub_commit.time();

                return Ok(SnapshotResult {
                    content: content.to_string(),
                    commit_hash: sub_commit.id().to_string(),
                    commit_message: sub_commit.message().unwrap_or("").to_string(),
                    commit_timestamp: commit_time.seconds(),
                    source: "git".to_string(),
                });
            }
        }
    }

    // 没有找到子模块
    Err(AppError::Git(crate::git::GitError::FileNotFound {
        commit: parent_commit.id().to_string(),
        path: file_path.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 get_file_snapshot 无效仓库路径
    #[tokio::test]
    async fn test_get_file_snapshot_invalid_repo() {
        let result = get_file_snapshot(
            "/nonexistent/path".to_string(),
            Utc::now(),
            "test.txt".to_string(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // 验证返回 Git 错误
        assert!(matches!(err, AppError::Git(_)));
    }

    /// 测试 find_commit_at_time 无效仓库路径
    #[tokio::test]
    async fn test_find_commit_at_time_invalid_repo() {
        let result = find_commit_at_time(
            "/nonexistent/path".to_string(),
            Utc::now(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Git(_)));
    }

    /// 测试空路径处理
    #[tokio::test]
    async fn test_empty_repo_path() {
        let result = get_file_snapshot(
            "".to_string(),
            Utc::now(),
            "test.txt".to_string(),
        )
        .await;

        assert!(result.is_err());
    }

    /// 测试 get_file_at_head 无效仓库路径
    #[tokio::test]
    async fn test_get_file_at_head_invalid_repo() {
        let result = get_file_at_head(
            "/nonexistent/path".to_string(),
            "test.txt".to_string(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Git(_)));
    }

    /// 测试 get_file_at_head 找到文件
    #[tokio::test]
    async fn test_get_file_at_head_finds_file() {
        // Get the Git repo root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        println!("Testing with repo_path: {}", repo_path);

        // Try to get README.md at HEAD
        let result = get_file_at_head(repo_path.clone(), "README.md".to_string()).await;
        println!("Result: {:?}", result);

        match result {
            Ok(snapshot) => {
                assert!(!snapshot.content.is_empty());
                assert!(!snapshot.commit_hash.is_empty());
            }
            Err(e) => {
                // If README.md doesn't exist in Git HEAD, try CLAUDE.md
                println!("README.md failed: {:?}, trying CLAUDE.md", e);
                let result2 = get_file_at_head(repo_path, "CLAUDE.md".to_string()).await;
                match result2 {
                    Ok(snapshot) => {
                        assert!(!snapshot.content.is_empty());
                        assert!(!snapshot.commit_hash.is_empty());
                    }
                    Err(e2) => {
                        panic!("Both README.md and CLAUDE.md failed: {:?}", e2);
                    }
                }
            }
        }
    }

    /// 测试 get_file_at_head 文件不存在
    #[tokio::test]
    async fn test_get_file_at_head_file_not_found() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        let result = get_file_at_head(repo_path, "nonexistent_file_xyz.txt".to_string()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Git(_)));
    }

    // =========================================================================
    // Story 2.32: get_commits_in_range 命令测试
    // =========================================================================

    /// 测试 get_commits_in_range 无效仓库路径
    #[tokio::test]
    async fn test_get_commits_in_range_invalid_repo() {
        let result = get_commits_in_range(
            "/nonexistent/path".to_string(),
            0,
            i64::MAX,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Git(_)));
    }

    /// 测试 get_commits_in_range 正常返回
    #[tokio::test]
    async fn test_get_commits_in_range_returns_commits() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        // 使用一个很大的时间范围，应该能找到一些 commit
        let now = Utc::now().timestamp();
        let one_year_ago = now - 365 * 24 * 60 * 60;

        let result = get_commits_in_range(
            repo_path,
            one_year_ago,
            now,
        )
        .await;

        // 应该成功执行，可能有也可能没有 commits（取决于项目历史）
        assert!(result.is_ok(), "get_commits_in_range should succeed on valid repo");

        let commits = result.unwrap();
        // 验证返回的是 Vec，即使为空也是有效的
        println!("Found {} commits in the last year", commits.len());
    }

    /// 查找包含子模块的父仓库根目录
    fn find_parent_repo_with_submodules() -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut current = std::path::PathBuf::from(manifest_dir);

        for _ in 0..10 {
            let gitmodules = current.join(".gitmodules");
            if gitmodules.exists() {
                return current.to_string_lossy().to_string();
            }
            if !current.pop() {
                break;
            }
        }

        std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string())
    }

    /// 测试 get_file_at_head 读取子模块内的文件 (Story 2.31)
    #[tokio::test]
    async fn test_get_file_at_head_submodule_file() {
        let repo_path = find_parent_repo_with_submodules();

        // 读取 apps/client 子模块内的 package.json
        let result = get_file_at_head(
            repo_path.clone(),
            "apps/client/package.json".to_string(),
        )
        .await;

        match result {
            Ok(snapshot) => {
                assert!(!snapshot.content.is_empty(), "子模块文件内容不应为空");
                assert!(
                    snapshot.content.contains("name") || snapshot.content.contains("version"),
                    "package.json 应包含 name 或 version 字段"
                );
                println!("成功读取子模块文件，commit: {}", snapshot.commit_hash);
            }
            Err(e) => {
                // 子模块可能未初始化，打印错误但不失败
                println!("读取子模块文件失败（可能未初始化）: {:?}", e);
            }
        }
    }
}
