//! MCP 配置解析与导入服务
//!
//! Story 11.3: 配置导入与接管
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 提供 MCP 配置文件的解析、扫描、备份、导入功能。
//!
//! ## 架构变更 (Story 11.8)
//!
//! - 使用 `adapter_id: String` 替代旧的 `ConfigSource` 枚举
//! - 通过 `ToolAdapterRegistry` 统一管理适配器
//! - 支持 Claude, Cursor, Codex, Gemini CLI 四大工具

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::mcp::{CreateMcpServiceRequest, McpService, McpServiceSource};
use crate::services::EnvManager;
use crate::services::mcp_adapters::{
    ConfigScope, DetectedConfig as AdapterDetectedConfig,
    DetectedService as AdapterDetectedService, GatewayInjectionConfig, ToolAdapterRegistry,
};
use crate::storage::{Database, StorageError};

// ===== 数据类型定义 =====

// Re-export 新的类型定义，保持向后兼容
// ConfigScope 已从 mcp_adapters 导入

/// 配置文件来源类型 (已弃用，保留向后兼容)
///
/// Story 11.8: 使用 `adapter_id: String` 替代此枚举
#[deprecated(since = "0.7.0", note = "Use adapter_id: String instead")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    /// Claude Code: .mcp.json
    ClaudeCode,
    /// Cursor: .cursor/mcp.json 或 ~/.cursor/mcp.json
    Cursor,
    /// Claude Desktop: claude_desktop_config.json (已弃用，合并到 ClaudeCode)
    ClaudeDesktop,
    /// Codex: .codex/config.toml
    Codex,
    /// Gemini CLI: .gemini/settings.json
    Gemini,
}

#[allow(deprecated)]
impl ConfigSource {
    /// 获取配置文件的典型路径描述
    pub fn description(&self) -> &'static str {
        match self {
            ConfigSource::ClaudeCode => ".mcp.json",
            ConfigSource::Cursor => ".cursor/mcp.json",
            ConfigSource::ClaudeDesktop => "claude_desktop_config.json",
            ConfigSource::Codex => ".codex/config.toml",
            ConfigSource::Gemini => ".gemini/settings.json",
        }
    }

    /// 从 adapter_id 转换
    pub fn from_adapter_id(id: &str) -> Option<Self> {
        match id {
            "claude" => Some(ConfigSource::ClaudeCode),
            "cursor" => Some(ConfigSource::Cursor),
            "codex" => Some(ConfigSource::Codex),
            "gemini" => Some(ConfigSource::Gemini),
            _ => None,
        }
    }

    /// 转换为 adapter_id
    pub fn to_adapter_id(&self) -> &'static str {
        match self {
            ConfigSource::ClaudeCode | ConfigSource::ClaudeDesktop => "claude",
            ConfigSource::Cursor => "cursor",
            ConfigSource::Codex => "codex",
            ConfigSource::Gemini => "gemini",
        }
    }
}

/// 检测到的 MCP 服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedService {
    /// 服务名称
    pub name: String,
    /// 传输类型（stdio 或 http）
    #[serde(default)]
    pub transport_type: crate::models::mcp::McpTransportType,
    /// 启动命令（stdio 模式）
    #[serde(default)]
    pub command: String,
    /// 命令参数（stdio 模式）
    pub args: Option<Vec<String>>,
    /// 环境变量
    pub env: Option<HashMap<String, String>>,
    /// HTTP 端点 URL（http 模式）
    pub url: Option<String>,
    /// HTTP 请求头（http 模式）
    pub headers: Option<HashMap<String, String>>,
    /// 来源配置文件路径
    pub source_file: PathBuf,
    /// 适配器 ID (Story 11.8: 替代旧的 source_type)
    pub adapter_id: String,
    /// 配置作用域 (Story 11.8: 新增)
    #[serde(default)]
    pub scope: Option<ConfigScope>,
}

impl From<AdapterDetectedService> for DetectedService {
    fn from(s: AdapterDetectedService) -> Self {
        Self {
            name: s.name,
            transport_type: s.transport_type,
            command: s.command,
            args: s.args,
            env: s.env,
            url: s.url,
            headers: s.headers,
            source_file: s.source_file,
            adapter_id: s.adapter_id,
            scope: Some(s.scope),
        }
    }
}

/// 检测到的配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedConfig {
    /// 适配器 ID (Story 11.8: 替代旧的 source)
    pub adapter_id: String,
    /// 配置文件路径
    pub path: PathBuf,
    /// 配置作用域 (Story 11.8: 新增)
    #[serde(default)]
    pub scope: Option<ConfigScope>,
    /// 检测到的服务列表
    pub services: Vec<DetectedService>,
    /// 解析错误（如有）
    pub parse_errors: Vec<String>,
}

impl From<AdapterDetectedConfig> for DetectedConfig {
    fn from(c: AdapterDetectedConfig) -> Self {
        Self {
            adapter_id: c.adapter_id,
            path: c.path,
            scope: Some(c.scope),
            services: c.services.into_iter().map(Into::into).collect(),
            parse_errors: c.parse_errors,
        }
    }
}

/// 服务冲突信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConflict {
    /// 服务名称
    pub name: String,
    /// 数据库中已存在的服务（如果有）
    pub existing: Option<McpService>,
    /// 多个来源检测到的同名服务
    pub candidates: Vec<DetectedService>,
}

/// 导入预览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    /// 检测到的配置文件
    pub configs: Vec<DetectedConfig>,
    /// 检测到的冲突
    pub conflicts: Vec<ServiceConflict>,
    /// 可直接导入的新服务
    pub new_services: Vec<DetectedService>,
    /// 需要用户提供值的环境变量
    pub env_vars_needed: Vec<String>,
    /// 总服务数量
    pub total_services: usize,
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// 保留现有服务，跳过导入
    Keep,
    /// 使用指定候选项替换现有服务
    Replace(usize),
    /// 重命名为新名称后导入
    Rename(String),
    /// 跳过此服务不导入
    Skip,
}

/// 导入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    /// 要导入的服务名称列表
    pub services_to_import: Vec<String>,
    /// 冲突解决策略映射
    pub conflict_resolutions: HashMap<String, ConflictResolution>,
    /// 环境变量值
    pub env_var_values: HashMap<String, String>,
    /// 是否启用影子模式（修改原配置文件）
    pub enable_shadow_mode: bool,
    /// 网关 URL（用于生成影子配置）
    pub gateway_url: Option<String>,
    /// 网关认证 Token (Story 11.8: 用于 HTTP Transport Authorization Header)
    #[serde(default)]
    pub gateway_token: Option<String>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// 成功导入的服务数量
    pub imported_count: usize,
    /// 跳过的服务数量
    pub skipped_count: usize,
    /// 创建的备份文件列表
    pub backup_files: Vec<PathBuf>,
    /// 修改为影子模式的配置文件
    pub shadow_configs: Vec<PathBuf>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 导入的服务 ID 列表
    pub imported_service_ids: Vec<String>,
}

/// 解析错误类型
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid config format: {0}")]
    InvalidFormat(String),
}

// ===== 配置文件 JSON 结构定义 =====

/// MCP 服务器配置（配置文件中的格式）
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum McpServerConfig {
    /// stdio 传输模式（命令行启动）
    Stdio {
        command: String,
        #[serde(default)]
        args: Option<Vec<String>>,
        #[serde(default)]
        env: Option<HashMap<String, String>>,
    },
    /// SSE 传输模式（URL 连接）
    Sse {
        url: String,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
    },
}

/// 通用 MCP 配置文件结构
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfigFile {
    #[serde(alias = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

// ===== 配置解析器实现 =====

/// 移除 JSON 注释（支持 JSONC）
///
/// 支持移除 // 单行注释和 /* */ 块注释
///
/// Note: 此函数为向后兼容保留，新代码应使用 `mcp_adapters::common::strip_json_comments`
pub fn strip_json_comments(input: &str) -> String {
    crate::services::mcp_adapters::common::strip_json_comments(input)
}

/// MCP 配置解析器 trait (已弃用)
///
/// Story 11.8: 使用 `McpToolAdapter` trait 替代
#[deprecated(since = "0.7.0", note = "Use McpToolAdapter trait from mcp_adapters module")]
#[allow(deprecated)]
pub trait McpConfigParser {
    /// 解析配置文件
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError>;

    /// 获取配置来源类型
    fn source_type(&self) -> ConfigSource;

    /// 生成影子模式配置
    fn generate_shadow_config(&self, gateway_url: &str) -> String;
}

/// Claude Code 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `ClaudeAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use ClaudeAdapter from mcp_adapters module")]
pub struct ClaudeCodeConfigParser;

#[allow(deprecated)]
impl McpConfigParser for ClaudeCodeConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "claude".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::ClaudeCode
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

/// Cursor 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `CursorAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use CursorAdapter from mcp_adapters module")]
pub struct CursorConfigParser;

#[allow(deprecated)]
impl McpConfigParser for CursorConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "cursor".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::Cursor
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

/// Claude Desktop 配置解析器 (已弃用)
///
/// Story 11.8: 使用 `ClaudeAdapter` 替代
#[deprecated(since = "0.7.0", note = "Use ClaudeAdapter from mcp_adapters module")]
pub struct ClaudeDesktopConfigParser;

#[allow(deprecated)]
impl McpConfigParser for ClaudeDesktopConfigParser {
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError> {
        let content = fs::read_to_string(path)?;
        let content = strip_json_comments(&content);
        let config: McpConfigFile = serde_json::from_str(&content)?;

        let mut services = Vec::new();
        if let Some(mcp_servers) = config.mcp_servers {
            for (name, server) in mcp_servers {
                if let McpServerConfig::Stdio { command, args, env } = server {
                    services.push(DetectedService {
                        name,
                        transport_type: Default::default(),
                        command,
                        args,
                        env,
                        url: None,
                        headers: None,
                        source_file: path.to_path_buf(),
                        adapter_id: "claude".to_string(),
                        scope: None,
                    });
                }
            }
        }

        Ok(services)
    }

    fn source_type(&self) -> ConfigSource {
        ConfigSource::ClaudeDesktop
    }

    fn generate_shadow_config(&self, gateway_url: &str) -> String {
        serde_json::json!({
            "mcpServers": {
                "mantra-gateway": {
                    "url": gateway_url
                }
            }
        })
        .to_string()
    }
}

// ===== 配置文件扫描器 =====

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 检测到的配置文件
    pub configs: Vec<DetectedConfig>,
    /// 扫描的路径
    pub scanned_paths: Vec<PathBuf>,
}

/// 获取平台相关的 Claude Desktop 配置路径
fn get_claude_desktop_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|p| p.join("claude-desktop").join("claude_desktop_config.json"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|p| p.join("Claude").join("claude_desktop_config.json"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|p| p.join("Claude").join("claude_desktop_config.json"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// 扫描 MCP 配置文件 (使用新的适配器架构)
///
/// Story 11.8: 使用 `ToolAdapterRegistry` 统一扫描所有工具的配置文件
///
/// # Arguments
/// * `project_path` - 项目路径（可选，用于扫描项目级配置）
///
/// # Returns
/// 扫描结果，包含所有检测到的配置文件和服务
pub fn scan_mcp_configs(project_path: Option<&Path>) -> ScanResult {
    let registry = ToolAdapterRegistry::new();
    let mut configs = Vec::new();
    let mut scanned_paths = Vec::new();

    let home_dir = dirs::home_dir();

    // 遍历所有适配器
    for adapter in registry.all() {
        for (scope, pattern) in adapter.scan_patterns() {
            // 解析路径模式
            let path = match scope {
                ConfigScope::Project => {
                    if let Some(project) = project_path {
                        project.join(&pattern)
                    } else {
                        continue;
                    }
                }
                ConfigScope::User => {
                    if let Some(ref home) = home_dir {
                        if pattern.starts_with("~/") {
                            home.join(&pattern[2..])
                        } else {
                            home.join(&pattern)
                        }
                    } else {
                        continue;
                    }
                }
            };

            scanned_paths.push(path.clone());

            if path.exists() {
                // 读取并解析配置文件
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match adapter.parse(&path, &content, scope) {
                            Ok(services) => {
                                configs.push(DetectedConfig {
                                    adapter_id: adapter.id().to_string(),
                                    path: path.clone(),
                                    scope: Some(scope),
                                    services: services.into_iter().map(Into::into).collect(),
                                    parse_errors: Vec::new(),
                                });
                            }
                            Err(e) => {
                                configs.push(DetectedConfig {
                                    adapter_id: adapter.id().to_string(),
                                    path: path.clone(),
                                    scope: Some(scope),
                                    services: Vec::new(),
                                    parse_errors: vec![e.to_string()],
                                });
                            }
                        }
                    }
                    Err(e) => {
                        configs.push(DetectedConfig {
                            adapter_id: adapter.id().to_string(),
                            path: path.clone(),
                            scope: Some(scope),
                            services: Vec::new(),
                            parse_errors: vec![format!("Failed to read file: {}", e)],
                        });
                    }
                }
            }
        }
    }

    // 向后兼容：扫描 Claude Desktop 配置
    if let Some(claude_desktop_path) = get_claude_desktop_config_path() {
        scanned_paths.push(claude_desktop_path.clone());
        if claude_desktop_path.exists() {
            #[allow(deprecated)]
            let config = parse_config_file_legacy(&claude_desktop_path, ConfigSource::ClaudeDesktop);
            configs.push(config);
        }
    }

    ScanResult {
        configs,
        scanned_paths,
    }
}

/// 解析单个配置文件 (旧版，向后兼容)
#[allow(deprecated)]
fn parse_config_file_legacy(path: &Path, source: ConfigSource) -> DetectedConfig {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
        ConfigSource::Codex | ConfigSource::Gemini => {
            // 新工具使用新的适配器架构
            return DetectedConfig {
                adapter_id: source.to_adapter_id().to_string(),
                path: path.to_path_buf(),
                scope: None,
                services: Vec::new(),
                parse_errors: vec!["Use new adapter architecture".to_string()],
            };
        }
    };

    match parser.parse(path) {
        Ok(services) => DetectedConfig {
            adapter_id: source.to_adapter_id().to_string(),
            path: path.to_path_buf(),
            scope: None,
            services,
            parse_errors: Vec::new(),
        },
        Err(e) => DetectedConfig {
            adapter_id: source.to_adapter_id().to_string(),
            path: path.to_path_buf(),
            scope: None,
            services: Vec::new(),
            parse_errors: vec![e.to_string()],
        },
    }
}

/// 使用新适配器架构生成影子配置
///
/// Story 11.8: 使用 HTTP Transport + Authorization Header
pub fn generate_shadow_config_v2(
    adapter_id: &str,
    gateway_url: &str,
    token: &str,
) -> Result<String, String> {
    let registry = ToolAdapterRegistry::new();
    let adapter = registry
        .get(adapter_id)
        .ok_or_else(|| format!("Unknown adapter: {}", adapter_id))?;

    let config = GatewayInjectionConfig::new(gateway_url, token);
    adapter
        .inject_gateway("", &config)
        .map_err(|e| e.to_string())
}

// ===== 导入预览生成器 =====

/// 提取环境变量引用
///
/// 从环境变量映射中识别 `$VAR_NAME` 或 `${VAR_NAME}` 格式的引用
pub fn extract_env_var_references(env: &Option<HashMap<String, String>>) -> Vec<String> {
    let mut vars = Vec::new();
    if let Some(env_map) = env {
        for value in env_map.values() {
            // 匹配 $VAR_NAME 或 ${VAR_NAME}
            if value.starts_with('$') {
                let var_name = value
                    .trim_start_matches('$')
                    .trim_start_matches('{')
                    .trim_end_matches('}');
                if !var_name.is_empty() && !vars.contains(&var_name.to_string()) {
                    vars.push(var_name.to_string());
                }
            }
        }
    }
    vars
}

/// 生成导入预览
///
/// # Arguments
/// * `configs` - 检测到的配置文件列表
/// * `db` - 数据库连接
///
/// # Returns
/// 导入预览，包含冲突检测和环境变量需求
pub fn generate_import_preview(
    configs: &[DetectedConfig],
    db: &Database,
) -> Result<ImportPreview, StorageError> {
    let mut service_map: HashMap<String, Vec<DetectedService>> = HashMap::new();
    let mut env_vars_needed: Vec<String> = Vec::new();

    // 收集所有服务并按名称分组
    for config in configs {
        for service in &config.services {
            service_map
                .entry(service.name.clone())
                .or_default()
                .push(service.clone());

            // 提取环境变量引用
            for var in extract_env_var_references(&service.env) {
                if !env_vars_needed.contains(&var) {
                    env_vars_needed.push(var);
                }
            }
        }
    }

    let mut conflicts = Vec::new();
    let mut new_services = Vec::new();
    let total_services = service_map.len();

    // 检查冲突
    for (name, candidates) in service_map {
        let existing = db.get_mcp_service_by_name(&name)?;

        if candidates.len() > 1 || existing.is_some() {
            // 存在冲突：多个候选或已存在同名服务
            conflicts.push(ServiceConflict {
                name,
                existing,
                candidates,
            });
        } else {
            // 无冲突，可直接导入
            new_services.extend(candidates);
        }
    }

    // 检查环境变量是否已存在
    let mut missing_env_vars = Vec::new();
    for var in &env_vars_needed {
        if !db.env_variable_exists(var)? {
            missing_env_vars.push(var.clone());
        }
    }

    Ok(ImportPreview {
        configs: configs.to_vec(),
        conflicts,
        new_services,
        env_vars_needed: missing_env_vars,
        total_services,
    })
}

// ===== 备份管理器 =====

/// 备份条目
struct BackupEntry {
    original_path: PathBuf,
    backup_path: PathBuf,
}

/// 备份管理器
///
/// 使用 RAII 模式管理备份文件，确保在出错时自动回滚
pub struct BackupManager {
    backups: Vec<BackupEntry>,
    committed: bool,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new() -> Self {
        Self {
            backups: Vec::new(),
            committed: false,
        }
    }

    /// 备份文件
    ///
    /// # Arguments
    /// * `path` - 要备份的文件路径
    ///
    /// # Returns
    /// 备份文件的路径
    pub fn backup(&mut self, path: &Path) -> io::Result<PathBuf> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {:?}", path),
            ));
        }

        // 构建备份文件名
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let backup_extension = if extension.is_empty() {
            "mantra-backup".to_string()
        } else {
            format!("{}.mantra-backup", extension)
        };

        let mut backup_path = path.with_extension(&backup_extension);

        // 如果备份已存在，添加时间戳
        if backup_path.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let timestamped_extension = format!("{}.{}", backup_extension, timestamp);
            backup_path = path.with_extension(timestamped_extension);
        }

        // 复制文件
        fs::copy(path, &backup_path)?;

        self.backups.push(BackupEntry {
            original_path: path.to_path_buf(),
            backup_path: backup_path.clone(),
        });

        Ok(backup_path)
    }

    /// 标记备份成功，不再需要回滚
    pub fn commit(&mut self) {
        self.committed = true;
    }

    /// 手动回滚所有备份
    pub fn rollback(&self) -> io::Result<()> {
        for entry in &self.backups {
            if entry.backup_path.exists() {
                fs::copy(&entry.backup_path, &entry.original_path)?;
            }
        }
        Ok(())
    }

    /// 获取所有备份文件路径
    pub fn backup_paths(&self) -> Vec<PathBuf> {
        self.backups.iter().map(|e| e.backup_path.clone()).collect()
    }

    /// 清理备份文件
    pub fn cleanup(&self) -> io::Result<()> {
        for entry in &self.backups {
            if entry.backup_path.exists() {
                fs::remove_file(&entry.backup_path)?;
            }
        }
        Ok(())
    }
}

impl Drop for BackupManager {
    fn drop(&mut self) {
        if !self.committed {
            // 自动回滚
            let _ = self.rollback();
        }
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 影子模式配置生成 =====

/// 生成影子模式配置 (旧版，向后兼容)
///
/// Story 11.8: 推荐使用 `generate_shadow_config_v2` 替代
///
/// # Arguments
/// * `source` - 配置来源类型
/// * `gateway_url` - Mantra Gateway URL
///
/// # Returns
/// 影子模式配置 JSON 字符串
#[allow(deprecated)]
pub fn generate_shadow_config(source: &ConfigSource, gateway_url: &str) -> String {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
        ConfigSource::Codex | ConfigSource::Gemini => {
            // 新工具使用 generate_shadow_config_v2
            return serde_json::json!({
                "mcpServers": {
                    "mantra-gateway": {
                        "url": gateway_url
                    }
                }
            })
            .to_string();
        }
    };
    parser.generate_shadow_config(gateway_url)
}

// ===== 导入执行器 =====

/// 导入执行器
pub struct ImportExecutor<'a> {
    db: &'a Database,
    env_manager: &'a EnvManager,
    backup_manager: BackupManager,
}

impl<'a> ImportExecutor<'a> {
    /// 创建导入执行器
    pub fn new(db: &'a Database, env_manager: &'a EnvManager) -> Self {
        Self {
            db,
            env_manager,
            backup_manager: BackupManager::new(),
        }
    }

    /// 执行导入
    ///
    /// # Arguments
    /// * `preview` - 导入预览
    /// * `request` - 导入请求
    ///
    /// # Returns
    /// 导入结果
    pub fn execute(
        mut self,
        preview: &ImportPreview,
        request: &ImportRequest,
    ) -> Result<ImportResult, StorageError> {
        let mut imported_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        let mut imported_service_ids = Vec::new();
        let mut shadow_configs = Vec::new();

        // 1. 存储环境变量
        for (name, value) in &request.env_var_values {
            if let Err(e) = self.db.set_env_variable(
                self.env_manager,
                name,
                value,
                Some("Imported from MCP config"),
            ) {
                errors.push(format!("Failed to set env var {}: {}", name, e));
            }
        }

        // 2. 导入新服务
        for service in &preview.new_services {
            if request.services_to_import.contains(&service.name) {
                match self.import_service(service) {
                    Ok(id) => {
                        imported_count += 1;
                        imported_service_ids.push(id);
                    }
                    Err(e) => {
                        errors.push(format!("Failed to import {}: {}", service.name, e));
                    }
                }
            } else {
                skipped_count += 1;
            }
        }

        // 3. 处理冲突
        for conflict in &preview.conflicts {
            if let Some(resolution) = request.conflict_resolutions.get(&conflict.name) {
                match self.resolve_conflict(conflict, resolution) {
                    Ok(Some(id)) => {
                        imported_count += 1;
                        imported_service_ids.push(id);
                    }
                    Ok(None) => {
                        skipped_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!(
                            "Failed to resolve conflict for {}: {}",
                            conflict.name, e
                        ));
                    }
                }
            } else {
                skipped_count += 1;
            }
        }

        // 4. 启用影子模式（如果请求）
        if request.enable_shadow_mode {
            if let Some(gateway_url) = &request.gateway_url {
                let gateway_token = request.gateway_token.as_deref();
                for config in &preview.configs {
                    if !config.services.is_empty() {
                        match self.apply_shadow_mode_v2(&config.path, &config.adapter_id, gateway_url, gateway_token) {
                            Ok(_) => {
                                shadow_configs.push(config.path.clone());
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "Failed to apply shadow mode to {:?}: {}",
                                    config.path, e
                                ));
                            }
                        }
                    }
                }
            } else {
                errors.push("Gateway URL required for shadow mode".to_string());
            }
        }

        // 5. 提交备份（成功则不回滚）
        if errors.is_empty() {
            self.backup_manager.commit();
        }

        Ok(ImportResult {
            imported_count,
            skipped_count,
            backup_files: self.backup_manager.backup_paths(),
            shadow_configs,
            errors,
            imported_service_ids,
        })
    }

    /// 导入单个服务
    fn import_service(&self, service: &DetectedService) -> Result<String, StorageError> {
        let request = CreateMcpServiceRequest {
            name: service.name.clone(),
            transport_type: service.transport_type.clone(),
            command: service.command.clone(),
            args: service.args.clone(),
            env: service.env.as_ref().map(|e| serde_json::to_value(e).unwrap()),
            url: service.url.clone(),
            headers: service.headers.clone(),
            source: McpServiceSource::Imported,
            source_file: Some(service.source_file.to_string_lossy().to_string()),
        };
        let created = self.db.create_mcp_service(&request)?;
        Ok(created.id)
    }

    /// 解决冲突
    fn resolve_conflict(
        &self,
        conflict: &ServiceConflict,
        resolution: &ConflictResolution,
    ) -> Result<Option<String>, StorageError> {
        match resolution {
            ConflictResolution::Keep => {
                // 保留现有，不导入
                Ok(None)
            }
            ConflictResolution::Replace(idx) => {
                // 替换现有服务
                if let Some(candidate) = conflict.candidates.get(*idx) {
                    if let Some(existing) = &conflict.existing {
                        // 删除现有服务
                        self.db.delete_mcp_service(&existing.id)?;
                    }
                    // 导入新服务
                    let id = self.import_service(candidate)?;
                    Ok(Some(id))
                } else {
                    Err(StorageError::InvalidInput(format!(
                        "Invalid candidate index: {}",
                        idx
                    )))
                }
            }
            ConflictResolution::Rename(new_name) => {
                // 使用新名称导入第一个候选
                if let Some(candidate) = conflict.candidates.first() {
                    let mut renamed = candidate.clone();
                    renamed.name = new_name.clone();
                    let id = self.import_service(&renamed)?;
                    Ok(Some(id))
                } else {
                    Err(StorageError::InvalidInput(
                        "No candidates to rename".to_string(),
                    ))
                }
            }
            ConflictResolution::Skip => {
                // 跳过
                Ok(None)
            }
        }
    }

    /// 应用影子模式 (旧版，向后兼容)
    #[allow(deprecated, dead_code)]
    fn apply_shadow_mode(
        &mut self,
        path: &Path,
        source: &ConfigSource,
        gateway_url: &str,
    ) -> io::Result<()> {
        // 备份原文件
        self.backup_manager.backup(path)?;

        // 生成影子配置
        let shadow_content = generate_shadow_config(source, gateway_url);

        // 写入影子配置
        fs::write(path, shadow_content)?;

        Ok(())
    }

    /// 应用影子模式 (新版，使用适配器架构)
    ///
    /// Story 11.8: 使用 HTTP Transport + Authorization Header
    fn apply_shadow_mode_v2(
        &mut self,
        path: &Path,
        adapter_id: &str,
        gateway_url: &str,
        gateway_token: Option<&str>,
    ) -> io::Result<()> {
        // 备份原文件
        self.backup_manager.backup(path)?;

        // 读取原始内容
        let original_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 使用新的适配器架构生成影子配置
        let registry = ToolAdapterRegistry::new();
        let token = gateway_token.unwrap_or("");
        let shadow_content = if let Some(adapter) = registry.get(adapter_id) {
            let config = GatewayInjectionConfig::new(gateway_url, token);
            adapter
                .inject_gateway(&original_content, &config)
                .unwrap_or_else(|_| original_content.clone())
        } else {
            // 回退到旧的生成方式
            serde_json::json!({
                "mcpServers": {
                    "mantra-gateway": {
                        "url": gateway_url
                    }
                }
            })
            .to_string()
        };

        // 写入影子配置
        fs::write(path, shadow_content)?;

        Ok(())
    }
}

// ===== 回滚功能 =====

/// 从备份文件回滚
///
/// # Arguments
/// * `backup_files` - 备份文件路径列表
///
/// # Returns
/// 成功恢复的文件数量
pub fn rollback_from_backups(backup_files: &[PathBuf]) -> io::Result<usize> {
    let mut restored = 0;

    for backup_path in backup_files {
        if backup_path.exists() {
            // 从备份路径推断原始路径
            let original_path = backup_path
                .to_string_lossy()
                .replace(".mantra-backup", "")
                // 移除时间戳后缀（如果有）
                .split(".mantra-backup.")
                .next()
                .map(PathBuf::from);

            if let Some(original) = original_path {
                // 恢复原始文件
                fs::copy(backup_path, &original)?;
                restored += 1;
            }
        }
    }

    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ===== Task 1.1-1.6: 配置解析器测试 =====

    #[test]
    fn test_strip_json_comments_single_line() {
        let input = r#"{
            "key": "value" // this is a comment
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("// this is a comment"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_json_comments_block() {
        let input = r#"{
            /* block comment */
            "key": "value"
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("/* block comment */"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_json_comments_in_string() {
        let input = r#"{"url": "http://example.com // not a comment"}"#;
        let result = strip_json_comments(input);
        // 字符串内的 // 应该保留
        assert!(result.contains("// not a comment"));
    }

    #[test]
    fn test_strip_json_comments_multiline_block() {
        let input = r#"{
            /*
             * Multi-line
             * block comment
             */
            "key": "value"
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_parse_claude_code_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "mcpServers": {
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                },
                "postgres-mcp": {
                    "command": "uvx",
                    "args": ["mcp-server-postgres"],
                    "env": {
                        "DATABASE_URL": "$DATABASE_URL"
                    }
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 2);

        let git_mcp = services.iter().find(|s| s.name == "git-mcp").unwrap();
        assert_eq!(git_mcp.command, "npx");
        assert_eq!(
            git_mcp.args,
            Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()])
        );

        let postgres_mcp = services.iter().find(|s| s.name == "postgres-mcp").unwrap();
        assert_eq!(postgres_mcp.command, "uvx");
        assert!(postgres_mcp.env.is_some());
        assert_eq!(
            postgres_mcp.env.as_ref().unwrap().get("DATABASE_URL"),
            Some(&"$DATABASE_URL".to_string())
        );
    }

    #[test]
    fn test_parse_cursor_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        let config_content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = CursorConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "filesystem");
        assert_eq!(services[0].command, "npx");
    }

    #[test]
    fn test_parse_config_with_jsonc_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            // MCP configuration
            "mcpServers": {
                /* Git MCP server */
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "git-mcp");
    }

    #[test]
    fn test_parse_config_with_sse_server() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "mcpServers": {
                "local-server": {
                    "command": "npx",
                    "args": ["-y", "local-mcp"]
                },
                "remote-server": {
                    "url": "http://remote.example.com/sse"
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        // SSE 服务应该被跳过
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "local-server");
    }

    #[test]
    fn test_generate_shadow_config() {
        let gateway_url = "http://127.0.0.1:8080/sse?token=test123";

        let shadow = generate_shadow_config(&ConfigSource::ClaudeCode, gateway_url);
        let parsed: serde_json::Value = serde_json::from_str(&shadow).unwrap();

        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            gateway_url
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_config_source_description() {
        // Story 11.8: 更新了 ConfigSource 描述
        assert_eq!(
            ConfigSource::ClaudeCode.description(),
            ".mcp.json"  // 更新为新的路径
        );
        assert_eq!(ConfigSource::Cursor.description(), ".cursor/mcp.json");
        assert_eq!(
            ConfigSource::ClaudeDesktop.description(),
            "claude_desktop_config.json"
        );
        assert_eq!(ConfigSource::Codex.description(), ".codex/config.toml");
        assert_eq!(ConfigSource::Gemini.description(), ".gemini/settings.json");
    }

    // ===== Task 2: 配置文件扫描器测试 =====

    #[test]
    fn test_scan_mcp_configs_project_level() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Story 11.8: 使用新的适配器架构扫描
        // 创建 .mcp.json (Claude Code 新路径)
        fs::write(
            project_path.join(".mcp.json"),
            r#"{"mcpServers": {"test": {"command": "test"}}}"#,
        )
        .unwrap();

        // 创建 .cursor/mcp.json
        let cursor_dir = project_path.join(".cursor");
        fs::create_dir_all(&cursor_dir).unwrap();
        fs::write(
            cursor_dir.join("mcp.json"),
            r#"{"mcpServers": {"cursor-test": {"command": "cursor"}}}"#,
        )
        .unwrap();

        let result = scan_mcp_configs(Some(project_path));

        // 项目级配置应该至少有 2 个（Claude + Cursor）
        // 注意：新架构扫描所有 4 个适配器的项目级配置
        let project_configs: Vec<_> = result
            .configs
            .iter()
            .filter(|c| c.path.starts_with(project_path))
            .collect();
        assert!(project_configs.len() >= 2, "Expected at least 2 project configs, got {}", project_configs.len());
        assert!(result.scanned_paths.len() >= 2);
    }

    #[test]
    fn test_scan_mcp_configs_no_project_configs() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_mcp_configs(Some(temp_dir.path()));

        // 项目级配置应该为空（但可能有全局配置）
        let project_configs: Vec<_> = result
            .configs
            .iter()
            .filter(|c| c.path.starts_with(temp_dir.path()))
            .collect();
        assert!(project_configs.is_empty());
    }

    // ===== Task 3: 导入预览逻辑测试 =====

    #[test]
    fn test_extract_env_var_references() {
        let env = Some(HashMap::from([
            ("API_KEY".to_string(), "$OPENAI_API_KEY".to_string()),
            ("DEBUG".to_string(), "true".to_string()),
            ("SECRET".to_string(), "${MY_SECRET}".to_string()),
        ]));

        let vars = extract_env_var_references(&env);

        assert!(vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(vars.contains(&"MY_SECRET".to_string()));
        assert!(!vars.contains(&"true".to_string()));
    }

    #[test]
    fn test_extract_env_var_references_empty() {
        let vars = extract_env_var_references(&None);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_generate_import_preview() {
        let db = Database::new_in_memory().unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/test/config.json"),
            scope: Some(ConfigScope::Project),
            services: vec![
                DetectedService {
                    name: "new-service".to_string(),
                    transport_type: Default::default(),
                    command: "npx".to_string(),
                    args: None,
                    env: Some(HashMap::from([(
                        "API_KEY".to_string(),
                        "$API_KEY".to_string(),
                    )])),
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                },
            ],
            parse_errors: Vec::new(),
        }];

        let preview = generate_import_preview(&configs, &db).unwrap();

        assert_eq!(preview.total_services, 1);
        assert_eq!(preview.new_services.len(), 1);
        assert!(preview.env_vars_needed.contains(&"API_KEY".to_string()));
    }

    #[test]
    fn test_generate_import_preview_with_conflict() {
        let db = Database::new_in_memory().unwrap();

        // 创建已存在的服务
        let request = CreateMcpServiceRequest {
            name: "existing-service".to_string(),
            transport_type: Default::default(),
            command: "old-command".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/test/config.json"),
            scope: Some(ConfigScope::Project),
            services: vec![DetectedService {
                name: "existing-service".to_string(),
                transport_type: Default::default(),
                command: "new-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            parse_errors: Vec::new(),
        }];

        let preview = generate_import_preview(&configs, &db).unwrap();

        assert_eq!(preview.conflicts.len(), 1);
        assert_eq!(preview.conflicts[0].name, "existing-service");
        assert!(preview.conflicts[0].existing.is_some());
    }

    // ===== Task 4: 备份与回滚测试 =====

    #[test]
    fn test_backup_manager_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        let mut manager = BackupManager::new();
        let backup_path = manager.backup(&test_file).unwrap();

        assert!(backup_path.exists());
        assert!(backup_path
            .to_string_lossy()
            .contains(".mantra-backup"));

        // 验证备份内容
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_backup_manager_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        let mut manager = BackupManager::new();
        manager.backup(&test_file).unwrap();

        // 修改原文件
        fs::write(&test_file, "modified content").unwrap();

        // 手动回滚
        manager.rollback().unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_backup_manager_auto_rollback_on_drop() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        {
            let mut manager = BackupManager::new();
            manager.backup(&test_file).unwrap();

            // 修改原文件
            fs::write(&test_file, "modified content").unwrap();

            // manager 在这里被 drop，但未 commit
        }

        // 应该已自动回滚
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_backup_manager_commit_prevents_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        {
            let mut manager = BackupManager::new();
            manager.backup(&test_file).unwrap();

            // 修改原文件
            fs::write(&test_file, "modified content").unwrap();

            // 提交
            manager.commit();
        }

        // commit 后不应回滚
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "modified content");
    }

    #[test]
    fn test_backup_manager_existing_backup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        let existing_backup = temp_dir.path().join("test.json.mantra-backup");

        fs::write(&test_file, "original content").unwrap();
        fs::write(&existing_backup, "old backup").unwrap();

        let mut manager = BackupManager::new();
        let backup_path = manager.backup(&test_file).unwrap();

        // 应该创建带时间戳的备份
        assert!(backup_path.exists());
        assert_ne!(backup_path, existing_backup);
        assert!(backup_path
            .to_string_lossy()
            .contains(".mantra-backup."));
    }

    // ===== Task 5: 影子模式配置测试 =====

    #[test]
    fn test_shadow_config_format() {
        let gateway_url = "http://127.0.0.1:8080/sse?token=abc123";

        for source in [
            ConfigSource::ClaudeCode,
            ConfigSource::Cursor,
            ConfigSource::ClaudeDesktop,
        ] {
            let shadow = generate_shadow_config(&source, gateway_url);
            let parsed: serde_json::Value = serde_json::from_str(&shadow).unwrap();

            assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
            assert_eq!(
                parsed["mcpServers"]["mantra-gateway"]["url"],
                gateway_url
            );
        }
    }

    // ===== Task 6: 导入执行器测试 =====

    #[test]
    fn test_import_executor_basic() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test-mcp".to_string()]),
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: vec!["test-service".to_string()],
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert!(result.errors.is_empty());

        // 验证服务已创建
        let service = db.get_mcp_service_by_name("test-service").unwrap();
        assert!(service.is_some());
        assert_eq!(service.unwrap().command, "npx");
    }

    #[test]
    fn test_import_executor_skip_service() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "skipped-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(), // 不包含任何服务
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_keep() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing.clone()),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Keep,
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 应该保持原服务不变
        let service = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(service.command, "old-command");
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_replace() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Replace(0),
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 服务应该被替换
        let service = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(service.command, "new-command");
        assert_eq!(result.imported_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_rename() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Rename("renamed-service".to_string()),
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 原服务应该保留
        let original = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(original.command, "old-command");

        // 新服务应该以新名称创建
        let renamed = db.get_mcp_service_by_name("renamed-service").unwrap().unwrap();
        assert_eq!(renamed.command, "new-command");

        assert_eq!(result.imported_count, 1);
    }

    #[test]
    fn test_import_executor_with_env_vars() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "api-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: Some(HashMap::from([("API_KEY".to_string(), "$API_KEY".to_string())])),
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: vec!["API_KEY".to_string()],
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: vec!["api-service".to_string()],
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::from([("API_KEY".to_string(), "secret-key-123".to_string())]),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 1);
        assert!(result.errors.is_empty());

        // 验证环境变量已存储
        assert!(db.env_variable_exists("API_KEY").unwrap());
    }

    // ===== 回滚功能测试 =====

    #[test]
    fn test_rollback_from_backups() {
        let temp_dir = TempDir::new().unwrap();
        let original_file = temp_dir.path().join("config.json");
        let backup_file = temp_dir.path().join("config.json.mantra-backup");

        fs::write(&original_file, "modified content").unwrap();
        fs::write(&backup_file, "original content").unwrap();

        let restored = rollback_from_backups(&[backup_file]).unwrap();

        assert_eq!(restored, 1);
        let content = fs::read_to_string(&original_file).unwrap();
        assert_eq!(content, "original content");
    }
}
