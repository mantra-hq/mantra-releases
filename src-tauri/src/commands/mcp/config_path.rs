//! 工具配置路径管理命令
//!
//! Story 13.1: 工具配置路径可配置化 - Task 5

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::ToolType;

use super::McpState;

/// 获取所有工具的配置路径信息
///
/// Story 13.1: AC 5
///
/// # Returns
/// 每个工具的 tool_type、display_name、默认目录、覆盖目录、配置文件相对路径
#[tauri::command]
pub fn get_tool_config_paths(
    state: State<'_, McpState>,
) -> Result<Vec<ToolConfigPathInfo>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    let overrides = db
        .get_all_tool_config_overrides()
        .map_err(|e| AppError::internal(e.to_string()))?;

    let mut results = Vec::new();
    for tool_type in ToolType::all() {
        let default_dir = tool_type
            .get_default_config_dir()
            .to_string_lossy()
            .to_string();
        let override_dir = overrides
            .iter()
            .find(|(t, _)| t == tool_type.as_str())
            .map(|(_, p)| p.clone());

        results.push(ToolConfigPathInfo {
            tool_type: tool_type.as_str().to_string(),
            display_name: tool_type.display_name().to_string(),
            default_dir,
            override_dir,
        });
    }

    Ok(results)
}

/// 设置工具的配置目录覆盖
///
/// Story 13.1: AC 5
///
/// # Arguments
/// * `tool_type` - 工具类型 (claude_code/cursor/codex/gemini_cli)
/// * `dir` - 自定义配置目录路径
#[tauri::command]
pub fn set_tool_config_path(
    tool_type: String,
    dir: String,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    // 验证 tool_type 合法
    if ToolType::from_str(&tool_type).is_none() {
        return Err(AppError::internal(format!(
            "Unknown tool type: {}",
            tool_type
        )));
    }

    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.upsert_tool_config_path(&tool_type, &dir)
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 重置工具的配置路径到默认值
///
/// Story 13.1: AC 5
///
/// # Arguments
/// * `tool_type` - 工具类型 (claude_code/cursor/codex/gemini_cli)
#[tauri::command]
pub fn reset_tool_config_path(
    tool_type: String,
    state: State<'_, McpState>,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::LockError)?;
    db.delete_tool_config_path(&tool_type)
        .map_err(|e| AppError::internal(e.to_string()))
}

/// 工具配置路径信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfigPathInfo {
    /// 工具类型 (claude_code/cursor/codex/gemini_cli)
    pub tool_type: String,
    /// 工具显示名称
    pub display_name: String,
    /// 默认配置目录
    pub default_dir: String,
    /// 覆盖目录 (None = 使用默认目录)
    pub override_dir: Option<String>,
}
