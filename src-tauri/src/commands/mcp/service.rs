//! MCP 服务 CRUD 和项目关联命令

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpService, McpServiceSource, McpServiceWithOverride,
    UpdateMcpServiceRequest,
};
use crate::GatewayServerState;

use super::McpState;

// ===== MCP 服务管理命令 =====

/// 获取所有 MCP 服务
#[tauri::command]
pub fn list_mcp_services(state: State<'_, McpState>) -> Result<Vec<McpService>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.list_mcp_services().map_err(AppError::from)
}

/// 按来源获取 MCP 服务
#[tauri::command]
pub fn list_mcp_services_by_source(
    source: String,
    state: State<'_, McpState>,
) -> Result<Vec<McpService>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let source = McpServiceSource::from_str(&source)
        .ok_or_else(|| AppError::InvalidInput(format!("Invalid source: {}", source)))?;
    db.list_mcp_services_by_source(&source)
        .map_err(AppError::from)
}

/// 获取单个 MCP 服务
#[tauri::command]
pub fn get_mcp_service(id: String, state: State<'_, McpState>) -> Result<McpService, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_mcp_service(&id).map_err(AppError::from)
}

/// 按名称获取 MCP 服务
#[tauri::command]
pub fn get_mcp_service_by_name(
    name: String,
    state: State<'_, McpState>,
) -> Result<Option<McpService>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_mcp_service_by_name(&name).map_err(AppError::from)
}

/// 创建 MCP 服务
#[tauri::command]
pub fn create_mcp_service(
    request: CreateMcpServiceRequest,
    state: State<'_, McpState>,
) -> Result<McpService, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.create_mcp_service(&request).map_err(AppError::from)
}

/// 更新 MCP 服务
///
/// Story 11.17: 更新后自动刷新 Gateway aggregator 缓存
#[tauri::command]
pub async fn update_mcp_service(
    id: String,
    updates: UpdateMcpServiceRequest,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<McpService, AppError> {
    // 1. 更新数据库
    let service = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.update_mcp_service(&id, &updates).map_err(AppError::from)?
    };

    // 2. 如果服务启用，刷新 aggregator 缓存 (Story 11.17 Task 8.4)
    if service.enabled {
        let manager = gateway_state.manager.lock().await;
        if let Some(aggregator) = manager.aggregator() {
            let aggregator = aggregator.clone();
            drop(manager);

            // 获取环境变量
            let env_vars = {
                let db = state.db.lock().map_err(|_| AppError::LockError)?;
                let env_manager = &state.env_manager;
                db.list_env_variables()
                    .map_err(|e| AppError::internal(e.to_string()))?
                    .into_iter()
                    .filter_map(|var| {
                        db.get_env_variable(env_manager, &var.name)
                            .ok()
                            .flatten()
                            .map(|value| (var.name, value))
                    })
                    .collect::<std::collections::HashMap<String, String>>()
            };

            let env_resolver = move |var_name: &str| -> Option<String> {
                env_vars.get(var_name).cloned()
            };

            // 更新服务配置并刷新
            let _ = aggregator.update_service(service.clone()).await;
            let _ = aggregator.refresh_service(&id, env_resolver).await;
        }
    }

    Ok(service)
}

/// 删除 MCP 服务
///
/// Story 11.17: 删除后自动从 Gateway aggregator 中移除
#[tauri::command]
pub async fn delete_mcp_service(
    id: String,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<(), AppError> {
    // 1. 从数据库删除
    {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.delete_mcp_service(&id).map_err(AppError::from)?;
    }

    // 2. 从 aggregator 中移除 (Story 11.17 Task 8.4)
    let manager = gateway_state.manager.lock().await;
    if let Some(aggregator) = manager.aggregator() {
        let aggregator = aggregator.clone();
        drop(manager);
        aggregator.remove_service(&id).await;
    }

    Ok(())
}

/// 切换 MCP 服务启用状态
///
/// Story 11.17: 变更后自动刷新 Gateway aggregator 缓存
#[tauri::command]
pub async fn toggle_mcp_service(
    id: String,
    enabled: bool,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<McpService, AppError> {
    // 1. 更新数据库
    let service = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.toggle_mcp_service(&id, enabled).map_err(AppError::from)?
    };

    // 2. 自动刷新 aggregator 缓存 (Story 11.17 Task 8.4)
    let manager = gateway_state.manager.lock().await;
    if let Some(aggregator) = manager.aggregator() {
        // 释放 manager 锁后再进行刷新
        let aggregator = aggregator.clone();
        drop(manager);

        if enabled {
            // 获取环境变量
            let env_vars = {
                let db = state.db.lock().map_err(|_| AppError::LockError)?;
                let env_manager = &state.env_manager;
                db.list_env_variables()
                    .map_err(|e| AppError::internal(e.to_string()))?
                    .into_iter()
                    .filter_map(|var| {
                        db.get_env_variable(env_manager, &var.name)
                            .ok()
                            .flatten()
                            .map(|value| (var.name, value))
                    })
                    .collect::<std::collections::HashMap<String, String>>()
            };

            let env_resolver = move |var_name: &str| -> Option<String> {
                env_vars.get(var_name).cloned()
            };

            // 服务启用：刷新以获取其 tools/resources/prompts
            let _ = aggregator.refresh_service(&id, env_resolver).await;
        } else {
            // 服务禁用：从 aggregator 中移除
            aggregator.remove_service(&id).await;
        }
    }

    Ok(service)
}

// ===== 项目关联命令 =====

/// 关联 MCP 服务到项目
#[tauri::command]
pub fn link_mcp_service_to_project(
    project_id: String,
    service_id: String,
    config_override: Option<serde_json::Value>,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.link_service_to_project(&project_id, &service_id, config_override.as_ref())
        .map_err(AppError::from)?;
    Ok(())
}

/// 解除项目与 MCP 服务的关联
#[tauri::command]
pub fn unlink_mcp_service_from_project(
    project_id: String,
    service_id: String,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.unlink_service_from_project(&project_id, &service_id)
        .map_err(AppError::from)
}

/// 获取项目的 MCP 服务列表
#[tauri::command]
pub fn get_project_mcp_services(
    project_id: String,
    state: State<'_, McpState>,
) -> Result<Vec<McpServiceWithOverride>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_project_services(&project_id).map_err(AppError::from)
}

/// 获取 MCP 服务关联的项目列表
#[tauri::command]
pub fn get_mcp_service_projects(
    service_id: String,
    state: State<'_, McpState>,
) -> Result<Vec<String>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_service_projects(&service_id).map_err(AppError::from)
}

/// 更新项目级 MCP 服务配置覆盖
#[tauri::command]
pub fn update_project_mcp_service_override(
    project_id: String,
    service_id: String,
    config_override: Option<serde_json::Value>,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.update_project_service_override(&project_id, &service_id, config_override.as_ref())
        .map_err(AppError::from)
}
