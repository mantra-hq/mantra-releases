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
                    return super::uri_to_local_path(uri);
                }
            }
        }

        // 回退到 rootUri
        if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
            return super::uri_to_local_path(root_uri);
        }

        // 再回退到 rootPath (deprecated but still used)
        if let Some(root_path) = params.get("rootPath").and_then(|v| v.as_str()) {
            return Some(PathBuf::from(root_path));
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

    /// 同步查询项目（无缓存）
    ///
    /// Story 11.27: Task 3 - 为 LPM 查询服务提供同步版本
    ///
    /// 注意：此方法不使用缓存，每次都直接查询数据库
    ///
    /// # Arguments
    /// * `work_dir` - 工作目录路径字符串
    ///
    /// # Returns
    /// 匹配的项目上下文，如果没有匹配则返回 None
    pub fn find_project_by_path_sync(&self, work_dir: &str) -> Option<ProjectContext> {
        let path = Path::new(work_dir);
        let normalized = self.normalize_path(path);
        self.perform_lpm(&normalized)
    }
}

#[cfg(test)]
mod tests;
