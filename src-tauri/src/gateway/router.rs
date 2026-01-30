//! 上下文路由器
//!
//! Story 11.5: 上下文路由 - Task 2
//!
//! 使用最长前缀匹配 (LPM) 算法将工作目录映射到项目

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lru::LruCache;
use tokio::sync::Mutex;

use crate::storage::Database;

/// 项目上下文
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// 项目 ID
    pub project_id: String,
    /// 项目名称
    pub project_name: String,
    /// 匹配的路径
    pub matched_path: PathBuf,
    /// 匹配长度
    pub match_length: usize,
}

/// LPM 路由器
///
/// 使用最长前缀匹配算法将工作目录映射到项目
pub struct ContextRouter {
    db: Arc<Database>,
    /// 路径匹配缓存
    cache: Mutex<LruCache<PathBuf, Option<ProjectContext>>>,
}

impl ContextRouter {
    /// 缓存大小
    const CACHE_SIZE: usize = 100;

    /// 创建新的路由器
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(Self::CACHE_SIZE).unwrap(),
            )),
        }
    }

    /// 根据工作目录查找关联项目
    ///
    /// 使用最长前缀匹配 (LPM) 算法：
    /// 1. 从 project_paths 表查询所有路径
    /// 2. 找到输入路径的最长匹配前缀
    /// 3. 返回对应的项目
    ///
    /// # Arguments
    /// * `work_dir` - 工作目录路径
    ///
    /// # Returns
    /// 匹配的项目上下文，如果没有匹配则返回 None
    pub async fn find_project_by_path(&self, work_dir: &Path) -> Option<ProjectContext> {
        // 规范化路径
        let normalized = self.normalize_path(work_dir);

        // 检查缓存
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&normalized) {
                return cached.clone();
            }
        }

        // 查询数据库执行 LPM
        let result = self.perform_lpm(&normalized);

        // 更新缓存
        {
            let mut cache = self.cache.lock().await;
            cache.put(normalized, result.clone());
        }

        result
    }

    /// 执行最长前缀匹配
    fn perform_lpm(&self, work_dir: &Path) -> Option<ProjectContext> {
        // SQL 策略：
        // 1. 使用 LIKE 查询找到所有是 work_dir 前缀的路径
        // 2. 按路径长度降序排列
        // 3. 返回第一条（最长匹配）
        //
        // 例如 work_dir = /home/user/projects/mantra/apps/client
        // 匹配:
        //   - /home/user/projects/mantra (长度 27)
        //   - /home/user/projects (长度 22)
        // 选择长度最长的: /home/user/projects/mantra

        let work_dir_str = work_dir.to_string_lossy().to_string();

        // 查询 project_paths 表
        // 使用 SQLite 的字符串比较来检查前缀匹配
        // work_dir 以 pp.path 开头，或者完全相等
        let result = self.db.connection().query_row(
            r#"
            SELECT pp.project_id, p.name, pp.path, LENGTH(pp.path) as path_len
            FROM project_paths pp
            JOIN projects p ON pp.project_id = p.id
            WHERE ?1 = pp.path
               OR (?1 LIKE (pp.path || '/%'))
               OR (?1 LIKE (pp.path || '\%'))
            ORDER BY path_len DESC
            LIMIT 1
            "#,
            [&work_dir_str],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, usize>(3)?,
                ))
            },
        );

        match result {
            Ok((project_id, project_name, matched_path, match_length)) => Some(ProjectContext {
                project_id,
                project_name,
                matched_path: PathBuf::from(matched_path),
                match_length,
            }),
            Err(_) => None,
        }
    }

    /// 规范化路径
    ///
    /// - 移除尾部斜杠
    /// - Windows: 统一为小写盘符
    /// - Unix: 保持原样（符号链接解析由调用方处理）
    fn normalize_path(&self, path: &Path) -> PathBuf {
        let mut normalized = path.to_path_buf();

        // 移除尾部斜杠
        if let Some(s) = normalized.to_str() {
            if s.ends_with('/') || s.ends_with('\\') {
                normalized = PathBuf::from(s.trim_end_matches(['/', '\\']));
            }
        }

        // Windows 路径小写化
        #[cfg(target_os = "windows")]
        {
            if let Some(s) = normalized.to_str() {
                normalized = PathBuf::from(s.to_lowercase());
            }
        }

        normalized
    }

    /// 解析 MCP initialize 请求中的工作目录
    ///
    /// 支持多种格式：
    /// - `rootUri`: string (file URI)
    /// - `workspaceFolders`: [{ uri: string, name: string }]
    /// - `rootPath`: string (deprecated but still used)
    ///
    /// # Arguments
    /// * `params` - initialize 请求的 params 对象
    ///
    /// # Returns
    /// 解析出的工作目录路径
    pub fn parse_work_dir_from_params(&self, params: &serde_json::Value) -> Option<PathBuf> {
        // 优先使用 workspaceFolders
        if let Some(folders) = params.get("workspaceFolders").and_then(|v| v.as_array()) {
            if let Some(first) = folders.first() {
                if let Some(uri) = first.get("uri").and_then(|v| v.as_str()) {
                    return self.uri_to_path(uri);
                }
            }
        }

        // 回退到 rootUri
        if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
            return self.uri_to_path(root_uri);
        }

        // 再回退到 rootPath (deprecated but still used)
        if let Some(root_path) = params.get("rootPath").and_then(|v| v.as_str()) {
            return Some(PathBuf::from(root_path));
        }

        None
    }

    /// 将 file:// URI 转换为本地路径
    fn uri_to_path(&self, uri: &str) -> Option<PathBuf> {
        if uri.starts_with("file://") {
            let path = &uri[7..];

            // Windows: file:///C:/path -> C:/path
            #[cfg(target_os = "windows")]
            {
                if path.starts_with('/') && path.len() > 2 && path.chars().nth(2) == Some(':') {
                    return Some(PathBuf::from(&path[1..]));
                }
            }

            // Unix: file:///path -> /path
            // URL 解码
            if let Ok(decoded) = urlencoding::decode(path) {
                return Some(PathBuf::from(decoded.as_ref()));
            }
            return Some(PathBuf::from(path));
        }
        None
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    /// 获取数据库连接引用
    pub fn database(&self) -> &Database {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn create_test_db() -> Arc<Database> {
        Arc::new(Database::new_in_memory().unwrap())
    }

    fn create_test_project(db: &Database, id: &str, name: &str, cwd: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        db.connection()
            .execute(
                "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![id, name, cwd, now],
            )
            .unwrap();
    }

    fn add_project_path(db: &Database, project_id: &str, path: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        db.connection()
            .execute(
                "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
                params![id, project_id, path, now],
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_find_project_exact_match() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);
        let result = router
            .find_project_by_path(Path::new("/home/user/projects/mantra"))
            .await;

        assert!(result.is_some());
        let ctx = result.unwrap();
        assert_eq!(ctx.project_id, "proj1");
        assert_eq!(ctx.project_name, "Project 1");
    }

    #[tokio::test]
    async fn test_find_project_prefix_match() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);
        // 查询子目录
        let result = router
            .find_project_by_path(Path::new("/home/user/projects/mantra/apps/client"))
            .await;

        assert!(result.is_some());
        let ctx = result.unwrap();
        assert_eq!(ctx.project_id, "proj1");
        assert_eq!(ctx.matched_path, PathBuf::from("/home/user/projects/mantra"));
    }

    #[tokio::test]
    async fn test_find_project_longest_prefix_match() {
        let db = create_test_db();

        // 创建两个项目，一个是另一个的父目录
        create_test_project(&db, "proj1", "Projects Root", "/home/user/projects");
        add_project_path(&db, "proj1", "/home/user/projects");

        create_test_project(&db, "proj2", "Mantra", "/home/user/projects/mantra");
        add_project_path(&db, "proj2", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);

        // 查询 mantra 子目录，应该匹配到 proj2（更长的前缀）
        let result = router
            .find_project_by_path(Path::new("/home/user/projects/mantra/apps/client"))
            .await;

        assert!(result.is_some());
        let ctx = result.unwrap();
        assert_eq!(ctx.project_id, "proj2");
        assert_eq!(ctx.project_name, "Mantra");
        // /home/user/projects/mantra = 26 characters
        assert_eq!(ctx.match_length, 26);
    }

    #[tokio::test]
    async fn test_find_project_no_match() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);
        // 查询不相关的路径
        let result = router
            .find_project_by_path(Path::new("/home/other/path"))
            .await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_project_partial_name_no_match() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);
        // 查询部分名称匹配但不是前缀的路径
        let result = router
            .find_project_by_path(Path::new("/home/user/projects/mantra-other"))
            .await;

        // 不应该匹配，因为 mantra-other 不是 mantra 的子目录
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);

        // 第一次查询
        let result1 = router
            .find_project_by_path(Path::new("/home/user/projects/mantra"))
            .await;
        assert!(result1.is_some());

        // 第二次查询应该命中缓存
        let result2 = router
            .find_project_by_path(Path::new("/home/user/projects/mantra"))
            .await;
        assert!(result2.is_some());
        assert_eq!(result1.unwrap().project_id, result2.unwrap().project_id);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let db = create_test_db();
        create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
        add_project_path(&db, "proj1", "/home/user/projects/mantra");

        let router = ContextRouter::new(db);

        // 查询以填充缓存
        let _ = router
            .find_project_by_path(Path::new("/home/user/projects/mantra"))
            .await;

        // 清除缓存
        router.clear_cache().await;

        // 缓存应该为空
        let cache = router.cache.lock().await;
        assert!(cache.is_empty());
    }

    #[test]
    fn test_normalize_path_trailing_slash() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let normalized = router.normalize_path(Path::new("/home/user/projects/"));
        assert_eq!(normalized, PathBuf::from("/home/user/projects"));
    }

    #[test]
    fn test_normalize_path_no_trailing_slash() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let normalized = router.normalize_path(Path::new("/home/user/projects"));
        assert_eq!(normalized, PathBuf::from("/home/user/projects"));
    }

    #[test]
    fn test_parse_work_dir_from_root_uri() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let params = serde_json::json!({
            "rootUri": "file:///home/user/projects/mantra"
        });

        let result = router.parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_from_workspace_folders() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let params = serde_json::json!({
            "workspaceFolders": [
                {
                    "uri": "file:///home/user/projects/mantra",
                    "name": "mantra"
                }
            ]
        });

        let result = router.parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_workspace_folders_priority() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        // workspaceFolders 应该优先于 rootUri
        let params = serde_json::json!({
            "rootUri": "file:///other/path",
            "workspaceFolders": [
                {
                    "uri": "file:///home/user/projects/mantra",
                    "name": "mantra"
                }
            ]
        });

        let result = router.parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_from_root_path() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let params = serde_json::json!({
            "rootPath": "/home/user/projects/mantra"
        });

        let result = router.parse_work_dir_from_params(&params);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
    }

    #[test]
    fn test_parse_work_dir_no_params() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let params = serde_json::json!({});

        let result = router.parse_work_dir_from_params(&params);
        assert!(result.is_none());
    }

    #[test]
    fn test_uri_to_path_unix() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let result = router.uri_to_path("file:///home/user/projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects"));
    }

    #[test]
    fn test_uri_to_path_with_spaces() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let result = router.uri_to_path("file:///home/user/my%20projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/my projects"));
    }

    #[test]
    fn test_uri_to_path_invalid() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let result = router.uri_to_path("http://example.com");
        assert!(result.is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_uri_to_path_windows() {
        let db = create_test_db();
        let router = ContextRouter::new(db);

        let result = router.uri_to_path("file:///C:/Users/user/projects");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("C:/Users/user/projects"));
    }
}
