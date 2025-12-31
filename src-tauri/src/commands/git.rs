//! Git 相关的 Tauri IPC 命令
//!
//! 提供前端调用的 Git Time Machine 功能接口。

use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use std::path::PathBuf;
use tauri::async_runtime::spawn_blocking;

use crate::error::AppError;
use crate::git::{GitTimeMachine, Snapshot};

/// 前端友好的快照结果（与 useTimeMachine.ts 对齐）
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotResult {
    /// 文件内容
    pub content: String,
    /// Commit Hash
    pub commit_hash: String,
    /// Commit 消息
    pub commit_message: String,
    /// Commit 时间戳 (Unix seconds)
    pub commit_timestamp: i64,
}

impl From<Snapshot> for SnapshotResult {
    fn from(snapshot: Snapshot) -> Self {
        Self {
            content: snapshot.content,
            commit_hash: snapshot.commit_hash,
            commit_message: snapshot.message,
            commit_timestamp: snapshot.committed_at.timestamp(),
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
}
