//! MCP 服务管理 Tauri 命令
//!
//! Story 11.2: MCP 服务数据模型 - Task 6
//! Story 11.3: 配置导入与接管 - Task 7
//! Story 11.9: 项目详情页 MCP 集成 - Task 1
//!
//! 提供 MCP 服务、项目关联、环境变量管理和配置导入的 Tauri IPC 命令

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{
    CreateMcpServiceRequest, EnvVariable, EnvVariableNameValidation, McpService, McpServiceSource,
    McpServiceWithOverride, SetEnvVariableRequest, UpdateMcpServiceRequest,
};
use crate::services::mcp_adapters::{ConfigScope, ToolAdapterRegistry};
use crate::services::mcp_config::{
    scan_mcp_configs, generate_import_preview, rollback_from_backups,
    ImportExecutor, ImportPreview, ImportRequest, ImportResult, ScanResult,
};
use crate::services::EnvManager;
use crate::storage::Database;
use crate::GatewayServerState;

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

    // 3. 构建服务摘要列表 (含运行状态)
    // 服务被视为"运行中"当：Gateway 运行 + 服务已启用
    let mut associated_services = Vec::new();
    for svc in &project_services {
        // 如果 Gateway 运行且服务已启用，视为运行状态
        let is_running = gateway_running && svc.service.enabled;
        associated_services.push(McpServiceSummary {
            id: svc.service.id.clone(),
            name: svc.service.name.clone(),
            adapter_id: svc.service.source_file
                .as_ref()
                .and_then(|path| infer_adapter_id_from_path(path))
                .unwrap_or_else(|| "unknown".to_string()),
            is_running,
            error_message: None,
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
