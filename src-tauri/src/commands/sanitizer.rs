//! Sanitizer IPC 命令
//!
//! 提供 Tauri IPC 接口用于文本和会话脱敏

use crate::error::AppError;
use crate::sanitizer::{PrivacyScanner, SanitizationResult, SanitizationRule, ScanResult, Sanitizer};

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

    // 序列化 session 为格式化 JSON (与前端保持一致)
    let session_json = serde_json::to_string_pretty(&session)
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

/// 获取所有内置脱敏规则
///
/// # Returns
/// * `Result<Vec<SanitizationRule>, AppError>` - 内置规则列表
#[tauri::command]
pub async fn get_builtin_rules() -> Result<Vec<SanitizationRule>, AppError> {
    use crate::sanitizer::BUILTIN_RULES;
    Ok(BUILTIN_RULES.clone())
}

/// 扫描文本中的隐私信息
///
/// 不修改原文，只返回检测结果供用户决策。
///
/// # Arguments
/// * `text` - 待扫描文本
///
/// # Returns
/// * `Result<ScanResult, AppError>` - 扫描结果，包含所有匹配项的详细信息
#[tauri::command]
pub async fn scan_text_for_privacy(text: String) -> Result<ScanResult, AppError> {
    let scanner = PrivacyScanner::with_defaults()
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(scanner.scan(&text))
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
            "phone_number",
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

    // Story 3-5: Test get_builtin_rules command
    #[tokio::test]
    async fn test_get_builtin_rules() {
        let rules = get_builtin_rules().await.unwrap();

        // Verify rules are returned
        assert!(!rules.is_empty(), "Should return non-empty builtin rules");

        // Verify expected rule types exist
        let rule_names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
        assert!(rule_names.contains(&"OpenAI API Key"), "Should contain OpenAI API Key rule");
        assert!(rule_names.contains(&"GitHub Token"), "Should contain GitHub Token rule");
        assert!(rule_names.contains(&"JWT Token"), "Should contain JWT Token rule");

        // Verify all rules have valid patterns (non-empty)
        for rule in &rules {
            assert!(!rule.pattern.is_empty(), "Rule {} should have non-empty pattern", rule.name);
            assert!(!rule.name.is_empty(), "Rule should have non-empty name");
        }
    }

    // Story 3-6: Test scan_text_for_privacy command
    #[tokio::test]
    async fn test_scan_text_for_privacy_basic() {
        let text = "My API key is sk-1234567890abcdefghij1234".to_string();
        let result = scan_text_for_privacy(text).await.unwrap();

        assert!(!result.matches.is_empty(), "Should find matches");
        assert!(result.has_critical, "Should have critical findings");
        assert_eq!(result.matches[0].sensitive_type, SensitiveType::ApiKey);
    }

    #[tokio::test]
    async fn test_scan_text_for_privacy_no_matches() {
        let text = "Hello, World!".to_string();
        let result = scan_text_for_privacy(text).await.unwrap();

        assert!(result.matches.is_empty(), "Should not find matches");
        assert!(!result.has_critical);
        assert!(!result.has_warning);
    }

    #[tokio::test]
    async fn test_scan_text_for_privacy_multiple_types() {
        let text = "API: sk-aaaaaaaaaaaaaaaaaaaaaaaa, Email: test@example.com".to_string();
        let result = scan_text_for_privacy(text).await.unwrap();

        assert!(result.stats.total >= 2, "Should find at least 2 matches");
        assert!(result.has_critical, "Should have critical (API key)");
        assert!(result.has_warning, "Should have warning (email)");
    }

    #[tokio::test]
    async fn test_scan_text_for_privacy_line_column() {
        let text = "Line 1\nLine 2 has sk-1234567890abcdefghij1234 here".to_string();
        let result = scan_text_for_privacy(text).await.unwrap();

        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].line, 2);
        assert!(result.matches[0].column > 0);
    }

    #[tokio::test]
    async fn test_scan_text_for_privacy_new_rules() {
        // Test new rules added in Story 3-6
        let text = r#"
            Email: user@example.com
            Phone: 13912345678
            ID: 110101199003076789
            password="secret123"
            -----BEGIN RSA PRIVATE KEY-----
        "#.to_string();

        let result = scan_text_for_privacy(text).await.unwrap();

        // Should find email
        assert!(result.matches.iter().any(|m| m.sensitive_type == SensitiveType::Email));
        // Should find phone
        assert!(result.matches.iter().any(|m| m.sensitive_type == SensitiveType::Phone));
        // Should find ID card
        assert!(result.matches.iter().any(|m| m.sensitive_type == SensitiveType::IdCard));
        // Should find password
        assert!(result.matches.iter().any(|m| m.sensitive_type == SensitiveType::Password));
        // Should find private key
        assert!(result.matches.iter().any(|m| m.sensitive_type == SensitiveType::PrivateKey));
    }
}
