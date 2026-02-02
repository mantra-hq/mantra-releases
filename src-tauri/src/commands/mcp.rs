//! MCP 服务管理 Tauri 命令
//!
//! Story 11.2: MCP 服务数据模型 - Task 6
//! Story 11.3: 配置导入与接管 - Task 7
//! Story 11.9: 项目详情页 MCP 集成 - Task 1
//!
//! 提供 MCP 服务、项目关联、环境变量管理和配置导入的 Tauri IPC 命令

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::gateway::McpHttpClient;
use crate::models::mcp::{
    CreateMcpServiceRequest, EnvVariable, EnvVariableNameValidation, McpService, McpServiceSource,
    McpServiceWithOverride, McpTransportType, SetEnvVariableRequest, TakeoverBackup, ToolType,
    UpdateMcpServiceRequest,
};
use crate::services::mcp_adapters::{ConfigScope, ToolAdapterRegistry};
use crate::services::mcp_config::{
    scan_mcp_configs, generate_import_preview, rollback_from_backups,
    restore_mcp_takeover, restore_mcp_takeover_by_tool, get_takeover_status,
    ImportExecutor, ImportPreview, ImportRequest, ImportResult, ScanResult,
};
use crate::services::EnvManager;
use crate::storage::Database;
use crate::GatewayServerState;

/// MCP 服务状态
pub struct McpState {
    pub db: Arc<Mutex<Database>>,
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

// ===== Story 11.15: 接管恢复命令 =====

/// 获取所有活跃的接管状态
///
/// Story 11.15: MCP 接管流程重构 - AC 5
///
/// # Returns
/// 所有活跃接管的备份记录列表
#[tauri::command]
pub fn list_active_takeovers(state: State<'_, McpState>) -> Result<Vec<TakeoverBackup>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    get_takeover_status(&db).map_err(AppError::from)
}

/// 恢复指定的接管配置
///
/// Story 11.15: MCP 接管流程重构 - AC 5
///
/// # Arguments
/// * `backup_id` - 备份记录 ID
///
/// # Returns
/// 恢复后的备份记录
#[tauri::command]
pub fn restore_takeover(
    backup_id: String,
    state: State<'_, McpState>,
) -> Result<TakeoverBackup, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    restore_mcp_takeover(&db, &backup_id).map_err(AppError::from)
}

/// 按工具类型恢复接管配置
///
/// Story 11.15: MCP 接管流程重构 - AC 5
///
/// # Arguments
/// * `tool_type` - 工具类型 ("claude_code" | "cursor" | "codex" | "gemini_cli")
///
/// # Returns
/// 恢复后的备份记录（如果存在活跃接管）
#[tauri::command]
pub fn restore_takeover_by_tool(
    tool_type: String,
    state: State<'_, McpState>,
) -> Result<Option<TakeoverBackup>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let tool = ToolType::from_str(&tool_type)
        .ok_or_else(|| AppError::InvalidInput(format!("Invalid tool type: {}", tool_type)))?;
    restore_mcp_takeover_by_tool(&db, &tool).map_err(AppError::from)
}

/// 获取指定工具类型的活跃接管
///
/// Story 11.15: MCP 接管流程重构 - AC 5
///
/// # Arguments
/// * `tool_type` - 工具类型 ("claude_code" | "cursor" | "codex" | "gemini_cli")
///
/// # Returns
/// 该工具类型的活跃备份记录（如果存在）
#[tauri::command]
pub fn get_active_takeover(
    tool_type: String,
    state: State<'_, McpState>,
) -> Result<Option<TakeoverBackup>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let tool = ToolType::from_str(&tool_type)
        .ok_or_else(|| AppError::InvalidInput(format!("Invalid tool type: {}", tool_type)))?;
    db.get_active_takeover_by_tool(&tool).map_err(AppError::from)
}

/// 获取指定项目的所有活跃接管 (Story 11.16)
///
/// # Arguments
/// * `project_path` - 项目路径
///
/// # Returns
/// 该项目的所有活跃接管备份记录
#[tauri::command]
pub fn get_active_takeovers_by_project(
    project_path: String,
    state: State<'_, McpState>,
) -> Result<Vec<TakeoverBackup>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_active_takeovers_by_project(&project_path)
        .map_err(AppError::from)
}

/// 读取配置文件内容用于预览 (Story 11.16 - AC5)
///
/// Security: 仅允许读取已知接管备份记录中的 original_path 或 backup_path，
/// 防止路径遍历攻击读取任意系统文件。
///
/// # Arguments
/// * `path` - 文件路径（必须是已知接管记录中的路径）
///
/// # Returns
/// 文件内容字符串
#[tauri::command]
pub fn read_config_file_content(
    path: String,
    state: State<'_, McpState>,
) -> Result<String, AppError> {
    let requested_path = PathBuf::from(&path);

    // Security: 验证路径属于已知的接管备份记录
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let backups = db.get_takeover_backups(None).map_err(AppError::from)?;

    let is_known_path = backups.iter().any(|b| {
        b.original_path == requested_path || b.backup_path == requested_path
    });

    if !is_known_path {
        return Err(AppError::InvalidInput(
            "Access denied: path is not a known takeover config file".to_string(),
        ));
    }

    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(AppError::NotFound(format!("File not found: {}", path)));
    }

    std::fs::read_to_string(file_path).map_err(|e| {
        AppError::internal(format!("Failed to read {}: {}", path, e))
    })
}

// ===== Story 11.9: 项目详情页 MCP 集成命令 =====

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

/// 检查项目的 MCP 状态
///
/// Story 11.9: 项目详情页 MCP 集成 - Task 1
///
/// 扫描项目目录下的 MCP 配置文件，查询已接管状态和服务运行状态
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `project_path` - 项目路径 (用于扫描配置文件)
///
/// # Returns
/// 项目的 MCP 状态，包含：
/// - 是否已接管
/// - 已关联的服务列表及运行状态
/// - 可检测到的配置文件
#[tauri::command]
pub async fn check_project_mcp_status(
    project_id: String,
    project_path: Option<String>,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<ProjectMcpStatus, AppError> {
    // 1. 查询项目已关联的 MCP 服务 (限制锁的作用域)
    let project_services = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_project_services(&project_id)?
    };

    // 2. 检查 Gateway 是否运行
    let gateway_running = {
        let manager = gateway_state.manager.lock().await;
        manager.is_running()
    };

    // 3. 构建服务摘要列表 (含运行状态和策略信息)
    // 服务被视为"运行中"当：Gateway 运行 + 服务已启用
    let mut associated_services = Vec::new();
    for svc in &project_services {
        // 如果 Gateway 运行且服务已启用，视为运行状态
        let is_running = gateway_running && svc.service.enabled;

        // Story 11.9 Phase 2: 提取 Tool Policy 信息
        // 优先级: 项目级 config_override.toolPolicy > 服务级 default_tool_policy > 默认 AllowAll
        let effective_policy = {
            // 尝试从项目级 config_override 获取
            let project_policy = svc.config_override
                .as_ref()
                .and_then(|config| config.get("toolPolicy"))
                .and_then(|v| serde_json::from_value::<crate::models::mcp::ToolPolicy>(v.clone()).ok());

            match project_policy {
                Some(p) if p.mode != crate::models::mcp::ToolPolicyMode::AllowAll
                    || !p.allowed_tools.is_empty()
                    || !p.denied_tools.is_empty() => Some(p),
                _ => {
                    // 回退到服务级默认
                    svc.service.default_tool_policy.clone()
                }
            }
        };

        let (tool_policy_mode, custom_tools_count) = match &effective_policy {
            Some(policy) => {
                let mode = match policy.mode {
                    crate::models::mcp::ToolPolicyMode::AllowAll => "allow_all",
                    crate::models::mcp::ToolPolicyMode::DenyAll => "deny_all",
                    crate::models::mcp::ToolPolicyMode::Custom => "custom",
                };
                let count = match policy.mode {
                    crate::models::mcp::ToolPolicyMode::Custom => {
                        Some(policy.allowed_tools.len() + policy.denied_tools.len())
                    }
                    crate::models::mcp::ToolPolicyMode::DenyAll => Some(0),
                    _ => None,
                };
                (Some(mode.to_string()), count)
            }
            None => (None, None),
        };

        associated_services.push(McpServiceSummary {
            id: svc.service.id.clone(),
            name: svc.service.name.clone(),
            adapter_id: svc.service.source_file
                .as_ref()
                .and_then(|path| infer_adapter_id_from_path(path))
                .unwrap_or_else(|| "unknown".to_string()),
            is_running,
            error_message: None,
            tool_policy_mode,
            custom_tools_count,
        });
    }

    // 4. 扫描可检测的配置文件
    let detectable_configs = if let Some(ref path) = project_path {
        scan_detectable_configs(path)
    } else {
        Vec::new()
    };

    // 5. 判断是否已接管
    let is_taken_over = !associated_services.is_empty();

    Ok(ProjectMcpStatus {
        is_taken_over,
        associated_services,
        detectable_configs,
    })
}

/// 从配置文件路径推断适配器 ID
fn infer_adapter_id_from_path(path: &str) -> Option<String> {
    if path.contains(".mcp.json") || path.contains(".claude.json") {
        Some("claude".to_string())
    } else if path.contains(".cursor") {
        Some("cursor".to_string())
    } else if path.contains(".codex") {
        Some("codex".to_string())
    } else if path.contains(".gemini") {
        Some("gemini".to_string())
    } else {
        None
    }
}

/// 扫描项目目录下的可检测配置文件
fn scan_detectable_configs(project_path: &str) -> Vec<DetectableConfig> {
    let registry = ToolAdapterRegistry::new();
    let project_path = PathBuf::from(project_path);
    let mut detectable_configs = Vec::new();

    // 获取用户主目录
    let home_dir = dirs::home_dir();

    for adapter in registry.all() {
        for (scope, pattern) in adapter.scan_patterns() {
            let full_path = match scope {
                ConfigScope::Project => {
                    // 项目级配置：相对于项目路径
                    project_path.join(&pattern)
                }
                ConfigScope::User => {
                    // 用户级配置：替换 ~ 为用户主目录
                    if let Some(ref home) = home_dir {
                        if pattern.starts_with('~') {
                            home.join(pattern.trim_start_matches("~/"))
                        } else {
                            PathBuf::from(&pattern)
                        }
                    } else {
                        continue;
                    }
                }
            };

            // 检查文件是否存在并读取内容
            if full_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    // 解析配置文件获取服务数量
                    let service_count = match adapter.parse(&full_path, &content, scope) {
                        Ok(services) => services.len(),
                        Err(_) => 0,
                    };

                    if service_count > 0 {
                        detectable_configs.push(DetectableConfig {
                            adapter_id: adapter.id().to_string(),
                            config_path: full_path.to_string_lossy().to_string(),
                            scope: match scope {
                                ConfigScope::Project => "project".to_string(),
                                ConfigScope::User => "user".to_string(),
                            },
                            service_count,
                        });
                    }
                }
            }
        }
    }

    detectable_configs
}

// ===== Story 11.10: 工具管理命令 =====

use crate::models::mcp::ToolPolicy;
use crate::services::{ToolDefinition, ToolDiscoveryResult};

/// 获取 MCP 服务的工具列表（带缓存）
///
/// Story 11.10: Project-Level Tool Management - Task 2.5
///
/// 首先尝试从缓存获取，如果缓存不存在或已过期，返回空列表
/// 前端需要调用 refresh_service_tools 强制刷新
///
/// # Arguments
/// * `service_id` - 服务 ID
/// * `force_refresh` - 是否强制刷新（清除缓存）
///
/// # Returns
/// 工具发现结果，包含工具列表和缓存状态
#[tauri::command]
pub async fn fetch_service_tools(
    service_id: String,
    force_refresh: Option<bool>,
    state: State<'_, McpState>,
) -> Result<ToolDiscoveryResult, AppError> {
    // 如果强制刷新，先清除缓存
    if force_refresh.unwrap_or(false) {
        let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
        db_lock.clear_service_tools_cache(&service_id)?;
        drop(db_lock);
        
        // 强制刷新时返回空列表，前端需要通过其他方式获取实际工具列表
        return Ok(ToolDiscoveryResult {
            service_id,
            tools: Vec::new(),
            from_cache: false,
            cached_at: None,
        });
    }

    // 尝试获取缓存，直接从数据库获取
    let cached_tools = {
        let db_lock = state.db.lock().map_err(|_| AppError::LockError)?;
        db_lock.get_cached_service_tools(&service_id)?
    };

    if cached_tools.is_empty() {
        // 无缓存，返回空列表
        Ok(ToolDiscoveryResult {
            service_id,
            tools: Vec::new(),
            from_cache: false,
            cached_at: None,
        })
    } else {
        // 检查是否过期 (5 分钟 TTL)
        let ttl_seconds = 300;
        let is_expired = cached_tools.first().map(|t| t.is_expired(ttl_seconds)).unwrap_or(true);

        if is_expired {
            Ok(ToolDiscoveryResult {
                service_id,
                tools: Vec::new(),
                from_cache: false,
                cached_at: cached_tools.first().map(|t| t.cached_at.clone()),
            })
        } else {
            let tools: Vec<ToolDefinition> = cached_tools
                .into_iter()
                .map(|t| ToolDefinition {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect();

            Ok(ToolDiscoveryResult {
                service_id,
                tools,
                from_cache: true,
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
            let mut config = link.config_override.unwrap_or_else(|| serde_json::json!({}));
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
    db.get_service_default_policy(&service_id).map_err(AppError::from)
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

// ===== Story 11.11: MCP Inspector 直接调用命令 =====

use crate::gateway::McpProcessManager;
use std::collections::HashMap;
use tokio::sync::RwLock;

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

/// 启动 MCP 服务并获取其工具和资源列表
///
/// Story 11.11: MCP Inspector - Task 实现
///
/// 此命令会：
/// 1. 根据服务配置启动 MCP 子进程（stdio）或连接 HTTP 端点（http）
/// 2. 发送 initialize 请求
/// 3. 发送 tools/list 和 resources/list 请求
/// 4. 返回服务的完整能力列表
#[tauri::command]
pub async fn mcp_get_service_capabilities(
    service_id: String,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    // 1. 从数据库获取服务配置
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    // 根据传输类型选择不同的处理路径
    match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输模式：使用 McpHttpClient
            get_http_service_capabilities(&service, &process_state).await
        }
        McpTransportType::Stdio => {
            // stdio 传输模式：使用子进程
            get_stdio_service_capabilities(&service, &mcp_state, &process_state).await
        }
    }
}

/// 获取 HTTP 传输类型服务的能力
///
/// 创建并缓存 HTTP 客户端，后续 tools/call 和 resources/read 可复用
async fn get_http_service_capabilities(
    service: &McpService,
    process_state: &State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    let url = service.url.as_ref().ok_or_else(|| {
        AppError::internal(format!(
            "HTTP service '{}' has no URL configured",
            service.name
        ))
    })?;

    // 创建 HTTP 客户端
    let client = McpHttpClient::new(url.clone(), service.headers.clone());

    // 1. 发送 initialize 请求
    client.initialize().await.map_err(|e| {
        AppError::internal(format!(
            "Initialize failed for HTTP service '{}' ({}): {}",
            service.name, url, e
        ))
    })?;

    // 2. 发送 initialized 通知
    let _ = client.send_initialized().await;

    // 3. 获取工具列表
    let tools: Vec<McpToolInfo> = match client.list_tools().await {
        Ok(response) => response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // 4. 获取资源列表
    let resources: Vec<McpResourceInfo> = match client.list_resources().await {
        Ok(response) => response
            .get("result")
            .and_then(|r| r.get("resources"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // 5. 缓存已初始化的客户端供后续 tools/call 复用
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.insert(service.id.clone(), Arc::new(client));
    }

    Ok(McpCapabilities { tools, resources })
}

/// 获取 stdio 传输类型服务的能力
async fn get_stdio_service_capabilities(
    service: &McpService,
    mcp_state: &State<'_, McpState>,
    process_state: &State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    // 2. 解析环境变量
    let env = resolve_service_env(service, mcp_state)?;

    // 3. 启动或获取进程
    {
        let manager = process_state.manager.read().await;
        if !manager.is_running(&service.id).await {
            drop(manager);
            let manager = process_state.manager.write().await;
            manager
                .get_or_spawn(service, env.clone())
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
        }
    }

    // 4. 等待进程准备就绪（特别是对于需要网络连接的服务如 mcp-remote）
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 检查进程是否仍在运行
    {
        let manager = process_state.manager.read().await;
        if !manager.is_running(&service.id).await {
            return Err(AppError::internal(format!(
                "MCP service '{}' process exited before initialization. \
                 Command: {} {:?}. \
                 Please check if the service is correctly configured and all dependencies are installed.",
                service.name,
                service.command,
                service.args
            )));
        }
    }

    // 5. 发送 initialize 请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "mantra-inspector",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    {
        let manager = process_state.manager.read().await;
        let _ = manager
            .send_request(&service.id, init_request)
            .await
            .map_err(|e| AppError::internal(format!(
                "Initialize failed for '{}': {}. \
                 Command: {} {:?}",
                service.name,
                e,
                service.command,
                service.args
            )))?;
    }

    // 6. 发送 notifications/initialized
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    {
        let manager = process_state.manager.read().await;
        // 通知不需要响应，忽略错误
        let _ = manager.send_request(&service.id, initialized_notification).await;
    }

    // 7. 获取工具列表
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let tools: Vec<McpToolInfo> = {
        let manager = process_state.manager.read().await;
        match manager.send_request(&service.id, tools_request).await {
            Ok(response) => {
                response
                    .get("result")
                    .and_then(|r| r.get("tools"))
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    };

    // 8. 获取资源列表
    let resources_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list",
        "params": {}
    });

    let resources: Vec<McpResourceInfo> = {
        let manager = process_state.manager.read().await;
        match manager.send_request(&service.id, resources_request).await {
            Ok(response) => {
                response
                    .get("result")
                    .and_then(|r| r.get("resources"))
                    .and_then(|t| serde_json::from_value(t.clone()).ok())
                    .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    };

    Ok(McpCapabilities { tools, resources })
}

/// 调用 MCP 工具
///
/// Story 11.11: MCP Inspector
/// 支持 stdio 和 HTTP 两种传输类型
#[tauri::command]
pub async fn mcp_call_tool(
    service_id: String,
    tool_name: String,
    arguments: serde_json::Value,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<serde_json::Value, AppError> {
    // 查询服务配置以确定传输类型
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": chrono::Utc::now().timestamp_millis(),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });

    let response = match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输：从缓存获取客户端或新建
            let client = get_or_create_http_client(&service, &process_state).await?;
            client
                .send_request(request)
                .await
                .map_err(|e| AppError::internal(format!("HTTP tool call failed: {}", e)))?
        }
        McpTransportType::Stdio => {
            // stdio 传输：通过进程管理器
            let manager = process_state.manager.read().await;
            manager
                .send_request(&service_id, request)
                .await
                .map_err(|e| AppError::internal(e.to_string()))?
        }
    };

    // 检查是否有错误
    if let Some(error) = response.get("error") {
        return Err(AppError::internal(format!(
            "Tool call failed: {}",
            error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error")
        )));
    }

    Ok(response.get("result").cloned().unwrap_or(serde_json::json!(null)))
}

/// 读取 MCP 资源
///
/// Story 11.11: MCP Inspector
/// 支持 stdio 和 HTTP 两种传输类型
#[tauri::command]
pub async fn mcp_read_resource(
    service_id: String,
    uri: String,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<serde_json::Value, AppError> {
    // 查询服务配置以确定传输类型
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": chrono::Utc::now().timestamp_millis(),
        "method": "resources/read",
        "params": {
            "uri": uri
        }
    });

    let response = match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输：从缓存获取客户端或新建
            let client = get_or_create_http_client(&service, &process_state).await?;
            client
                .send_request(request)
                .await
                .map_err(|e| AppError::internal(format!("HTTP resource read failed: {}", e)))?
        }
        McpTransportType::Stdio => {
            // stdio 传输：通过进程管理器
            let manager = process_state.manager.read().await;
            manager
                .send_request(&service_id, request)
                .await
                .map_err(|e| AppError::internal(e.to_string()))?
        }
    };

    // 检查是否有错误
    if let Some(error) = response.get("error") {
        return Err(AppError::internal(format!(
            "Resource read failed: {}",
            error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error")
        )));
    }

    Ok(response.get("result").cloned().unwrap_or(serde_json::json!(null)))
}

/// 获取或创建 HTTP 客户端
///
/// 优先从缓存中获取已初始化的客户端，如果不存在则创建新客户端并初始化
async fn get_or_create_http_client(
    service: &McpService,
    process_state: &State<'_, McpProcessState>,
) -> Result<Arc<McpHttpClient>, AppError> {
    // 先尝试从缓存获取
    {
        let http_clients = process_state.http_clients.read().await;
        if let Some(client) = http_clients.get(&service.id) {
            return Ok(Arc::clone(client));
        }
    }

    // 缓存中没有，创建新客户端
    let url = service.url.as_ref().ok_or_else(|| {
        AppError::internal(format!(
            "HTTP service '{}' has no URL configured",
            service.name
        ))
    })?;

    let client = McpHttpClient::new(url.clone(), service.headers.clone());

    // 初始化连接
    client.initialize().await.map_err(|e| {
        AppError::internal(format!(
            "Initialize failed for HTTP service '{}' ({}): {}",
            service.name, url, e
        ))
    })?;
    let _ = client.send_initialized().await;

    let client = Arc::new(client);

    // 缓存客户端
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.insert(service.id.clone(), Arc::clone(&client));
    }

    Ok(client)
}

/// 停止 MCP 服务进程
///
/// Story 11.11: MCP Inspector
/// 清理 stdio 进程和 HTTP 客户端缓存
#[tauri::command]
pub async fn mcp_stop_service(
    service_id: String,
    process_state: State<'_, McpProcessState>,
) -> Result<(), AppError> {
    // 停止 stdio 进程
    let manager = process_state.manager.read().await;
    manager.stop_process(&service_id).await;

    // 清理 HTTP 客户端缓存
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.remove(&service_id);
    }

    Ok(())
}

/// 获取运行中的 MCP 服务列表
///
/// Story 11.11: MCP Inspector
#[tauri::command]
pub async fn mcp_list_running_services(
    process_state: State<'_, McpProcessState>,
) -> Result<Vec<String>, AppError> {
    let manager = process_state.manager.read().await;
    let running = manager.list_running().await;
    Ok(running.into_iter().map(|p| p.service_id).collect())
}

/// 解析服务的环境变量
fn resolve_service_env(
    service: &McpService,
    mcp_state: &State<'_, McpState>,
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
