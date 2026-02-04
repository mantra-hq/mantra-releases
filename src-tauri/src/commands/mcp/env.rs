//! 环境变量管理命令

use tauri::State;

use crate::error::AppError;
use crate::models::mcp::{EnvVariable, EnvVariableNameValidation, SetEnvVariableRequest, McpService};

use super::McpState;

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
            error_message: Some(
                "变量名必须为 SCREAMING_SNAKE_CASE 格式（大写字母、数字和下划线）".to_string(),
            ),
        }
    }
}
