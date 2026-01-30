//! MCP 配置解析与导入服务
//!
//! Story 11.3: 配置导入与接管
//!
//! 提供 MCP 配置文件的解析、扫描、备份、导入功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::mcp::{CreateMcpServiceRequest, McpService, McpServiceSource};
use crate::services::EnvManager;
use crate::storage::{Database, StorageError};

// ===== 数据类型定义 =====

/// 配置文件来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    /// Claude Code: .claude/config.json
    ClaudeCode,
    /// Cursor: .cursor/mcp.json 或 ~/.cursor/mcp.json
    Cursor,
    /// Claude Desktop: claude_desktop_config.json
    ClaudeDesktop,
}

impl ConfigSource {
    /// 获取配置文件的典型路径描述
    pub fn description(&self) -> &'static str {
        match self {
            ConfigSource::ClaudeCode => ".claude/config.json",
            ConfigSource::Cursor => ".cursor/mcp.json",
            ConfigSource::ClaudeDesktop => "claude_desktop_config.json",
        }
    }
}

/// 检测到的 MCP 服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedService {
    /// 服务名称
    pub name: String,
    /// 启动命令
    pub command: String,
    /// 命令参数
    pub args: Option<Vec<String>>,
    /// 环境变量
    pub env: Option<HashMap<String, String>>,
    /// 来源配置文件路径
    pub source_file: PathBuf,
    /// 来源类型
    pub source_type: ConfigSource,
}

/// 检测到的配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedConfig {
    /// 配置来源类型
    pub source: ConfigSource,
    /// 配置文件路径
    pub path: PathBuf,
    /// 检测到的服务列表
    pub services: Vec<DetectedService>,
    /// 解析错误（如有）
    pub parse_errors: Vec<String>,
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
pub fn strip_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' && !escape_next {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if in_string {
            result.push(c);
            continue;
        }

        // 不在字符串中，检查注释
        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // 单行注释，跳过到行末
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        if nc == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    // 块注释，跳过到 */
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' {
                            if let Some(&'/' ) = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

/// MCP 配置解析器 trait
pub trait McpConfigParser {
    /// 解析配置文件
    fn parse(&self, path: &Path) -> Result<Vec<DetectedService>, ParseError>;

    /// 获取配置来源类型
    fn source_type(&self) -> ConfigSource;

    /// 生成影子模式配置
    fn generate_shadow_config(&self, gateway_url: &str) -> String;
}

/// Claude Code 配置解析器
pub struct ClaudeCodeConfigParser;

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
                        command,
                        args,
                        env,
                        source_file: path.to_path_buf(),
                        source_type: ConfigSource::ClaudeCode,
                    });
                }
                // 跳过 SSE 模式的服务（通常是已配置的 gateway）
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

/// Cursor 配置解析器
pub struct CursorConfigParser;

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
                        command,
                        args,
                        env,
                        source_file: path.to_path_buf(),
                        source_type: ConfigSource::Cursor,
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

/// Claude Desktop 配置解析器
pub struct ClaudeDesktopConfigParser;

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
                        command,
                        args,
                        env,
                        source_file: path.to_path_buf(),
                        source_type: ConfigSource::ClaudeDesktop,
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

/// 扫描 MCP 配置文件
///
/// # Arguments
/// * `project_path` - 项目路径（可选，用于扫描项目级配置）
///
/// # Returns
/// 扫描结果，包含所有检测到的配置文件和服务
pub fn scan_mcp_configs(project_path: Option<&Path>) -> ScanResult {
    let mut configs = Vec::new();
    let mut scanned_paths = Vec::new();

    // 1. 项目级配置扫描
    if let Some(project) = project_path {
        // Claude Code: {project}/.claude/config.json
        let claude_code_path = project.join(".claude").join("config.json");
        scanned_paths.push(claude_code_path.clone());
        if claude_code_path.exists() {
            configs.push(parse_config_file(&claude_code_path, ConfigSource::ClaudeCode));
        }

        // Cursor: {project}/.cursor/mcp.json
        let cursor_path = project.join(".cursor").join("mcp.json");
        scanned_paths.push(cursor_path.clone());
        if cursor_path.exists() {
            configs.push(parse_config_file(&cursor_path, ConfigSource::Cursor));
        }
    }

    // 2. 全局配置扫描
    if let Some(home) = dirs::home_dir() {
        // Cursor 全局: ~/.cursor/mcp.json
        let cursor_global = home.join(".cursor").join("mcp.json");
        scanned_paths.push(cursor_global.clone());
        if cursor_global.exists() {
            configs.push(parse_config_file(&cursor_global, ConfigSource::Cursor));
        }
    }

    // Claude Desktop
    if let Some(claude_desktop_path) = get_claude_desktop_config_path() {
        scanned_paths.push(claude_desktop_path.clone());
        if claude_desktop_path.exists() {
            configs.push(parse_config_file(
                &claude_desktop_path,
                ConfigSource::ClaudeDesktop,
            ));
        }
    }

    ScanResult {
        configs,
        scanned_paths,
    }
}

/// 解析单个配置文件
fn parse_config_file(path: &Path, source: ConfigSource) -> DetectedConfig {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
    };

    match parser.parse(path) {
        Ok(services) => DetectedConfig {
            source,
            path: path.to_path_buf(),
            services,
            parse_errors: Vec::new(),
        },
        Err(e) => DetectedConfig {
            source,
            path: path.to_path_buf(),
            services: Vec::new(),
            parse_errors: vec![e.to_string()],
        },
    }
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

/// 生成影子模式配置
///
/// # Arguments
/// * `source` - 配置来源类型
/// * `gateway_url` - Mantra Gateway URL
///
/// # Returns
/// 影子模式配置 JSON 字符串
pub fn generate_shadow_config(source: &ConfigSource, gateway_url: &str) -> String {
    let parser: Box<dyn McpConfigParser> = match source {
        ConfigSource::ClaudeCode => Box::new(ClaudeCodeConfigParser),
        ConfigSource::Cursor => Box::new(CursorConfigParser),
        ConfigSource::ClaudeDesktop => Box::new(ClaudeDesktopConfigParser),
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
                Some(&format!("Imported from MCP config")),
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
                for config in &preview.configs {
                    if !config.services.is_empty() {
                        match self.apply_shadow_mode(&config.path, &config.source, gateway_url) {
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
            command: service.command.clone(),
            args: service.args.clone(),
            env: service.env.as_ref().map(|e| serde_json::to_value(e).unwrap()),
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

    /// 应用影子模式
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
    use std::io::Write;
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
    fn test_config_source_description() {
        assert_eq!(
            ConfigSource::ClaudeCode.description(),
            ".claude/config.json"
        );
        assert_eq!(ConfigSource::Cursor.description(), ".cursor/mcp.json");
        assert_eq!(
            ConfigSource::ClaudeDesktop.description(),
            "claude_desktop_config.json"
        );
    }

    // ===== Task 2: 配置文件扫描器测试 =====

    #[test]
    fn test_scan_mcp_configs_project_level() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // 创建 .claude/config.json
        let claude_dir = project_path.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join("config.json"),
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
        // 注意：全局配置也会被扫描，所以可能会多于 2 个
        let project_configs: Vec<_> = result
            .configs
            .iter()
            .filter(|c| c.path.starts_with(project_path))
            .collect();
        assert_eq!(project_configs.len(), 2);
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
            source: ConfigSource::ClaudeCode,
            path: PathBuf::from("/test/config.json"),
            services: vec![
                DetectedService {
                    name: "new-service".to_string(),
                    command: "npx".to_string(),
                    args: None,
                    env: Some(HashMap::from([(
                        "API_KEY".to_string(),
                        "$API_KEY".to_string(),
                    )])),
                    source_file: PathBuf::from("/test/config.json"),
                    source_type: ConfigSource::ClaudeCode,
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
            command: "old-command".to_string(),
            args: None,
            env: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let configs = vec![DetectedConfig {
            source: ConfigSource::ClaudeCode,
            path: PathBuf::from("/test/config.json"),
            services: vec![DetectedService {
                name: "existing-service".to_string(),
                command: "new-command".to_string(),
                args: None,
                env: None,
                source_file: PathBuf::from("/test/config.json"),
                source_type: ConfigSource::ClaudeCode,
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
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test-mcp".to_string()]),
                env: None,
                source_file: PathBuf::from("/test/config.json"),
                source_type: ConfigSource::ClaudeCode,
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
                command: "npx".to_string(),
                args: None,
                env: None,
                source_file: PathBuf::from("/test/config.json"),
                source_type: ConfigSource::ClaudeCode,
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
                command: "old-command".to_string(),
                args: None,
                env: None,
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
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    source_file: PathBuf::from("/test/config.json"),
                    source_type: ConfigSource::ClaudeCode,
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
                command: "old-command".to_string(),
                args: None,
                env: None,
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
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    source_file: PathBuf::from("/test/config.json"),
                    source_type: ConfigSource::ClaudeCode,
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
                command: "old-command".to_string(),
                args: None,
                env: None,
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
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    source_file: PathBuf::from("/test/config.json"),
                    source_type: ConfigSource::ClaudeCode,
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
                command: "npx".to_string(),
                args: None,
                env: Some(HashMap::from([("API_KEY".to_string(), "$API_KEY".to_string())])),
                source_file: PathBuf::from("/test/config.json"),
                source_type: ConfigSource::ClaudeCode,
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
