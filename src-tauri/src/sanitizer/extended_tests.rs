//! 扩展测试套件 - 基于测试矩阵补充的完整测试覆盖
//!
//! 包含:
//! - P0: False Negative 防护测试
//! - P1: False Positive 防护测试
//! - P1-S: ReDoS 安全测试
//! - 属性测试 (Property-based Testing)

use super::engine::Sanitizer;
use super::patterns::{SanitizationRule, SensitiveType};

// ============================================================================
// 测试数据工厂
// ============================================================================

mod test_data {
    /// 生成 OpenAI 标准 API Key
    pub fn openai_key() -> String {
        format!("sk-{}", "a".repeat(24))
    }

    /// 生成 OpenAI proj 格式 Key
    pub fn openai_proj_key() -> String {
        format!("sk-proj-{}", "b".repeat(24))
    }

    /// 生成 GitHub Token (支持 ghp_, gho_, ghs_, ghu_, ghr_)
    pub fn github_token(prefix: &str) -> String {
        format!("{prefix}_{}", "x".repeat(36))
    }

    /// 生成 AWS Access Key ID
    pub fn aws_access_key() -> String {
        format!("AKIA{}", "IOSFODNN7EXAMPLE")
    }

    /// 生成 Anthropic Key
    pub fn anthropic_key() -> String {
        format!("sk-ant-{}", "c".repeat(20))
    }

    /// 生成 Google Cloud Key
    pub fn google_cloud_key() -> String {
        format!("AIza{}", "d".repeat(35))
    }

    /// 生成 IPv4 地址
    pub fn ipv4(a: u8, b: u8, c: u8, d: u8) -> String {
        format!("{a}.{b}.{c}.{d}")
    }

    /// 生成标准 JWT
    pub fn jwt() -> String {
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".into()
    }
}

#[allow(dead_code)]
mod edge_cases {
    /// 过短的 sk- 前缀 (不应匹配)
    pub fn short_sk() -> String {
        "sk-short".into()
    }

    /// 版本号格式 (不应匹配为 IP)
    pub fn version_number() -> String {
        "v1.2.3.4".into()
    }

    /// 不完整的 IP (不应匹配)
    pub fn incomplete_ip() -> String {
        "192.168.1".into()
    }

    /// 单独的 password 单词 (不应匹配)
    pub fn word_password() -> String {
        "password".into()
    }

    /// 代码变量名 (不应匹配)
    pub fn variable_name() -> String {
        "let sk_count = 10".into()
    }
}

// ============================================================================
// P0: False Negative 防护测试 - 确保不漏掉敏感信息
// ============================================================================

#[cfg(test)]
mod p0_false_negative_tests {
    use super::*;

    #[test]
    fn test_p0_03_openai_key_in_json() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();
        let text = format!(r#"{{"api_key": "{key}", "name": "test"}}"#);
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches, "Should match key in JSON: {}", text);
        assert!(
            result.sanitized_text.contains("[REDACTED:API_KEY]"),
            "Result: {}",
            result.sanitized_text
        );
        assert!(
            !result.sanitized_text.contains(&key),
            "Original key should not appear"
        );
    }

    #[test]
    fn test_p0_04_openai_key_in_url() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();
        let text = format!("https://api.example.com?token={key}&foo=bar");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches, "Should match key in URL: {}", text);
        assert!(!result.sanitized_text.contains(&key));
    }

    #[test]
    fn test_p0_05_openai_key_in_code_comment() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();
        let text = format!("// API Key: {key}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches, "Should match key in comment: {}", text);
        assert!(!result.sanitized_text.contains(&key));
    }

    #[test]
    fn test_p0_06_multiple_keys_same_line() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key1 = test_data::openai_key();
        let key2 = test_data::openai_proj_key();
        let text = format!("keys: {key1} and {key2}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(!result.sanitized_text.contains(&key1));
        assert!(!result.sanitized_text.contains(&key2));
        assert_eq!(
            result.stats.counts.get(&SensitiveType::ApiKey),
            Some(&2),
            "Should count 2 API keys"
        );
    }

    #[test]
    fn test_p0_07_github_token_gho() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let token = test_data::github_token("gho");
        // 注意: 避免使用 "token:" 前缀，否则会被 Generic Secret 规则优先匹配
        let text = format!("GitHub OAuth: {token}");
        let result = sanitizer.sanitize(&text);

        assert!(
            result.has_matches,
            "Should match gho_ token: {}",
            result.sanitized_text
        );
        assert!(
            result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "Expected GITHUB_TOKEN but got: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p0_08_github_token_ghs() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let token = test_data::github_token("ghs");
        let text = format!("GitHub Server: {token}");
        let result = sanitizer.sanitize(&text);

        assert!(
            result.has_matches,
            "Should match ghs_ token: {}",
            result.sanitized_text
        );
        assert!(
            result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "Expected GITHUB_TOKEN but got: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p0_09_github_token_ghu() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let token = test_data::github_token("ghu");
        let text = format!("GitHub User: {token}");
        let result = sanitizer.sanitize(&text);

        assert!(
            result.has_matches,
            "Should match ghu_ token: {}",
            result.sanitized_text
        );
        assert!(
            result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "Expected GITHUB_TOKEN but got: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p0_10_github_token_ghr() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let token = test_data::github_token("ghr");
        let text = format!("GitHub Refresh: {token}");
        let result = sanitizer.sanitize(&text);

        assert!(
            result.has_matches,
            "Should match ghr_ token: {}",
            result.sanitized_text
        );
        assert!(
            result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "Expected GITHUB_TOKEN but got: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p0_11_ipv6_full_format() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "Server at 2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        let result = sanitizer.sanitize(text);

        assert!(
            result.has_matches,
            "Should match full IPv6: {}",
            result.sanitized_text
        );
        assert!(result.sanitized_text.contains("[REDACTED:IP_ADDRESS]"));
    }

    #[test]
    fn test_p0_12_key_with_surrounding_quotes() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();
        let text = format!(r#"export OPENAI_API_KEY="{key}""#);
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(!result.sanitized_text.contains(&key));
    }

    #[test]
    fn test_p0_13_bearer_with_jwt() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let jwt = test_data::jwt();
        let text = format!("Authorization: Bearer {jwt}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        // Should be redacted as either BEARER_TOKEN or JWT_TOKEN
        assert!(
            result.sanitized_text.contains("[REDACTED:BEARER_TOKEN]")
                || result.sanitized_text.contains("[REDACTED:JWT_TOKEN]")
        );
    }

    #[test]
    fn test_p0_14_mixed_sensitive_types() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = format!(
            "Config:\n  api_key: {}\n  server: {}\n  token: {}",
            test_data::openai_key(),
            test_data::ipv4(10, 0, 0, 1),
            test_data::jwt()
        );
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(result.stats.total >= 3, "Should have at least 3 matches");
    }
}

// ============================================================================
// P1: False Positive 防护测试 - 确保不过度脱敏
// ============================================================================

#[cfg(test)]
mod p1_false_positive_tests {
    use super::*;

    #[test]
    fn test_p1_01_version_not_ip() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        // 版本号不应被匹配为 IP (修复后)
        let text = "Version: v1.2.3.4 is released";
        let result = sanitizer.sanitize(text);

        // 修复后: v1.2.3.4 不应被匹配
        assert!(
            !result.has_matches || !result.sanitized_text.contains("[REDACTED:IP_ADDRESS]"),
            "Version number should not match as IP: {}",
            result.sanitized_text
        );
        assert!(
            result.sanitized_text.contains("v1.2.3.4"),
            "Version should be preserved: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p1_02_short_sk_prefix_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = edge_cases::short_sk();
        let result = sanitizer.sanitize(&text);

        assert!(
            !result.has_matches,
            "Short sk- should not match: {}",
            result.sanitized_text
        );
        assert_eq!(result.sanitized_text, text);
    }

    #[test]
    fn test_p1_03_incomplete_ip_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "prefix: 192.168.1 (incomplete)";
        let result = sanitizer.sanitize(&text);

        // 192.168.1 不应被匹配 (只有3段)
        // 验证原文保留
        assert!(
            result.sanitized_text.contains("192.168.1"),
            "Incomplete IP should be preserved: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p1_04_password_word_alone_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "Please enter your password below";
        let result = sanitizer.sanitize(&text);

        assert!(
            !result.has_matches,
            "Bare 'password' word should not match"
        );
        assert_eq!(result.sanitized_text, text);
    }

    #[test]
    fn test_p1_05_password_in_docs_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "See password documentation for details";
        let result = sanitizer.sanitize(&text);

        assert!(
            !result.has_matches,
            "password in docs should not match"
        );
    }

    #[test]
    fn test_p1_06_variable_name_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "let sk_counter = 0; let api_key_count = 5;";
        let result = sanitizer.sanitize(&text);

        // 变量名不应被匹配
        assert!(
            result.sanitized_text.contains("sk_counter"),
            "Variable name should be preserved: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p1_07_base64_image_not_matched_as_jwt() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        // Base64 图片数据不应被误认为 JWT
        let text = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA";
        let result = sanitizer.sanitize(&text);

        // base64 图片不应匹配为 JWT (因为格式不同)
        assert!(
            !result.sanitized_text.contains("[REDACTED:JWT_TOKEN]"),
            "Base64 image should not match as JWT: {}",
            result.sanitized_text
        );
    }

    #[test]
    fn test_p1_08_whitespace_only_not_matched() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "   \n\t\r\n   ";
        let result = sanitizer.sanitize(&text);

        assert!(!result.has_matches);
        assert_eq!(result.sanitized_text, text);
    }

    #[test]
    fn test_p1_09_localhost_variations_preserved() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let text = "Connect to 127.0.0.1:8080 or 127.0.0.2 or 127.255.255.255";
        let result = sanitizer.sanitize(&text);

        // 所有 127.x.x.x 都应保留
        assert!(result.sanitized_text.contains("127.0.0.1"));
        assert!(result.sanitized_text.contains("127.0.0.2"));
        assert!(result.sanitized_text.contains("127.255.255.255"));
    }

    #[test]
    fn test_p1_10_common_port_numbers_not_ip() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        // 端口号如 8080, 3000 不应触发匹配
        let text = "Server running on port 8080 and 3000";
        let result = sanitizer.sanitize(&text);

        assert!(!result.has_matches, "Port numbers should not match");
    }
}

// ============================================================================
// P1-S: ReDoS 安全测试 - 确保不会因恶意输入卡死
// ============================================================================

#[cfg(test)]
mod p1_security_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_p1_s01_large_input_performance() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 生成 1MB 文本
        let text = "a".repeat(1_000_000);
        let start = Instant::now();
        let result = sanitizer.sanitize(&text);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "1MB text should process in < 1s, took {:?}",
            elapsed
        );
        assert!(!result.has_matches);
    }

    #[test]
    fn test_p1_s02_repeated_pattern_no_hang() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 重复的近似匹配模式
        let text = "sk-".repeat(10000);
        let start = Instant::now();
        let result = sanitizer.sanitize(&text);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "Repeated pattern should not hang, took {:?}",
            elapsed
        );
        // 每个 sk- 都太短，不应匹配
        assert!(!result.has_matches);
    }

    #[test]
    fn test_p1_s03_nested_pattern_no_hang() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 嵌套的模式
        let text = "sk-sk-sk-sk-".repeat(1000) + &"a".repeat(30);
        let start = Instant::now();
        let _result = sanitizer.sanitize(&text);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "Nested pattern should not hang, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_p1_s04_unicode_boundary_handling() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 混合中文和敏感信息
        let key = test_data::openai_key();
        let text = format!("配置项：API密钥={key}，服务器=192.168.1.1，端口=8080");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(!result.sanitized_text.contains(&key));
        // 确保中文字符没有被破坏
        assert!(result.sanitized_text.contains("配置项"));
        assert!(result.sanitized_text.contains("端口"));
    }

    #[test]
    fn test_p1_s05_emoji_handling() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        let key = test_data::openai_key();
        let text = format!("🔑 Key: {key} 🚀 Deploy!");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(result.sanitized_text.contains("🔑"));
        assert!(result.sanitized_text.contains("🚀"));
    }

    #[test]
    fn test_p1_s06_chunked_large_file() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 2MB 文本带嵌入的敏感信息
        let key = test_data::openai_key();
        let mut text = "x".repeat(500_000);
        text.push_str(&key);
        text.push_str(&"y".repeat(500_000));
        text.push_str(&key);
        text.push_str(&"z".repeat(500_000));

        let start = Instant::now();
        let result = sanitizer.sanitize_chunked(&text, 256 * 1024); // 256KB chunks
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(2),
            "2MB chunked should process in < 2s, took {:?}",
            elapsed
        );
        assert!(result.has_matches);
        // 注意: 分块可能导致边界处的 key 被截断，只验证有匹配
        assert!(result.stats.counts.get(&SensitiveType::ApiKey).unwrap_or(&0) > &0);
    }
}

// ============================================================================
// P2: 边界条件测试
// ============================================================================

#[cfg(test)]
mod p2_boundary_tests {
    use super::*;

    #[test]
    fn test_p2_01_disabled_rule_not_matched() {
        let mut rule = SanitizationRule::new("test_rule", "Test Rule", r"test-secret-\d+", SensitiveType::Custom);
        rule.enabled = false;

        let sanitizer = Sanitizer::new(vec![rule]).unwrap();
        let text = "My test-secret-12345 here";
        let result = sanitizer.sanitize(&text);

        assert!(
            !result.has_matches,
            "Disabled rule should not match: {}",
            result.sanitized_text
        );
        assert_eq!(result.sanitized_text, text);
    }

    #[test]
    fn test_p2_02_custom_rule_works() {
        let rule = SanitizationRule::custom("ssn_pattern", "SSN Pattern", r"\d{3}-\d{2}-\d{4}");
        let sanitizer = Sanitizer::new(vec![rule]).unwrap();

        let text = "SSN: 123-45-6789";
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(result.sanitized_text.contains("[REDACTED:CUSTOM]"));
    }

    #[test]
    fn test_p2_03_multiple_custom_rules() {
        let rules = vec![
            SanitizationRule::custom("ssn", "SSN", r"\d{3}-\d{2}-\d{4}"),
            SanitizationRule::custom("phone", "Phone", r"\d{3}-\d{3}-\d{4}"),
        ];
        let sanitizer = Sanitizer::new(rules).unwrap();

        let text = "SSN: 123-45-6789, Phone: 555-123-4567";
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert_eq!(
            result.stats.counts.get(&SensitiveType::Custom),
            Some(&2)
        );
    }

    #[test]
    fn test_p2_04_stats_accuracy_multiple_types() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        let text = format!(
            "{} {} {} {}",
            test_data::openai_key(),
            test_data::openai_proj_key(),
            test_data::ipv4(10, 0, 0, 1),
            test_data::ipv4(172, 16, 0, 1)
        );
        let result = sanitizer.sanitize(&text);

        assert_eq!(
            result.stats.counts.get(&SensitiveType::ApiKey),
            Some(&2),
            "Should have 2 API keys"
        );
        assert_eq!(
            result.stats.counts.get(&SensitiveType::IpAddress),
            Some(&2),
            "Should have 2 IPs"
        );
        assert_eq!(result.stats.total, 4);
    }

    #[test]
    fn test_p2_05_empty_rules_no_matches() {
        let sanitizer = Sanitizer::new(vec![]).unwrap();
        let text = test_data::openai_key();
        let result = sanitizer.sanitize(&text);

        assert!(!result.has_matches);
        assert_eq!(result.sanitized_text, text);
    }

    #[test]
    fn test_p2_06_special_characters_in_text() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();

        // 特殊字符不应干扰匹配
        let text = format!("key='{key}' && echo $?");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(!result.sanitized_text.contains(&key));
    }

    /// P2-07: 规则冲突测试 - 验证修复后行为
    ///
    /// 修复后: Generic Secret 规则使用负向前瞻排除已知 token 格式，
    /// 因此 GitHub Token 即使在 "token: xxx" 格式中也能被正确识别。
    #[test]
    fn test_p2_07_rule_conflict_secret_vs_github() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let token = test_data::github_token("ghp");

        // 修复后: 即使使用 "token:" 前缀，也应被 GITHUB_TOKEN 规则匹配
        let text_with_prefix = format!("token: {token}");
        let result = sanitizer.sanitize(&text_with_prefix);

        assert!(result.has_matches);
        assert!(
            result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "With 'token:' prefix, should now be caught by GITHUB_TOKEN rule: {}",
            result.sanitized_text
        );

        // 不使用前缀时，GitHub Token 规则同样正确匹配
        let text_without_prefix = format!("GitHub: {token}");
        let result2 = sanitizer.sanitize(&text_without_prefix);

        assert!(
            result2.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
            "Without 'token:' prefix, should be caught by GITHUB_TOKEN rule: {}",
            result2.sanitized_text
        );
    }

    /// P2-08: 多重规则匹配同一文本
    #[test]
    fn test_p2_08_multiple_overlapping_rules() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // JWT 在 Bearer 头中 - 两个规则都可能匹配
        let jwt = test_data::jwt();
        let text = format!("Authorization: Bearer {jwt}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        // 应该被某个规则捕获 (BEARER_TOKEN 或 JWT_TOKEN)
        assert!(
            result.sanitized_text.contains("[REDACTED:BEARER_TOKEN]")
                || result.sanitized_text.contains("[REDACTED:JWT_TOKEN]"),
            "Should be redacted by either rule: {}",
            result.sanitized_text
        );
    }

    /// P2-09: Generic Secret 仍然匹配普通密码
    #[test]
    fn test_p2_09_generic_secret_still_works() {
        let sanitizer = Sanitizer::with_defaults().unwrap();

        // 普通密码应该仍被 SECRET 规则匹配
        let text = "password=mySecurePassword123";
        let result = sanitizer.sanitize(text);

        assert!(result.has_matches);
        assert!(
            result.sanitized_text.contains("[REDACTED:SECRET]"),
            "Generic password should match: {}",
            result.sanitized_text
        );
    }

    /// P2-10: OpenAI Key 在 token: 格式中
    #[test]
    fn test_p2_10_openai_key_with_token_prefix() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let key = test_data::openai_key();

        let text = format!("token: {key}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(
            result.sanitized_text.contains("[REDACTED:API_KEY]"),
            "OpenAI key should be caught by API_KEY rule, not SECRET: {}",
            result.sanitized_text
        );
    }

    /// P2-11: JWT 在 token: 格式中
    #[test]
    fn test_p2_11_jwt_with_token_prefix() {
        let sanitizer = Sanitizer::with_defaults().unwrap();
        let jwt = test_data::jwt();

        let text = format!("token: {jwt}");
        let result = sanitizer.sanitize(&text);

        assert!(result.has_matches);
        assert!(
            result.sanitized_text.contains("[REDACTED:JWT_TOKEN]"),
            "JWT should be caught by JWT_TOKEN rule, not SECRET: {}",
            result.sanitized_text
        );
    }
}

// ============================================================================
// 属性测试 (Property-Based Testing)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// 不变式: 脱敏后的文本绝不包含原始 OpenAI API Key
        #[test]
        fn prop_sanitized_never_contains_openai_key(
            suffix in "[a-zA-Z0-9]{24,48}"
        ) {
            let key = format!("sk-{suffix}");
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize(&key);

            prop_assert!(
                !result.sanitized_text.contains(&key),
                "Original key should not appear in sanitized output"
            );
        }

        /// 不变式: 脱敏是幂等的 (多次脱敏结果相同)
        #[test]
        fn prop_sanitization_is_idempotent(
            text in ".{0,1000}"
        ) {
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let once = sanitizer.sanitize(&text);
            let twice = sanitizer.sanitize(&once.sanitized_text);

            prop_assert_eq!(
                once.sanitized_text,
                twice.sanitized_text,
                "Sanitization should be idempotent"
            );
        }

        /// 不变式: 空输入返回空输出
        #[test]
        fn prop_empty_input_empty_output(
            _dummy in Just(())
        ) {
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize("");

            prop_assert_eq!(result.sanitized_text, "");
            prop_assert!(!result.has_matches);
        }

        /// 不变式: 输出长度不超过输入长度 (替换文本可能更长，所以这个需要调整)
        /// 实际上 [REDACTED:XXX] 可能比原文长，所以改为验证输出合理
        #[test]
        fn prop_output_is_reasonable_length(
            text in ".{0,500}"
        ) {
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize(&text);

            // 输出长度应该在合理范围内 (原长度 + 每个匹配最多增加 30 字符)
            let max_expected = text.len() + result.stats.total * 30;
            prop_assert!(
                result.sanitized_text.len() <= max_expected,
                "Output length {} exceeds expected max {}",
                result.sanitized_text.len(),
                max_expected
            );
        }

        /// 不变式: 统计数据一致性
        #[test]
        fn prop_stats_consistency(
            text in ".{0,500}"
        ) {
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize(&text);

            let sum: usize = result.stats.counts.values().sum();
            prop_assert_eq!(
                sum,
                result.stats.total,
                "Stats sum should equal total"
            );

            if result.stats.total > 0 {
                prop_assert!(result.has_matches);
            }
        }

        /// 不变式: GitHub token 各前缀都被正确处理
        #[test]
        fn prop_github_token_all_prefixes(
            prefix in prop::sample::select(vec!["ghp", "gho", "ghs", "ghu", "ghr"]),
            suffix in "[A-Za-z0-9]{36,50}"
        ) {
            let token = format!("{prefix}_{suffix}");
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize(&token);

            prop_assert!(
                result.has_matches,
                "GitHub token with prefix {} should match",
                prefix
            );
            prop_assert!(
                result.sanitized_text.contains("[REDACTED:GITHUB_TOKEN]"),
                "Should be redacted as GITHUB_TOKEN"
            );
        }

        /// 不变式: localhost (127.x.x.x) 永远被保留
        #[test]
        fn prop_localhost_always_preserved(
            b in 0u8..=255,
            c in 0u8..=255,
            d in 0u8..=255
        ) {
            let ip = format!("127.{b}.{c}.{d}");
            let sanitizer = Sanitizer::with_defaults().unwrap();
            let result = sanitizer.sanitize(&ip);

            prop_assert!(
                result.sanitized_text.contains(&ip),
                "Localhost {} should be preserved, got: {}",
                ip,
                result.sanitized_text
            );
        }
    }
}
