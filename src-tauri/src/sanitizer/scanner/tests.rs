use super::*;

#[test]
fn test_scanner_creation() {
    let scanner = PrivacyScanner::with_defaults();
    assert!(scanner.is_ok());
}

#[test]
fn test_empty_text() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let result = scanner.scan("");
    assert!(result.matches.is_empty());
    assert!(!result.has_critical);
    assert!(!result.has_warning);
}

#[test]
fn test_no_matches() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let result = scanner.scan("Hello, World!");
    assert!(result.matches.is_empty());
    assert_eq!(result.stats.total, 0);
}

#[test]
fn test_openai_key_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "My API key is sk-1234567890abcdefghij1234";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_critical);
    assert_eq!(result.matches[0].sensitive_type, SensitiveType::ApiKey);
    assert_eq!(result.matches[0].severity, Severity::Critical);
    assert_eq!(result.matches[0].line, 1);
}

#[test]
fn test_email_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "Contact: user@example.com";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_warning);

    let email_match = result.matches.iter().find(|m| m.sensitive_type == SensitiveType::Email);
    assert!(email_match.is_some());
    assert_eq!(email_match.unwrap().masked_text, "u****@example.com");
}

#[test]
fn test_phone_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "手机号: 13812345678";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_warning);

    let phone_match = result.matches.iter().find(|m| m.sensitive_type == SensitiveType::Phone);
    assert!(phone_match.is_some());
    assert_eq!(phone_match.unwrap().masked_text, "138****5678");
}

#[test]
fn test_id_card_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "身份证: 110101199003076789";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_critical);

    let id_match = result.matches.iter().find(|m| m.sensitive_type == SensitiveType::IdCard);
    assert!(id_match.is_some());
    assert_eq!(id_match.unwrap().masked_text, "110101****6789");
}

#[test]
fn test_private_key_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpA...";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_critical);

    let key_match = result.matches.iter().find(|m| m.sensitive_type == SensitiveType::PrivateKey);
    assert!(key_match.is_some());
}

#[test]
fn test_password_detection() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = r#"password="mySecret123""#;
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.has_critical);

    let pwd_match = result.matches.iter().find(|m| m.sensitive_type == SensitiveType::Password);
    assert!(pwd_match.is_some());
}

#[test]
fn test_line_column_calculation() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "Line 1\nLine 2 with sk-1234567890abcdefghij1234";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert_eq!(result.matches[0].line, 2);
    // "Line 2 with " = 12 chars, key starts at column 13 (1-based)
    assert_eq!(result.matches[0].column, 13);
}

#[test]
fn test_context_extraction() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "Some prefix text sk-1234567890abcdefghij1234 some suffix text";
    let result = scanner.scan(text);

    assert!(!result.matches.is_empty());
    assert!(result.matches[0].context.contains("prefix"));
    assert!(result.matches[0].context.contains("suffix"));
}

#[test]
fn test_localhost_preserved() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "Server at 127.0.0.1:8080";
    let result = scanner.scan(text);

    // localhost 应该不被检测
    let ip_matches: Vec<_> = result.matches.iter()
        .filter(|m| m.sensitive_type == SensitiveType::IpAddress)
        .collect();
    assert!(ip_matches.is_empty());
}

#[test]
fn test_version_not_matched() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "Version v1.2.3.4 released";
    let result = scanner.scan(text);

    // 版本号不应被检测为 IP
    let ip_matches: Vec<_> = result.matches.iter()
        .filter(|m| m.sensitive_type == SensitiveType::IpAddress)
        .collect();
    assert!(ip_matches.is_empty());
}

#[test]
fn test_multiple_matches() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "API: sk-aaaaaaaaaaaaaaaaaaaaaaaa, Email: test@example.com, Phone: 13912345678";
    let result = scanner.scan(text);

    assert!(result.stats.total >= 3);
    assert!(result.has_critical); // API key
    assert!(result.has_warning); // Email, Phone
}

#[test]
fn test_stats_accuracy() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "sk-aaaaaaaaaaaaaaaaaaaaaaaa sk-bbbbbbbbbbbbbbbbbbbbbbbb";
    let result = scanner.scan(text);

    assert_eq!(result.stats.total, 2);
    assert_eq!(result.stats.critical_count, 2);
    assert_eq!(*result.stats.by_type.get("API_KEY").unwrap_or(&0), 2);
}

#[test]
fn test_unicode_handling() {
    let scanner = PrivacyScanner::with_defaults().unwrap();
    let text = "配置: sk-中文测试1234567890abcd 结束";
    let result = scanner.scan(text);

    // 应该正确处理 UTF-8
    for m in &result.matches {
        assert!(m.line > 0);
        assert!(m.column > 0);
    }
}
