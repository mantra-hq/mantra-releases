//! 内置正则规则定义
//!
//! 包含常见敏感信息类型的识别模式

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 严重程度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// 必须处理：API Key、密码、私钥、身份证号
    Critical,
    /// 警告：邮箱、电话、JWT Token
    #[default]
    Warning,
    /// 仅提示：通用模式匹配
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

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
    /// 邮箱地址
    Email,
    /// 电话号码
    Phone,
    /// 身份证号
    IdCard,
    /// 私钥 (SSH/GPG)
    PrivateKey,
    /// 密码赋值
    Password,
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
            Self::Email => "EMAIL",
            Self::Phone => "PHONE",
            Self::IdCard => "ID_CARD",
            Self::PrivateKey => "PRIVATE_KEY",
            Self::Password => "PASSWORD",
            Self::Custom => "CUSTOM",
        }
    }

    /// 获取该类型的默认严重程度
    pub fn default_severity(&self) -> Severity {
        match self {
            // Critical: API Key、密码、私钥、身份证号
            Self::ApiKey
            | Self::AwsKey
            | Self::AnthropicKey
            | Self::GoogleCloudKey
            | Self::PrivateKey
            | Self::Password
            | Self::IdCard => Severity::Critical,
            // Warning: 邮箱、电话、JWT Token、Bearer Token、GitHub Token
            Self::Email
            | Self::Phone
            | Self::JwtToken
            | Self::BearerToken
            | Self::GithubToken => Severity::Warning,
            // Info: IP 地址、通用模式、自定义
            Self::IpAddress | Self::Secret | Self::Custom => Severity::Info,
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
    /// 规则唯一标识符
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 正则表达式模式
    pub pattern: String,
    /// 敏感信息类型
    pub sensitive_type: SensitiveType,
    /// 严重程度
    pub severity: Severity,
    /// 是否启用
    pub enabled: bool,
}

impl SanitizationRule {
    /// 创建新规则 (使用敏感类型的默认严重程度)
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        pattern: impl Into<String>,
        sensitive_type: SensitiveType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pattern: pattern.into(),
            sensitive_type,
            severity: sensitive_type.default_severity(),
            enabled: true,
        }
    }

    /// 创建带自定义严重程度的规则
    pub fn with_severity(
        id: impl Into<String>,
        name: impl Into<String>,
        pattern: impl Into<String>,
        sensitive_type: SensitiveType,
        severity: Severity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pattern: pattern.into(),
            sensitive_type,
            severity,
            enabled: true,
        }
    }

    /// 创建自定义规则
    pub fn custom(id: impl Into<String>, name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(id, name, pattern, SensitiveType::Custom)
    }
}

/// 内置脱敏规则集
pub static BUILTIN_RULES: Lazy<Vec<SanitizationRule>> = Lazy::new(|| {
    vec![
        // === Critical Severity ===
        // OpenAI API Key: sk-xxx 或 sk-proj-xxx
        SanitizationRule::new(
            "openai_api_key",
            "OpenAI API Key",
            r"sk-(?:proj-)?[a-zA-Z0-9]{20,}",
            SensitiveType::ApiKey,
        ),
        // AWS Access Key ID
        SanitizationRule::new(
            "aws_access_key_id",
            "AWS Access Key ID",
            r"AKIA[0-9A-Z]{16}",
            SensitiveType::AwsKey,
        ),
        // AWS Secret Access Key (40 char base64-like)
        SanitizationRule::new(
            "aws_secret_access_key",
            "AWS Secret Access Key",
            r"(?i)aws[_\-]?secret[_\-]?(?:access[_\-]?)?key\s*[:=]\s*[A-Za-z0-9/+=]{40}",
            SensitiveType::AwsKey,
        ),
        // Anthropic API Key
        SanitizationRule::new(
            "anthropic_api_key",
            "Anthropic API Key",
            r"sk-ant-[a-zA-Z0-9\-]{20,}",
            SensitiveType::AnthropicKey,
        ),
        // Google Cloud API Key
        SanitizationRule::new(
            "google_cloud_api_key",
            "Google Cloud API Key",
            r"AIza[0-9A-Za-z\-_]{35}",
            SensitiveType::GoogleCloudKey,
        ),
        // 中国身份证号
        SanitizationRule::new(
            "id_card_cn",
            "China ID Card",
            r"\b\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]\b",
            SensitiveType::IdCard,
        ),
        // 私钥检测 (RSA, EC, OPENSSH, etc.)
        SanitizationRule::new(
            "private_key",
            "Private Key",
            r"-----BEGIN\s+(?:RSA\s+|EC\s+|OPENSSH\s+)?PRIVATE\s+KEY-----",
            SensitiveType::PrivateKey,
        ),
        // 密码赋值检测
        SanitizationRule::new(
            "password_assign",
            "Password Assignment",
            r#"(?i)(?:password|passwd|pwd)\s*[=:]\s*["'][^"']{4,}["']"#,
            SensitiveType::Password,
        ),
        // === Warning Severity ===
        // GitHub Token
        SanitizationRule::new(
            "github_token",
            "GitHub Token",
            r"gh[pousr]_[A-Za-z0-9]{36,}",
            SensitiveType::GithubToken,
        ),
        // Bearer Token
        SanitizationRule::new(
            "bearer_token",
            "Bearer Token",
            r"Bearer\s+[A-Za-z0-9\-._~+/]+=*",
            SensitiveType::BearerToken,
        ),
        // JWT Token
        SanitizationRule::new(
            "jwt_token",
            "JWT Token",
            r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]+",
            SensitiveType::JwtToken,
        ),
        // 邮箱地址
        SanitizationRule::new(
            "email",
            "Email Address",
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
            SensitiveType::Email,
        ),
        // 中国手机号
        SanitizationRule::new(
            "phone_cn",
            "China Phone Number",
            r"\b1[3-9]\d{9}\b",
            SensitiveType::Phone,
        ),
        // === Info Severity ===
        // IPv4 地址 (版本号如 v1.2.3.4 通过后处理过滤)
        SanitizationRule::new(
            "ipv4_address",
            "IPv4 Address",
            r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b",
            SensitiveType::IpAddress,
        ),
        // IPv6 地址 (支持完整形式和常见压缩形式)
        SanitizationRule::new(
            "ipv6_address",
            "IPv6 Address",
            r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b|\b(?:[0-9a-fA-F]{1,4}:){1,7}:\b|\b(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}\b|\b::(?:[0-9a-fA-F]{1,4}:){0,5}[0-9a-fA-F]{1,4}\b",
            SensitiveType::IpAddress,
        ),
        // Generic Secret/Password patterns
        // 已知 token 格式 (GitHub, OpenAI, Anthropic, JWT) 通过后处理过滤
        SanitizationRule::new(
            "generic_secret",
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
