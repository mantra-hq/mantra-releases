//! 内置正则规则定义
//!
//! 包含常见敏感信息类型的识别模式

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 敏感信息类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveType {
    /// OpenAI API Key
    ApiKey,
    /// AWS Access Key
    AwsKey,
    /// GitHub Token
    GithubToken,
    /// Anthropic API Key
    AnthropicKey,
    /// Google Cloud API Key
    GoogleCloudKey,
    /// IP 地址
    IpAddress,
    /// Bearer Token
    BearerToken,
    /// JWT Token
    JwtToken,
    /// 通用 Secret/Password
    Secret,
    /// 用户自定义规则
    Custom,
}

impl SensitiveType {
    /// 获取显示名称 (用于 [REDACTED:xxx] 格式)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "API_KEY",
            Self::AwsKey => "AWS_KEY",
            Self::GithubToken => "GITHUB_TOKEN",
            Self::AnthropicKey => "ANTHROPIC_KEY",
            Self::GoogleCloudKey => "GOOGLE_CLOUD_KEY",
            Self::IpAddress => "IP_ADDRESS",
            Self::BearerToken => "BEARER_TOKEN",
            Self::JwtToken => "JWT_TOKEN",
            Self::Secret => "SECRET",
            Self::Custom => "CUSTOM",
        }
    }
}

impl fmt::Display for SensitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 脱敏规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationRule {
    /// 规则名称
    pub name: String,
    /// 正则表达式模式
    pub pattern: String,
    /// 敏感信息类型
    pub sensitive_type: SensitiveType,
    /// 是否启用
    pub enabled: bool,
}

impl SanitizationRule {
    /// 创建新规则
    pub fn new(name: impl Into<String>, pattern: impl Into<String>, sensitive_type: SensitiveType) -> Self {
        Self {
            name: name.into(),
            pattern: pattern.into(),
            sensitive_type,
            enabled: true,
        }
    }

    /// 创建自定义规则
    pub fn custom(name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(name, pattern, SensitiveType::Custom)
    }
}

/// 内置脱敏规则集
pub static BUILTIN_RULES: Lazy<Vec<SanitizationRule>> = Lazy::new(|| {
    vec![
        // OpenAI API Key: sk-xxx 或 sk-proj-xxx
        SanitizationRule::new(
            "OpenAI API Key",
            r"sk-(?:proj-)?[a-zA-Z0-9]{20,}",
            SensitiveType::ApiKey,
        ),
        // AWS Access Key ID
        SanitizationRule::new(
            "AWS Access Key ID",
            r"AKIA[0-9A-Z]{16}",
            SensitiveType::AwsKey,
        ),
        // AWS Secret Access Key (40 char base64-like)
        SanitizationRule::new(
            "AWS Secret Access Key",
            r"(?i)aws[_\-]?secret[_\-]?(?:access[_\-]?)?key\s*[:=]\s*[A-Za-z0-9/+=]{40}",
            SensitiveType::AwsKey,
        ),
        // GitHub Token
        SanitizationRule::new(
            "GitHub Token",
            r"gh[pousr]_[A-Za-z0-9]{36,}",
            SensitiveType::GithubToken,
        ),
        // Anthropic API Key
        SanitizationRule::new(
            "Anthropic API Key",
            r"sk-ant-[a-zA-Z0-9\-]{20,}",
            SensitiveType::AnthropicKey,
        ),
        // Google Cloud API Key
        SanitizationRule::new(
            "Google Cloud API Key",
            r"AIza[0-9A-Za-z\-_]{35}",
            SensitiveType::GoogleCloudKey,
        ),
        // IPv4 地址 (匹配所有 IPv4，localhost 通过后处理过滤)
        SanitizationRule::new(
            "IPv4 Address",
            r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b",
            SensitiveType::IpAddress,
        ),
        // IPv6 地址 (支持完整形式和常见压缩形式)
        SanitizationRule::new(
            "IPv6 Address",
            r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b|\b(?:[0-9a-fA-F]{1,4}:){1,7}:\b|\b(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}\b|\b::(?:[0-9a-fA-F]{1,4}:){0,5}[0-9a-fA-F]{1,4}\b",
            SensitiveType::IpAddress,
        ),
        // Bearer Token
        SanitizationRule::new(
            "Bearer Token",
            r"Bearer\s+[A-Za-z0-9\-._~+/]+=*",
            SensitiveType::BearerToken,
        ),
        // JWT Token
        SanitizationRule::new(
            "JWT Token",
            r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]+",
            SensitiveType::JwtToken,
        ),
        // Generic Secret/Password patterns
        SanitizationRule::new(
            "Generic Secret",
            r#"(?i)(password|secret|token|api_key|apikey)\s*[:=]\s*["']?[^\s,;'"]{8,}["']?"#,
            SensitiveType::Secret,
        ),
    ]
});

#[cfg(test)]
mod pattern_tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_all_patterns_compile() {
        for rule in BUILTIN_RULES.iter() {
            let result = Regex::new(&rule.pattern);
            assert!(result.is_ok(), "Pattern '{}' failed to compile: {:?}", rule.name, result.err());
        }
    }

    #[test]
    fn test_sensitive_type_display() {
        assert_eq!(SensitiveType::ApiKey.as_str(), "API_KEY");
        assert_eq!(SensitiveType::IpAddress.as_str(), "IP_ADDRESS");
        assert_eq!(SensitiveType::Custom.as_str(), "CUSTOM");
    }

    #[test]
    fn test_github_token_pattern() {
        // gh[pousr]_[A-Za-z0-9]{36,}
        let re = Regex::new(r"gh[pousr]_[A-Za-z0-9]{36,}").unwrap();
        // 需要 ghp_ 后面跟 36+ 个字母数字
        let valid = "ghp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 36 a's
        assert!(re.is_match(valid), "Should match: {}", valid);
    }

    #[test]
    fn test_anthropic_key_pattern() {
        // sk-ant-[a-zA-Z0-9\-]{20,}
        let re = Regex::new(r"sk-ant-[a-zA-Z0-9\-]{20,}").unwrap();
        // 需要 sk-ant- 后面跟 20+ 个字母数字或连字符
        let valid = "sk-ant-aaaaaaaaaaaaaaaaaaaa"; // 20 a's after sk-ant-
        assert!(re.is_match(valid), "Should match: {}", valid);
    }

    #[test]
    fn test_google_cloud_key_pattern() {
        // AIza[0-9A-Za-z\-_]{35}
        let re = Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap();
        // 需要 AIza 后面恰好 35 个字符
        let valid = "AIzaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 35 a's after AIza
        assert!(re.is_match(valid), "Should match: {} (len after AIza: {})", valid, valid.len() - 4);
    }

    #[test]
    fn test_jwt_token_pattern() {
        // eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]+
        let re = Regex::new(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]+").unwrap();
        let valid = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(re.is_match(valid), "Should match JWT: {}", valid);
    }
}
