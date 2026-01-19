//! 拦截记录数据模型
//!
//! 定义隐私扫描拦截记录的数据结构，用于持久化存储和统计分析。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::scanner::ScanMatch;

/// 拦截来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterceptionSource {
    /// 客户端上传前检查
    PreUpload { session_id: String },
    /// Claude Code Hook
    ClaudeCodeHook { session_id: Option<String> },
    /// 其他 AI 工具 Hook
    ExternalHook { tool_name: String },
    /// PreToolUse 文件内容检测 (Story 3.11)
    PreToolUseFileCheck {
        /// 触发的工具名 (Read/Grep/Bash/Edit)
        tool_name: String,
        /// 被检测的文件路径列表
        file_paths: Vec<String>,
    },
}

impl InterceptionSource {
    /// 获取来源类型字符串 (用于数据库存储)
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::PreUpload { .. } => "pre_upload",
            Self::ClaudeCodeHook { .. } => "claude_code_hook",
            Self::ExternalHook { .. } => "external_hook",
            Self::PreToolUseFileCheck { .. } => "pre_tool_use_file_check",
        }
    }

    /// 从类型字符串和上下文 JSON 反序列化
    pub fn from_db(source_type: &str, context: Option<&str>) -> Option<Self> {
        match source_type {
            "pre_upload" => {
                let ctx: serde_json::Value = context
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Some(Self::PreUpload {
                    session_id: ctx
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
            }
            "claude_code_hook" => {
                let ctx: serde_json::Value = context
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Some(Self::ClaudeCodeHook {
                    session_id: ctx
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            }
            "external_hook" => {
                let ctx: serde_json::Value = context
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Some(Self::ExternalHook {
                    tool_name: ctx
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                })
            }
            "pre_tool_use_file_check" => {
                let ctx: serde_json::Value = context
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                Some(Self::PreToolUseFileCheck {
                    tool_name: ctx
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    file_paths: ctx
                        .get("file_paths")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                        .unwrap_or_default(),
                })
            }
            _ => None,
        }
    }

    /// 序列化上下文为 JSON 字符串
    pub fn context_json(&self) -> String {
        match self {
            Self::PreUpload { session_id } => {
                serde_json::json!({ "session_id": session_id }).to_string()
            }
            Self::ClaudeCodeHook { session_id } => {
                serde_json::json!({ "session_id": session_id }).to_string()
            }
            Self::ExternalHook { tool_name } => {
                serde_json::json!({ "tool_name": tool_name }).to_string()
            }
            Self::PreToolUseFileCheck { tool_name, file_paths } => {
                serde_json::json!({
                    "tool_name": tool_name,
                    "file_paths": file_paths
                }).to_string()
            }
        }
    }
}

/// 用户操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    /// 已脱敏
    Redacted,
    /// 忽略并继续
    Ignored,
    /// 取消操作
    Cancelled,
    /// 禁用了该规则
    RuleDisabled,
}

impl UserAction {
    /// 获取操作类型字符串 (用于数据库存储)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Redacted => "redacted",
            Self::Ignored => "ignored",
            Self::Cancelled => "cancelled",
            Self::RuleDisabled => "rule_disabled",
        }
    }

    /// 从字符串反序列化
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "redacted" => Some(Self::Redacted),
            "ignored" => Some(Self::Ignored),
            "cancelled" => Some(Self::Cancelled),
            "rule_disabled" => Some(Self::RuleDisabled),
            _ => None,
        }
    }
}

/// 拦截记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptionRecord {
    /// 记录 ID (UUID)
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 拦截来源
    pub source: InterceptionSource,
    /// 匹配结果列表
    pub matches: Vec<ScanMatch>,
    /// 用户操作
    pub user_action: UserAction,
    /// 原文哈希 (不存原文，仅用于去重)
    pub original_text_hash: String,
    /// 项目名称 (可选)
    pub project_name: Option<String>,
}

impl InterceptionRecord {
    /// 创建新的拦截记录
    pub fn new(
        source: InterceptionSource,
        matches: Vec<ScanMatch>,
        user_action: UserAction,
        original_text_hash: String,
        project_name: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source,
            matches,
            user_action,
            original_text_hash,
            project_name,
        }
    }
}

/// 拦截统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InterceptionStats {
    /// 总拦截数
    pub total_interceptions: u64,
    /// 按敏感类型分组统计 (SensitiveType -> count)
    pub by_type: HashMap<String, u64>,
    /// 按严重程度分组统计 (Severity -> count)
    pub by_severity: HashMap<String, u64>,
    /// 按用户操作分组统计 (UserAction -> count)
    pub by_action: HashMap<String, u64>,
    /// 最近 7 天拦截数
    pub recent_7_days: u64,
}

/// 分页记录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedRecords {
    /// 记录列表
    pub records: Vec<InterceptionRecord>,
    /// 总记录数
    pub total: u64,
    /// 当前页码 (1-based)
    pub page: u32,
    /// 每页记录数
    pub per_page: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sanitizer::patterns::{SensitiveType, Severity};
    use crate::sanitizer::scanner::ScanMatch;

    #[test]
    fn test_interception_source_type() {
        let pre_upload = InterceptionSource::PreUpload {
            session_id: "sess123".to_string(),
        };
        assert_eq!(pre_upload.source_type(), "pre_upload");

        let claude_hook = InterceptionSource::ClaudeCodeHook {
            session_id: Some("sess456".to_string()),
        };
        assert_eq!(claude_hook.source_type(), "claude_code_hook");

        let external_hook = InterceptionSource::ExternalHook {
            tool_name: "github-copilot".to_string(),
        };
        assert_eq!(external_hook.source_type(), "external_hook");
    }

    #[test]
    fn test_interception_source_context_json() {
        let pre_upload = InterceptionSource::PreUpload {
            session_id: "sess123".to_string(),
        };
        let json = pre_upload.context_json();
        assert!(json.contains("sess123"));

        let claude_hook = InterceptionSource::ClaudeCodeHook { session_id: None };
        let json = claude_hook.context_json();
        assert!(json.contains("null") || json.contains("session_id"));
    }

    #[test]
    fn test_interception_source_from_db() {
        let ctx = r#"{"session_id": "sess123"}"#;
        let source = InterceptionSource::from_db("pre_upload", Some(ctx));
        assert!(source.is_some());
        if let Some(InterceptionSource::PreUpload { session_id }) = source {
            assert_eq!(session_id, "sess123");
        }

        let source = InterceptionSource::from_db("invalid", None);
        assert!(source.is_none());
    }

    #[test]
    fn test_user_action_str() {
        assert_eq!(UserAction::Redacted.as_str(), "redacted");
        assert_eq!(UserAction::Ignored.as_str(), "ignored");
        assert_eq!(UserAction::Cancelled.as_str(), "cancelled");
        assert_eq!(UserAction::RuleDisabled.as_str(), "rule_disabled");
    }

    #[test]
    fn test_user_action_from_str() {
        assert_eq!(UserAction::from_str("redacted"), Some(UserAction::Redacted));
        assert_eq!(UserAction::from_str("ignored"), Some(UserAction::Ignored));
        assert_eq!(UserAction::from_str("cancelled"), Some(UserAction::Cancelled));
        assert_eq!(
            UserAction::from_str("rule_disabled"),
            Some(UserAction::RuleDisabled)
        );
        assert_eq!(UserAction::from_str("invalid"), None);
    }

    #[test]
    fn test_interception_record_new() {
        let source = InterceptionSource::PreUpload {
            session_id: "sess123".to_string(),
        };
        let matches = vec![ScanMatch {
            rule_id: "openai_api_key".to_string(),
            sensitive_type: SensitiveType::ApiKey,
            severity: Severity::Critical,
            line: 1,
            column: 10,
            matched_text: "sk-test123".to_string(),
            masked_text: "sk-****".to_string(),
            context: "API key: sk-test123".to_string(),
        }];
        let record = InterceptionRecord::new(
            source,
            matches.clone(),
            UserAction::Redacted,
            "abc123hash".to_string(),
            Some("my-project".to_string()),
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.matches.len(), 1);
        assert_eq!(record.user_action, UserAction::Redacted);
        assert_eq!(record.original_text_hash, "abc123hash");
        assert_eq!(record.project_name, Some("my-project".to_string()));
    }

    #[test]
    fn test_interception_stats_default() {
        let stats = InterceptionStats::default();
        assert_eq!(stats.total_interceptions, 0);
        assert!(stats.by_type.is_empty());
        assert!(stats.by_severity.is_empty());
        assert!(stats.by_action.is_empty());
        assert_eq!(stats.recent_7_days, 0);
    }

    #[test]
    fn test_paginated_records() {
        let records = PaginatedRecords {
            records: vec![],
            total: 100,
            page: 1,
            per_page: 20,
        };
        assert_eq!(records.total, 100);
        assert_eq!(records.page, 1);
        assert_eq!(records.per_page, 20);
        assert!(records.records.is_empty());
    }

    #[test]
    fn test_serde_interception_source() {
        let source = InterceptionSource::ClaudeCodeHook {
            session_id: Some("test-session".to_string()),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("claude_code_hook"));
        assert!(json.contains("test-session"));

        let deserialized: InterceptionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, deserialized);
    }

    #[test]
    fn test_serde_user_action() {
        let action = UserAction::Redacted;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("redacted"));

        let deserialized: UserAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }
}
