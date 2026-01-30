//! MCP 服务管理 Tauri 命令
//!
//! Story 11.2: MCP 服务数据模型 - Task 6
//! Story 11.3: 配置导入与接管 - Task 7
//!
//! 提供 MCP 服务、项目关联、环境变量管理和配置导入的 Tauri IPC 命令

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{
    CreateMcpServiceRequest, EnvVariable, EnvVariableNameValidation, McpService, McpServiceSource,
    McpServiceWithOverride, SetEnvVariableRequest, UpdateMcpServiceRequest,
};
use crate::services::mcp_config::{
    scan_mcp_configs, generate_import_preview, rollback_from_backups,
    ImportExecutor, ImportPreview, ImportRequest, ImportResult, ScanResult,
};
use crate::services::EnvManager;
use crate::storage::Database;

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

// ===== Story 11.4: 环境变量管理扩展命令 =====

/// 获取解密后的环境变量值（临时显示用）
///
/// 用于前端"显示完整值"功能，返回解密后的明文值
///
/// # Arguments
/// * `name` - 环境变量名称
///
/// # Returns
/// 解密后的变量值，如果不存在则返回 None
#[tauri::command]
pub fn get_env_variable_decrypted(
    name: String,
    state: State<'_, McpState>,
) -> Result<Option<String>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_env_variable(&state.env_manager, &name)
        .map_err(AppError::from)
}

/// 获取引用指定环境变量的 MCP 服务列表
///
/// 用于删除/编辑环境变量前显示受影响的服务
///
/// # Arguments
/// * `var_name` - 环境变量名称
///
/// # Returns
/// 引用该变量的 MCP 服务列表
#[tauri::command]
pub fn get_affected_mcp_services(
    var_name: String,
    state: State<'_, McpState>,
) -> Result<Vec<McpService>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.find_services_using_env_var(&var_name)
        .map_err(AppError::from)
}

/// 批量设置环境变量
///
/// 一次性设置多个环境变量，用于批量导入场景
///
/// # Arguments
/// * `variables` - 环境变量列表
///
/// # Returns
/// 创建/更新的环境变量列表
#[tauri::command]
pub fn batch_set_env_variables(
    variables: Vec<SetEnvVariableRequest>,
    state: State<'_, McpState>,
) -> Result<Vec<EnvVariable>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let mut results = Vec::with_capacity(variables.len());

    for var in variables {
        let result = db.set_env_variable(
            &state.env_manager,
            &var.name,
            &var.value,
            var.description.as_deref(),
        )?;
        results.push(result);
    }

    Ok(results)
}

/// 校验环境变量名格式
///
/// 检查变量名是否符合 SCREAMING_SNAKE_CASE 格式
/// 如果不符合，提供格式化建议
///
/// # Arguments
/// * `name` - 待校验的变量名
///
/// # Returns
/// 校验结果，包含是否有效和格式化建议
#[tauri::command]
pub fn validate_env_variable_name(name: String) -> EnvVariableNameValidation {
    // SCREAMING_SNAKE_CASE 格式：以大写字母开头，只包含大写字母、数字和下划线
    let re = regex::Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap();
    let is_valid = re.is_match(&name);

    if is_valid {
        EnvVariableNameValidation {
            is_valid: true,
            suggestion: None,
            error_message: None,
        }
    } else {
        // 生成格式化建议
        let suggestion = name
            .to_uppercase()
            .replace('-', "_")
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>();

        // 确保以字母开头
        let suggestion = if suggestion.starts_with(|c: char| c.is_ascii_digit()) {
            format!("VAR_{}", suggestion)
        } else if suggestion.is_empty() {
            "VARIABLE_NAME".to_string()
        } else {
            suggestion
        };

        EnvVariableNameValidation {
            is_valid: false,
            suggestion: Some(suggestion),
            error_message: Some("变量名必须为 SCREAMING_SNAKE_CASE 格式（大写字母、数字和下划线）".to_string()),
        }
    }
}

// ===== Story 11.3: 配置导入命令 =====

/// 扫描 MCP 配置文件
///
/// 扫描指定项目路径和全局配置目录，检测所有 MCP 配置文件
///
/// # Arguments
/// * `project_path` - 项目路径（可选）
///
/// # Returns
/// 扫描结果，包含检测到的配置文件和服务
#[tauri::command]
pub fn scan_mcp_configs_cmd(project_path: Option<String>) -> Result<ScanResult, AppError> {
    let path = project_path.as_ref().map(PathBuf::from);
    Ok(scan_mcp_configs(path.as_deref()))
}

/// 生成 MCP 配置导入预览
///
/// 分析扫描结果，检测冲突和需要的环境变量
///
/// # Arguments
/// * `scan_result` - 扫描结果
///
/// # Returns
/// 导入预览，包含冲突信息和环境变量需求
#[tauri::command]
pub fn preview_mcp_import(
    scan_result: ScanResult,
    state: State<'_, McpState>,
) -> Result<ImportPreview, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    generate_import_preview(&scan_result.configs, &db).map_err(AppError::from)
}

/// 执行 MCP 配置导入
///
/// 根据预览结果和用户选择执行导入
///
/// # Arguments
/// * `preview` - 导入预览
/// * `request` - 导入请求（包含服务选择、冲突解决策略、环境变量值等）
///
/// # Returns
/// 导入结果
#[tauri::command]
pub fn execute_mcp_import(
    preview: ImportPreview,
    request: ImportRequest,
    state: State<'_, McpState>,
) -> Result<ImportResult, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let executor = ImportExecutor::new(&db, &state.env_manager);
    executor.execute(&preview, &request).map_err(AppError::from)
}

/// 回滚 MCP 配置导入
///
/// 从备份文件恢复原始配置
///
/// # Arguments
/// * `backup_files` - 备份文件路径列表
///
/// # Returns
/// 成功恢复的文件数量
#[tauri::command]
pub fn rollback_mcp_import(backup_files: Vec<String>) -> Result<usize, AppError> {
    let paths: Vec<PathBuf> = backup_files.iter().map(PathBuf::from).collect();
    rollback_from_backups(&paths).map_err(AppError::Io)
}
