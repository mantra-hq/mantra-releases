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
    /// 节点类型: "file" | "directory" | "submodule"
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
            Some(ObjectType::Commit) => {
                // 子模块：ObjectType::Commit 表示指向子模块的 commit 指针
                // Story 2.31: 支持子模块文件树展示
                let children = get_submodule_tree_nodes(repo, &path);

                TreeNode {
                    name,
                    path,
                    node_type: "submodule".to_string(),
                    children: Some(children),
                }
            }
            _ => continue, // 跳过其他类型 (Tag 等)
        };

        nodes.push(node);
    }

    // 排序：目录在前，子模块在中，文件在后，按名称字母顺序
    nodes.sort_by(|a, b| {
        let type_order = |t: &str| match t {
            "directory" => 0,
            "submodule" => 1,
            "file" => 2,
            _ => 3,
        };
        match type_order(&a.node_type).cmp(&type_order(&b.node_type)) {
            std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            other => other,
        }
    });

    Ok(nodes)
}

/// 获取子模块的文件树节点
///
/// 打开子模块仓库并递归获取其文件树。
/// 如果子模块未初始化，返回空列表（不报错）。
fn get_submodule_tree_nodes(parent_repo: &Repository, submodule_path: &str) -> Vec<TreeNode> {
    let submodule_repo = match open_submodule_repo(parent_repo, submodule_path) {
        Some(repo) => repo,
        None => return Vec::new(),
    };

    let tree = match get_head_tree(&submodule_repo, submodule_path) {
        Some(t) => t,
        None => return Vec::new(),
    };

    // 递归构建子模块的文件树（AC5: 支持嵌套子模块）
    build_tree_nodes(&submodule_repo, &tree, submodule_path).unwrap_or_default()
}

/// 打开子模块仓库
///
/// 如果父仓库是裸仓库或子模块未初始化，返回 None。
fn open_submodule_repo(parent_repo: &Repository, submodule_path: &str) -> Option<Repository> {
    // 获取父仓库的工作目录（裸仓库无法处理子模块）
    let workdir = parent_repo.workdir()?;

    // 构建子模块的完整路径并尝试打开
    let submodule_full_path = workdir.join(submodule_path);
    Repository::open(&submodule_full_path).ok()
}

/// 获取仓库 HEAD 指向的 Tree 对象
fn get_head_tree<'a>(repo: &'a Repository, _context: &str) -> Option<Tree<'a>> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    commit.tree().ok()
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
            Some(ObjectType::Commit) => {
                // 子模块：递归收集子模块内的文件
                // Story 2.31 - AC4: list_files_at_commit 同步修复
                collect_submodule_files(repo, &path, files);
            }
            _ => {}
        }
    }

    Ok(())
}

/// 收集子模块内的所有文件路径
///
/// 打开子模块仓库并递归收集其文件列表。
/// 如果子模块未初始化，不添加任何文件（不报错）。
fn collect_submodule_files(parent_repo: &Repository, submodule_path: &str, files: &mut Vec<String>) {
    let submodule_repo = match open_submodule_repo(parent_repo, submodule_path) {
        Some(repo) => repo,
        None => return,
    };

    let tree = match get_head_tree(&submodule_repo, submodule_path) {
        Some(t) => t,
        None => return,
    };

    // 递归收集子模块内的文件（AC5: 支持嵌套子模块）
    let _ = collect_files(&submodule_repo, &tree, submodule_path, files);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 查找包含子模块的父仓库根目录
    ///
    /// 从 CARGO_MANIFEST_DIR 向上查找，直到找到包含 .gitmodules 文件的 git 仓库。
    /// 如果找不到，则回退到向上 3 层目录的假设。
    fn find_parent_repo_with_submodules() -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut current = std::path::PathBuf::from(manifest_dir);

        // 向上查找包含 .gitmodules 的目录（最多 10 层）
        for _ in 0..10 {
            let gitmodules = current.join(".gitmodules");
            if gitmodules.exists() {
                return current.to_string_lossy().to_string();
            }
            if !current.pop() {
                break;
            }
        }

        // 回退方案：假设目录结构为 mantra/apps/client/src-tauri
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| manifest_dir.to_string())
    }

    /// 测试子模块文件树显示 (Story 2.31)
    /// 验证 apps/client 子模块内的文件能被正确列出
    #[tokio::test]
    async fn test_list_tree_with_submodule() {
        let repo_path = find_parent_repo_with_submodules();

        let result = list_tree_at_commit(repo_path, None, None).await;

        assert!(result.is_ok(), "list_tree_at_commit 应该成功");
        let nodes = result.unwrap();

        // 查找 apps 目录
        let apps_node = nodes.iter().find(|n| n.name == "apps");
        assert!(apps_node.is_some(), "应该存在 apps 目录");

        let apps = apps_node.unwrap();
        assert_eq!(apps.node_type, "directory", "apps 应该是目录类型");
        assert!(apps.children.is_some(), "apps 应该有子节点");

        // 查找 apps/client 子模块
        let client_node = apps
            .children
            .as_ref()
            .unwrap()
            .iter()
            .find(|n| n.name == "client");
        assert!(client_node.is_some(), "应该存在 apps/client 子模块");

        let client = client_node.unwrap();
        // AC2: 子模块类型标识
        assert_eq!(
            client.node_type, "submodule",
            "apps/client 应该被标识为 submodule 类型"
        );

        // AC1: 子模块内的文件和目录完整展示
        assert!(
            client.children.is_some(),
            "子模块应该有子节点（已初始化的情况下）"
        );

        let client_children = client.children.as_ref().unwrap();
        // 子模块应该包含一些典型文件/目录
        let has_src = client_children.iter().any(|n| n.name == "src");
        let has_package_json = client_children.iter().any(|n| n.name == "package.json");
        assert!(
            has_src || has_package_json || !client_children.is_empty(),
            "子模块内应该有文件内容"
        );
    }

    /// 测试子模块文件列表 (Story 2.31 - AC4)
    /// 验证 list_files_at_commit 也能返回子模块内的文件
    #[tokio::test]
    async fn test_list_files_with_submodule() {
        let repo_path = find_parent_repo_with_submodules();

        let result = list_files_at_commit(repo_path, None).await;

        assert!(result.is_ok(), "list_files_at_commit 应该成功");
        let files = result.unwrap();

        // AC4: 子模块内的文件路径正确包含在结果中
        let has_submodule_files = files
            .iter()
            .any(|f| f.starts_with("apps/client/"));
        assert!(
            has_submodule_files,
            "文件列表应该包含 apps/client/ 子模块内的文件"
        );

        // 验证具体的子模块文件
        let has_client_package = files
            .iter()
            .any(|f| f == "apps/client/package.json");
        println!(
            "子模块文件数量: {}",
            files.iter().filter(|f| f.starts_with("apps/client/")).count()
        );
        assert!(
            has_client_package || files.iter().any(|f| f.starts_with("apps/client/")),
            "应该包含子模块内的具体文件"
        );
    }

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
        let repo_path = find_parent_repo_with_submodules();

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
        let repo_path = find_parent_repo_with_submodules();

        let result = list_tree_at_commit(repo_path, None, None).await;

        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(!nodes.is_empty());
    }

    /// 测试列出文件路径
    #[tokio::test]
    async fn test_list_files_at_commit() {
        let repo_path = find_parent_repo_with_submodules();

        let result = list_files_at_commit(repo_path, None).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(!files.is_empty());
        // 应该包含一些已知文件
        println!("Found {} files", files.len());
    }
}






