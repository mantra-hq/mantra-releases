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

    /// 使用配置目录中的规则创建扫描器
    ///
    /// 从配置目录加载规则配置，合并内置规则和自定义规则
    pub fn with_config(config_dir: &std::path::Path) -> Result<Self, SanitizerError> {
        use super::config::PrivacyRulesConfig;

        let config = PrivacyRulesConfig::load(config_dir)?;
        let rules = config.get_merged_rules();
        Self::new(rules)
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
mod tests;
