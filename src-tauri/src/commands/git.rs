//! Git 相关的 Tauri IPC 命令
//!
//! 提供前端调用的 Git Time Machine 功能接口。

use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tauri::async_runtime::spawn_blocking;

use crate::error::AppError;
use crate::git::{GitTimeMachine, Snapshot};

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
