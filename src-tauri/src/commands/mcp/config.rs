//! 配置导入/扫描和项目 MCP 状态命令

use std::path::PathBuf;

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{TakeoverBackup, ToolType};
use crate::services::mcp_adapters::{ConfigScope, ToolAdapterRegistry};
use crate::services::mcp_config::{
    delete_invalid_backups, generate_import_preview, get_takeover_status,
    list_takeover_backups_with_integrity, restore_mcp_takeover, restore_mcp_takeover_by_tool,
    rollback_from_backups, scan_mcp_configs, ImportExecutor, ImportPreview, ImportRequest,
    ImportResult, ScanResult,
};
use crate::GatewayServerState;

use super::{
    DetectableConfig, McpServiceSummary, McpState, ProjectMcpStatus,
};

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

    let is_known_path = backups
        .iter()
        .any(|b| b.original_path == requested_path || b.backup_path == requested_path);

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

        // Story 11.18: 简化的 Tool Policy 信息提取
        // 优先级: 项目级 config_override.toolPolicy > 服务级 default_tool_policy > 默认 AllowAll
        let effective_policy = {
            // 尝试从项目级 config_override 获取
            let project_policy = svc.config_override
                .as_ref()
                .and_then(|config| config.get("toolPolicy"))
                .and_then(|v| serde_json::from_value::<crate::models::mcp::ToolPolicy>(v.clone()).ok());

            match project_policy {
                // 如果项目级不是继承模式，使用项目级策略
                Some(p) if !p.is_inherit() => Some(p),
                _ => {
                    // 回退到服务级默认
                    svc.service.default_tool_policy.clone()
                }
            }
        };

        let (tool_policy_mode, custom_tools_count) = match &effective_policy {
            Some(policy) => {
                // Story 11.18: 简化的模式判断
                let mode = if policy.is_inherit() {
                    "inherit" // 继承全局
                } else if policy.is_allow_all() {
                    "allow_all" // 全选
                } else {
                    "custom" // 部分选
                };
                let count = if policy.is_custom() {
                    policy.allowed_tools.as_ref().map(|tools| tools.len())
                } else {
                    None
                };
                (Some(mode.to_string()), count)
            }
            None => (Some("allow_all".to_string()), None), // 无策略 = 默认全选
        };

        associated_services.push(McpServiceSummary {
            id: svc.service.id.clone(),
            name: svc.service.name.clone(),
            // Story 11.19 AC6: 三级回退逻辑获取 adapter_id
            // 1. project_mcp_map.detected_adapter_id (项目关联时记录)
            // 2. mcp_services.source_adapter_id (首次导入时记录)
            // 3. infer_adapter_id_from_path(source_file) (回退推断)
            adapter_id: svc
                .detected_adapter_id
                .clone()
                .or_else(|| svc.service.source_adapter_id.clone())
                .or_else(|| {
                    svc.service
                        .source_file
                        .as_ref()
                        .and_then(|path| infer_adapter_id_from_path(path))
                })
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

    let _ = project_id; // project_id 用于日志追踪

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
                ConfigScope::User | ConfigScope::Local => {
                    // 用户级配置：替换 ~ 为用户主目录
                    // Local scope 在扫描模式中不会出现，但为完整性处理
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
                                ConfigScope::Local => "local".to_string(),
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

// ===== Story 11.22: 备份完整性检查命令 =====

/// 获取带完整性信息的活跃备份列表
///
/// Story 11.22: 原子性备份恢复机制 - AC 4
///
/// 检查每个备份记录的文件存在状态和 hash 完整性
///
/// # Returns
/// 带完整性信息的备份记录列表
#[tauri::command]
pub fn list_active_takeovers_with_integrity(
    state: State<'_, McpState>,
) -> Result<Vec<crate::models::mcp::TakeoverBackupIntegrity>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    list_takeover_backups_with_integrity(&db).map_err(AppError::from)
}

/// 删除无效的备份记录
///
/// Story 11.22: 原子性备份恢复机制 - AC 4
///
/// 无效备份定义为：备份文件不存在或 hash 验证失败的记录
///
/// # Returns
/// 删除的记录数量
#[tauri::command]
pub fn delete_invalid_takeover_backups(
    state: State<'_, McpState>,
) -> Result<usize, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    delete_invalid_backups(&db).map_err(AppError::from)
}

// ===== Story 11.23: 备份版本管理命令 =====

/// 清理旧备份，只保留最近 keep_count 个
///
/// Story 11.23: 备份版本管理 - AC 1, 2
///
/// 每个 (工具 + Scope + 项目路径) 组合独立计算
/// 按照 DD-015 清理优先级：先删除 DB 记录，再删除文件
///
/// # Arguments
/// * `tool_type` - 工具类型
/// * `scope` - 作用域 ("user" | "project" | "local")
/// * `project_path` - 项目路径 (project/local scope 需要)
/// * `keep_count` - 保留数量 (默认 5)
///
/// # Returns
/// 清理结果
#[tauri::command]
pub fn cleanup_old_takeover_backups(
    tool_type: String,
    scope: String,
    project_path: Option<String>,
    keep_count: Option<usize>,
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::CleanupResult, AppError> {
    use crate::models::mcp::TakeoverScope;
    use crate::services::mcp_config::cleanup_old_backups;

    let tool = ToolType::from_str(&tool_type)
        .ok_or_else(|| AppError::InvalidInput(format!("Invalid tool type: {}", tool_type)))?;

    let scope = TakeoverScope::from_str(&scope)
        .ok_or_else(|| AppError::InvalidInput(format!("Invalid scope: {}", scope)))?;

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    cleanup_old_backups(&db, &tool, &scope, project_path.as_deref(), keep_count.unwrap_or(5))
        .map_err(AppError::from)
}

/// 批量清理所有组合的旧备份
///
/// Story 11.23: 备份版本管理 - AC 5
///
/// 对每个 (工具 + Scope + 项目路径) 组合执行清理
///
/// # Arguments
/// * `keep_per_group` - 每组保留数量 (默认 5)
///
/// # Returns
/// 批量清理结果
#[tauri::command]
pub fn cleanup_all_old_takeover_backups(
    keep_per_group: Option<usize>,
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::BatchCleanupResult, AppError> {
    use crate::services::mcp_config::cleanup_all_old_backups;

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    cleanup_all_old_backups(&db, keep_per_group.unwrap_or(5)).map_err(AppError::from)
}

/// 获取备份统计信息
///
/// Story 11.23: 备份版本管理 - AC 5
///
/// # Returns
/// 备份统计信息
#[tauri::command]
pub fn get_backup_stats(
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::BackupStats, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_backup_stats().map_err(AppError::from)
}

/// 获取带版本序号的备份列表
///
/// Story 11.23: 备份版本管理 - AC 3
///
/// 为每个备份添加版本序号信息
///
/// # Returns
/// 带版本序号的备份列表
#[tauri::command]
pub fn list_takeover_backups_with_version(
    state: State<'_, McpState>,
) -> Result<Vec<crate::models::mcp::TakeoverBackupWithVersion>, AppError> {
    use crate::services::mcp_config::list_backups_with_version;

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    list_backups_with_version(&db).map_err(AppError::from)
}

/// 删除单个备份记录
///
/// Story 11.23: 备份版本管理 - AC 3
///
/// 按照 DD-015 清理优先级：先删除 DB 记录，再删除文件
///
/// # Arguments
/// * `backup_id` - 备份 ID
///
/// # Returns
/// 删除的文件大小 (如果文件存在)
#[tauri::command]
pub fn delete_single_takeover_backup(
    backup_id: String,
    state: State<'_, McpState>,
) -> Result<u64, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // 1. 获取备份记录
    let backup = db
        .get_takeover_backup_by_id(&backup_id)?
        .ok_or_else(|| AppError::NotFound(format!("Backup not found: {}", backup_id)))?;

    // 2. 获取文件大小
    let file_size = if backup.backup_path.exists() {
        std::fs::metadata(&backup.backup_path)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    // 3. 先删除 DB 记录
    db.delete_takeover_backup(&backup_id)?;

    // 4. 再删除文件（失败只警告）
    if backup.backup_path.exists() {
        if let Err(e) = std::fs::remove_file(&backup.backup_path) {
            eprintln!("[Backup] Warning: Failed to delete backup file {:?}: {}", backup.backup_path, e);
        }
    }

    Ok(file_size)
}
