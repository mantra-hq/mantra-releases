use super::*;

#[test]
fn test_mcp_service_source_serialization() {
    let imported = McpServiceSource::Imported;
    let manual = McpServiceSource::Manual;

    assert_eq!(serde_json::to_string(&imported).unwrap(), r#""imported""#);
    assert_eq!(serde_json::to_string(&manual).unwrap(), r#""manual""#);
}

#[test]
fn test_mcp_service_source_deserialization() {
    let imported: McpServiceSource = serde_json::from_str(r#""imported""#).unwrap();
    let manual: McpServiceSource = serde_json::from_str(r#""manual""#).unwrap();

    assert_eq!(imported, McpServiceSource::Imported);
    assert_eq!(manual, McpServiceSource::Manual);
}

#[test]
fn test_mcp_service_source_as_str() {
    assert_eq!(McpServiceSource::Imported.as_str(), "imported");
    assert_eq!(McpServiceSource::Manual.as_str(), "manual");
}

#[test]
fn test_mcp_service_source_from_str() {
    assert_eq!(
        McpServiceSource::from_str("imported"),
        Some(McpServiceSource::Imported)
    );
    assert_eq!(
        McpServiceSource::from_str("manual"),
        Some(McpServiceSource::Manual)
    );
    assert_eq!(McpServiceSource::from_str("unknown"), None);
}

#[test]
fn test_mcp_service_serialization() {
    let service = McpService {
        id: "test-id".to_string(),
        name: "git-mcp".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "npx".to_string(),
        args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
        env: Some(serde_json::json!({"DEBUG": "true"})),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: "2026-01-30T00:00:00Z".to_string(),
        updated_at: "2026-01-30T00:00:00Z".to_string(),
        default_tool_policy: None,
    };

    let json = serde_json::to_string(&service).unwrap();
    let deserialized: McpService = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, service.id);
    assert_eq!(deserialized.name, service.name);
    assert_eq!(deserialized.transport_type, McpTransportType::Stdio);
    assert_eq!(deserialized.command, service.command);
    assert_eq!(deserialized.args, service.args);
    assert_eq!(deserialized.source, service.source);
    assert!(deserialized.enabled);
}

#[test]
fn test_mcp_service_http_type() {
    let service = McpService {
        id: "deepwiki-id".to_string(),
        name: "deepwiki".to_string(),
        transport_type: McpTransportType::Http,
        command: String::new(),
        args: None,
        env: None,
        url: Some("https://mcp.deepwiki.com/mcp".to_string()),
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some(".mcp.json".to_string()),
        source_adapter_id: Some("claude".to_string()),
        source_scope: Some("project".to_string()),
        enabled: true,
        created_at: "2026-01-30T00:00:00Z".to_string(),
        updated_at: "2026-01-30T00:00:00Z".to_string(),
        default_tool_policy: None,
    };

    let json = serde_json::to_string(&service).unwrap();
    assert!(json.contains("deepwiki"));
    assert!(json.contains("https://mcp.deepwiki.com/mcp"));
    assert!(json.contains(r#""transport_type":"http""#));

    let deserialized: McpService = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.transport_type, McpTransportType::Http);
    assert_eq!(deserialized.url, Some("https://mcp.deepwiki.com/mcp".to_string()));
}

#[test]
fn test_create_mcp_service_request() {
    let request = CreateMcpServiceRequest {
        name: "filesystem".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "npx".to_string(),
        args: Some(vec!["-y".to_string(), "@anthropic/filesystem-mcp".to_string()]),
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some("/home/user/.mcp.json".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("filesystem"));
    assert!(json.contains("imported"));
}

#[test]
fn test_create_mcp_service_request_http() {
    let request = CreateMcpServiceRequest {
        name: "deepwiki".to_string(),
        transport_type: McpTransportType::Http,
        command: String::new(),
        args: None,
        env: None,
        url: Some("https://mcp.deepwiki.com/mcp".to_string()),
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some(".mcp.json".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("deepwiki"));
    assert!(json.contains("https://mcp.deepwiki.com/mcp"));
}

#[test]
fn test_update_mcp_service_request_partial() {
    let request = UpdateMcpServiceRequest {
        name: Some("new-name".to_string()),
        ..Default::default()
    };

    assert!(request.name.is_some());
    assert!(request.command.is_none());
    assert!(request.args.is_none());
    assert!(request.env.is_none());
    assert!(request.enabled.is_none());
}

#[test]
fn test_project_mcp_service() {
    let link = ProjectMcpService {
        project_id: "project-123".to_string(),
        service_id: "service-456".to_string(),
        config_override: Some(serde_json::json!({"args": ["--custom"]})),
        detected_adapter_id: Some("claude".to_string()),
        detected_config_path: Some("/project/.mcp.json".to_string()),
        created_at: "2026-01-30T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&link).unwrap();
    assert!(json.contains("project-123"));
    assert!(json.contains("service-456"));
    assert!(json.contains("--custom"));
    assert!(json.contains("claude"));
    assert!(json.contains("/project/.mcp.json"));
}

#[test]
fn test_env_variable() {
    let env_var = EnvVariable {
        id: "env-123".to_string(),
        name: "OPENAI_API_KEY".to_string(),
        masked_value: "sk-****...****xyz".to_string(),
        description: Some("OpenAI API Key".to_string()),
        created_at: "2026-01-30T00:00:00Z".to_string(),
        updated_at: "2026-01-30T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&env_var).unwrap();
    assert!(json.contains("OPENAI_API_KEY"));
    assert!(json.contains("sk-****...****xyz"));
}

#[test]
fn test_env_variable_name_validation() {
    let valid = EnvVariableNameValidation {
        is_valid: true,
        suggestion: None,
        error_message: None,
    };
    assert!(valid.is_valid);
    assert!(valid.suggestion.is_none());

    let invalid = EnvVariableNameValidation {
        is_valid: false,
        suggestion: Some("OPENAI_API_KEY".to_string()),
        error_message: Some("Name must be in SCREAMING_SNAKE_CASE format".to_string()),
    };
    assert!(!invalid.is_valid);
    assert_eq!(invalid.suggestion, Some("OPENAI_API_KEY".to_string()));
}

#[test]
fn test_mcp_service_with_override() {
    let service = McpService {
        id: "test-id".to_string(),
        name: "git-mcp".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: "2026-01-30T00:00:00Z".to_string(),
        updated_at: "2026-01-30T00:00:00Z".to_string(),
        default_tool_policy: None,
    };

    let with_override = McpServiceWithOverride {
        service,
        config_override: Some(serde_json::json!({"args": ["--verbose"]})),
        detected_adapter_id: None,
        detected_config_path: None,
    };

    let json = serde_json::to_string(&with_override).unwrap();
    // 由于 #[serde(flatten)]，service 字段会被展开
    assert!(json.contains("git-mcp"));
    assert!(json.contains("--verbose"));
}

#[test]
fn test_transport_type_serialization() {
    assert_eq!(serde_json::to_string(&McpTransportType::Stdio).unwrap(), r#""stdio""#);
    assert_eq!(serde_json::to_string(&McpTransportType::Http).unwrap(), r#""http""#);

    let stdio: McpTransportType = serde_json::from_str(r#""stdio""#).unwrap();
    assert_eq!(stdio, McpTransportType::Stdio);
    let http: McpTransportType = serde_json::from_str(r#""http""#).unwrap();
    assert_eq!(http, McpTransportType::Http);
}

#[test]
fn test_transport_type_default() {
    let default = McpTransportType::default();
    assert_eq!(default, McpTransportType::Stdio);
}

#[test]
fn test_transport_type_as_str() {
    assert_eq!(McpTransportType::Stdio.as_str(), "stdio");
    assert_eq!(McpTransportType::Http.as_str(), "http");
}

#[test]
fn test_transport_type_from_str() {
    assert_eq!(McpTransportType::from_str("stdio"), Some(McpTransportType::Stdio));
    assert_eq!(McpTransportType::from_str("http"), Some(McpTransportType::Http));
    assert_eq!(McpTransportType::from_str("unknown"), None);
}

// ===== Story 11.18: 简化 Tool Policy 测试 =====

#[test]
fn test_tool_policy_default() {
    let policy = ToolPolicy::default();
    assert!(policy.is_allow_all());
    assert!(!policy.is_inherit());
    assert!(!policy.is_custom());
    assert_eq!(policy.allowed_tools, Some(vec![]));
}

#[test]
fn test_tool_policy_allow_all() {
    let policy = ToolPolicy::allow_all();
    assert!(policy.is_allow_all());
    assert!(policy.is_tool_allowed("any_tool"));
    assert!(policy.is_tool_allowed("another_tool"));
}

#[test]
fn test_tool_policy_inherit() {
    let policy = ToolPolicy::inherit();
    assert!(policy.is_inherit());
    assert!(!policy.is_allow_all());
    assert!(!policy.is_custom());
    // 继承模式下默认允许（实际继承由 PolicyResolver 处理）
    assert!(policy.is_tool_allowed("any_tool"));
}

#[test]
fn test_tool_policy_custom() {
    let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);
    assert!(policy.is_custom());
    assert!(!policy.is_allow_all());
    assert!(!policy.is_inherit());
    assert!(policy.is_tool_allowed("read_file"));
    assert!(policy.is_tool_allowed("list_commits"));
    assert!(!policy.is_tool_allowed("write_file"));
}

#[test]
fn test_tool_policy_serialization_new_format() {
    // 全选
    let allow_all = ToolPolicy::allow_all();
    let json = serde_json::to_string(&allow_all).unwrap();
    assert!(json.contains(r#""allowedTools":[]"#));
    assert!(!json.contains("mode")); // mode 不再序列化
    assert!(!json.contains("deniedTools")); // deniedTools 不再序列化

    // 部分选
    let custom = ToolPolicy::custom(vec!["read_file".to_string()]);
    let json = serde_json::to_string(&custom).unwrap();
    assert!(json.contains("read_file"));
    assert!(!json.contains("mode"));

    // 继承
    let inherit = ToolPolicy::inherit();
    let json = serde_json::to_string(&inherit).unwrap();
    assert!(json.contains(r#""allowedTools":null"#));
}

#[test]
fn test_tool_policy_backward_compat_deserialization() {
    // 旧格式: allow_all
    let old_json = r#"{"mode":"allow_all","allowedTools":[],"deniedTools":[]}"#;
    let policy: ToolPolicy = serde_json::from_str(old_json).unwrap();
    assert!(policy.is_allow_all());

    // 旧格式: custom
    let old_json = r#"{"mode":"custom","allowedTools":["read_file","list_commits"],"deniedTools":["write_file"]}"#;
    let policy: ToolPolicy = serde_json::from_str(old_json).unwrap();
    // 新模型忽略 mode 和 deniedTools，只看 allowedTools
    assert!(policy.is_custom());
    assert!(policy.is_tool_allowed("read_file"));
    assert!(policy.is_tool_allowed("list_commits"));
    // write_file 在 deniedTools 中，但新模型不再使用 deniedTools
    assert!(!policy.is_tool_allowed("write_file")); // 不在 allowedTools 中所以不允许

    // 新格式: 只有 allowedTools
    let new_json = r#"{"allowedTools":["read_file"]}"#;
    let policy: ToolPolicy = serde_json::from_str(new_json).unwrap();
    assert!(policy.is_custom());
    assert!(policy.is_tool_allowed("read_file"));
    assert!(!policy.is_tool_allowed("write_file"));
}

#[test]
fn test_tool_policy_from_config_override() {
    // 新格式
    let config_override = serde_json::json!({
        "toolPolicy": {
            "allowedTools": ["read_file", "list_commits"]
        }
    });

    let tool_policy_value = config_override.get("toolPolicy").unwrap();
    let policy: ToolPolicy = serde_json::from_value(tool_policy_value.clone()).unwrap();

    assert!(policy.is_custom());
    assert!(policy.is_tool_allowed("read_file"));
    assert!(policy.is_tool_allowed("list_commits"));
    assert!(!policy.is_tool_allowed("write_file"));
}

#[test]
fn test_tool_policy_migrate_from_legacy() {
    // deny_all → None (删除关联)
    let deny_all = serde_json::json!({"mode": "deny_all", "allowedTools": [], "deniedTools": []});
    assert!(ToolPolicy::migrate_from_legacy(&deny_all).is_none());

    // allow_all → 全选
    let allow_all = serde_json::json!({"mode": "allow_all", "allowedTools": [], "deniedTools": []});
    let migrated = ToolPolicy::migrate_from_legacy(&allow_all).unwrap();
    assert!(migrated.is_allow_all());

    // custom → 保留 allowedTools
    let custom = serde_json::json!({"mode": "custom", "allowedTools": ["read_file", "list"], "deniedTools": ["write"]});
    let migrated = ToolPolicy::migrate_from_legacy(&custom).unwrap();
    assert!(migrated.is_custom());
    assert_eq!(migrated.allowed_tools, Some(vec!["read_file".to_string(), "list".to_string()]));
}

#[test]
fn test_tool_policy_filter_tools() {
    let tools = vec!["read_file", "write_file", "list_commits", "execute"];
    let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);

    let filtered = policy.filter_tools(&tools, |t| t);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.contains(&&"read_file"));
    assert!(filtered.contains(&&"list_commits"));
}

#[test]
fn test_project_mcp_service_get_tool_policy_none() {
    let service = ProjectMcpService {
        project_id: "proj-123".to_string(),
        service_id: "service-456".to_string(),
        config_override: None,
        detected_adapter_id: None,
        detected_config_path: None,
        created_at: "2026-01-31T00:00:00Z".to_string(),
    };

    // Story 11.18: 无配置时返回继承策略 (inherit)，以回退到服务默认
    let policy = service.get_tool_policy();
    assert!(policy.is_inherit());
}

#[test]
fn test_project_mcp_service_get_tool_policy_with_override() {
    let service = ProjectMcpService {
        project_id: "proj-123".to_string(),
        service_id: "service-456".to_string(),
        config_override: Some(serde_json::json!({
            "toolPolicy": {
                "allowedTools": ["read_file"]
            }
        })),
        detected_adapter_id: None,
        detected_config_path: None,
        created_at: "2026-01-31T00:00:00Z".to_string(),
    };

    let policy = service.get_tool_policy();
    assert!(policy.is_custom());
    assert!(policy.is_tool_allowed("read_file"));
    assert!(!policy.is_tool_allowed("write_file"));
}

#[test]
fn test_project_mcp_service_get_tool_policy_invalid_json() {
    // Story 11.18: 如果 toolPolicy 格式无效，返回继承策略 (inherit)
    let service = ProjectMcpService {
        project_id: "proj-123".to_string(),
        service_id: "service-456".to_string(),
        config_override: Some(serde_json::json!({
            "toolPolicy": "invalid_not_an_object"
        })),
        detected_adapter_id: None,
        detected_config_path: None,
        created_at: "2026-01-31T00:00:00Z".to_string(),
    };

    let policy = service.get_tool_policy();
    assert!(policy.is_inherit());
}

#[test]
fn test_project_mcp_service_get_tool_policy_inherit() {
    // Story 11.18: 测试继承模式
    let service = ProjectMcpService {
        project_id: "proj-123".to_string(),
        service_id: "service-456".to_string(),
        config_override: Some(serde_json::json!({
            "toolPolicy": {
                "allowedTools": null
            }
        })),
        detected_adapter_id: None,
        detected_config_path: None,
        created_at: "2026-01-31T00:00:00Z".to_string(),
    };

    let policy = service.get_tool_policy();
    assert!(policy.is_inherit());
}

// ===== Story 11.15: TakeoverBackup 模型测试 =====

#[test]
fn test_tool_type_serialization() {
    assert_eq!(serde_json::to_string(&ToolType::ClaudeCode).unwrap(), r#""claude_code""#);
    assert_eq!(serde_json::to_string(&ToolType::Cursor).unwrap(), r#""cursor""#);
    assert_eq!(serde_json::to_string(&ToolType::Codex).unwrap(), r#""codex""#);
    assert_eq!(serde_json::to_string(&ToolType::GeminiCli).unwrap(), r#""gemini_cli""#);
}

#[test]
fn test_tool_type_deserialization() {
    let claude: ToolType = serde_json::from_str(r#""claude_code""#).unwrap();
    assert_eq!(claude, ToolType::ClaudeCode);
    let cursor: ToolType = serde_json::from_str(r#""cursor""#).unwrap();
    assert_eq!(cursor, ToolType::Cursor);
    let codex: ToolType = serde_json::from_str(r#""codex""#).unwrap();
    assert_eq!(codex, ToolType::Codex);
    let gemini: ToolType = serde_json::from_str(r#""gemini_cli""#).unwrap();
    assert_eq!(gemini, ToolType::GeminiCli);
}

#[test]
fn test_tool_type_as_str() {
    assert_eq!(ToolType::ClaudeCode.as_str(), "claude_code");
    assert_eq!(ToolType::Cursor.as_str(), "cursor");
    assert_eq!(ToolType::Codex.as_str(), "codex");
    assert_eq!(ToolType::GeminiCli.as_str(), "gemini_cli");
}

#[test]
fn test_tool_type_from_str() {
    assert_eq!(ToolType::from_str("claude_code"), Some(ToolType::ClaudeCode));
    assert_eq!(ToolType::from_str("cursor"), Some(ToolType::Cursor));
    assert_eq!(ToolType::from_str("codex"), Some(ToolType::Codex));
    assert_eq!(ToolType::from_str("gemini_cli"), Some(ToolType::GeminiCli));
    assert_eq!(ToolType::from_str("unknown"), None);
}

#[test]
fn test_tool_type_from_adapter_id() {
    assert_eq!(ToolType::from_adapter_id("claude"), Some(ToolType::ClaudeCode));
    assert_eq!(ToolType::from_adapter_id("cursor"), Some(ToolType::Cursor));
    assert_eq!(ToolType::from_adapter_id("codex"), Some(ToolType::Codex));
    assert_eq!(ToolType::from_adapter_id("gemini"), Some(ToolType::GeminiCli));
    assert_eq!(ToolType::from_adapter_id("unknown"), None);
}

#[test]
fn test_tool_type_display_name() {
    assert_eq!(ToolType::ClaudeCode.display_name(), "Claude Code");
    assert_eq!(ToolType::Cursor.display_name(), "Cursor");
    assert_eq!(ToolType::Codex.display_name(), "Codex");
    assert_eq!(ToolType::GeminiCli.display_name(), "Gemini CLI");
}

#[test]
fn test_tool_type_user_config_path() {
    // 测试路径包含正确的文件名
    let claude_path = ToolType::ClaudeCode.get_user_config_path();
    assert!(claude_path.to_string_lossy().ends_with(".claude.json"));

    let cursor_path = ToolType::Cursor.get_user_config_path();
    assert!(cursor_path.to_string_lossy().contains(".cursor"));
    assert!(cursor_path.to_string_lossy().ends_with("mcp.json"));

    let codex_path = ToolType::Codex.get_user_config_path();
    assert!(codex_path.to_string_lossy().contains(".codex"));
    assert!(codex_path.to_string_lossy().ends_with("config.toml"));

    let gemini_path = ToolType::GeminiCli.get_user_config_path();
    assert!(gemini_path.to_string_lossy().contains(".gemini"));
    assert!(gemini_path.to_string_lossy().ends_with("settings.json"));
}

#[test]
fn test_takeover_status_serialization() {
    assert_eq!(serde_json::to_string(&TakeoverStatus::Active).unwrap(), r#""active""#);
    assert_eq!(serde_json::to_string(&TakeoverStatus::Restored).unwrap(), r#""restored""#);
}

#[test]
fn test_takeover_status_deserialization() {
    let active: TakeoverStatus = serde_json::from_str(r#""active""#).unwrap();
    assert_eq!(active, TakeoverStatus::Active);
    let restored: TakeoverStatus = serde_json::from_str(r#""restored""#).unwrap();
    assert_eq!(restored, TakeoverStatus::Restored);
}

#[test]
fn test_takeover_status_as_str() {
    assert_eq!(TakeoverStatus::Active.as_str(), "active");
    assert_eq!(TakeoverStatus::Restored.as_str(), "restored");
}

#[test]
fn test_takeover_status_from_str() {
    assert_eq!(TakeoverStatus::from_str("active"), Some(TakeoverStatus::Active));
    assert_eq!(TakeoverStatus::from_str("restored"), Some(TakeoverStatus::Restored));
    assert_eq!(TakeoverStatus::from_str("unknown"), None);
}

#[test]
fn test_takeover_status_default() {
    let status = TakeoverStatus::default();
    assert_eq!(status, TakeoverStatus::Active);
}

#[test]
fn test_takeover_backup_new() {
    let backup = TakeoverBackup::new(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"),
    );

    assert!(!backup.id.is_empty());
    assert_eq!(backup.tool_type, ToolType::ClaudeCode);
    assert_eq!(backup.original_path, PathBuf::from("/home/user/.claude.json"));
    assert_eq!(backup.backup_path, PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"));
    assert!(backup.restored_at.is_none());
    assert_eq!(backup.status, TakeoverStatus::Active);
}

#[test]
fn test_takeover_backup_serialization() {
    let backup = TakeoverBackup {
        id: "backup-123".to_string(),
        tool_type: ToolType::Cursor,
        scope: TakeoverScope::User,
        project_path: None,
        original_path: PathBuf::from("/home/user/.cursor/mcp.json"),
        backup_path: PathBuf::from("/home/user/.cursor/mcp.json.mantra-backup.20260201"),
        taken_over_at: "2026-02-01T10:00:00Z".to_string(),
        restored_at: None,
        status: TakeoverStatus::Active,
    };

    let json = serde_json::to_string(&backup).unwrap();
    assert!(json.contains("backup-123"));
    assert!(json.contains("cursor"));
    assert!(json.contains("takenOverAt")); // camelCase
    assert!(json.contains("active"));

    let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, backup.id);
    assert_eq!(deserialized.tool_type, backup.tool_type);
    assert_eq!(deserialized.status, TakeoverStatus::Active);
}

#[test]
fn test_takeover_backup_can_restore() {
    // Active 状态但备份文件不存在
    let backup = TakeoverBackup {
        id: "backup-123".to_string(),
        tool_type: ToolType::ClaudeCode,
        scope: TakeoverScope::User,
        project_path: None,
        original_path: PathBuf::from("/nonexistent/original.json"),
        backup_path: PathBuf::from("/nonexistent/backup.json"),
        taken_over_at: "2026-02-01T10:00:00Z".to_string(),
        restored_at: None,
        status: TakeoverStatus::Active,
    };
    assert!(!backup.can_restore()); // 文件不存在

    // Restored 状态
    let restored_backup = TakeoverBackup {
        id: "backup-456".to_string(),
        tool_type: ToolType::Cursor,
        scope: TakeoverScope::User,
        project_path: None,
        original_path: PathBuf::from("/home/user/.cursor/mcp.json"),
        backup_path: PathBuf::from("/home/user/.cursor/mcp.json.backup"),
        taken_over_at: "2026-02-01T10:00:00Z".to_string(),
        restored_at: Some("2026-02-01T12:00:00Z".to_string()),
        status: TakeoverStatus::Restored,
    };
    assert!(!restored_backup.can_restore()); // 已恢复
}

// ===== Story 11.16: TakeoverScope 模型测试 =====

#[test]
fn test_takeover_scope_serialization() {
    assert_eq!(serde_json::to_string(&TakeoverScope::User).unwrap(), r#""user""#);
    assert_eq!(serde_json::to_string(&TakeoverScope::Project).unwrap(), r#""project""#);
}

#[test]
fn test_takeover_scope_deserialization() {
    let user: TakeoverScope = serde_json::from_str(r#""user""#).unwrap();
    assert_eq!(user, TakeoverScope::User);
    let project: TakeoverScope = serde_json::from_str(r#""project""#).unwrap();
    assert_eq!(project, TakeoverScope::Project);
}

#[test]
fn test_takeover_scope_as_str() {
    assert_eq!(TakeoverScope::User.as_str(), "user");
    assert_eq!(TakeoverScope::Project.as_str(), "project");
}

#[test]
fn test_takeover_scope_from_str() {
    assert_eq!(TakeoverScope::from_str("user"), Some(TakeoverScope::User));
    assert_eq!(TakeoverScope::from_str("project"), Some(TakeoverScope::Project));
    assert_eq!(TakeoverScope::from_str("unknown"), None);
}

#[test]
fn test_takeover_scope_default() {
    let scope = TakeoverScope::default();
    assert_eq!(scope, TakeoverScope::User);
}

#[test]
fn test_takeover_backup_new_default_scope() {
    let backup = TakeoverBackup::new(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.claude.json.backup"),
    );
    assert_eq!(backup.scope, TakeoverScope::User);
    assert!(backup.project_path.is_none());
    assert!(backup.is_user_level());
    assert!(!backup.is_project_level());
}

#[test]
fn test_takeover_backup_new_with_scope_user() {
    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.claude.json.backup"),
        TakeoverScope::User,
        None,
    );
    assert_eq!(backup.scope, TakeoverScope::User);
    assert!(backup.project_path.is_none());
    assert!(backup.is_user_level());
}

#[test]
fn test_takeover_backup_new_with_scope_project() {
    let project_path = PathBuf::from("/home/user/my-project");
    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/my-project/.mcp.json"),
        PathBuf::from("/home/user/my-project/.mcp.json.backup"),
        TakeoverScope::Project,
        Some(project_path.clone()),
    );
    assert_eq!(backup.scope, TakeoverScope::Project);
    assert_eq!(backup.project_path, Some(project_path));
    assert!(backup.is_project_level());
    assert!(!backup.is_user_level());
}

#[test]
fn test_takeover_backup_serialization_with_scope() {
    let backup = TakeoverBackup {
        id: "backup-789".to_string(),
        tool_type: ToolType::ClaudeCode,
        scope: TakeoverScope::Project,
        project_path: Some(PathBuf::from("/home/user/my-project")),
        original_path: PathBuf::from("/home/user/my-project/.mcp.json"),
        backup_path: PathBuf::from("/home/user/my-project/.mcp.json.backup"),
        taken_over_at: "2026-02-01T10:00:00Z".to_string(),
        restored_at: None,
        status: TakeoverStatus::Active,
    };

    let json = serde_json::to_string(&backup).unwrap();
    assert!(json.contains(r#""scope":"project""#));
    assert!(json.contains("projectPath"));
    assert!(json.contains("my-project"));

    let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.scope, TakeoverScope::Project);
    assert_eq!(deserialized.project_path, Some(PathBuf::from("/home/user/my-project")));
}

// ===== Story 11.21: Local Scope 支持 =====

#[test]
fn test_takeover_scope_local_as_str() {
    assert_eq!(TakeoverScope::Local.as_str(), "local");
}

#[test]
fn test_takeover_scope_local_from_str() {
    assert_eq!(TakeoverScope::from_str("local"), Some(TakeoverScope::Local));
    assert_eq!(TakeoverScope::from_str("user"), Some(TakeoverScope::User));
    assert_eq!(TakeoverScope::from_str("project"), Some(TakeoverScope::Project));
    assert_eq!(TakeoverScope::from_str("invalid"), None);
}

#[test]
fn test_takeover_scope_requires_project_path() {
    assert!(!TakeoverScope::User.requires_project_path());
    assert!(TakeoverScope::Project.requires_project_path());
    assert!(TakeoverScope::Local.requires_project_path());
}

#[test]
fn test_takeover_scope_is_local() {
    assert!(!TakeoverScope::User.is_local());
    assert!(!TakeoverScope::Project.is_local());
    assert!(TakeoverScope::Local.is_local());
}

#[test]
fn test_takeover_scope_local_serialization() {
    let scope = TakeoverScope::Local;
    let json = serde_json::to_string(&scope).unwrap();
    assert_eq!(json, r#""local""#);

    let deserialized: TakeoverScope = serde_json::from_str(r#""local""#).unwrap();
    assert_eq!(deserialized, TakeoverScope::Local);
}

#[test]
fn test_takeover_backup_local_scope() {
    let backup = TakeoverBackup {
        id: "backup-local-123".to_string(),
        tool_type: ToolType::ClaudeCode,
        scope: TakeoverScope::Local,
        project_path: Some(PathBuf::from("/home/user/project-a")),
        original_path: PathBuf::from("/home/user/.claude.json"),
        backup_path: PathBuf::from("/home/user/.mantra/backups/project-a-local.backup"),
        taken_over_at: "2026-02-03T10:00:00Z".to_string(),
        restored_at: None,
        status: TakeoverStatus::Active,
    };

    // 序列化测试
    let json = serde_json::to_string(&backup).unwrap();
    assert!(json.contains(r#""scope":"local""#));
    assert!(json.contains("projectPath"));
    assert!(json.contains("project-a"));
    assert!(json.contains(".claude.json"));

    // 反序列化测试
    let deserialized: TakeoverBackup = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.scope, TakeoverScope::Local);
    assert_eq!(deserialized.project_path, Some(PathBuf::from("/home/user/project-a")));
    assert_eq!(deserialized.original_path, PathBuf::from("/home/user/.claude.json"));

    // 方法测试
    assert!(deserialized.is_local_level());
    assert!(!deserialized.is_user_level());
    assert!(!deserialized.is_project_level());
}

#[test]
fn test_takeover_backup_new_with_scope_local() {
    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.mantra/backups/project-b-local.backup"),
        TakeoverScope::Local,
        Some(PathBuf::from("/home/user/project-b")),
    );

    assert_eq!(backup.tool_type, ToolType::ClaudeCode);
    assert_eq!(backup.scope, TakeoverScope::Local);
    assert_eq!(backup.project_path, Some(PathBuf::from("/home/user/project-b")));
    assert!(backup.is_local_level());
    assert_eq!(backup.status, TakeoverStatus::Active);
}
