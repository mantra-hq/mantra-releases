//! MCP 服务管理 Tauri 命令
//!
//! Story 11.2: MCP 服务数据模型 - Task 6
//!
//! 提供 MCP 服务、项目关联和环境变量管理的 Tauri IPC 命令

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{
    CreateMcpServiceRequest, EnvVariable, McpService, McpServiceSource, McpServiceWithOverride,
    UpdateMcpServiceRequest,
};
use crate::services::EnvManager;
use crate::storage::Database;

use std::sync::Mutex;

/// MCP 服务状态
pub struct McpState {
    pub db: Mutex<Database>,
    pub env_manager: EnvManager,
}

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
#[tauri::command]
pub fn update_mcp_service(
    id: String,
    updates: UpdateMcpServiceRequest,
    state: State<'_, McpState>,
) -> Result<McpService, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.update_mcp_service(&id, &updates).map_err(AppError::from)
}

/// 删除 MCP 服务
#[tauri::command]
pub fn delete_mcp_service(id: String, state: State<'_, McpState>) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.delete_mcp_service(&id).map_err(AppError::from)
}

/// 切换 MCP 服务启用状态
#[tauri::command]
pub fn toggle_mcp_service(
    id: String,
    enabled: bool,
    state: State<'_, McpState>,
) -> Result<McpService, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.toggle_mcp_service(&id, enabled).map_err(AppError::from)
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

// ===== 环境变量命令 =====

/// 设置环境变量
#[tauri::command]
pub fn set_env_variable(
    name: String,
    value: String,
    description: Option<String>,
    state: State<'_, McpState>,
) -> Result<EnvVariable, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.set_env_variable(&state.env_manager, &name, &value, description.as_deref())
        .map_err(AppError::from)
}

/// 获取环境变量列表（值已脱敏）
#[tauri::command]
pub fn list_env_variables(state: State<'_, McpState>) -> Result<Vec<EnvVariable>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.list_env_variables().map_err(AppError::from)
}

/// 删除环境变量
#[tauri::command]
pub fn delete_env_variable(name: String, state: State<'_, McpState>) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.delete_env_variable(&name).map_err(AppError::from)
}

/// 检查环境变量是否存在
#[tauri::command]
pub fn env_variable_exists(name: String, state: State<'_, McpState>) -> Result<bool, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.env_variable_exists(&name).map_err(AppError::from)
}
