//! MCP 服务数据模型
//!
//! Story 11.2: MCP 服务数据模型 - Task 1
//!
//! 定义 MCP 服务配置、项目关联和环境变量的数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// MCP 服务参数类型别名
pub type McpServiceArgs = Vec<String>;

/// MCP 服务环境变量类型别名
pub type McpServiceEnv = serde_json::Value;

/// MCP 传输类型
///
/// 支持 stdio（子进程）和 http（Streamable HTTP）两种传输模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpTransportType {
    /// stdio 传输：通过子进程 stdin/stdout 通信
    #[default]
    Stdio,
    /// HTTP 传输：Streamable HTTP（MCP 2025-03-26 规范）
    Http,
}

impl McpTransportType {
    /// 转换为数据库存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            McpTransportType::Stdio => "stdio",
            McpTransportType::Http => "http",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "stdio" => Some(McpTransportType::Stdio),
            "http" => Some(McpTransportType::Http),
            _ => None,
        }
    }
}

/// MCP 服务来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpServiceSource {
    /// 从外部配置文件导入（如 Claude 的 .mcp.json）
    Imported,
    /// 用户手动添加
    Manual,
}

impl McpServiceSource {
    /// 转换为数据库存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            McpServiceSource::Imported => "imported",
            McpServiceSource::Manual => "manual",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "imported" => Some(McpServiceSource::Imported),
            "manual" => Some(McpServiceSource::Manual),
            _ => None,
        }
    }
}

/// MCP 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpService {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 服务名称，如 "git-mcp"、"filesystem"
    pub name: String,
    /// 传输类型（stdio 或 http）
    #[serde(default)]
    pub transport_type: McpTransportType,
    /// 启动命令，如 "npx"、"uvx"（仅 stdio 模式）
    pub command: String,
    /// 命令参数，JSON 数组格式（仅 stdio 模式）
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用，JSON 对象格式
    /// 值可以是字面量或变量引用（如 "$OPENAI_API_KEY"）
    pub env: Option<McpServiceEnv>,
    /// HTTP 端点 URL（仅 http 模式）
    pub url: Option<String>,
    /// HTTP 请求头（仅 http 模式）
    pub headers: Option<HashMap<String, String>>,
    /// 服务来源
    pub source: McpServiceSource,
    /// 导入来源的原始配置文件路径
    pub source_file: Option<String>,
    /// 首次导入时的工具来源 (Story 11.19)
    /// 如 "claude"、"cursor"、"codex"、"gemini"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_adapter_id: Option<String>,
    /// 首次导入时的 scope (Story 11.19)
    /// 'project' 或 'user'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_scope: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间 (ISO 8601)
    pub created_at: String,
    /// 更新时间 (ISO 8601)
    pub updated_at: String,
    /// 服务级默认 Tool Policy (Story 11.9 Phase 2)
    /// 当项目未配置项目级 Policy 时，使用此默认策略
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_tool_policy: Option<ToolPolicy>,
}

/// 创建 MCP 服务的请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMcpServiceRequest {
    /// 服务名称
    pub name: String,
    /// 传输类型（默认 stdio）
    #[serde(default)]
    pub transport_type: McpTransportType,
    /// 启动命令（stdio 模式必填）
    #[serde(default)]
    pub command: String,
    /// 命令参数（stdio 模式）
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用
    pub env: Option<McpServiceEnv>,
    /// HTTP 端点 URL（http 模式必填）
    pub url: Option<String>,
    /// HTTP 请求头（http 模式可选）
    pub headers: Option<HashMap<String, String>>,
    /// 服务来源
    pub source: McpServiceSource,
    /// 导入来源的原始配置文件路径
    pub source_file: Option<String>,
}

/// 更新 MCP 服务的请求参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateMcpServiceRequest {
    /// 服务名称
    pub name: Option<String>,
    /// 传输类型
    pub transport_type: Option<McpTransportType>,
    /// 启动命令
    pub command: Option<String>,
    /// 命令参数
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用
    pub env: Option<McpServiceEnv>,
    /// HTTP 端点 URL
    pub url: Option<String>,
    /// HTTP 请求头
    pub headers: Option<HashMap<String, String>>,
    /// 是否启用
    pub enabled: Option<bool>,
}

/// 项目与 MCP 服务的关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMcpService {
    /// 关联的项目 ID
    pub project_id: String,
    /// 关联的服务 ID
    pub service_id: String,
    /// 项目级配置覆盖，JSON 对象格式
    /// 支持 `toolPolicy` 字段 (Story 11.10)
    pub config_override: Option<serde_json::Value>,
    /// 该项目发现此服务时的工具 ID (Story 11.19)
    /// 如 "claude"、"cursor"、"codex"、"gemini"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_adapter_id: Option<String>,
    /// 该项目发现此服务时的配置文件路径 (Story 11.19)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_config_path: Option<String>,
    /// 创建时间 (ISO 8601)
    pub created_at: String,
}

impl McpService {
    /// 获取服务级默认 Tool Policy (Story 11.9 Phase 2)
    ///
    /// 返回 `default_tool_policy` 字段值。
    /// 如果未配置，返回默认策略 (AllowAll)。
    pub fn get_default_tool_policy(&self) -> ToolPolicy {
        self.default_tool_policy.clone().unwrap_or_default()
    }

    /// 设置服务级默认 Tool Policy (Story 11.9 Phase 2)
    pub fn set_default_tool_policy(&mut self, policy: Option<ToolPolicy>) {
        self.default_tool_policy = policy;
    }
}

impl ProjectMcpService {
    /// 获取项目的 Tool Policy
    ///
    /// Story 11.10 → Story 11.18: Project-Level Tool Management
    ///
    /// 从 `config_override.toolPolicy` 解析 Tool Policy。
    /// 如果未配置或解析失败，返回继承策略 (inherit) 以回退到服务默认。
    pub fn get_tool_policy(&self) -> ToolPolicy {
        self.config_override
            .as_ref()
            .and_then(|config| config.get("toolPolicy"))
            .and_then(|policy_value| serde_json::from_value(policy_value.clone()).ok())
            .unwrap_or_else(ToolPolicy::inherit)
    }

    /// 设置 Tool Policy
    ///
    /// Story 11.10: Project-Level Tool Management
    ///
    /// 更新 `config_override.toolPolicy` 字段。
    pub fn set_tool_policy(&mut self, policy: &ToolPolicy) {
        let policy_value = serde_json::to_value(policy).unwrap_or_default();

        match &mut self.config_override {
            Some(config) => {
                if let Some(obj) = config.as_object_mut() {
                    obj.insert("toolPolicy".to_string(), policy_value);
                }
            }
            None => {
                self.config_override = Some(serde_json::json!({
                    "toolPolicy": policy_value
                }));
            }
        }
    }
}

/// MCP 服务及其项目级配置覆盖
///
/// Story 11.19: 扩展支持 detected_adapter_id 和 detected_config_path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceWithOverride {
    /// 服务配置
    #[serde(flatten)]
    pub service: McpService,
    /// 项目级配置覆盖
    pub config_override: Option<serde_json::Value>,
    /// 项目发现此服务时的工具 ID (Story 11.19)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_adapter_id: Option<String>,
    /// 项目发现此服务时的配置文件路径 (Story 11.19)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_config_path: Option<String>,
}

/// 环境变量（列表展示用，值已脱敏）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 变量名称，如 "OPENAI_API_KEY"
    pub name: String,
    /// 脱敏后的值，如 "sk-****...****xyz"
    pub masked_value: String,
    /// 变量描述
    pub description: Option<String>,
    /// 创建时间 (ISO 8601)
    pub created_at: String,
    /// 更新时间 (ISO 8601)
    pub updated_at: String,
}

/// 创建/更新环境变量的请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEnvVariableRequest {
    /// 变量名称
    pub name: String,
    /// 变量值（明文，将被加密存储）
    pub value: String,
    /// 变量描述
    pub description: Option<String>,
}

/// 环境变量名称校验结果
///
/// Story 11.4: 环境变量管理 - Task 1.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariableNameValidation {
    /// 是否有效
    pub is_valid: bool,
    /// 格式化建议（如果名称无效）
    pub suggestion: Option<String>,
    /// 错误信息（如果名称无效）
    pub error_message: Option<String>,
}

// ===== Story 11.10 → Story 11.18: Tool Policy 简化 =====

/// Tool Policy 模式 (已废弃，仅用于向后兼容反序列化)
///
/// Story 11.18: 从旧格式迁移时使用
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolPolicyMode {
    #[default]
    AllowAll,
    DenyAll,
    Custom,
}

/// Tool Policy 配置
///
/// Story 11.18: MCP 权限管理 UX 系统性重构 - AC 1
///
/// 简化的权限模型:
/// - `allowed_tools = None` → 继承全局默认（仅项目级有效）
/// - `allowed_tools = Some([])` → 全选（允许所有工具）
/// - `allowed_tools = Some([...])` → 部分选（仅允许指定工具）
/// - 不关联服务 = 禁用（等同于旧 deny_all）
///
/// ## 示例
/// ```json
/// // 全选（允许所有工具）
/// { "allowedTools": [] }
///
/// // 部分选
/// { "allowedTools": ["read_file", "list_commits"] }
///
/// // 继承全局默认
/// { "allowedTools": null }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPolicy {
    /// 允许的工具列表
    ///
    /// - `None`: 继承全局默认（仅项目级有效）
    /// - `Some([])`: 全选（允许所有工具）
    /// - `Some([...])`: 部分选（仅允许指定工具）
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,

    // === 向后兼容字段（反序列化时接收，不再使用） ===
    /// 已废弃：旧模式字段，反序列化后忽略
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    mode: Option<ToolPolicyMode>,
    /// 已废弃：旧禁止列表字段，反序列化后忽略
    #[serde(default, skip_serializing, rename = "deniedTools")]
    #[allow(dead_code)]
    denied_tools: Option<Vec<String>>,
}

impl Default for ToolPolicy {
    /// 默认策略：全选（允许所有工具）
    fn default() -> Self {
        Self {
            allowed_tools: Some(vec![]),
            mode: None,
            denied_tools: None,
        }
    }
}

impl ToolPolicy {
    /// 创建全选策略（允许所有工具）
    pub fn allow_all() -> Self {
        Self {
            allowed_tools: Some(vec![]),
            mode: None,
            denied_tools: None,
        }
    }

    /// 创建继承策略（继承全局默认）
    pub fn inherit() -> Self {
        Self {
            allowed_tools: None,
            mode: None,
            denied_tools: None,
        }
    }

    /// 创建部分选策略
    pub fn custom(tools: Vec<String>) -> Self {
        Self {
            allowed_tools: Some(tools),
            mode: None,
            denied_tools: None,
        }
    }

    /// 是否为继承模式
    pub fn is_inherit(&self) -> bool {
        self.allowed_tools.is_none()
    }

    /// 是否为全选模式
    pub fn is_allow_all(&self) -> bool {
        matches!(&self.allowed_tools, Some(tools) if tools.is_empty())
    }

    /// 是否为部分选模式
    pub fn is_custom(&self) -> bool {
        matches!(&self.allowed_tools, Some(tools) if !tools.is_empty())
    }

    /// 检查工具是否被允许
    ///
    /// - `None` (继承): 返回 true（实际继承由 PolicyResolver 处理）
    /// - `Some([])` (全选): 返回 true
    /// - `Some([...])` (部分选): 工具在列表中才返回 true
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        match &self.allowed_tools {
            None => true, // 继承 = 由上层决定，此处默认允许
            Some(tools) if tools.is_empty() => true, // 全选
            Some(tools) => tools.iter().any(|t| t == tool_name), // 部分选
        }
    }

    /// 过滤工具列表，返回被允许的工具
    pub fn filter_tools<'a, T>(&self, tools: &'a [T], get_name: impl Fn(&T) -> &str) -> Vec<&'a T> {
        tools
            .iter()
            .filter(|tool| self.is_tool_allowed(get_name(tool)))
            .collect()
    }

    /// 从旧格式 ToolPolicy 迁移到新格式
    ///
    /// Story 11.18: 数据迁移逻辑
    /// - deny_all → None（表示应删除关联）
    /// - allow_all → Some([])
    /// - custom → Some(allowed_tools)
    pub fn migrate_from_legacy(json: &serde_json::Value) -> Option<Self> {
        let mode = json.get("mode").and_then(|v| v.as_str()).unwrap_or("allow_all");
        let allowed_tools: Vec<String> = json
            .get("allowedTools")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        match mode {
            "deny_all" => None, // 表示应删除关联
            "allow_all" => Some(ToolPolicy::allow_all()),
            "custom" => Some(ToolPolicy::custom(allowed_tools)),
            _ => Some(ToolPolicy::allow_all()),
        }
    }
}

/// MCP 服务工具定义
///
/// Story 11.10: Project-Level Tool Management - AC 2
///
/// 用于缓存 MCP 服务提供的工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServiceTool {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 关联的服务 ID
    pub service_id: String,
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: Option<String>,
    /// 输入参数 JSON Schema
    pub input_schema: Option<serde_json::Value>,
    /// 缓存时间 (ISO 8601)
    pub cached_at: String,
}

impl McpServiceTool {
    /// 检查缓存是否已过期
    ///
    /// # Arguments
    /// * `ttl_seconds` - 缓存有效期（秒）
    pub fn is_expired(&self, ttl_seconds: i64) -> bool {
        if let Ok(cached_at) = chrono::DateTime::parse_from_rfc3339(&self.cached_at) {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(cached_at);
            elapsed.num_seconds() > ttl_seconds
        } else {
            true // 无法解析时间，视为过期
        }
    }
}

// ===== Story 11.15: MCP 接管备份模型 =====

/// 工具类型
///
/// Story 11.15: MCP 接管流程重构 - AC 6
///
/// 支持的 AI 编程工具类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// Claude Code
    ClaudeCode,
    /// Cursor
    Cursor,
    /// Codex (OpenAI)
    Codex,
    /// Gemini CLI
    GeminiCli,
}

impl ToolType {
    /// 转换为数据库存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolType::ClaudeCode => "claude_code",
            ToolType::Cursor => "cursor",
            ToolType::Codex => "codex",
            ToolType::GeminiCli => "gemini_cli",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude_code" => Some(ToolType::ClaudeCode),
            "cursor" => Some(ToolType::Cursor),
            "codex" => Some(ToolType::Codex),
            "gemini_cli" => Some(ToolType::GeminiCli),
            _ => None,
        }
    }

    /// 从 adapter_id 解析
    pub fn from_adapter_id(id: &str) -> Option<Self> {
        match id {
            "claude" => Some(ToolType::ClaudeCode),
            "cursor" => Some(ToolType::Cursor),
            "codex" => Some(ToolType::Codex),
            "gemini" => Some(ToolType::GeminiCli),
            _ => None,
        }
    }

    /// 转换为适配器 ID
    pub fn to_adapter_id(&self) -> &'static str {
        match self {
            ToolType::ClaudeCode => "claude",
            ToolType::Cursor => "cursor",
            ToolType::Codex => "codex",
            ToolType::GeminiCli => "gemini",
        }
    }

    /// 获取用户级配置文件路径
    ///
    /// 根据工具类型返回对应的用户级配置文件路径
    pub fn get_user_config_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        match self {
            ToolType::ClaudeCode => home.join(".claude.json"),
            ToolType::Cursor => home.join(".cursor").join("mcp.json"),
            ToolType::Codex => home.join(".codex").join("config.toml"),
            ToolType::GeminiCli => home.join(".gemini").join("settings.json"),
        }
    }

    /// 获取工具显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ToolType::ClaudeCode => "Claude Code",
            ToolType::Cursor => "Cursor",
            ToolType::Codex => "Codex",
            ToolType::GeminiCli => "Gemini CLI",
        }
    }

    /// 获取所有支持的工具类型
    ///
    /// Story 11.20: 全工具自动接管生成 - AC 1
    pub fn all() -> Vec<Self> {
        vec![
            ToolType::ClaudeCode,
            ToolType::Cursor,
            ToolType::Codex,
            ToolType::GeminiCli,
        ]
    }
}

// ===== Story 11.20: 全工具自动接管生成 =====

/// 单个工具的检测结果
///
/// Story 11.20: 全工具自动接管生成 - AC 1
///
/// 表示对单个 AI 编程工具的安装检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDetectionResult {
    /// 工具类型
    pub tool_type: ToolType,
    /// 是否已安装（用户配置文件存在）
    pub installed: bool,
    /// 用户级配置文件路径
    pub user_config_path: PathBuf,
    /// 用户级配置文件是否存在
    pub user_config_exists: bool,
    /// 工具显示名称
    pub display_name: String,
    /// 适配器 ID
    pub adapter_id: String,
}

/// 所有工具的检测结果
///
/// Story 11.20: 全工具自动接管生成 - AC 1
///
/// 聚合所有支持工具的检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllToolsDetectionResult {
    /// 各工具的检测结果列表
    pub tools: Vec<ToolDetectionResult>,
    /// 已安装工具数量
    pub installed_count: usize,
    /// 总工具数量
    pub total_count: usize,
}

// ===== Story 11.20: 全 Scope 扫描结果 =====

/// Scope 扫描结果
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// 单个 Scope 的扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeScanResult {
    /// 配置文件路径
    pub config_path: PathBuf,
    /// 配置文件是否存在
    pub exists: bool,
    /// 检测到的服务数量
    pub service_count: usize,
    /// 检测到的服务名称列表
    pub service_names: Vec<String>,
    /// 解析错误（如有）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parse_errors: Vec<String>,
}

impl ScopeScanResult {
    /// 创建空结果（配置文件不存在）
    pub fn not_found(config_path: PathBuf) -> Self {
        Self {
            config_path,
            exists: false,
            service_count: 0,
            service_names: Vec::new(),
            parse_errors: Vec::new(),
        }
    }

    /// 创建成功结果
    pub fn success(config_path: PathBuf, service_names: Vec<String>) -> Self {
        Self {
            config_path,
            exists: true,
            service_count: service_names.len(),
            service_names,
            parse_errors: Vec::new(),
        }
    }

    /// 创建错误结果
    pub fn with_error(config_path: PathBuf, error: String) -> Self {
        Self {
            config_path,
            exists: true,
            service_count: 0,
            service_names: Vec::new(),
            parse_errors: vec![error],
        }
    }
}

/// Local Scope 扫描结果 (Claude Code projects.*)
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// Claude Code 特有的 Local Scope，对应 ~/.claude.json 中的 projects.* 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalScopeScanResult {
    /// 项目路径 (projects 的 key)
    pub project_path: String,
    /// 检测到的服务数量
    pub service_count: usize,
    /// 检测到的服务名称列表
    pub service_names: Vec<String>,
}

/// 单个工具的扫描结果
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// 包含工具在各 Scope 下的配置扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolScanResult {
    /// 工具类型
    pub tool_type: ToolType,
    /// 工具显示名称
    pub display_name: String,
    /// 适配器 ID
    pub adapter_id: String,
    /// 是否已安装（用户配置文件存在）
    pub installed: bool,
    /// User Scope 扫描结果
    pub user_scope: Option<ScopeScanResult>,
    /// Local Scope 扫描结果列表 (仅 Claude Code)
    /// 对应 ~/.claude.json 中的 projects.* 配置
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_scopes: Vec<LocalScopeScanResult>,
    /// Project Scope 扫描结果
    pub project_scope: Option<ScopeScanResult>,
    /// 总服务数量（所有 Scope 累计）
    pub total_service_count: usize,
}

impl ToolScanResult {
    /// 创建新的工具扫描结果
    pub fn new(tool_type: ToolType) -> Self {
        Self {
            display_name: tool_type.display_name().to_string(),
            adapter_id: tool_type.to_adapter_id().to_string(),
            installed: false,
            tool_type,
            user_scope: None,
            local_scopes: Vec::new(),
            project_scope: None,
            total_service_count: 0,
        }
    }

    /// 计算并更新总服务数量
    pub fn update_total_service_count(&mut self) {
        let user_count = self.user_scope.as_ref().map_or(0, |s| s.service_count);
        let local_count: usize = self.local_scopes.iter().map(|s| s.service_count).sum();
        let project_count = self.project_scope.as_ref().map_or(0, |s| s.service_count);
        self.total_service_count = user_count + local_count + project_count;
    }

    /// 检查是否有任何配置
    pub fn has_any_config(&self) -> bool {
        self.total_service_count > 0
    }
}

/// 全工具扫描结果
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// 聚合所有支持工具的扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllToolsScanResult {
    /// 各工具的扫描结果
    pub tools: Vec<ToolScanResult>,
    /// 扫描的项目路径
    pub project_path: String,
    /// 已安装工具数量
    pub installed_count: usize,
    /// 有配置的工具数量
    pub tools_with_config_count: usize,
    /// 总服务数量（所有工具累计）
    pub total_service_count: usize,
}

// ===== Story 11.20: 全工具接管预览 =====

/// 单个工具的接管预览
///
/// Story 11.20: 全工具自动接管生成 - AC 3
///
/// 包含工具检测信息 + 三档分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolTakeoverPreview {
    /// 工具类型
    pub tool_type: ToolType,
    /// 工具显示名称
    pub display_name: String,
    /// 适配器 ID
    pub adapter_id: String,
    /// 是否已安装
    pub installed: bool,
    /// 是否选中接管（默认 true）
    pub selected: bool,
    /// User Scope 的三档分类结果
    pub user_scope_preview: Option<ScopeTakeoverPreview>,
    /// Project Scope 的三档分类结果
    pub project_scope_preview: Option<ScopeTakeoverPreview>,
    /// Local Scope 项目列表 (Claude Code 特有)
    /// Story 11.21: 支持 ~/.claude.json 中 projects.{path}.mcpServers 配置
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_scopes: Vec<LocalScopeScanResult>,
    /// 总服务数量
    pub total_service_count: usize,
    /// 需要决策的冲突数量
    pub conflict_count: usize,
}

/// 单个 Scope 的接管预览
///
/// Story 11.20: 全工具自动接管生成 - AC 3
///
/// 包含该 Scope 下的三档分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeTakeoverPreview {
    /// Scope 类型
    pub scope: TakeoverScope,
    /// 配置文件路径
    pub config_path: String,
    /// 配置文件是否存在
    pub exists: bool,
    /// 自动创建项
    pub auto_create: Vec<AutoCreateItem>,
    /// 自动跳过项
    pub auto_skip: Vec<AutoSkipItem>,
    /// 需要决策项
    pub needs_decision: Vec<ConflictDetail>,
    /// 服务数量
    pub service_count: usize,
}

impl ScopeTakeoverPreview {
    /// 创建空预览
    pub fn empty(scope: TakeoverScope, config_path: String) -> Self {
        Self {
            scope,
            config_path,
            exists: false,
            auto_create: Vec::new(),
            auto_skip: Vec::new(),
            needs_decision: Vec::new(),
            service_count: 0,
        }
    }

    /// 是否有需要决策的冲突
    pub fn has_conflicts(&self) -> bool {
        !self.needs_decision.is_empty()
    }

    /// 获取分类统计
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.auto_create.len(),
            self.auto_skip.len(),
            self.needs_decision.len(),
        )
    }
}

/// 全工具接管预览
///
/// Story 11.20: 全工具自动接管生成 - AC 3
///
/// 聚合所有工具的接管预览，用于前端展示
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullToolTakeoverPreview {
    /// 项目路径
    pub project_path: String,
    /// 各工具的接管预览
    pub tools: Vec<ToolTakeoverPreview>,
    /// 已安装工具数量
    pub installed_count: usize,
    /// 需要的环境变量列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars_needed: Vec<String>,
    /// 总服务数量
    pub total_service_count: usize,
    /// 总冲突数量
    pub total_conflict_count: usize,
    /// 是否可以一键执行（无冲突）
    pub can_auto_execute: bool,
}

impl FullToolTakeoverPreview {
    /// 创建空预览
    pub fn empty(project_path: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            tools: Vec::new(),
            installed_count: 0,
            env_vars_needed: Vec::new(),
            total_service_count: 0,
            total_conflict_count: 0,
            can_auto_execute: true,
        }
    }

    /// 是否有需要决策的冲突
    pub fn has_conflicts(&self) -> bool {
        self.total_conflict_count > 0
    }

    /// 获取选中的工具
    pub fn get_selected_tools(&self) -> Vec<&ToolTakeoverPreview> {
        self.tools.iter().filter(|t| t.selected).collect()
    }

    /// 计算汇总统计
    pub fn update_stats(&mut self) {
        self.installed_count = self.tools.iter().filter(|t| t.installed).count();
        self.total_service_count = self.tools.iter().map(|t| t.total_service_count).sum();
        self.total_conflict_count = self.tools.iter().map(|t| t.conflict_count).sum();
        self.can_auto_execute = self.total_conflict_count == 0;
    }
}

/// 接管状态
///
/// Story 11.15: MCP 接管流程重构 - AC 3, AC 5
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TakeoverStatus {
    /// 接管中（原配置已被替换）
    #[default]
    Active,
    /// 已恢复（原配置已被恢复）
    Restored,
}

impl TakeoverStatus {
    /// 转换为数据库存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            TakeoverStatus::Active => "active",
            TakeoverStatus::Restored => "restored",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(TakeoverStatus::Active),
            "restored" => Some(TakeoverStatus::Restored),
            _ => None,
        }
    }
}

/// 接管作用域
///
/// Story 11.16: 接管状态模块系统性重构 - AC 1, AC 2
/// Story 11.21: Claude Code Local Scope 完整支持 - AC 2
///
/// 区分用户级、项目级和 Local Scope 配置接管
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TakeoverScope {
    /// 用户级配置（如 ~/.claude.json 顶层 mcpServers）
    #[default]
    User,
    /// 项目级配置（如 project/.mcp.json 独立文件）
    Project,
    /// Local Scope 配置（~/.claude.json 中 projects.{path}.mcpServers）
    ///
    /// Story 11.21: Claude Code 的 local scope 配置存储在用户配置文件中，
    /// 但属于特定项目。需要单独备份和恢复。
    Local,
}

impl TakeoverScope {
    /// 转换为数据库存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            TakeoverScope::User => "user",
            TakeoverScope::Project => "project",
            TakeoverScope::Local => "local",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(TakeoverScope::User),
            "project" => Some(TakeoverScope::Project),
            "local" => Some(TakeoverScope::Local),
            _ => None,
        }
    }

    /// 检查是否需要 local_project_path 字段
    pub fn requires_project_path(&self) -> bool {
        matches!(self, TakeoverScope::Project | TakeoverScope::Local)
    }

    /// 检查是否是 local scope (Story 11.21)
    pub fn is_local(&self) -> bool {
        matches!(self, TakeoverScope::Local)
    }
}

/// 接管备份记录
///
/// Story 11.15: MCP 接管流程重构 - AC 3, AC 4, AC 5
/// Story 11.16: 接管状态模块系统性重构 - AC 1, AC 2
/// Story 11.21: Claude Code Local Scope 完整支持 - AC 2
///
/// 记录 MCP 配置接管的备份信息，支持一键恢复
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeoverBackup {
    /// 唯一标识符 (UUID)
    pub id: String,
    /// 工具类型
    pub tool_type: ToolType,
    /// 接管作用域 (Story 11.16, 11.21)
    /// - user: 用户级配置 (~/.claude.json 顶层 mcpServers)
    /// - project: 项目级配置 (project/.mcp.json)
    /// - local: Local Scope 配置 (~/.claude.json 中 projects.{path}.mcpServers)
    #[serde(default)]
    pub scope: TakeoverScope,
    /// 项目路径（project/local scope 时有值）(Story 11.16, 11.21)
    /// - project scope: 配置文件所在的项目目录
    /// - local scope: ~/.claude.json 中 projects 下的项目路径键
    pub project_path: Option<PathBuf>,
    /// 原始配置文件路径
    pub original_path: PathBuf,
    /// 备份文件路径
    /// - local scope: 备份的是该项目的 mcpServers JSON 片段
    pub backup_path: PathBuf,
    /// 接管时间 (ISO 8601)
    pub taken_over_at: String,
    /// 恢复时间 (ISO 8601)，如果未恢复则为 None
    pub restored_at: Option<String>,
    /// 接管状态
    pub status: TakeoverStatus,
}

impl TakeoverBackup {
    /// 创建新的备份记录
    pub fn new(tool_type: ToolType, original_path: PathBuf, backup_path: PathBuf) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_type,
            scope: TakeoverScope::User,
            project_path: None,
            original_path,
            backup_path,
            taken_over_at: chrono::Utc::now().to_rfc3339(),
            restored_at: None,
            status: TakeoverStatus::Active,
        }
    }

    /// 创建带作用域的备份记录 (Story 11.16)
    pub fn new_with_scope(
        tool_type: ToolType,
        original_path: PathBuf,
        backup_path: PathBuf,
        scope: TakeoverScope,
        project_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_type,
            scope,
            project_path,
            original_path,
            backup_path,
            taken_over_at: chrono::Utc::now().to_rfc3339(),
            restored_at: None,
            status: TakeoverStatus::Active,
        }
    }

    /// 检查备份文件是否存在
    pub fn backup_exists(&self) -> bool {
        self.backup_path.exists()
    }

    /// 检查是否可以恢复
    pub fn can_restore(&self) -> bool {
        self.status == TakeoverStatus::Active && self.backup_exists()
    }

    /// 检查是否是项目级接管 (Story 11.16)
    pub fn is_project_level(&self) -> bool {
        self.scope == TakeoverScope::Project
    }

    /// 检查是否是用户级接管 (Story 11.16)
    pub fn is_user_level(&self) -> bool {
        self.scope == TakeoverScope::User
    }

    /// 检查是否是 local scope 接管 (Story 11.21)
    ///
    /// Local scope 是 Claude Code 特有的概念，配置存储在 ~/.claude.json 的
    /// projects.{path}.mcpServers 中，属于特定项目但位于用户配置文件内。
    pub fn is_local_level(&self) -> bool {
        self.scope == TakeoverScope::Local
    }
}

// ===== Story 11.19: 智能接管合并引擎 =====

/// 合并分类 (Story 11.19)
///
/// 三档分类用于智能接管预览
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeClassification {
    /// 自动创建：全局池无此服务，将自动创建
    AutoCreate,
    /// 自动跳过：全局池有同名服务且配置完全一致，自动跳过
    AutoSkip,
    /// 需要决策：配置冲突或多 Scope 冲突
    NeedsDecision,
}

impl Default for MergeClassification {
    fn default() -> Self {
        MergeClassification::NeedsDecision
    }
}

/// 冲突类型 (Story 11.19)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// 配置差异：同名服务的配置不同
    ConfigDiff,
    /// Scope 冲突：同一服务名在 project + user 级都存在
    ScopeConflict,
    /// 多来源冲突：多个工具都有同名服务配置
    MultiSource,
}

/// 配置字段差异详情 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiffDetail {
    /// 差异字段名
    pub field: String,
    /// 现有值
    pub existing_value: Option<String>,
    /// 新值
    pub new_value: Option<String>,
}

/// 冲突候选项 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCandidate {
    /// 来源适配器 ID (claude/cursor/codex/gemini)
    pub adapter_id: String,
    /// 配置文件路径
    pub config_path: String,
    /// Scope (project/user)
    pub scope: TakeoverScope,
    /// 服务配置摘要
    pub config_summary: ServiceConfigSummary,
}

/// 服务配置摘要 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfigSummary {
    /// 传输类型
    pub transport_type: McpTransportType,
    /// 命令 (stdio 模式)
    pub command: Option<String>,
    /// 参数数量
    pub args_count: usize,
    /// 环境变量数量
    pub env_count: usize,
    /// URL (http 模式)
    pub url: Option<String>,
}

/// 冲突详情 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetail {
    /// 服务名称
    pub service_name: String,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 现有服务 (如有)
    pub existing_service: Option<McpServiceSummary>,
    /// 冲突候选项列表
    pub candidates: Vec<ConflictCandidate>,
    /// 配置差异详情 (仅 ConfigDiff 类型)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diff_details: Vec<ConfigDiffDetail>,
}

/// MCP 服务摘要 (Story 11.19)
///
/// 用于展示现有服务的简要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceSummary {
    /// 服务 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 来源适配器 ID
    pub source_adapter_id: Option<String>,
    /// 来源 Scope
    pub source_scope: Option<String>,
    /// 配置摘要
    pub config_summary: ServiceConfigSummary,
}

impl McpServiceSummary {
    /// 从 McpService 创建摘要
    pub fn from_service(service: &McpService) -> Self {
        let config_summary = ServiceConfigSummary {
            transport_type: service.transport_type.clone(),
            command: if service.command.is_empty() { None } else { Some(service.command.clone()) },
            args_count: service.args.as_ref().map_or(0, |a| a.len()),
            env_count: service.env.as_ref().map_or(0, |e| {
                e.as_object().map_or(0, |o| o.len())
            }),
            url: service.url.clone(),
        };

        Self {
            id: service.id.clone(),
            name: service.name.clone(),
            source_adapter_id: service.source_adapter_id.clone(),
            source_scope: service.source_scope.clone(),
            config_summary,
        }
    }
}

/// 自动跳过项 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSkipItem {
    /// 服务名称
    pub service_name: String,
    /// 检测到的适配器 ID
    pub detected_adapter_id: String,
    /// 检测到的配置文件路径
    pub detected_config_path: String,
    /// 检测到的 Scope
    pub detected_scope: TakeoverScope,
    /// 现有服务摘要
    pub existing_service: McpServiceSummary,
}

/// 自动创建项 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCreateItem {
    /// 服务名称
    pub service_name: String,
    /// 来源适配器 ID
    pub adapter_id: String,
    /// 配置文件路径
    pub config_path: String,
    /// Scope
    pub scope: TakeoverScope,
    /// 配置摘要
    pub config_summary: ServiceConfigSummary,
}

/// 用户决策选项 (Story 11.19)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TakeoverDecisionOption {
    /// 保留现有：跳过导入
    KeepExisting,
    /// 使用新配置：更新现有服务
    UseNew,
    /// 都保留：重命名新服务为 `{name}-{adapter_id}`
    KeepBoth,
    /// 使用 Project 级配置 (Scope 冲突专用)
    UseProjectScope,
    /// 使用 User 级配置 (Scope 冲突专用)
    UseUserScope,
}

/// 用户决策 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverDecision {
    /// 服务名称
    pub service_name: String,
    /// 选择的决策选项
    pub decision: TakeoverDecisionOption,
    /// 如果是 KeepBoth，选择的候选项索引
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_candidate_index: Option<usize>,
}

/// 智能接管预览结果 (Story 11.19)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverPreview {
    /// 项目路径
    pub project_path: String,
    /// 自动创建项：全局池无此服务，将自动创建
    pub auto_create: Vec<AutoCreateItem>,
    /// 自动跳过项：全局池有同名服务且配置完全一致
    pub auto_skip: Vec<AutoSkipItem>,
    /// 需要决策项：配置冲突或多 Scope 冲突
    pub needs_decision: Vec<ConflictDetail>,
    /// 需要的环境变量列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars_needed: Vec<String>,
    /// 总服务数
    pub total_services: usize,
}

impl TakeoverPreview {
    /// 创建空预览
    pub fn empty(project_path: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            auto_create: Vec::new(),
            auto_skip: Vec::new(),
            needs_decision: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 0,
        }
    }

    /// 检查是否有需要用户决策的冲突
    pub fn has_conflicts(&self) -> bool {
        !self.needs_decision.is_empty()
    }

    /// 检查是否可以一键执行（无冲突）
    pub fn can_auto_execute(&self) -> bool {
        self.needs_decision.is_empty()
    }

    /// 获取分类统计
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.auto_create.len(),
            self.auto_skip.len(),
            self.needs_decision.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_service_source_serialization() {
        let imported = McpServiceSource::Imported;
        let manual = McpServiceSource::Manual;

        assert_eq!(serde_json::to_string(&imported).unwrap(), r#""imported""#);
        assert_eq!(serde_json::to_string(&manual).unwrap(), r#""manual""#);
    }

    #[test]
    fn test_mcp_service_source_deserialization() {
        let imported: McpServiceSource = serde_json::from_str(r#""imported""#).unwrap();
        let manual: McpServiceSource = serde_json::from_str(r#""manual""#).unwrap();

        assert_eq!(imported, McpServiceSource::Imported);
        assert_eq!(manual, McpServiceSource::Manual);
    }

    #[test]
    fn test_mcp_service_source_as_str() {
        assert_eq!(McpServiceSource::Imported.as_str(), "imported");
        assert_eq!(McpServiceSource::Manual.as_str(), "manual");
    }

    #[test]
    fn test_mcp_service_source_from_str() {
        assert_eq!(
            McpServiceSource::from_str("imported"),
            Some(McpServiceSource::Imported)
        );
        assert_eq!(
            McpServiceSource::from_str("manual"),
            Some(McpServiceSource::Manual)
        );
        assert_eq!(McpServiceSource::from_str("unknown"), None);
    }

    #[test]
    fn test_mcp_service_serialization() {
        let service = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: Some(serde_json::json!({"DEBUG": "true"})),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let json = serde_json::to_string(&service).unwrap();
        let deserialized: McpService = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, service.id);
        assert_eq!(deserialized.name, service.name);
        assert_eq!(deserialized.transport_type, McpTransportType::Stdio);
        assert_eq!(deserialized.command, service.command);
        assert_eq!(deserialized.args, service.args);
        assert_eq!(deserialized.source, service.source);
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_mcp_service_http_type() {
        let service = McpService {
            id: "deepwiki-id".to_string(),
            name: "deepwiki".to_string(),
            transport_type: McpTransportType::Http,
            command: String::new(),
            args: None,
            env: None,
            url: Some("https://mcp.deepwiki.com/mcp".to_string()),
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some(".mcp.json".to_string()),
            source_adapter_id: Some("claude".to_string()),
            source_scope: Some("project".to_string()),
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let json = serde_json::to_string(&service).unwrap();
        assert!(json.contains("deepwiki"));
        assert!(json.contains("https://mcp.deepwiki.com/mcp"));
        assert!(json.contains(r#""transport_type":"http""#));

        let deserialized: McpService = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.transport_type, McpTransportType::Http);
        assert_eq!(deserialized.url, Some("https://mcp.deepwiki.com/mcp".to_string()));
    }

    #[test]
    fn test_create_mcp_service_request() {
        let request = CreateMcpServiceRequest {
            name: "filesystem".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/filesystem-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/home/user/.mcp.json".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("filesystem"));
        assert!(json.contains("imported"));
    }

    #[test]
    fn test_create_mcp_service_request_http() {
        let request = CreateMcpServiceRequest {
            name: "deepwiki".to_string(),
            transport_type: McpTransportType::Http,
            command: String::new(),
            args: None,
            env: None,
            url: Some("https://mcp.deepwiki.com/mcp".to_string()),
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some(".mcp.json".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("deepwiki"));
        assert!(json.contains("https://mcp.deepwiki.com/mcp"));
    }

    #[test]
    fn test_update_mcp_service_request_partial() {
        let request = UpdateMcpServiceRequest {
            name: Some("new-name".to_string()),
            ..Default::default()
        };

        assert!(request.name.is_some());
        assert!(request.command.is_none());
        assert!(request.args.is_none());
        assert!(request.env.is_none());
        assert!(request.enabled.is_none());
    }

    #[test]
    fn test_project_mcp_service() {
        let link = ProjectMcpService {
            project_id: "project-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({"args": ["--custom"]})),
            detected_adapter_id: Some("claude".to_string()),
            detected_config_path: Some("/project/.mcp.json".to_string()),
            created_at: "2026-01-30T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("project-123"));
        assert!(json.contains("service-456"));
        assert!(json.contains("--custom"));
        assert!(json.contains("claude"));
        assert!(json.contains("/project/.mcp.json"));
    }

    #[test]
    fn test_env_variable() {
        let env_var = EnvVariable {
            id: "env-123".to_string(),
            name: "OPENAI_API_KEY".to_string(),
            masked_value: "sk-****...****xyz".to_string(),
            description: Some("OpenAI API Key".to_string()),
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&env_var).unwrap();
        assert!(json.contains("OPENAI_API_KEY"));
        assert!(json.contains("sk-****...****xyz"));
    }

    #[test]
    fn test_env_variable_name_validation() {
        let valid = EnvVariableNameValidation {
            is_valid: true,
            suggestion: None,
            error_message: None,
        };
        assert!(valid.is_valid);
        assert!(valid.suggestion.is_none());

        let invalid = EnvVariableNameValidation {
            is_valid: false,
            suggestion: Some("OPENAI_API_KEY".to_string()),
            error_message: Some("Name must be in SCREAMING_SNAKE_CASE format".to_string()),
        };
        assert!(!invalid.is_valid);
        assert_eq!(invalid.suggestion, Some("OPENAI_API_KEY".to_string()));
    }

    #[test]
    fn test_mcp_service_with_override() {
        let service = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let with_override = McpServiceWithOverride {
            service,
            config_override: Some(serde_json::json!({"args": ["--verbose"]})),
            detected_adapter_id: None,
            detected_config_path: None,
        };

        let json = serde_json::to_string(&with_override).unwrap();
        // 由于 #[serde(flatten)]，service 字段会被展开
        assert!(json.contains("git-mcp"));
        assert!(json.contains("--verbose"));
    }

    #[test]
    fn test_transport_type_serialization() {
        assert_eq!(serde_json::to_string(&McpTransportType::Stdio).unwrap(), r#""stdio""#);
        assert_eq!(serde_json::to_string(&McpTransportType::Http).unwrap(), r#""http""#);

        let stdio: McpTransportType = serde_json::from_str(r#""stdio""#).unwrap();
        assert_eq!(stdio, McpTransportType::Stdio);
        let http: McpTransportType = serde_json::from_str(r#""http""#).unwrap();
        assert_eq!(http, McpTransportType::Http);
    }

    #[test]
    fn test_transport_type_default() {
        let default = McpTransportType::default();
        assert_eq!(default, McpTransportType::Stdio);
    }

    #[test]
    fn test_transport_type_as_str() {
        assert_eq!(McpTransportType::Stdio.as_str(), "stdio");
        assert_eq!(McpTransportType::Http.as_str(), "http");
    }

    #[test]
    fn test_transport_type_from_str() {
        assert_eq!(McpTransportType::from_str("stdio"), Some(McpTransportType::Stdio));
        assert_eq!(McpTransportType::from_str("http"), Some(McpTransportType::Http));
        assert_eq!(McpTransportType::from_str("unknown"), None);
    }

    // ===== Story 11.18: 简化 Tool Policy 测试 =====

    #[test]
    fn test_tool_policy_default() {
        let policy = ToolPolicy::default();
        assert!(policy.is_allow_all());
        assert!(!policy.is_inherit());
        assert!(!policy.is_custom());
        assert_eq!(policy.allowed_tools, Some(vec![]));
    }

    #[test]
    fn test_tool_policy_allow_all() {
        let policy = ToolPolicy::allow_all();
        assert!(policy.is_allow_all());
        assert!(policy.is_tool_allowed("any_tool"));
        assert!(policy.is_tool_allowed("another_tool"));
    }

    #[test]
    fn test_tool_policy_inherit() {
        let policy = ToolPolicy::inherit();
        assert!(policy.is_inherit());
        assert!(!policy.is_allow_all());
        assert!(!policy.is_custom());
        // 继承模式下默认允许（实际继承由 PolicyResolver 处理）
        assert!(policy.is_tool_allowed("any_tool"));
    }

    #[test]
    fn test_tool_policy_custom() {
        let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);
        assert!(policy.is_custom());
        assert!(!policy.is_allow_all());
        assert!(!policy.is_inherit());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("list_commits"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_tool_policy_serialization_new_format() {
        // 全选
        let allow_all = ToolPolicy::allow_all();
        let json = serde_json::to_string(&allow_all).unwrap();
        assert!(json.contains(r#""allowedTools":[]"#));
        assert!(!json.contains("mode")); // mode 不再序列化
        assert!(!json.contains("deniedTools")); // deniedTools 不再序列化

        // 部分选
        let custom = ToolPolicy::custom(vec!["read_file".to_string()]);
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("read_file"));
        assert!(!json.contains("mode"));

        // 继承
        let inherit = ToolPolicy::inherit();
        let json = serde_json::to_string(&inherit).unwrap();
        assert!(json.contains(r#""allowedTools":null"#));
    }

    #[test]
    fn test_tool_policy_backward_compat_deserialization() {
        // 旧格式: allow_all
        let old_json = r#"{"mode":"allow_all","allowedTools":[],"deniedTools":[]}"#;
        let policy: ToolPolicy = serde_json::from_str(old_json).unwrap();
        assert!(policy.is_allow_all());

        // 旧格式: custom
        let old_json = r#"{"mode":"custom","allowedTools":["read_file","list_commits"],"deniedTools":["write_file"]}"#;
        let policy: ToolPolicy = serde_json::from_str(old_json).unwrap();
        // 新模型忽略 mode 和 deniedTools，只看 allowedTools
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("list_commits"));
        // write_file 在 deniedTools 中，但新模型不再使用 deniedTools
        assert!(!policy.is_tool_allowed("write_file")); // 不在 allowedTools 中所以不允许

        // 新格式: 只有 allowedTools
        let new_json = r#"{"allowedTools":["read_file"]}"#;
        let policy: ToolPolicy = serde_json::from_str(new_json).unwrap();
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_tool_policy_from_config_override() {
        // 新格式
        let config_override = serde_json::json!({
            "toolPolicy": {
                "allowedTools": ["read_file", "list_commits"]
            }
        });

        let tool_policy_value = config_override.get("toolPolicy").unwrap();
        let policy: ToolPolicy = serde_json::from_value(tool_policy_value.clone()).unwrap();

        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("list_commits"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_tool_policy_migrate_from_legacy() {
        // deny_all → None (删除关联)
        let deny_all = serde_json::json!({"mode": "deny_all", "allowedTools": [], "deniedTools": []});
        assert!(ToolPolicy::migrate_from_legacy(&deny_all).is_none());

        // allow_all → 全选
        let allow_all = serde_json::json!({"mode": "allow_all", "allowedTools": [], "deniedTools": []});
        let migrated = ToolPolicy::migrate_from_legacy(&allow_all).unwrap();
        assert!(migrated.is_allow_all());

        // custom → 保留 allowedTools
        let custom = serde_json::json!({"mode": "custom", "allowedTools": ["read_file", "list"], "deniedTools": ["write"]});
        let migrated = ToolPolicy::migrate_from_legacy(&custom).unwrap();
        assert!(migrated.is_custom());
        assert_eq!(migrated.allowed_tools, Some(vec!["read_file".to_string(), "list".to_string()]));
    }

    #[test]
    fn test_tool_policy_filter_tools() {
        let tools = vec!["read_file", "write_file", "list_commits", "execute"];
        let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);

        let filtered = policy.filter_tools(&tools, |t| t);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&&"read_file"));
        assert!(filtered.contains(&&"list_commits"));
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_none() {
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: None,
            detected_adapter_id: None,
            detected_config_path: None,
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        // Story 11.18: 无配置时返回继承策略 (inherit)，以回退到服务默认
        let policy = service.get_tool_policy();
        assert!(policy.is_inherit());
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_with_override() {
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({
                "toolPolicy": {
                    "allowedTools": ["read_file"]
                }
            })),
            detected_adapter_id: None,
            detected_config_path: None,
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        let policy = service.get_tool_policy();
        assert!(policy.is_custom());
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_invalid_json() {
        // Story 11.18: 如果 toolPolicy 格式无效，返回继承策略 (inherit)
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({
                "toolPolicy": "invalid_not_an_object"
            })),
            detected_adapter_id: None,
            detected_config_path: None,
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        let policy = service.get_tool_policy();
        assert!(policy.is_inherit());
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_inherit() {
        // Story 11.18: 测试继承模式
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({
                "toolPolicy": {
                    "allowedTools": null
                }
            })),
            detected_adapter_id: None,
            detected_config_path: None,
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        let policy = service.get_tool_policy();
        assert!(policy.is_inherit());
    }

    // ===== Story 11.15: TakeoverBackup 模型测试 =====

    #[test]
    fn test_tool_type_serialization() {
        assert_eq!(serde_json::to_string(&ToolType::ClaudeCode).unwrap(), r#""claude_code""#);
        assert_eq!(serde_json::to_string(&ToolType::Cursor).unwrap(), r#""cursor""#);
        assert_eq!(serde_json::to_string(&ToolType::Codex).unwrap(), r#""codex""#);
        assert_eq!(serde_json::to_string(&ToolType::GeminiCli).unwrap(), r#""gemini_cli""#);
    }

    #[test]
    fn test_tool_type_deserialization() {
        let claude: ToolType = serde_json::from_str(r#""claude_code""#).unwrap();
        assert_eq!(claude, ToolType::ClaudeCode);
        let cursor: ToolType = serde_json::from_str(r#""cursor""#).unwrap();
        assert_eq!(cursor, ToolType::Cursor);
        let codex: ToolType = serde_json::from_str(r#""codex""#).unwrap();
        assert_eq!(codex, ToolType::Codex);
        let gemini: ToolType = serde_json::from_str(r#""gemini_cli""#).unwrap();
        assert_eq!(gemini, ToolType::GeminiCli);
    }

    #[test]
    fn test_tool_type_as_str() {
        assert_eq!(ToolType::ClaudeCode.as_str(), "claude_code");
        assert_eq!(ToolType::Cursor.as_str(), "cursor");
        assert_eq!(ToolType::Codex.as_str(), "codex");
        assert_eq!(ToolType::GeminiCli.as_str(), "gemini_cli");
    }

    #[test]
    fn test_tool_type_from_str() {
        assert_eq!(ToolType::from_str("claude_code"), Some(ToolType::ClaudeCode));
        assert_eq!(ToolType::from_str("cursor"), Some(ToolType::Cursor));
        assert_eq!(ToolType::from_str("codex"), Some(ToolType::Codex));
        assert_eq!(ToolType::from_str("gemini_cli"), Some(ToolType::GeminiCli));
        assert_eq!(ToolType::from_str("unknown"), None);
    }

    #[test]
    fn test_tool_type_from_adapter_id() {
        assert_eq!(ToolType::from_adapter_id("claude"), Some(ToolType::ClaudeCode));
        assert_eq!(ToolType::from_adapter_id("cursor"), Some(ToolType::Cursor));
        assert_eq!(ToolType::from_adapter_id("codex"), Some(ToolType::Codex));
        assert_eq!(ToolType::from_adapter_id("gemini"), Some(ToolType::GeminiCli));
        assert_eq!(ToolType::from_adapter_id("unknown"), None);
    }

    #[test]
    fn test_tool_type_display_name() {
        assert_eq!(ToolType::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(ToolType::Cursor.display_name(), "Cursor");
        assert_eq!(ToolType::Codex.display_name(), "Codex");
        assert_eq!(ToolType::GeminiCli.display_name(), "Gemini CLI");
    }

    #[test]
    fn test_tool_type_user_config_path() {
        // 测试路径包含正确的文件名
        let claude_path = ToolType::ClaudeCode.get_user_config_path();
        assert!(claude_path.to_string_lossy().ends_with(".claude.json"));

        let cursor_path = ToolType::Cursor.get_user_config_path();
        assert!(cursor_path.to_string_lossy().contains(".cursor"));
        assert!(cursor_path.to_string_lossy().ends_with("mcp.json"));

        let codex_path = ToolType::Codex.get_user_config_path();
        assert!(codex_path.to_string_lossy().contains(".codex"));
        assert!(codex_path.to_string_lossy().ends_with("config.toml"));

        let gemini_path = ToolType::GeminiCli.get_user_config_path();
        assert!(gemini_path.to_string_lossy().contains(".gemini"));
        assert!(gemini_path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_takeover_status_serialization() {
        assert_eq!(serde_json::to_string(&TakeoverStatus::Active).unwrap(), r#""active""#);
        assert_eq!(serde_json::to_string(&TakeoverStatus::Restored).unwrap(), r#""restored""#);
    }

    #[test]
    fn test_takeover_status_deserialization() {
        let active: TakeoverStatus = serde_json::from_str(r#""active""#).unwrap();
        assert_eq!(active, TakeoverStatus::Active);
        let restored: TakeoverStatus = serde_json::from_str(r#""restored""#).unwrap();
        assert_eq!(restored, TakeoverStatus::Restored);
    }

    #[test]
    fn test_takeover_status_as_str() {
        assert_eq!(TakeoverStatus::Active.as_str(), "active");
        assert_eq!(TakeoverStatus::Restored.as_str(), "restored");
    }

    #[test]
    fn test_takeover_status_from_str() {
        assert_eq!(TakeoverStatus::from_str("active"), Some(TakeoverStatus::Active));
        assert_eq!(TakeoverStatus::from_str("restored"), Some(TakeoverStatus::Restored));
        assert_eq!(TakeoverStatus::from_str("unknown"), None);
    }

    #[test]
    fn test_takeover_status_default() {
        let status = TakeoverStatus::default();
        assert_eq!(status, TakeoverStatus::Active);
    }

    #[test]
    fn test_takeover_backup_new() {
        let backup = TakeoverBackup::new(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"),
        );

        assert!(!backup.id.is_empty());
        assert_eq!(backup.tool_type, ToolType::ClaudeCode);
        assert_eq!(backup.original_path, PathBuf::from("/home/user/.claude.json"));
        assert_eq!(backup.backup_path, PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"));
        assert!(backup.restored_at.is_none());
        assert_eq!(backup.status, TakeoverStatus::Active);
    }

    #[test]
    fn test_takeover_backup_serialization() {
        let backup = TakeoverBackup {
            id: "backup-123".to_string(),
            tool_type: ToolType::Cursor,
            scope: TakeoverScope::User,
            project_path: None,
            original_path: PathBuf::from("/home/user/.cursor/mcp.json"),
            backup_path: PathBuf::from("/home/user/.cursor/mcp.json.mantra-backup.20260201"),
            taken_over_at: "2026-02-01T10:00:00Z".to_string(),
            restored_at: None,
            status: TakeoverStatus::Active,
        };

        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains("backup-123"));
        assert!(json.contains("cursor"));
        assert!(json.contains("takenOverAt")); // camelCase
        assert!(json.contains("active"));

        let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, backup.id);
        assert_eq!(deserialized.tool_type, backup.tool_type);
        assert_eq!(deserialized.status, TakeoverStatus::Active);
    }

    #[test]
    fn test_takeover_backup_can_restore() {
        // Active 状态但备份文件不存在
        let backup = TakeoverBackup {
            id: "backup-123".to_string(),
            tool_type: ToolType::ClaudeCode,
            scope: TakeoverScope::User,
            project_path: None,
            original_path: PathBuf::from("/nonexistent/original.json"),
            backup_path: PathBuf::from("/nonexistent/backup.json"),
            taken_over_at: "2026-02-01T10:00:00Z".to_string(),
            restored_at: None,
            status: TakeoverStatus::Active,
        };
        assert!(!backup.can_restore()); // 文件不存在

        // Restored 状态
        let restored_backup = TakeoverBackup {
            id: "backup-456".to_string(),
            tool_type: ToolType::Cursor,
            scope: TakeoverScope::User,
            project_path: None,
            original_path: PathBuf::from("/home/user/.cursor/mcp.json"),
            backup_path: PathBuf::from("/home/user/.cursor/mcp.json.backup"),
            taken_over_at: "2026-02-01T10:00:00Z".to_string(),
            restored_at: Some("2026-02-01T12:00:00Z".to_string()),
            status: TakeoverStatus::Restored,
        };
        assert!(!restored_backup.can_restore()); // 已恢复
    }

    // ===== Story 11.16: TakeoverScope 模型测试 =====

    #[test]
    fn test_takeover_scope_serialization() {
        assert_eq!(serde_json::to_string(&TakeoverScope::User).unwrap(), r#""user""#);
        assert_eq!(serde_json::to_string(&TakeoverScope::Project).unwrap(), r#""project""#);
    }

    #[test]
    fn test_takeover_scope_deserialization() {
        let user: TakeoverScope = serde_json::from_str(r#""user""#).unwrap();
        assert_eq!(user, TakeoverScope::User);
        let project: TakeoverScope = serde_json::from_str(r#""project""#).unwrap();
        assert_eq!(project, TakeoverScope::Project);
    }

    #[test]
    fn test_takeover_scope_as_str() {
        assert_eq!(TakeoverScope::User.as_str(), "user");
        assert_eq!(TakeoverScope::Project.as_str(), "project");
    }

    #[test]
    fn test_takeover_scope_from_str() {
        assert_eq!(TakeoverScope::from_str("user"), Some(TakeoverScope::User));
        assert_eq!(TakeoverScope::from_str("project"), Some(TakeoverScope::Project));
        assert_eq!(TakeoverScope::from_str("unknown"), None);
    }

    #[test]
    fn test_takeover_scope_default() {
        let scope = TakeoverScope::default();
        assert_eq!(scope, TakeoverScope::User);
    }

    #[test]
    fn test_takeover_backup_new_default_scope() {
        let backup = TakeoverBackup::new(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from("/home/user/.claude.json.backup"),
        );
        assert_eq!(backup.scope, TakeoverScope::User);
        assert!(backup.project_path.is_none());
        assert!(backup.is_user_level());
        assert!(!backup.is_project_level());
    }

    #[test]
    fn test_takeover_backup_new_with_scope_user() {
        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from("/home/user/.claude.json.backup"),
            TakeoverScope::User,
            None,
        );
        assert_eq!(backup.scope, TakeoverScope::User);
        assert!(backup.project_path.is_none());
        assert!(backup.is_user_level());
    }

    #[test]
    fn test_takeover_backup_new_with_scope_project() {
        let project_path = PathBuf::from("/home/user/my-project");
        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/my-project/.mcp.json"),
            PathBuf::from("/home/user/my-project/.mcp.json.backup"),
            TakeoverScope::Project,
            Some(project_path.clone()),
        );
        assert_eq!(backup.scope, TakeoverScope::Project);
        assert_eq!(backup.project_path, Some(project_path));
        assert!(backup.is_project_level());
        assert!(!backup.is_user_level());
    }

    #[test]
    fn test_takeover_backup_serialization_with_scope() {
        let backup = TakeoverBackup {
            id: "backup-789".to_string(),
            tool_type: ToolType::ClaudeCode,
            scope: TakeoverScope::Project,
            project_path: Some(PathBuf::from("/home/user/my-project")),
            original_path: PathBuf::from("/home/user/my-project/.mcp.json"),
            backup_path: PathBuf::from("/home/user/my-project/.mcp.json.backup"),
            taken_over_at: "2026-02-01T10:00:00Z".to_string(),
            restored_at: None,
            status: TakeoverStatus::Active,
        };

        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains(r#""scope":"project""#));
        assert!(json.contains("projectPath"));
        assert!(json.contains("my-project"));

        let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scope, TakeoverScope::Project);
        assert_eq!(deserialized.project_path, Some(PathBuf::from("/home/user/my-project")));
    }

    // ===== Story 11.21: Local Scope 支持 =====

    #[test]
    fn test_takeover_scope_local_as_str() {
        assert_eq!(TakeoverScope::Local.as_str(), "local");
    }

    #[test]
    fn test_takeover_scope_local_from_str() {
        assert_eq!(TakeoverScope::from_str("local"), Some(TakeoverScope::Local));
        assert_eq!(TakeoverScope::from_str("user"), Some(TakeoverScope::User));
        assert_eq!(TakeoverScope::from_str("project"), Some(TakeoverScope::Project));
        assert_eq!(TakeoverScope::from_str("invalid"), None);
    }

    #[test]
    fn test_takeover_scope_requires_project_path() {
        assert!(!TakeoverScope::User.requires_project_path());
        assert!(TakeoverScope::Project.requires_project_path());
        assert!(TakeoverScope::Local.requires_project_path());
    }

    #[test]
    fn test_takeover_scope_is_local() {
        assert!(!TakeoverScope::User.is_local());
        assert!(!TakeoverScope::Project.is_local());
        assert!(TakeoverScope::Local.is_local());
    }

    #[test]
    fn test_takeover_scope_local_serialization() {
        let scope = TakeoverScope::Local;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, r#""local""#);

        let deserialized: TakeoverScope = serde_json::from_str(r#""local""#).unwrap();
        assert_eq!(deserialized, TakeoverScope::Local);
    }

    #[test]
    fn test_takeover_backup_local_scope() {
        let backup = TakeoverBackup {
            id: "backup-local-123".to_string(),
            tool_type: ToolType::ClaudeCode,
            scope: TakeoverScope::Local,
            project_path: Some(PathBuf::from("/home/user/project-a")),
            original_path: PathBuf::from("/home/user/.claude.json"),
            backup_path: PathBuf::from("/home/user/.mantra/backups/project-a-local.backup"),
            taken_over_at: "2026-02-03T10:00:00Z".to_string(),
            restored_at: None,
            status: TakeoverStatus::Active,
        };

        // 序列化测试
        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains(r#""scope":"local""#));
        assert!(json.contains("projectPath"));
        assert!(json.contains("project-a"));
        assert!(json.contains(".claude.json"));

        // 反序列化测试
        let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scope, TakeoverScope::Local);
        assert_eq!(deserialized.project_path, Some(PathBuf::from("/home/user/project-a")));
        assert_eq!(deserialized.original_path, PathBuf::from("/home/user/.claude.json"));

        // 方法测试
        assert!(deserialized.is_local_level());
        assert!(!deserialized.is_user_level());
        assert!(!deserialized.is_project_level());
    }

    #[test]
    fn test_takeover_backup_new_with_scope_local() {
        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from("/home/user/.mantra/backups/project-b-local.backup"),
            TakeoverScope::Local,
            Some(PathBuf::from("/home/user/project-b")),
        );

        assert_eq!(backup.tool_type, ToolType::ClaudeCode);
        assert_eq!(backup.scope, TakeoverScope::Local);
        assert_eq!(backup.project_path, Some(PathBuf::from("/home/user/project-b")));
        assert!(backup.is_local_level());
        assert_eq!(backup.status, TakeoverStatus::Active);
    }
}
