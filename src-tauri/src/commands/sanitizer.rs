//! Sanitizer IPC 命令
//!
//! 提供 Tauri IPC 接口用于文本和会话脱敏

use crate::error::AppError;
use crate::sanitizer::{SanitizationResult, SanitizationRule, Sanitizer};

/// 对文本进行脱敏处理
///
/// # Arguments
/// * `text` - 待脱敏文本
/// * `custom_patterns` - 可选的自定义正则规则
///
/// # Returns
/// * `Result<SanitizationResult, AppError>` - 脱敏结果
#[tauri::command]
pub async fn sanitize_text(
    text: String,
    custom_patterns: Option<Vec<SanitizationRule>>,
) -> Result<SanitizationResult, AppError> {
    let sanitizer = match custom_patterns {
        Some(patterns) => Sanitizer::with_custom_rules(patterns)
            .map_err(|e| AppError::internal(e.to_string()))?,
        None => Sanitizer::with_defaults()
            .map_err(|e| AppError::internal(e.to_string()))?,
    };

    // 对于大文本使用分块处理
    const CHUNK_THRESHOLD: usize = 1024 * 1024; // 1MB
    let result = if text.len() > CHUNK_THRESHOLD {
        sanitizer.sanitize_chunked(&text, CHUNK_THRESHOLD)
    } else {
        sanitizer.sanitize(&text)
    };

    Ok(result)
}

/// 对会话进行脱敏处理
///
/// # Arguments
/// * `session_id` - 会话 ID
/// * `custom_patterns` - 可选的自定义正则规则
///
/// # Returns
/// * `Result<SanitizationResult, AppError>` - 脱敏结果
#[tauri::command]
pub async fn sanitize_session(
    session_id: String,
    custom_patterns: Option<Vec<SanitizationRule>>,
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<SanitizationResult, AppError> {
    // 获取数据库锁
    let db = state.db.lock().map_err(|_| AppError::LockError)?;

    // 从数据库获取 session
    let session = db.get_session(&session_id)?;

    // 序列化 session 为 JSON
    let session_json = serde_json::to_string(&session)
        .map_err(|e| AppError::internal(format!("序列化失败: {}", e)))?;

    // 构建 sanitizer
    let sanitizer = match custom_patterns {
        Some(patterns) => Sanitizer::with_custom_rules(patterns)
            .map_err(|e| AppError::internal(e.to_string()))?,
        None => Sanitizer::with_defaults()
            .map_err(|e| AppError::internal(e.to_string()))?,
    };

    // 脱敏处理
    let result = sanitizer.sanitize(&session_json);

    Ok(result)
}

/// 正则表达式验证结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 错误信息 (如果无效)
    pub error: Option<String>,
}

/// 验证正则表达式是否有效
///
/// # Arguments
/// * `pattern` - 正则表达式模式
///
/// # Returns
/// * `Result<ValidationResult, AppError>` - 验证结果
#[tauri::command]
pub async fn validate_regex(pattern: String) -> Result<ValidationResult, AppError> {
    match regex::Regex::new(&pattern) {
        Ok(_) => Ok(ValidationResult {
            valid: true,
            error: None,
        }),
        Err(e) => Ok(ValidationResult {
            valid: false,
            error: Some(e.to_string()),
        }),
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::sanitizer::SensitiveType;

    #[tokio::test]
    async fn test_sanitize_text_basic() {
        let text = "My API key is sk-1234567890abcdefghij1234".to_string();
        let result = sanitize_text(text, None).await.unwrap();
        assert!(result.has_matches);
        assert!(result.sanitized_text.contains("[REDACTED:API_KEY]"));
    }

    #[tokio::test]
    async fn test_sanitize_text_no_matches() {
        let text = "Hello, World!".to_string();
        let result = sanitize_text(text, None).await.unwrap();
        assert!(!result.has_matches);
        assert_eq!(result.sanitized_text, "Hello, World!");
    }

    #[tokio::test]
    async fn test_sanitize_text_with_custom_patterns() {
        let custom_rule = SanitizationRule::new(
            "Phone Number",
            r"\d{3}-\d{3}-\d{4}",
            SensitiveType::Custom,
        );
        let text = "Phone: 123-456-7890".to_string();
        let result = sanitize_text(text, Some(vec![custom_rule])).await.unwrap();
        assert!(result.has_matches);
        assert!(result.sanitized_text.contains("[REDACTED:CUSTOM]"));
    }

    #[tokio::test]
    async fn test_sanitize_text_multiple_types() {
        let text = "key=sk-abcdefghij1234567890 ip=10.0.0.1".to_string();
        let result = sanitize_text(text, None).await.unwrap();
        assert!(result.has_matches);
        assert!(result.stats.total >= 2);
    }
}
