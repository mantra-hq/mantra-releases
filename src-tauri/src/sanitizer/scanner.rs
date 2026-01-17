//! 隐私扫描引擎
//!
//! 提供敏感信息扫描检测功能，返回匹配详情供用户决策。
//! 与 Sanitizer (脱敏引擎) 不同，Scanner 不修改原文，只报告检测结果。

use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use super::error::SanitizerError;
use super::patterns::{SanitizationRule, SensitiveType, Severity, BUILTIN_RULES};

/// 扫描匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    /// 规则 ID
    pub rule_id: String,
    /// 敏感信息类型
    pub sensitive_type: SensitiveType,
    /// 严重程度
    pub severity: Severity,
    /// 行号 (1-based)
    pub line: usize,
    /// 列号 (1-based)
    pub column: usize,
    /// 原始匹配文本
    pub matched_text: String,
    /// 脱敏显示文本 (如 sk-****xxxx)
    pub masked_text: String,
    /// 上下文片段
    pub context: String,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 所有匹配项
    pub matches: Vec<ScanMatch>,
    /// 是否包含 Critical 级别匹配
    pub has_critical: bool,
    /// 是否包含 Warning 级别匹配
    pub has_warning: bool,
    /// 扫描耗时 (毫秒)
    pub scan_time_ms: u64,
    /// 按严重程度分组的统计
    pub stats: ScanStats,
}

/// 扫描统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanStats {
    /// Critical 数量
    pub critical_count: usize,
    /// Warning 数量
    pub warning_count: usize,
    /// Info 数量
    pub info_count: usize,
    /// 总匹配数
    pub total: usize,
    /// 按类型统计
    pub by_type: HashMap<String, usize>,
}

/// 隐私扫描器
pub struct PrivacyScanner {
    /// 规则列表
    rules: Vec<SanitizationRule>,
    /// 编译后的正则表达式
    regexes: Vec<Regex>,
    /// RegexSet 用于批量匹配
    regex_set: RegexSet,
}

impl PrivacyScanner {
    /// 使用指定规则创建扫描器
    pub fn new(rules: Vec<SanitizationRule>) -> Result<Self, SanitizerError> {
        // 只保留启用的规则
        let enabled_rules: Vec<_> = rules.into_iter().filter(|r| r.enabled).collect();

        if enabled_rules.is_empty() {
            return Ok(Self {
                rules: vec![],
                regexes: vec![],
                regex_set: RegexSet::new::<_, &str>([])?,
            });
        }

        // 编译每个正则
        let mut regexes = Vec::with_capacity(enabled_rules.len());
        let mut patterns = Vec::with_capacity(enabled_rules.len());

        for rule in &enabled_rules {
            let regex = Regex::new(&rule.pattern)?;
            regexes.push(regex);
            patterns.push(rule.pattern.as_str());
        }

        // 创建 RegexSet 用于批量匹配
        let regex_set = RegexSet::new(&patterns)?;

        Ok(Self {
            rules: enabled_rules,
            regexes,
            regex_set,
        })
    }

    /// 使用默认规则创建扫描器
    pub fn with_defaults() -> Result<Self, SanitizerError> {
        Self::new(BUILTIN_RULES.clone())
    }

    /// 扫描文本内容
    pub fn scan(&self, text: &str) -> ScanResult {
        let start = Instant::now();
        let mut matches = Vec::new();
        let mut stats = ScanStats::default();

        if self.rules.is_empty() || text.is_empty() {
            return ScanResult {
                matches,
                has_critical: false,
                has_warning: false,
                scan_time_ms: start.elapsed().as_millis() as u64,
                stats,
            };
        }

        // 使用 RegexSet 快速确定哪些规则可能匹配
        let matching_indices: Vec<_> = self.regex_set.matches(text).into_iter().collect();

        // 只对匹配的规则进行详细处理
        for idx in matching_indices {
            let rule = &self.rules[idx];
            let regex = &self.regexes[idx];

            for mat in regex.find_iter(text) {
                let matched_text = mat.as_str().to_string();

                // 跳过 localhost IP (127.x.x.x)
                if rule.sensitive_type == SensitiveType::IpAddress
                    && matched_text.starts_with("127.")
                {
                    continue;
                }

                // 跳过版本号 (如 v1.2.3.4)
                if rule.sensitive_type == SensitiveType::IpAddress {
                    let start_byte = mat.start();
                    if start_byte > 0 {
                        let prev_char = text[..start_byte].chars().last();
                        if prev_char == Some('v') || prev_char == Some('V') {
                            continue;
                        }
                    }
                }

                let (line, column) = calculate_position(text, mat.start());
                let context = extract_context(text, mat.start(), mat.end(), 50);
                let masked_text = mask_text(&matched_text, &rule.sensitive_type);

                let scan_match = ScanMatch {
                    rule_id: rule.id.clone(),
                    sensitive_type: rule.sensitive_type,
                    severity: rule.severity,
                    line,
                    column,
                    matched_text,
                    masked_text,
                    context,
                };

                // 更新统计
                match rule.severity {
                    Severity::Critical => stats.critical_count += 1,
                    Severity::Warning => stats.warning_count += 1,
                    Severity::Info => stats.info_count += 1,
                }
                *stats
                    .by_type
                    .entry(rule.sensitive_type.as_str().to_string())
                    .or_insert(0) += 1;
                stats.total += 1;

                matches.push(scan_match);
            }
        }

        // 按行号、列号排序
        matches.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        ScanResult {
            has_critical: stats.critical_count > 0,
            has_warning: stats.warning_count > 0,
            scan_time_ms: start.elapsed().as_millis() as u64,
            matches,
            stats,
        }
    }
}

/// 计算行号和列号 (1-based)
fn calculate_position(text: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;

    for (i, c) in text[..byte_offset].char_indices() {
        if c == '\n' {
            line += 1;
            line_start = i + 1;
        }
    }

    // 计算 UTF-8 安全的列号
    let column = text[line_start..byte_offset].chars().count() + 1;
    (line, column)
}

/// 提取上下文片段
fn extract_context(text: &str, start: usize, end: usize, context_size: usize) -> String {
    // 找到安全的 UTF-8 边界
    let context_start = text[..start]
        .char_indices()
        .rev()
        .nth(context_size)
        .map(|(i, _)| i)
        .unwrap_or(0);

    let context_end = text[end..]
        .char_indices()
        .nth(context_size)
        .map(|(i, _)| end + i)
        .unwrap_or(text.len());

    let mut context = String::new();

    if context_start > 0 {
        context.push_str("...");
    }
    context.push_str(&text[context_start..context_end]);
    if context_end < text.len() {
        context.push_str("...");
    }

    // 替换换行符为空格，使上下文更易读
    context.replace('\n', " ").replace('\r', "")
}

/// 脱敏显示函数
fn mask_text(matched: &str, sensitive_type: &SensitiveType) -> String {
    if matched.len() <= 8 {
        return "****".to_string();
    }

    // 辅助函数：获取后 n 个字符
    fn get_suffix(s: &str, n: usize) -> String {
        let char_count = s.chars().count();
        s.chars().skip(char_count.saturating_sub(n)).collect()
    }

    match sensitive_type {
        SensitiveType::ApiKey | SensitiveType::AnthropicKey => {
            // sk-****xxxx (保留前3个字符和后4个字符)
            let prefix: String = matched.chars().take(3).collect();
            let suffix = get_suffix(matched, 4);
            format!("{}****{}", prefix, suffix)
        }
        SensitiveType::Email => {
            // a****@example.com (仅显示首字母和域名)
            if let Some(at_pos) = matched.find('@') {
                let first_char: String = matched.chars().take(1).collect();
                format!("{}****{}", first_char, &matched[at_pos..])
            } else {
                "****".to_string()
            }
        }
        SensitiveType::Phone => {
            // 138****1234 (中间隐藏)
            let prefix: String = matched.chars().take(3).collect();
            let suffix = get_suffix(matched, 4);
            format!("{}****{}", prefix, suffix)
        }
        SensitiveType::IdCard => {
            // 前6位 + **** + 后4位
            let prefix: String = matched.chars().take(6).collect();
            let suffix = get_suffix(matched, 4);
            format!("{}****{}", prefix, suffix)
        }
        SensitiveType::PrivateKey => {
            // 只显示类型标识
            "-----BEGIN ****-----".to_string()
        }
        SensitiveType::Password => {
            // 完全隐藏密码值
            if matched.contains('=') {
                let parts: Vec<&str> = matched.splitn(2, '=').collect();
                if parts.len() == 2 {
                    return format!("{}=****", parts[0]);
                }
            }
            if matched.contains(':') {
                let parts: Vec<&str> = matched.splitn(2, ':').collect();
                if parts.len() == 2 {
                    return format!("{}:****", parts[0]);
                }
            }
            "****".to_string()
        }
        _ => {
            // 默认：首尾各保留一部分
            let char_count = matched.chars().count();
            let prefix_len = std::cmp::min(3, char_count / 4);
            let suffix_len = std::cmp::min(4, char_count / 4);

            let prefix: String = matched.chars().take(prefix_len).collect();
            let suffix = get_suffix(matched, suffix_len);
            format!("{}****{}", prefix, suffix)
        }
    }
}

#[cfg(test)]
mod scanner_tests {
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
}
