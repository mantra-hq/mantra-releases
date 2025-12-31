//! 文件树相关的 Tauri IPC 命令
//!
//! Story 2.13: Task 5 - AC #9, #19
//! 提供前端调用的文件树功能接口。

use git2::{ObjectType, Repository, Tree};
use serde::Serialize;
use std::path::Path;
use tauri::async_runtime::spawn_blocking;

use crate::error::AppError;

/// 树节点数据结构
#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    /// 文件/目录名
    pub name: String,
    /// 完整相对路径
    pub path: String,
    /// 节点类型: "file" | "directory"
    #[serde(rename = "type")]
    pub node_type: String,
    /// 子节点 (仅目录)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNode>>,
}

/// 列出指定 commit 的文件树
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `commit_hash` - Commit hash (可选，默认 HEAD)
/// * `subpath` - 子目录路径 (可选，默认根目录)
///
/// # Returns
/// 返回 TreeNode 列表，包含完整的目录结构
#[tauri::command]
pub async fn list_tree_at_commit(
    repo_path: String,
    commit_hash: Option<String>,
    subpath: Option<String>,
) -> Result<Vec<TreeNode>, AppError> {
    spawn_blocking(move || {
        let repo = Repository::open(&repo_path)
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

        // 获取 commit
        let commit = if let Some(hash) = commit_hash {
            let oid = git2::Oid::from_str(&hash)
                .map_err(|e| AppError::Internal(format!("无效的 commit hash: {}", e)))?;
            repo.find_commit(oid)
                .map_err(|e| AppError::Internal(format!("未找到 commit: {}", e)))?
        } else {
            let head = repo
                .head()
                .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
            head.peel_to_commit()
                .map_err(|e| AppError::Internal(format!("HEAD 不是 commit: {}", e)))?
        };

        // 获取树对象
        let tree = commit
            .tree()
            .map_err(|e| AppError::Internal(format!("无法获取树对象: {}", e)))?;

        // 如果指定了路径，获取子树
        let target_tree = if let Some(ref sp) = subpath {
            let entry = tree
                .get_path(Path::new(sp))
                .map_err(|e| AppError::Internal(format!("路径不存在: {}", e)))?;
            let obj = entry
                .to_object(&repo)
                .map_err(|e| AppError::Internal(format!("无法获取对象: {}", e)))?;
            obj.into_tree()
                .map_err(|_| AppError::Internal("路径不是目录".to_string()))?
        } else {
            tree
        };

        // 构建树节点
        let nodes = build_tree_nodes(&repo, &target_tree, subpath.as_deref().unwrap_or(""))?;

        Ok(nodes)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// 递归构建树节点
fn build_tree_nodes(
    repo: &Repository,
    tree: &Tree,
    parent_path: &str,
) -> Result<Vec<TreeNode>, AppError> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    for entry in tree.iter() {
        let name = entry
            .name()
            .ok_or_else(|| AppError::Internal("无效的文件名".to_string()))?
            .to_string();

        let path = if parent_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", parent_path, name)
        };

        let node = match entry.kind() {
            Some(ObjectType::Tree) => {
                // 目录：递归获取子节点
                let subtree = entry
                    .to_object(repo)
                    .map_err(|e| AppError::Internal(format!("无法获取子树: {}", e)))?
                    .into_tree()
                    .map_err(|_| AppError::Internal("对象不是树".to_string()))?;

                let children = build_tree_nodes(repo, &subtree, &path)?;

                TreeNode {
                    name,
                    path,
                    node_type: "directory".to_string(),
                    children: Some(children),
                }
            }
            Some(ObjectType::Blob) => {
                // 文件
                TreeNode {
                    name,
                    path,
                    node_type: "file".to_string(),
                    children: None,
                }
            }
            _ => continue, // 跳过其他类型
        };

        nodes.push(node);
    }

    // 排序：目录在前，按名称字母顺序
    nodes.sort_by(|a, b| {
        match (&a.node_type[..], &b.node_type[..]) {
            ("directory", "file") => std::cmp::Ordering::Less,
            ("file", "directory") => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(nodes)
}

/// 列出指定 commit 的所有文件路径 (用于 QuickOpen)
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
/// * `commit_hash` - Commit hash (可选，默认 HEAD)
///
/// # Returns
/// 返回所有文件的相对路径列表
#[tauri::command]
pub async fn list_files_at_commit(
    repo_path: String,
    commit_hash: Option<String>,
) -> Result<Vec<String>, AppError> {
    spawn_blocking(move || {
        let repo = Repository::open(&repo_path)
            .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;

        // 获取 commit
        let commit = if let Some(hash) = commit_hash {
            let oid = git2::Oid::from_str(&hash)
                .map_err(|e| AppError::Internal(format!("无效的 commit hash: {}", e)))?;
            repo.find_commit(oid)
                .map_err(|e| AppError::Internal(format!("未找到 commit: {}", e)))?
        } else {
            let head = repo
                .head()
                .map_err(|e| AppError::Git(crate::git::GitError::RepositoryError(e)))?;
            head.peel_to_commit()
                .map_err(|e| AppError::Internal(format!("HEAD 不是 commit: {}", e)))?
        };

        // 获取树对象
        let tree = commit
            .tree()
            .map_err(|e| AppError::Internal(format!("无法获取树对象: {}", e)))?;

        // 收集所有文件路径
        let mut files: Vec<String> = Vec::new();
        collect_files(&repo, &tree, "", &mut files)?;

        // 按路径排序
        files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        Ok(files)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// 递归收集文件路径
fn collect_files(
    repo: &Repository,
    tree: &Tree,
    parent_path: &str,
    files: &mut Vec<String>,
) -> Result<(), AppError> {
    for entry in tree.iter() {
        let name = entry
            .name()
            .ok_or_else(|| AppError::Internal("无效的文件名".to_string()))?;

        let path = if parent_path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", parent_path, name)
        };

        match entry.kind() {
            Some(ObjectType::Tree) => {
                let subtree = entry
                    .to_object(repo)
                    .map_err(|e| AppError::Internal(format!("无法获取子树: {}", e)))?
                    .into_tree()
                    .map_err(|_| AppError::Internal("对象不是树".to_string()))?;

                collect_files(repo, &subtree, &path, files)?;
            }
            Some(ObjectType::Blob) => {
                files.push(path);
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试无效仓库路径
    #[tokio::test]
    async fn test_list_tree_invalid_repo() {
        let result = list_tree_at_commit(
            "/nonexistent/path".to_string(),
            None,
            None,
        )
        .await;

        assert!(result.is_err());
    }

    /// 测试无效 commit hash
    #[tokio::test]
    async fn test_list_tree_invalid_commit() {
        // 使用当前项目仓库
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        let result = list_tree_at_commit(
            repo_path,
            Some("invalid_hash".to_string()),
            None,
        )
        .await;

        assert!(result.is_err());
    }

    /// 测试列出根目录
    #[tokio::test]
    async fn test_list_tree_root() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        let result = list_tree_at_commit(repo_path, None, None).await;

        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(!nodes.is_empty());
    }

    /// 测试列出文件路径
    #[tokio::test]
    async fn test_list_files_at_commit() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let repo_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string());

        let result = list_files_at_commit(repo_path, None).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(!files.is_empty());
        // 应该包含一些已知文件
        println!("Found {} files", files.len());
    }
}

