//! MCP 服务管理 Tauri 命令
//!
//! Story 11.2: MCP 服务数据模型 - Task 6
//! Story 11.3: 配置导入与接管 - Task 7
//! Story 11.9: 项目详情页 MCP 集成 - Task 1
//! Story 11.19: MCP 智能接管合并引擎 - Task 4
//!
//! 提供 MCP 服务、项目关联、环境变量管理和配置导入的 Tauri IPC 命令

mod config;
mod config_path;
mod env;
mod runtime;
mod service;
mod takeover;
mod tools;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::gateway::{McpHttpClient, McpProcessManager};
use crate::models::mcp::McpService;
use crate::services::EnvManager;
use crate::storage::Database;

// Re-export all submodule contents
pub use config::*;
pub use config_path::*;
pub use env::*;
pub use runtime::*;
pub use service::*;
pub use takeover::*;
pub use tools::*;

/// MCP 服务状态
pub struct McpState {
    pub db: Arc<Mutex<Database>>,
    pub env_manager: EnvManager,
}

/// MCP 进程管理器状态
///
/// 管理 stdio 子进程和 HTTP 客户端连接
pub struct McpProcessState {
    pub manager: Arc<RwLock<McpProcessManager>>,
    /// HTTP 传输客户端缓存（service_id -> 已初始化的客户端）
    pub http_clients: Arc<RwLock<HashMap<String, Arc<McpHttpClient>>>>,
}

impl McpProcessState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(McpProcessManager::new())),
            http_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for McpProcessState {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Story 11.9: 项目详情页 MCP 集成 - 共享类型 =====

/// 项目 MCP 状态 (AC: 1, 2, 4, 5)
///
/// 用于前端 McpContextCard 组件显示项目的 MCP 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMcpStatus {
    /// 项目是否已接管 MCP 配置
    pub is_taken_over: bool,
    /// 已关联的服务列表 (来自 project_mcp_services)
    pub associated_services: Vec<McpServiceSummary>,
    /// 检测到的可接管配置文件 (来自 ToolAdapterRegistry)
    pub detectable_configs: Vec<DetectableConfig>,
}

/// MCP 服务摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceSummary {
    /// 服务 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 适配器 ID: "claude" | "cursor" | "codex" | "gemini"
    pub adapter_id: String,
    /// 是否正在运行 (Gateway 子进程存活)
    pub is_running: bool,
    /// 错误信息 (如果启动失败)
    pub error_message: Option<String>,
    /// 当前生效的 Tool Policy 模式 (Story 11.9 Phase 2)
    /// "allow_all" | "deny_all" | "custom"
    pub tool_policy_mode: Option<String>,
    /// Custom 模式下允许/禁止的工具数量 (Story 11.9 Phase 2)
    pub custom_tools_count: Option<usize>,
}

/// 检测到的可接管配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectableConfig {
    /// 适配器 ID: "claude" | "cursor" | "codex" | "gemini"
    pub adapter_id: String,
    /// 配置文件路径
    pub config_path: String,
    /// 配置作用域: "project" | "user"
    pub scope: String,
    /// 检测到的服务数量
    pub service_count: usize,
}

// ===== Story 11.11: MCP Inspector - 共享类型 =====

/// MCP 工具定义（JSON-RPC 返回格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// MCP 资源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceInfo {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP 服务能力响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub tools: Vec<McpToolInfo>,
    pub resources: Vec<McpResourceInfo>,
}

/// 解析服务的环境变量
pub(crate) fn resolve_service_env(
    service: &McpService,
    mcp_state: &tauri::State<'_, McpState>,
) -> Result<HashMap<String, String>, AppError> {
    let mut env = HashMap::new();

    if let Some(env_config) = &service.env {
        if let Some(obj) = env_config.as_object() {
            let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;

            for (key, value) in obj {
                let resolved_value = if let Some(s) = value.as_str() {
                    if s.starts_with('$') {
                        // 变量引用，从数据库获取
                        let var_name = &s[1..];
                        db.get_env_variable(&mcp_state.env_manager, var_name)?
                            .unwrap_or_default()
                    } else {
                        s.to_string()
                    }
                } else {
                    value.to_string()
                };
                env.insert(key.clone(), resolved_value);
            }
        }
    }

    Ok(env)
}
