//! 工具管理和 Tool Policy 命令

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{McpTransportType, ToolPolicy};
use crate::services::{ToolDefinition, ToolDiscoveryResult};

use super::{McpProcessState, McpState};
use super::runtime::{get_http_service_capabilities, get_stdio_service_capabilities};

// ===== Story 11.10: 工具管理命令 =====

/// 获取 MCP 服务的工具列表（带缓存）
///
/// Story 11.10: Project-Level Tool Management - Task 2.5
///
/// 首先尝试从缓存获取，如果缓存不存在或已过期，从 MCP 服务获取并缓存
///
/// # Arguments
/// * `service_id` - 服务 ID
/// * `force_refresh` - 是否强制刷新（清除缓存并重新获取）
///
/// # Returns
/// 工具发现结果，包含工具列表和缓存状态
#[tauri::command]
pub async fn fetch_service_tools(
    service_id: String,
    force_refresh: Option<bool>,
    state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<ToolDiscoveryResult, AppError> {
    let force = force_refresh.unwrap_or(false);

    // 如果强制刷新，先清除缓存
    if force {
        let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
        db_lock.clear_service_tools_cache(&service_id)?;
        drop(db_lock);
    }

    // 尝试获取缓存
    let cached_tools = if !force {
        let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
        db_lock.get_cached_service_tools(&service_id)?
    } else {
        Vec::new()
    };

    // 检查缓存是否有效 (5 分钟 TTL)
    let ttl_seconds = 300;
    let cache_valid = !cached_tools.is_empty()
        && !cached_tools
            .first()
            .map(|t| t.is_expired(ttl_seconds))
            .unwrap_or(true);

    if cache_valid {
        // 返回缓存的工具列表
        let tools: Vec<ToolDefinition> = cached_tools
            .into_iter()
            .map(|t| ToolDefinition {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
            })
            .collect();

        return Ok(ToolDiscoveryResult {
            service_id,
            tools,
            from_cache: true,
            cached_at: None,
        });
    }

    // 缓存无效或不存在，从 MCP 服务获取工具列表
    let service = {
        let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
        db_lock.get_mcp_service(&service_id)?
    };

    let capabilities = match service.transport_type {
        McpTransportType::Http => get_http_service_capabilities(&service, &process_state).await,
        McpTransportType::Stdio => {
            get_stdio_service_capabilities(&service, &state, &process_state).await
        }
    };

    match capabilities {
        Ok(caps) => {
            let tools: Vec<ToolDefinition> = caps
                .tools
                .into_iter()
                .map(|t| ToolDefinition {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect();

            // 缓存到数据库
            if !tools.is_empty() {
                let tool_data: Vec<(String, Option<String>, Option<serde_json::Value>)> = tools
                    .iter()
                    .map(|t| (t.name.clone(), t.description.clone(), t.input_schema.clone()))
                    .collect();
                let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
                let _ = db_lock.cache_service_tools(&service_id, &tool_data);
            }

            Ok(ToolDiscoveryResult {
                service_id,
                tools,
                from_cache: false,
                cached_at: None,
            })
        }
        Err(e) => {
            // 获取失败，返回空列表
            eprintln!(
                "[fetch_service_tools] Failed to fetch tools for service {}: {}",
                service_id, e
            );
            Ok(ToolDiscoveryResult {
                service_id,
                tools: Vec::new(),
                from_cache: false,
                cached_at: None,
            })
        }
    }
}

/// 缓存 MCP 服务的工具列表
///
/// Story 11.10: Project-Level Tool Management - Task 2.5
///
/// 前端在调用 MCP 服务的 tools/list 后，将结果通过此命令缓存
///
/// # Arguments
/// * `service_id` - 服务 ID
/// * `tools` - 工具列表
#[tauri::command]
pub async fn cache_service_tools(
    service_id: String,
    tools: Vec<ToolDefinition>,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    let tool_data: Vec<(String, Option<String>, Option<serde_json::Value>)> = tools
        .into_iter()
        .map(|t| (t.name, t.description, t.input_schema))
        .collect();

    db.cache_service_tools(&service_id, &tool_data)
        .map_err(AppError::from)
}

/// 刷新 MCP 服务的工具缓存
///
/// Story 11.10: Project-Level Tool Management - Task 2.5
///
/// 清除指定服务的工具缓存，强制下次获取时重新从服务获取
///
/// # Arguments
/// * `service_id` - 服务 ID
#[tauri::command]
pub async fn refresh_service_tools(
    service_id: String,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.clear_service_tools_cache(&service_id)
        .map_err(AppError::from)
}

/// 获取项目的 Tool Policy
///
/// Story 11.10: Project-Level Tool Management - AC 1
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `service_id` - 服务 ID
///
/// # Returns
/// Tool Policy 配置
#[tauri::command]
pub fn get_project_tool_policy(
    project_id: String,
    service_id: String,
    state: State<'_, McpState>,
) -> Result<ToolPolicy, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    let link = db.get_project_service_link(&project_id, &service_id)?;
    match link {
        Some(pms) => Ok(pms.get_tool_policy()),
        None => Ok(ToolPolicy::default()),
    }
}

/// 更新项目的 Tool Policy
///
/// Story 11.10: Project-Level Tool Management - AC 3
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `service_id` - 服务 ID
/// * `policy` - Tool Policy 配置
#[tauri::command]
pub fn update_project_tool_policy(
    project_id: String,
    service_id: String,
    policy: ToolPolicy,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // 获取现有的 config_override
    let existing_link = db.get_project_service_link(&project_id, &service_id)?;

    // 构建新的 config_override
    let policy_value = serde_json::to_value(&policy).unwrap_or_default();
    let new_config = match existing_link {
        Some(link) => {
            let mut config = link
                .config_override
                .unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = config.as_object_mut() {
                obj.insert("toolPolicy".to_string(), policy_value);
            }
            config
        }
        None => {
            // 链接不存在，返回错误
            return Err(AppError::NotFound(format!(
                "Project-service link not found: {} - {}",
                project_id, service_id
            )));
        }
    };

    db.update_project_service_override(&project_id, &service_id, Some(&new_config))
        .map_err(AppError::from)
}

// ===== Story 11.9 Phase 2: 服务级默认 Tool Policy =====

/// 获取服务的默认 Tool Policy
///
/// Story 11.9 Phase 2: Task 10 - Hub 页面全局 Policy 入口
///
/// # Arguments
/// * `service_id` - 服务 ID
///
/// # Returns
/// 服务的默认 Tool Policy，如果未配置则返回 AllowAll
#[tauri::command]
pub fn get_service_default_policy(
    service_id: String,
    state: State<'_, McpState>,
) -> Result<ToolPolicy, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_service_default_policy(&service_id)
        .map_err(AppError::from)
}

/// 更新服务的默认 Tool Policy
///
/// Story 11.9 Phase 2: Task 10 - Hub 页面全局 Policy 入口
///
/// # Arguments
/// * `service_id` - 服务 ID
/// * `policy` - Tool Policy 配置，传 None 清除默认策略
#[tauri::command]
pub fn update_service_default_policy(
    service_id: String,
    policy: Option<ToolPolicy>,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.update_service_default_policy(&service_id, policy.as_ref())?;
    Ok(())
}
