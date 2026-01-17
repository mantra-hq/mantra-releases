//! 隐私规则配置模块
//!
//! 支持用户自定义规则和内置规则启用/禁用状态的持久化存储

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use std::path::Path;

use super::error::SanitizerError;
use super::patterns::{SanitizationRule, BUILTIN_RULES};

/// 配置文件名
const CONFIG_FILENAME: &str = "privacy-rules.json";

/// 隐私规则配置
///
/// 存储用户对内置规则的启用/禁用状态，以及用户自定义规则列表
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyRulesConfig {
    /// 内置规则的启用状态 (rule_id -> enabled)
    /// 缺失的规则默认启用
    pub builtin_enabled: HashMap<String, bool>,
    /// 用户自定义规则列表
    pub custom_rules: Vec<SanitizationRule>,
}

impl PrivacyRulesConfig {
    /// 创建空配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从配置目录加载配置
    ///
    /// # Arguments
    /// * `config_dir` - 配置目录路径 (app_data_dir)
    ///
    /// # Returns
    /// 配置对象，如果文件不存在则返回默认配置
    pub fn load(config_dir: &Path) -> Result<Self, SanitizerError> {
        let config_path = config_dir.join(CONFIG_FILENAME);

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            SanitizerError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            SanitizerError::ConfigError(format!("Failed to parse config file: {}", e))
        })
    }

    /// 保存配置到配置目录
    ///
    /// # Arguments
    /// * `config_dir` - 配置目录路径 (app_data_dir)
    pub fn save(&self, config_dir: &Path) -> Result<(), SanitizerError> {
        let config_path = config_dir.join(CONFIG_FILENAME);

        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).map_err(|e| {
                SanitizerError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            SanitizerError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&config_path, content).map_err(|e| {
            SanitizerError::ConfigError(format!("Failed to write config file: {}", e))
        })
    }

    /// 获取合并后的规则列表
    ///
    /// 合并内置规则和用户自定义规则，并应用启用/禁用状态
    pub fn get_merged_rules(&self) -> Vec<SanitizationRule> {
        let mut rules = Vec::new();

        // 添加内置规则 (应用启用状态)
        for builtin_rule in BUILTIN_RULES.iter() {
            let mut rule = builtin_rule.clone();
            // 如果配置中有该规则的启用状态，则使用配置值；否则使用规则默认值
            if let Some(&enabled) = self.builtin_enabled.get(&rule.id) {
                rule.enabled = enabled;
            }
            rules.push(rule);
        }

        // 添加用户自定义规则
        rules.extend(self.custom_rules.clone());

        rules
    }

    /// 设置内置规则的启用状态
    pub fn set_builtin_enabled(&mut self, rule_id: &str, enabled: bool) {
        self.builtin_enabled.insert(rule_id.to_string(), enabled);
    }

    /// 添加自定义规则
    ///
    /// # Arguments
    /// * `rule` - 要添加的规则
    ///
    /// # Returns
    /// 如果规则 ID 已存在则返回错误
    pub fn add_custom_rule(&mut self, rule: SanitizationRule) -> Result<(), SanitizerError> {
        // 验证规则 ID 唯一性
        if self.custom_rules.iter().any(|r| r.id == rule.id) {
            return Err(SanitizerError::ValidationError(format!(
                "Custom rule with id '{}' already exists",
                rule.id
            )));
        }

        // 验证规则名称非空
        if rule.name.trim().is_empty() {
            return Err(SanitizerError::ValidationError(
                "Rule name cannot be empty".to_string(),
            ));
        }

        // 验证正则表达式有效性
        validate_regex_pattern(&rule.pattern)?;

        self.custom_rules.push(rule);
        Ok(())
    }

    /// 删除自定义规则
    ///
    /// # Arguments
    /// * `rule_id` - 要删除的规则 ID
    ///
    /// # Returns
    /// 如果规则不存在则返回错误
    pub fn remove_custom_rule(&mut self, rule_id: &str) -> Result<(), SanitizerError> {
        let original_len = self.custom_rules.len();
        self.custom_rules.retain(|r| r.id != rule_id);

        if self.custom_rules.len() == original_len {
            return Err(SanitizerError::ValidationError(format!(
                "Custom rule with id '{}' not found",
                rule_id
            )));
        }

        Ok(())
    }

    /// 更新自定义规则
    pub fn update_custom_rule(&mut self, rule: SanitizationRule) -> Result<(), SanitizerError> {
        // 验证规则名称非空
        if rule.name.trim().is_empty() {
            return Err(SanitizerError::ValidationError(
                "Rule name cannot be empty".to_string(),
            ));
        }

        // 验证正则表达式有效性
        validate_regex_pattern(&rule.pattern)?;

        // 查找并更新规则
        if let Some(existing) = self.custom_rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule;
            Ok(())
        } else {
            Err(SanitizerError::ValidationError(format!(
                "Custom rule with id '{}' not found",
                rule.id
            )))
        }
    }
}

/// 验证正则表达式是否有效
///
/// # Arguments
/// * `pattern` - 正则表达式字符串
///
/// # Returns
/// 如果正则无效则返回包含错误信息的 Err
pub fn validate_regex_pattern(pattern: &str) -> Result<(), SanitizerError> {
    if pattern.trim().is_empty() {
        return Err(SanitizerError::ValidationError(
            "Regex pattern cannot be empty".to_string(),
        ));
    }

    Regex::new(pattern).map_err(|e| {
        SanitizerError::InvalidRegex(format!("Invalid regex pattern: {}", e))
    })?;

    Ok(())
}

/// 验证正则表达式并返回详细结果
///
/// # Arguments
/// * `pattern` - 正则表达式字符串
///
/// # Returns
/// 验证结果，包含是否有效和错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexValidationResult {
    pub valid: bool,
    pub error: Option<String>,
}

pub fn validate_regex_with_details(pattern: &str) -> RegexValidationResult {
    match validate_regex_pattern(pattern) {
        Ok(()) => RegexValidationResult {
            valid: true,
            error: None,
        },
        Err(SanitizerError::InvalidRegex(msg)) => RegexValidationResult {
            valid: false,
            error: Some(msg),
        },
        Err(SanitizerError::ValidationError(msg)) => RegexValidationResult {
            valid: false,
            error: Some(msg),
        },
        Err(e) => RegexValidationResult {
            valid: false,
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sanitizer::SensitiveType;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = PrivacyRulesConfig::new();
        assert!(config.builtin_enabled.is_empty());
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_load_nonexistent_config() {
        let dir = tempdir().unwrap();
        let config = PrivacyRulesConfig::load(dir.path()).unwrap();
        assert!(config.builtin_enabled.is_empty());
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let mut config = PrivacyRulesConfig::new();

        // 设置一些状态
        config.set_builtin_enabled("openai_api_key", false);
        config.set_builtin_enabled("email", true);

        // 添加自定义规则
        let custom_rule = SanitizationRule::custom("custom_1", "My Rule", r"\btest\b");
        config.add_custom_rule(custom_rule).unwrap();

        // 保存
        config.save(dir.path()).unwrap();

        // 重新加载
        let loaded = PrivacyRulesConfig::load(dir.path()).unwrap();
        assert_eq!(loaded.builtin_enabled.get("openai_api_key"), Some(&false));
        assert_eq!(loaded.builtin_enabled.get("email"), Some(&true));
        assert_eq!(loaded.custom_rules.len(), 1);
        assert_eq!(loaded.custom_rules[0].id, "custom_1");
    }

    #[test]
    fn test_get_merged_rules() {
        let mut config = PrivacyRulesConfig::new();
        config.set_builtin_enabled("openai_api_key", false);

        let custom_rule = SanitizationRule::custom("custom_1", "My Rule", r"\btest\b");
        config.add_custom_rule(custom_rule).unwrap();

        let merged = config.get_merged_rules();

        // 应该包含所有内置规则 + 1个自定义规则
        let builtin_count = BUILTIN_RULES.len();
        assert_eq!(merged.len(), builtin_count + 1);

        // 检查禁用状态
        let openai_rule = merged.iter().find(|r| r.id == "openai_api_key").unwrap();
        assert!(!openai_rule.enabled);

        // 检查自定义规则
        let custom = merged.iter().find(|r| r.id == "custom_1").unwrap();
        assert_eq!(custom.name, "My Rule");
        assert_eq!(custom.sensitive_type, SensitiveType::Custom);
    }

    #[test]
    fn test_add_duplicate_custom_rule() {
        let mut config = PrivacyRulesConfig::new();
        let rule1 = SanitizationRule::custom("dup_id", "Rule 1", r"\btest\b");
        let rule2 = SanitizationRule::custom("dup_id", "Rule 2", r"\btest2\b");

        config.add_custom_rule(rule1).unwrap();
        let result = config.add_custom_rule(rule2);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_rule_with_empty_name() {
        let mut config = PrivacyRulesConfig::new();
        let rule = SanitizationRule::custom("empty_name", "  ", r"\btest\b");
        let result = config.add_custom_rule(rule);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_rule_with_invalid_regex() {
        let mut config = PrivacyRulesConfig::new();
        let rule = SanitizationRule::custom("invalid_regex", "Invalid", r"[unclosed");
        let result = config.add_custom_rule(rule);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_custom_rule() {
        let mut config = PrivacyRulesConfig::new();
        let rule = SanitizationRule::custom("to_remove", "Remove Me", r"\btest\b");
        config.add_custom_rule(rule).unwrap();
        assert_eq!(config.custom_rules.len(), 1);

        config.remove_custom_rule("to_remove").unwrap();
        assert_eq!(config.custom_rules.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_rule() {
        let mut config = PrivacyRulesConfig::new();
        let result = config.remove_custom_rule("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_regex_pattern() {
        // 有效模式
        assert!(validate_regex_pattern(r"\btest\b").is_ok());
        assert!(validate_regex_pattern(r"[a-zA-Z0-9]+").is_ok());

        // 无效模式
        assert!(validate_regex_pattern(r"[unclosed").is_err());
        assert!(validate_regex_pattern(r"(?P<dup>a)(?P<dup>b)").is_err());

        // 空模式
        assert!(validate_regex_pattern("").is_err());
        assert!(validate_regex_pattern("   ").is_err());
    }

    #[test]
    fn test_validate_regex_with_details() {
        let valid = validate_regex_with_details(r"\btest\b");
        assert!(valid.valid);
        assert!(valid.error.is_none());

        let invalid = validate_regex_with_details(r"[unclosed");
        assert!(!invalid.valid);
        assert!(invalid.error.is_some());
    }

    #[test]
    fn test_update_custom_rule() {
        let mut config = PrivacyRulesConfig::new();
        let rule = SanitizationRule::custom("update_me", "Original", r"\boriginal\b");
        config.add_custom_rule(rule).unwrap();

        let updated = SanitizationRule::custom("update_me", "Updated", r"\bupdated\b");
        config.update_custom_rule(updated).unwrap();

        assert_eq!(config.custom_rules[0].name, "Updated");
        assert_eq!(config.custom_rules[0].pattern, r"\bupdated\b");
    }
}
