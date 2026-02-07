//! 智能接管、全工具接管和 Local Scope 命令

use std::collections::HashMap;

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{TakeoverDecision, TakeoverPreview};
use crate::services::mcp_config::{
    execute_smart_takeover, generate_smart_takeover_preview, scan_mcp_configs, SmartTakeoverResult,
};
use crate::GatewayServerState;

use super::McpState;

// ===== Story 11.19: MCP 智能接管合并引擎命令 =====

/// 智能接管预览
///
/// Story 11.19: MCP 智能接管合并引擎 - AC 2, 3, 4
///
/// 扫描项目和全局配置，生成三档分类预览：
/// - auto_create: 全局池无此服务，将自动创建
/// - auto_skip: 全局池有同名服务且配置完全一致，自动跳过
/// - needs_decision: 需用户决策（配置冲突 / 多 scope 冲突）
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `project_path` - 项目路径
///
/// # Returns
/// 智能接管预览结果
#[tauri::command]
pub fn preview_smart_takeover(
    project_id: String,
    project_path: String,
    state: State<'_, McpState>,
) -> Result<TakeoverPreview, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // 1. 扫描配置文件
    let scan_result = scan_mcp_configs(Some(std::path::Path::new(&project_path)));

    // 2. 生成智能预览
    let preview = generate_smart_takeover_preview(&scan_result.configs, &db, &project_path)
        .map_err(AppError::from)?;

    // 3. 记录项目 ID 以便后续使用（通过 project_path 关联）
    let _ = project_id; // project_id 用于日志追踪

    Ok(preview)
}

/// 执行智能接管
///
/// Story 11.19: MCP 智能接管合并引擎 - AC 5, 7, 8
///
/// 根据预览结果和用户决策执行合并：
/// - auto_create 项: 创建服务 + 写入 source_adapter_id/source_scope + 关联项目
/// - auto_skip 项: 仅创建项目关联
/// - needs_decision 项: 按用户决策执行
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `preview` - 智能接管预览结果
/// * `decisions` - 用户决策列表
///
/// # Returns
/// 执行结果
#[tauri::command]
pub async fn execute_smart_takeover_cmd(
    project_id: String,
    preview: TakeoverPreview,
    decisions: Vec<TakeoverDecision>,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<SmartTakeoverResult, AppError> {
    // 1. 检查 Gateway 运行状态
    let (gateway_running, gateway_url, gateway_token) = {
        let manager = gateway_state.manager.lock().await;
        let running = manager.is_running();
        if running {
            let port = manager.current_port();
            let token = manager.auth_token().to_string();
            let url = format!("http://127.0.0.1:{}", port);
            (true, Some(url), Some(token))
        } else {
            (false, None, None)
        }
    };

    // 2. 执行智能接管
    let result = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        execute_smart_takeover(
            &preview,
            &decisions,
            &project_id,
            &db,
            &state.env_manager,
            gateway_url.as_deref(),
            gateway_token.as_deref(),
            gateway_running,
        )
        .map_err(AppError::from)?
    };

    // 3. 如果 Gateway 运行中且有新创建的服务，刷新 aggregator 缓存
    if gateway_running && !result.created_service_ids.is_empty() {
        let manager = gateway_state.manager.lock().await;
        if let Some(aggregator) = manager.aggregator() {
            let aggregator = aggregator.clone();
            drop(manager);

            // 获取环境变量用于刷新
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
                    .collect::<HashMap<String, String>>()
            };

            let env_resolver = move |var_name: &str| -> Option<String> {
                env_vars.get(var_name).cloned()
            };

            // 刷新新创建的服务
            for service_id in &result.created_service_ids {
                let _ = aggregator.refresh_service(service_id, &env_resolver).await;
            }
        }
    }

    Ok(result)
}

// ===== Story 11.20: 全工具接管命令 =====

/// 生成全工具接管预览
///
/// Story 11.20: 全工具自动接管生成 - AC 3
///
/// 扫描所有已安装工具的配置，为每个工具每个 Scope 生成接管预览。
/// 预览包含三档分类：auto_create, auto_skip, needs_decision
///
/// # Arguments
/// * `project_path` - 项目路径
///
/// # Returns
/// 全工具接管预览，按工具分组
#[tauri::command]
pub fn preview_full_tool_takeover(
    project_path: String,
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::FullToolTakeoverPreview, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let path = std::path::Path::new(&project_path);

    crate::services::mcp_config::generate_full_tool_takeover_preview(path, &db)
        .map_err(AppError::from)
}

/// 检测已安装的 AI 编程工具
///
/// Story 11.20: 全工具自动接管生成 - AC 1
///
/// 扫描所有支持的 AI 编程工具，检测其用户级配置文件是否存在
/// 配置文件存在即视为工具已安装
///
/// # Returns
/// 所有工具的检测结果，包括各工具的安装状态和配置路径
#[tauri::command]
pub fn detect_installed_tools(
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::AllToolsDetectionResult, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    Ok(crate::services::mcp_config::detect_installed_tools(&db))
}

/// 扫描所有工具的配置（按工具分组）
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// # Arguments
/// * `project_path` - 项目路径
///
/// # Returns
/// 所有工具的扫描结果，按工具分组
#[tauri::command]
pub fn scan_all_tool_configs(
    project_path: String,
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::AllToolsScanResult, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let path = std::path::Path::new(&project_path);
    Ok(crate::services::mcp_config::scan_all_tool_configs(path, &db))
}

/// 执行全工具接管（带事务支持）
///
/// Story 11.20: 全工具自动接管生成 - Task 5
///
/// 遍历所有检测到的工具配置，执行统一的接管操作。
/// 任意工具接管失败时，回滚所有已执行的操作。
///
/// # Arguments
/// * `project_id` - 项目 ID
/// * `preview` - 智能接管预览结果
/// * `decisions` - 用户决策列表
///
/// # Returns
/// 执行结果（包含回滚状态）
#[tauri::command]
pub async fn execute_full_tool_takeover_cmd(
    project_id: String,
    preview: TakeoverPreview,
    decisions: Vec<TakeoverDecision>,
    state: State<'_, McpState>,
    gateway_state: State<'_, GatewayServerState>,
) -> Result<crate::services::mcp_config::FullTakeoverResult, AppError> {
    // 1. 检查 Gateway 运行状态
    let (gateway_running, gateway_url, gateway_token) = {
        let manager = gateway_state.manager.lock().await;
        let running = manager.is_running();
        if running {
            let port = manager.current_port();
            let token = manager.auth_token().to_string();
            let url = format!("http://127.0.0.1:{}", port);
            (true, Some(url), Some(token))
        } else {
            (false, None, None)
        }
    };

    // 2. Gateway 必须运行才能执行接管
    let gateway_url = match gateway_url {
        Some(url) => url,
        None => {
            return Ok(
                crate::services::mcp_config::FullTakeoverResult::empty()
                    .fail("Gateway 未运行，无法执行接管".to_string()),
            );
        }
    };

    // 3. 执行全工具接管
    let result = {
        let db = state.db.lock().map_err(|_| AppError::LockError)?;
        crate::services::mcp_config::execute_full_tool_takeover(
            &preview,
            &decisions,
            &project_id,
            &db,
            &state.env_manager,
            &gateway_url,
            gateway_token.as_deref(),
            gateway_running,
        )
    };

    // 4. 如果成功且有新创建的服务，刷新 aggregator 缓存
    if result.success && !result.created_service_ids.is_empty() {
        let manager = gateway_state.manager.lock().await;
        if let Some(aggregator) = manager.aggregator() {
            let aggregator = aggregator.clone();
            drop(manager);

            // 获取环境变量用于刷新
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
                    .collect::<HashMap<String, String>>()
            };

            let env_resolver = move |var_name: &str| -> Option<String> {
                env_vars.get(var_name).cloned()
            };

            // 刷新新创建的服务
            for service_id in &result.created_service_ids {
                let _ = aggregator.refresh_service(service_id, &env_resolver).await;
            }
        }
    }

    Ok(result)
}

// ===== Story 11.21: Local Scope 相关命令 =====

/// 扫描 Claude Code Local Scope 项目列表
///
/// Story 11.21: Claude Code Local Scope 完整支持 - Task 8.1
///
/// 扫描 ~/.claude.json 中的 projects.* 配置，返回所有包含 mcpServers 的项目列表。
///
/// # Returns
/// Local Scope 项目列表，每项包含项目路径、服务数量、服务名称
#[tauri::command]
pub fn scan_local_scopes(
    state: State<'_, McpState>,
) -> Result<Vec<crate::models::mcp::LocalScopeScanResult>, AppError> {
    use crate::models::mcp::{LocalScopeScanResult, ToolType};
    use crate::services::mcp_adapters::ClaudeAdapter;

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let user_config = ToolType::ClaudeCode.resolve_config_path(&db);

    if !user_config.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&user_config)
        .map_err(|e| AppError::internal(format!("Failed to read ~/.claude.json: {}", e)))?;

    let adapter = ClaudeAdapter;
    let projects = adapter
        .list_local_scope_projects(&content)
        .map_err(|e| AppError::internal(format!("Failed to parse local scopes: {}", e)))?;

    // 转换类型
    let results = projects
        .into_iter()
        .map(|p| LocalScopeScanResult {
            project_path: p.project_path,
            service_count: p.service_count,
            service_names: p.service_names,
        })
        .collect();

    Ok(results)
}

/// 恢复单个 Local Scope 接管备份
///
/// Story 11.21: Claude Code Local Scope 完整支持 - Task 8.2
///
/// 从备份文件恢复指定项目的 mcpServers 配置到 ~/.claude.json
///
/// # Arguments
/// * `backup_id` - 备份记录 ID
///
/// # Returns
/// 恢复后的备份记录
#[tauri::command]
pub fn restore_local_scope_takeover_cmd(
    backup_id: String,
    state: State<'_, McpState>,
) -> Result<crate::models::mcp::TakeoverBackup, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    crate::services::mcp_config::restore_local_scope_takeover(&db, &backup_id)
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 恢复所有活跃的 Local Scope 接管备份
///
/// Story 11.21: Claude Code Local Scope 完整支持 - Task 8.3
///
/// 恢复所有 scope=local 的活跃备份
///
/// # Returns
/// (成功恢复的数量, 失败的错误列表)
#[tauri::command]
pub fn restore_all_local_scope_takeovers_cmd(
    state: State<'_, McpState>,
) -> Result<(usize, Vec<String>), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    crate::services::mcp_config::restore_all_local_scope_takeovers(&db)
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 获取所有活跃的 Local Scope 备份列表
///
/// Story 11.21: Claude Code Local Scope 完整支持 - Task 8.4
///
/// # Returns
/// 活跃的 Local Scope 备份列表
#[tauri::command]
pub fn get_active_local_scope_takeovers(
    state: State<'_, McpState>,
) -> Result<Vec<crate::models::mcp::TakeoverBackup>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.get_active_local_scope_takeovers()
        .map_err(|e| AppError::internal(e.to_string()))
}
