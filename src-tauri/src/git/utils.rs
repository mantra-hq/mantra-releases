//! Git 工具函数
//!
//! Story 1.9: Git Remote URL 提取和规范化

use std::path::Path;

use git2::Repository;

use super::error::GitError;

/// 从 Git 仓库中提取 remote URL
///
/// # Arguments
/// * `repo_path` - Git 仓库路径
///
/// # Returns
/// 返回规范化后的 remote URL，如果不是 Git 仓库或没有 remote 则返回 None
///
/// # Logic
/// 1. 优先使用 `origin` remote
/// 2. 如果没有 `origin`，使用第一个可用的 remote
/// 3. 返回规范化后的 URL
pub fn get_git_remote_url(repo_path: &Path) -> Result<Option<String>, GitError> {
    let repo = match Repository::open(repo_path) {
        Ok(r) => r,
        Err(e) => {
            // 非 Git 仓库不是错误，返回 None
            if e.code() == git2::ErrorCode::NotFound {
                return Ok(None);
            }
            return Err(GitError::RepositoryError(e));
        }
    };

    // 获取 remote 名称列表
    let remotes = match repo.remotes() {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    // 优先使用 origin，否则使用第一个 remote
    let remote_name = if remotes.iter().any(|r| r == Some("origin")) {
        "origin"
    } else {
        match remotes.get(0) {
            Some(name) => name,
            None => return Ok(None),
        }
    };

    // 获取 remote 的 URL
    let remote = match repo.find_remote(remote_name) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    match remote.url() {
        Some(url) => Ok(Some(normalize_git_url(url))),
        None => Ok(None),
    }
}

/// 规范化 Git URL
///
/// 将各种格式的 Git URL 转换为统一格式：
/// - 去除 `.git` 后缀
/// - SSH 格式转换为 HTTPS 格式 (便于比较)
///
/// # Examples
/// ```
/// use client_lib::git::normalize_git_url;
///
/// // SSH format
/// assert_eq!(
///     normalize_git_url("git@github.com:user/repo.git"),
///     "https://github.com/user/repo"
/// );
///
/// // HTTPS format with .git
/// assert_eq!(
///     normalize_git_url("https://github.com/user/repo.git"),
///     "https://github.com/user/repo"
/// );
///
/// // Already normalized
/// assert_eq!(
///     normalize_git_url("https://github.com/user/repo"),
///     "https://github.com/user/repo"
/// );
/// ```
pub fn normalize_git_url(url: &str) -> String {
    let mut url = url.trim().to_string();

    // 1. 去除 .git 后缀
    if url.ends_with(".git") {
        url = url[..url.len() - 4].to_string();
    }

    // 2. SSH 格式转换为 HTTPS
    // git@github.com:user/repo -> https://github.com/user/repo
    if url.starts_with("git@") {
        // Remove "git@" prefix
        let without_prefix = &url[4..];
        // Replace first ":" with "/"
        if let Some(colon_pos) = without_prefix.find(':') {
            let host = &without_prefix[..colon_pos];
            let path = &without_prefix[colon_pos + 1..];
            url = format!("https://{}/{}", host, path);
        }
    }

    // 3. ssh://git@host/path -> https://host/path
    if url.starts_with("ssh://git@") {
        url = url.replacen("ssh://git@", "https://", 1);
    } else if url.starts_with("ssh://") {
        url = url.replacen("ssh://", "https://", 1);
    }

    // 4. git:// -> https://
    if url.starts_with("git://") {
        url = url.replacen("git://", "https://", 1);
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    // normalize_git_url tests
    #[test]
    fn test_normalize_git_url_ssh_format() {
        assert_eq!(
            normalize_git_url("git@github.com:user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_https_with_git_suffix() {
        assert_eq!(
            normalize_git_url("https://github.com/user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_https_no_suffix() {
        assert_eq!(
            normalize_git_url("https://github.com/user/repo"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_ssh_protocol() {
        assert_eq!(
            normalize_git_url("ssh://git@github.com/user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_git_protocol() {
        assert_eq!(
            normalize_git_url("git://github.com/user/repo"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_gitlab() {
        assert_eq!(
            normalize_git_url("git@gitlab.com:group/project.git"),
            "https://gitlab.com/group/project"
        );
    }

    #[test]
    fn test_normalize_git_url_bitbucket() {
        assert_eq!(
            normalize_git_url("git@bitbucket.org:team/repo.git"),
            "https://bitbucket.org/team/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_with_whitespace() {
        assert_eq!(
            normalize_git_url("  git@github.com:user/repo.git  "),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_nested_path() {
        // Handle nested paths like organization/team/repo
        assert_eq!(
            normalize_git_url("git@github.com:org/team/repo.git"),
            "https://github.com/org/team/repo"
        );
    }

    // get_git_remote_url tests
    #[test]
    fn test_get_git_remote_url_current_repo() {
        // Test on the current Mantra project (which should have a Git repo)
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let result = get_git_remote_url(Path::new(manifest_dir));

        // Should succeed (either return Some URL or None if no remote)
        assert!(result.is_ok());

        if let Ok(Some(url)) = result {
            // URL should be normalized (https format)
            assert!(url.starts_with("https://"), "URL should be https format: {}", url);
            // URL should not end with .git
            assert!(!url.ends_with(".git"), "URL should not end with .git: {}", url);
        }
    }

    #[test]
    fn test_get_git_remote_url_non_git_dir() {
        // /tmp is typically not a Git repo
        let result = get_git_remote_url(Path::new("/tmp"));
        assert!(result.is_ok());
        // Most likely None, but could be Some if /tmp happens to be in a Git repo
    }

    #[test]
    fn test_get_git_remote_url_nonexistent_path() {
        let result = get_git_remote_url(Path::new("/nonexistent/path/xyz123"));
        // Should return Ok(None) for non-existent paths (not a Git repo)
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Additional boundary tests for normalize_git_url (Story 1.9 Code Review)

    #[test]
    fn test_normalize_git_url_ssh_without_git_suffix() {
        // SSH format without .git suffix should still work
        assert_eq!(
            normalize_git_url("git@github.com:user/repo"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_ssh_protocol_without_git_user() {
        // ssh://host/path without git@ prefix
        assert_eq!(
            normalize_git_url("ssh://github.com/user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_self_hosted_gitlab() {
        // Self-hosted GitLab with custom domain
        assert_eq!(
            normalize_git_url("git@git.company.com:team/project.git"),
            "https://git.company.com/team/project"
        );
    }

    #[test]
    fn test_normalize_git_url_with_port_in_https() {
        // HTTPS with port number
        assert_eq!(
            normalize_git_url("https://github.com:443/user/repo.git"),
            "https://github.com:443/user/repo"
        );
    }

    #[test]
    fn test_normalize_git_url_empty_string() {
        // Empty string should return empty string
        assert_eq!(normalize_git_url(""), "");
    }

    #[test]
    fn test_normalize_git_url_only_git_suffix() {
        // Edge case: only ".git"
        assert_eq!(normalize_git_url(".git"), "");
    }

    #[test]
    fn test_normalize_git_url_case_sensitivity() {
        // URL paths are case-sensitive, should be preserved
        assert_eq!(
            normalize_git_url("https://github.com/User/Repo.git"),
            "https://github.com/User/Repo"
        );
    }

    #[test]
    fn test_normalize_git_url_azure_devops() {
        // Azure DevOps SSH format
        assert_eq!(
            normalize_git_url("git@ssh.dev.azure.com:v3/org/project/repo"),
            "https://ssh.dev.azure.com/v3/org/project/repo"
        );
    }
}
