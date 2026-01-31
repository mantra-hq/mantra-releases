//! MCP 服务数据模型
//!
//! Story 11.2: MCP 服务数据模型 - Task 1
//!
//! 定义 MCP 服务配置、项目关联和环境变量的数据结构

use serde::{Deserialize, Serialize};

/// MCP 服务参数类型别名
pub type McpServiceArgs = Vec<String>;

/// MCP 服务环境变量类型别名
pub type McpServiceEnv = serde_json::Value;

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
    /// 启动命令，如 "npx"、"uvx"
    pub command: String,
    /// 命令参数，JSON 数组格式
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用，JSON 对象格式
    /// 值可以是字面量或变量引用（如 "$OPENAI_API_KEY"）
    pub env: Option<McpServiceEnv>,
    /// 服务来源
    pub source: McpServiceSource,
    /// 导入来源的原始配置文件路径
    pub source_file: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间 (ISO 8601)
    pub created_at: String,
    /// 更新时间 (ISO 8601)
    pub updated_at: String,
}

/// 创建 MCP 服务的请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMcpServiceRequest {
    /// 服务名称
    pub name: String,
    /// 启动命令
    pub command: String,
    /// 命令参数
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用
    pub env: Option<McpServiceEnv>,
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
    /// 启动命令
    pub command: Option<String>,
    /// 命令参数
    pub args: Option<McpServiceArgs>,
    /// 环境变量引用
    pub env: Option<McpServiceEnv>,
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
    /// 创建时间 (ISO 8601)
    pub created_at: String,
}

impl ProjectMcpService {
    /// 获取项目的 Tool Policy
    ///
    /// Story 11.10: Project-Level Tool Management - AC 1, AC 6
    ///
    /// 从 `config_override.toolPolicy` 解析 Tool Policy。
    /// 如果未配置或解析失败，返回默认策略 (AllowAll)。
    pub fn get_tool_policy(&self) -> ToolPolicy {
        self.config_override
            .as_ref()
            .and_then(|config| config.get("toolPolicy"))
            .and_then(|policy_value| serde_json::from_value(policy_value.clone()).ok())
            .unwrap_or_default()
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceWithOverride {
    /// 服务配置
    #[serde(flatten)]
    pub service: McpService,
    /// 项目级配置覆盖
    pub config_override: Option<serde_json::Value>,
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

// ===== Story 11.10: Project-Level Tool Management =====

/// Tool Policy 模式
///
/// Story 11.10: Project-Level Tool Management - AC 1
///
/// 定义工具策略的三种模式:
/// - `allow_all`: 允许所有工具（默认）
/// - `deny_all`: 禁止所有工具
/// - `custom`: 自定义模式，需配合 allowed_tools 和 denied_tools 使用
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
/// Story 11.10: Project-Level Tool Management - AC 1
///
/// 用于控制项目级别的 MCP 工具访问权限。
///
/// ## 优先级规则
/// `denied_tools` > `allowed_tools` > `mode`
///
/// 即:
/// 1. 如果工具在 `denied_tools` 中，无论其他设置如何都被禁止
/// 2. 当 `mode = custom` 时，工具必须在 `allowed_tools` 中且不在 `denied_tools` 中才可用
/// 3. 当 `mode = allow_all` 时，允许所有不在 `denied_tools` 中的工具
/// 4. 当 `mode = deny_all` 时，禁止所有工具
///
/// ## 示例
/// ```json
/// {
///   "toolPolicy": {
///     "mode": "custom",
///     "allowedTools": ["git-mcp/read_file", "git-mcp/list_commits"],
///     "deniedTools": ["git-mcp/write_file", "git-mcp/execute_command"]
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolPolicy {
    /// 策略模式
    #[serde(default)]
    pub mode: ToolPolicyMode,
    /// 允许的工具列表（仅在 Custom 模式下生效）
    /// 格式: "tool_name" (不含 service 前缀)
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// 禁止的工具列表（优先级最高）
    /// 格式: "tool_name" (不含 service 前缀)
    #[serde(default)]
    pub denied_tools: Vec<String>,
}

impl ToolPolicy {
    /// 检查工具是否被允许
    ///
    /// ## 优先级规则
    /// 1. `denied_tools` 优先级最高 - 任何在此列表中的工具都被禁止
    /// 2. 根据 `mode` 判断:
    ///    - `AllowAll`: 允许所有工具
    ///    - `DenyAll`: 禁止所有工具
    ///    - `Custom`: 仅允许在 `allowed_tools` 中的工具
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        // 1. denied_tools 优先级最高
        if self.denied_tools.iter().any(|t| t == tool_name) {
            return false;
        }

        // 2. 根据 mode 判断
        match self.mode {
            ToolPolicyMode::AllowAll => true,
            ToolPolicyMode::DenyAll => false,
            ToolPolicyMode::Custom => self.allowed_tools.iter().any(|t| t == tool_name),
        }
    }

    /// 过滤工具列表，返回被允许的工具
    pub fn filter_tools<'a, T>(&self, tools: &'a [T], get_name: impl Fn(&T) -> &str) -> Vec<&'a T> {
        tools
            .iter()
            .filter(|tool| self.is_tool_allowed(get_name(tool)))
            .collect()
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
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: Some(serde_json::json!({"DEBUG": "true"})),
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&service).unwrap();
        let deserialized: McpService = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, service.id);
        assert_eq!(deserialized.name, service.name);
        assert_eq!(deserialized.command, service.command);
        assert_eq!(deserialized.args, service.args);
        assert_eq!(deserialized.source, service.source);
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_create_mcp_service_request() {
        let request = CreateMcpServiceRequest {
            name: "filesystem".to_string(),
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/filesystem-mcp".to_string()]),
            env: None,
            source: McpServiceSource::Imported,
            source_file: Some("/home/user/.mcp.json".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("filesystem"));
        assert!(json.contains("imported"));
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
            created_at: "2026-01-30T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("project-123"));
        assert!(json.contains("service-456"));
        assert!(json.contains("--custom"));
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
            command: "npx".to_string(),
            args: None,
            env: None,
            source: McpServiceSource::Manual,
            source_file: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
        };

        let with_override = McpServiceWithOverride {
            service,
            config_override: Some(serde_json::json!({"args": ["--verbose"]})),
        };

        let json = serde_json::to_string(&with_override).unwrap();
        // 由于 #[serde(flatten)]，service 字段会被展开
        assert!(json.contains("git-mcp"));
        assert!(json.contains("--verbose"));
    }

    // ===== Story 11.10: Tool Policy 测试 =====

    #[test]
    fn test_tool_policy_mode_serialization() {
        // 测试序列化
        assert_eq!(
            serde_json::to_string(&ToolPolicyMode::AllowAll).unwrap(),
            r#""allow_all""#
        );
        assert_eq!(
            serde_json::to_string(&ToolPolicyMode::DenyAll).unwrap(),
            r#""deny_all""#
        );
        assert_eq!(
            serde_json::to_string(&ToolPolicyMode::Custom).unwrap(),
            r#""custom""#
        );

        // 测试反序列化
        let allow_all: ToolPolicyMode = serde_json::from_str(r#""allow_all""#).unwrap();
        assert_eq!(allow_all, ToolPolicyMode::AllowAll);
        let deny_all: ToolPolicyMode = serde_json::from_str(r#""deny_all""#).unwrap();
        assert_eq!(deny_all, ToolPolicyMode::DenyAll);
        let custom: ToolPolicyMode = serde_json::from_str(r#""custom""#).unwrap();
        assert_eq!(custom, ToolPolicyMode::Custom);
    }

    #[test]
    fn test_tool_policy_default() {
        let policy = ToolPolicy::default();
        assert_eq!(policy.mode, ToolPolicyMode::AllowAll);
        assert!(policy.allowed_tools.is_empty());
        assert!(policy.denied_tools.is_empty());
    }

    #[test]
    fn test_tool_policy_is_tool_allowed_allow_all_mode() {
        let policy = ToolPolicy {
            mode: ToolPolicyMode::AllowAll,
            allowed_tools: vec![],
            denied_tools: vec![],
        };

        // AllowAll 模式下，所有工具都被允许
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("write_file"));
        assert!(policy.is_tool_allowed("execute_command"));
    }

    #[test]
    fn test_tool_policy_is_tool_allowed_deny_all_mode() {
        let policy = ToolPolicy {
            mode: ToolPolicyMode::DenyAll,
            allowed_tools: vec![],
            denied_tools: vec![],
        };

        // DenyAll 模式下，所有工具都被禁止
        assert!(!policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
        assert!(!policy.is_tool_allowed("execute_command"));
    }

    #[test]
    fn test_tool_policy_is_tool_allowed_custom_mode() {
        let policy = ToolPolicy {
            mode: ToolPolicyMode::Custom,
            allowed_tools: vec!["read_file".to_string(), "list_commits".to_string()],
            denied_tools: vec![],
        };

        // Custom 模式下，只有 allowed_tools 中的工具被允许
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("list_commits"));
        assert!(!policy.is_tool_allowed("write_file"));
        assert!(!policy.is_tool_allowed("execute_command"));
    }

    #[test]
    fn test_tool_policy_denied_tools_highest_priority() {
        // denied_tools 优先级最高，即使在 AllowAll 模式下也应该被禁止
        let policy = ToolPolicy {
            mode: ToolPolicyMode::AllowAll,
            allowed_tools: vec![],
            denied_tools: vec!["write_file".to_string(), "execute_command".to_string()],
        };

        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
        assert!(!policy.is_tool_allowed("execute_command"));
    }

    #[test]
    fn test_tool_policy_denied_overrides_allowed() {
        // denied_tools 优先于 allowed_tools
        let policy = ToolPolicy {
            mode: ToolPolicyMode::Custom,
            allowed_tools: vec![
                "read_file".to_string(),
                "write_file".to_string(),
                "execute_command".to_string(),
            ],
            denied_tools: vec!["write_file".to_string()],
        };

        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file")); // denied 优先
        assert!(policy.is_tool_allowed("execute_command"));
    }

    #[test]
    fn test_tool_policy_serialization() {
        let policy = ToolPolicy {
            mode: ToolPolicyMode::Custom,
            allowed_tools: vec!["read_file".to_string(), "list_commits".to_string()],
            denied_tools: vec!["write_file".to_string()],
        };

        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("custom"));
        assert!(json.contains("read_file"));
        assert!(json.contains("write_file"));

        let deserialized: ToolPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mode, ToolPolicyMode::Custom);
        assert_eq!(deserialized.allowed_tools.len(), 2);
        assert_eq!(deserialized.denied_tools.len(), 1);
    }

    #[test]
    fn test_tool_policy_from_config_override() {
        // 测试从 config_override JSON 中解析 ToolPolicy
        let config_override = serde_json::json!({
            "toolPolicy": {
                "mode": "custom",
                "allowedTools": ["read_file", "list_commits"],
                "deniedTools": ["write_file"]
            }
        });

        let tool_policy_value = config_override.get("toolPolicy").unwrap();
        let policy: ToolPolicy = serde_json::from_value(tool_policy_value.clone()).unwrap();

        assert_eq!(policy.mode, ToolPolicyMode::Custom);
        assert!(policy.is_tool_allowed("read_file"));
        assert!(policy.is_tool_allowed("list_commits"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_tool_policy_missing_fields_use_defaults() {
        // 测试缺少字段时使用默认值
        let partial_json = r#"{"mode": "custom"}"#;
        let policy: ToolPolicy = serde_json::from_str(partial_json).unwrap();

        assert_eq!(policy.mode, ToolPolicyMode::Custom);
        assert!(policy.allowed_tools.is_empty());
        assert!(policy.denied_tools.is_empty());
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_none() {
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: None,
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        // 无配置时返回默认策略 (AllowAll)
        let policy = service.get_tool_policy();
        assert_eq!(policy.mode, ToolPolicyMode::AllowAll);
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_with_override() {
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({
                "toolPolicy": {
                    "mode": "custom",
                    "allowedTools": ["read_file"],
                    "deniedTools": ["write_file"]
                }
            })),
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        let policy = service.get_tool_policy();
        assert_eq!(policy.mode, ToolPolicyMode::Custom);
        assert!(policy.is_tool_allowed("read_file"));
        assert!(!policy.is_tool_allowed("write_file"));
    }

    #[test]
    fn test_project_mcp_service_get_tool_policy_invalid_json() {
        // 如果 toolPolicy 格式无效，应该返回默认策略
        let service = ProjectMcpService {
            project_id: "proj-123".to_string(),
            service_id: "service-456".to_string(),
            config_override: Some(serde_json::json!({
                "toolPolicy": "invalid_not_an_object"
            })),
            created_at: "2026-01-31T00:00:00Z".to_string(),
        };

        let policy = service.get_tool_policy();
        assert_eq!(policy.mode, ToolPolicyMode::AllowAll);
    }
}
