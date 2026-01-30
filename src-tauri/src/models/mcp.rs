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
    pub config_override: Option<serde_json::Value>,
    /// 创建时间 (ISO 8601)
    pub created_at: String,
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
}
