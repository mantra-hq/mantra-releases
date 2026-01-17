//! Sanitizer 模块集成测试

use super::*;

#[test]
fn test_module_exports() {
    // 验证公共 API 可访问
    let _type = SensitiveType::ApiKey;
    let _rule = SanitizationRule::new("test", "test", r"test", SensitiveType::Custom);
    let sanitizer = Sanitizer::with_defaults();
    assert!(sanitizer.is_ok());
}

#[test]
fn test_builtin_rules_accessible() {
    assert!(!BUILTIN_RULES.is_empty());
}

#[test]
fn test_end_to_end_sanitization() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    
    let input = r#"
        API Key: sk-1234567890abcdefghij1234
        GitHub: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1234
        AWS: AKIAIOSFODNN7EXAMPLE
        Server: 192.168.1.100
        JWT: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123
        password=secretValue123
    "#;
    
    let result = sanitizer.sanitize(input);
    
    assert!(result.has_matches);
    assert!(result.sanitized_text.contains("[REDACTED:API_KEY]"));
    assert!(result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"));
    assert!(result.sanitized_text.contains("[REDACTED:AWS_KEY]"));
    assert!(result.sanitized_text.contains("[REDACTED:IP_ADDRESS]"));
    assert!(result.sanitized_text.contains("[REDACTED:JWT_TOKEN]"));
    assert!(result.sanitized_text.contains("[REDACTED:SECRET]"));
    
    // 验证统计 (GitHub token + OpenAI key + AWS key + IP + JWT + Secret = 6)
    assert!(result.stats.total >= 5, "Expected at least 5 matches, got {}", result.stats.total);
}

#[test]
fn test_anthropic_key() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    // Anthropic key: sk-ant- + 20+ alphanumeric/hyphen chars (exactly 20 a's)
    // 避免使用 KEY= 格式，否则会被 Generic Secret 规则匹配
    let text = "My Anthropic key is sk-ant-aaaaaaaaaaaaaaaaaaaa here";
    let result = sanitizer.sanitize(text);
    assert!(result.has_matches, "Text: {}, Result: {}", text, result.sanitized_text);
    assert!(result.sanitized_text.contains("[REDACTED:ANTHROPIC_KEY]"));
}

#[test]
fn test_google_cloud_key() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    // Google Cloud key: AIza + exactly 35 alphanumeric/hyphen/underscore chars
    // 避免使用 KEY= 格式
    let text = "My Google key is AIzaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa here";
    let result = sanitizer.sanitize(text);
    assert!(result.has_matches, "Text: {}, Result: {}", text, result.sanitized_text);
    assert!(result.sanitized_text.contains("[REDACTED:GOOGLE_CLOUD_KEY]"));
}

#[test]
fn test_bearer_token() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.xxx";
    let result = sanitizer.sanitize(text);
    assert!(result.has_matches);
    // Bearer token or JWT should be redacted
    assert!(
        result.sanitized_text.contains("[REDACTED:BEARER_TOKEN]")
            || result.sanitized_text.contains("[REDACTED:JWT_TOKEN]")
    );
}

#[test]
fn test_empty_text() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    let result = sanitizer.sanitize("");
    assert!(!result.has_matches);
    assert_eq!(result.sanitized_text, "");
}

#[test]
fn test_stats_accuracy() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    let text = "sk-aaaaaaaaaaaaaaaaaaaaaaaa sk-bbbbbbbbbbbbbbbbbbbbbbbb";
    let result = sanitizer.sanitize(text);
    assert_eq!(result.stats.counts.get(&SensitiveType::ApiKey), Some(&2));
    assert_eq!(result.stats.total, 2);
}

#[test]
fn test_ipv6_compressed() {
    let sanitizer = Sanitizer::with_defaults().unwrap();
    // 测试压缩格式 IPv6
    let text = "Server at 2001:db8::1 and ::1";
    let result = sanitizer.sanitize(text);
    assert!(result.has_matches, "Text: {}, Result: {}", text, result.sanitized_text);
    assert!(result.sanitized_text.contains("[REDACTED:IP_ADDRESS]"));
}
