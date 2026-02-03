//! MCP 配置相关数据类型定义
//!
//! Story 11.3: 配置导入与接管
//! Story 11.8: MCP Gateway Architecture Refactor

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crate::models::mcp::{McpService, McpTransportType};
use crate::services::mcp_adapters::{
    ConfigScope, DetectedConfig as AdapterDetectedConfig, DetectedService as AdapterDetectedService,
};

// ===== 配置来源类型 =====

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

// ===== 检测到的服务和配置 =====

/// 检测到的 MCP 服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedService {
    /// 服务名称
    pub name: String,
    /// 传输类型（stdio 或 http）
    #[serde(default)]
    pub transport_type: McpTransportType,
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

// ===== 冲突和导入相关类型 =====

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
    ///
    /// Story 11.15: 此字段已弃用，导入时总是强制接管
    #[deprecated(since = "0.7.0", note = "Shadow mode is now always enabled during import")]
    #[serde(default)]
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
    /// 修改为影子模式的配置文件（已接管的工具配置）
    pub shadow_configs: Vec<PathBuf>,
    /// 错误信息
    pub errors: Vec<String>,
    /// 导入的服务 ID 列表
    pub imported_service_ids: Vec<String>,
    /// 接管备份记录 ID 列表 (Story 11.15)
    #[serde(default)]
    pub takeover_backup_ids: Vec<String>,
}

// ===== 扫描结果 =====

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 检测到的配置文件
    pub configs: Vec<DetectedConfig>,
    /// 扫描的路径
    pub scanned_paths: Vec<PathBuf>,
}

// ===== 错误类型 =====

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

// ===== 同步接管结果 =====

/// 同步接管配置的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTakeoverResult {
    /// 成功同步的配置数量
    pub synced_count: usize,
    /// 同步失败的配置数量
    pub failed_count: usize,
    /// 错误信息列表
    pub errors: Vec<String>,
}

// ===== 智能接管执行结果 =====

/// 智能接管执行结果 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTakeoverResult {
    /// 成功创建的服务数量
    pub created_count: usize,
    /// 跳过的服务数量（auto_skip + keep_existing）
    pub skipped_count: usize,
    /// 更新的服务数量（use_new）
    pub updated_count: usize,
    /// 重命名创建的服务数量（keep_both）
    pub renamed_count: usize,
    /// 错误信息列表
    pub errors: Vec<String>,
    /// 创建的服务 ID 列表
    pub created_service_ids: Vec<String>,
    /// 接管备份记录 ID 列表
    pub takeover_backup_ids: Vec<String>,
    /// 配置文件接管路径列表
    pub takeover_config_paths: Vec<PathBuf>,
    /// Gateway 是否运行中
    pub gateway_running: bool,
}

impl SmartTakeoverResult {
    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            created_count: 0,
            skipped_count: 0,
            updated_count: 0,
            renamed_count: 0,
            errors: Vec::new(),
            created_service_ids: Vec::new(),
            takeover_backup_ids: Vec::new(),
            takeover_config_paths: Vec::new(),
            gateway_running: false,
        }
    }

    /// 是否完全成功（无错误）
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

// ===== 全工具接管结果 =====

/// 全工具接管执行结果 (Story 11.20)
///
/// 包含执行状态和详细的回滚信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTakeoverResult {
    /// 是否成功（无致命错误）
    pub success: bool,
    /// 是否已回滚
    pub rolled_back: bool,
    /// 执行的操作统计
    pub stats: TakeoverStats,
    /// 错误信息列表
    pub errors: Vec<String>,
    /// 警告信息列表
    pub warnings: Vec<String>,
    /// 创建的服务 ID 列表
    pub created_service_ids: Vec<String>,
    /// 接管备份记录 ID 列表
    pub takeover_backup_ids: Vec<String>,
    /// 配置文件接管路径列表
    pub takeover_config_paths: Vec<PathBuf>,
    /// Gateway 是否运行中
    pub gateway_running: bool,
}

/// 接管操作统计 (Story 11.20)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TakeoverStats {
    /// 成功创建的服务数量
    pub created_count: usize,
    /// 跳过的服务数量
    pub skipped_count: usize,
    /// 更新的服务数量
    pub updated_count: usize,
    /// 重命名创建的服务数量
    pub renamed_count: usize,
    /// 成功接管的配置文件数量
    pub takeover_count: usize,
    /// 处理的工具数量
    pub tool_count: usize,
}

impl FullTakeoverResult {
    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            success: true,
            rolled_back: false,
            stats: TakeoverStats::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
            created_service_ids: Vec::new(),
            takeover_backup_ids: Vec::new(),
            takeover_config_paths: Vec::new(),
            gateway_running: false,
        }
    }

    /// 标记为失败
    pub fn fail(mut self, error: String) -> Self {
        self.success = false;
        self.errors.push(error);
        self
    }
}
