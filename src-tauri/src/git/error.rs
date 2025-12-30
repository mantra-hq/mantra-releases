//! Git 模块错误类型
//!
//! 定义 GitError 枚举，包含所有 Git 操作可能的错误类型。

use thiserror::Error;

/// Git 操作错误类型
#[derive(Error, Debug)]
pub enum GitError {
    /// 路径不是有效的 Git 仓库
    #[error("路径不是有效的 Git 仓库: {0}")]
    NotARepository(String),

    /// 找不到符合条件的 Commit
    #[error("找不到 Commit: {0}")]
    CommitNotFound(String),

    /// 在指定 Commit 中找不到文件
    #[error("在 Commit {commit} 中找不到文件: {path}")]
    FileNotFound {
        /// Commit SHA
        commit: String,
        /// 文件路径
        path: String,
    },

    /// 文件内容不是有效的 UTF-8
    #[error("文件内容不是有效的 UTF-8")]
    InvalidUtf8,

    /// Git 仓库底层错误
    #[error("Git 仓库错误: {0}")]
    RepositoryError(#[from] git2::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GitError::NotARepository("/invalid/path".to_string());
        assert!(err.to_string().contains("不是有效的 Git 仓库"));
    }

    #[test]
    fn test_file_not_found_display() {
        let err = GitError::FileNotFound {
            commit: "abc123".to_string(),
            path: "src/main.rs".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("src/main.rs"));
    }
}
