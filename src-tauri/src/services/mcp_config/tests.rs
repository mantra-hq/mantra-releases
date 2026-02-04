use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use super::*;
use crate::models::mcp::{
    ConflictType, CreateMcpServiceRequest, McpServiceSource, McpTransportType,
    MergeClassification, TakeoverScope,
};
use crate::services::mcp_adapters::ConfigScope;
use crate::storage::Database;

// ===== Task 1.1-1.6: 配置解析器测试 =====

    #[test]
    fn test_strip_json_comments_single_line() {
        let input = r#"{
            "key": "value" // this is a comment
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("// this is a comment"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_json_comments_block() {
        let input = r#"{
            /* block comment */
            "key": "value"
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("/* block comment */"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_json_comments_in_string() {
        let input = r#"{"url": "http://example.com // not a comment"}"#;
        let result = strip_json_comments(input);
        // 字符串内的 // 应该保留
        assert!(result.contains("// not a comment"));
    }

    #[test]
    fn test_strip_json_comments_multiline_block() {
        let input = r#"{
            /*
             * Multi-line
             * block comment
             */
            "key": "value"
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_parse_claude_code_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "mcpServers": {
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                },
                "postgres-mcp": {
                    "command": "uvx",
                    "args": ["mcp-server-postgres"],
                    "env": {
                        "DATABASE_URL": "$DATABASE_URL"
                    }
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 2);

        let git_mcp = services.iter().find(|s| s.name == "git-mcp").unwrap();
        assert_eq!(git_mcp.command, "npx");
        assert_eq!(
            git_mcp.args,
            Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()])
        );

        let postgres_mcp = services.iter().find(|s| s.name == "postgres-mcp").unwrap();
        assert_eq!(postgres_mcp.command, "uvx");
        assert!(postgres_mcp.env.is_some());
        assert_eq!(
            postgres_mcp.env.as_ref().unwrap().get("DATABASE_URL"),
            Some(&"$DATABASE_URL".to_string())
        );
    }

    #[test]
    fn test_parse_cursor_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        let config_content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = CursorConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "filesystem");
        assert_eq!(services[0].command, "npx");
    }

    #[test]
    fn test_parse_config_with_jsonc_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            // MCP configuration
            "mcpServers": {
                /* Git MCP server */
                "git-mcp": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/git-mcp"]
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "git-mcp");
    }

    #[test]
    fn test_parse_config_with_sse_server() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "mcpServers": {
                "local-server": {
                    "command": "npx",
                    "args": ["-y", "local-mcp"]
                },
                "remote-server": {
                    "url": "http://remote.example.com/sse"
                }
            }
        }"#;

        fs::write(&config_path, config_content).unwrap();

        let parser = ClaudeCodeConfigParser;
        let services = parser.parse(&config_path).unwrap();

        // SSE 服务应该被跳过
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "local-server");
    }

    #[test]
    fn test_generate_shadow_config() {
        let gateway_url = "http://127.0.0.1:8080/mcp";

        let shadow = generate_shadow_config(&ConfigSource::ClaudeCode, gateway_url);
        let parsed: serde_json::Value = serde_json::from_str(&shadow).unwrap();

        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            gateway_url
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_config_source_description() {
        // Story 11.8: 更新了 ConfigSource 描述
        assert_eq!(
            ConfigSource::ClaudeCode.description(),
            ".mcp.json"  // 更新为新的路径
        );
        assert_eq!(ConfigSource::Cursor.description(), ".cursor/mcp.json");
        assert_eq!(
            ConfigSource::ClaudeDesktop.description(),
            "claude_desktop_config.json"
        );
        assert_eq!(ConfigSource::Codex.description(), ".codex/config.toml");
        assert_eq!(ConfigSource::Gemini.description(), ".gemini/settings.json");
    }

    // ===== Task 2: 配置文件扫描器测试 =====

    #[test]
    fn test_scan_mcp_configs_project_level() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Story 11.8: 使用新的适配器架构扫描
        // 创建 .mcp.json (Claude Code 新路径)
        fs::write(
            project_path.join(".mcp.json"),
            r#"{"mcpServers": {"test": {"command": "test"}}}"#,
        )
        .unwrap();

        // 创建 .cursor/mcp.json
        let cursor_dir = project_path.join(".cursor");
        fs::create_dir_all(&cursor_dir).unwrap();
        fs::write(
            cursor_dir.join("mcp.json"),
            r#"{"mcpServers": {"cursor-test": {"command": "cursor"}}}"#,
        )
        .unwrap();

        let result = scan_mcp_configs(Some(project_path));

        // 项目级配置应该至少有 2 个（Claude + Cursor）
        // 注意：新架构扫描所有 4 个适配器的项目级配置
        let project_configs: Vec<_> = result
            .configs
            .iter()
            .filter(|c| c.path.starts_with(project_path))
            .collect();
        assert!(project_configs.len() >= 2, "Expected at least 2 project configs, got {}", project_configs.len());
        assert!(result.scanned_paths.len() >= 2);
    }

    #[test]
    fn test_scan_mcp_configs_no_project_configs() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_mcp_configs(Some(temp_dir.path()));

        // 项目级配置应该为空（但可能有全局配置）
        let project_configs: Vec<_> = result
            .configs
            .iter()
            .filter(|c| c.path.starts_with(temp_dir.path()))
            .collect();
        assert!(project_configs.is_empty());
    }

    // ===== Task 3: 导入预览逻辑测试 =====

    #[test]
    fn test_extract_env_var_references() {
        let env = Some(HashMap::from([
            ("API_KEY".to_string(), "$OPENAI_API_KEY".to_string()),
            ("DEBUG".to_string(), "true".to_string()),
            ("SECRET".to_string(), "${MY_SECRET}".to_string()),
        ]));

        let vars = extract_env_var_references(&env);

        assert!(vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(vars.contains(&"MY_SECRET".to_string()));
        assert!(!vars.contains(&"true".to_string()));
    }

    #[test]
    fn test_extract_env_var_references_empty() {
        let vars = extract_env_var_references(&None);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_generate_import_preview() {
        let db = Database::new_in_memory().unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/test/config.json"),
            scope: Some(ConfigScope::Project),
            services: vec![
                DetectedService {
                    name: "new-service".to_string(),
                    transport_type: Default::default(),
                    command: "npx".to_string(),
                    args: None,
                    env: Some(HashMap::from([(
                        "API_KEY".to_string(),
                        "$API_KEY".to_string(),
                    )])),
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                },
            ],
            parse_errors: Vec::new(),
        }];

        let preview = generate_import_preview(&configs, &db).unwrap();

        assert_eq!(preview.total_services, 1);
        assert_eq!(preview.new_services.len(), 1);
        assert!(preview.env_vars_needed.contains(&"API_KEY".to_string()));
    }

    #[test]
    fn test_generate_import_preview_with_conflict() {
        let db = Database::new_in_memory().unwrap();

        // 创建已存在的服务
        let request = CreateMcpServiceRequest {
            name: "existing-service".to_string(),
            transport_type: Default::default(),
            command: "old-command".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/test/config.json"),
            scope: Some(ConfigScope::Project),
            services: vec![DetectedService {
                name: "existing-service".to_string(),
                transport_type: Default::default(),
                command: "new-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            parse_errors: Vec::new(),
        }];

        let preview = generate_import_preview(&configs, &db).unwrap();

        assert_eq!(preview.conflicts.len(), 1);
        assert_eq!(preview.conflicts[0].name, "existing-service");
        assert!(preview.conflicts[0].existing.is_some());
    }

    // ===== Task 4: 备份与回滚测试 =====

    #[test]
    fn test_backup_manager_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        let mut manager = BackupManager::new();
        let (backup_path, _hash) = manager.backup(&test_file).unwrap();

        assert!(backup_path.exists());
        assert!(backup_path
            .to_string_lossy()
            .contains(".mantra-backup"));

        // 验证备份内容
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "original content");
    }

    #[test]
    fn test_backup_manager_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        let mut manager = BackupManager::new();
        let _ = manager.backup(&test_file).unwrap();

        // 修改原文件
        fs::write(&test_file, "modified content").unwrap();

        // 手动回滚
        manager.rollback().unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_backup_manager_auto_rollback_on_drop() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        {
            let mut manager = BackupManager::new();
            let _ = manager.backup(&test_file).unwrap();

            // 修改原文件
            fs::write(&test_file, "modified content").unwrap();

            // manager 在这里被 drop，但未 commit
        }

        // 应该已自动回滚
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_backup_manager_commit_prevents_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        fs::write(&test_file, "original content").unwrap();

        {
            let mut manager = BackupManager::new();
            let _ = manager.backup(&test_file).unwrap();

            // 修改原文件
            fs::write(&test_file, "modified content").unwrap();

            // 提交
            manager.commit();
        }

        // commit 后不应回滚
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "modified content");
    }

    #[test]
    fn test_backup_manager_existing_backup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");
        let existing_backup = temp_dir.path().join("test.json.mantra-backup");

        fs::write(&test_file, "original content").unwrap();
        fs::write(&existing_backup, "old backup").unwrap();

        let mut manager = BackupManager::new();
        let (backup_path, _hash) = manager.backup(&test_file).unwrap();

        // 应该创建带时间戳的备份
        assert!(backup_path.exists());
        assert_ne!(backup_path, existing_backup);
        assert!(backup_path
            .to_string_lossy()
            .contains(".mantra-backup."));
    }

    // ===== Task 5: 影子模式配置测试 =====

    #[test]
    fn test_shadow_config_format() {
        let gateway_url = "http://127.0.0.1:8080/mcp";

        for source in [
            ConfigSource::ClaudeCode,
            ConfigSource::Cursor,
            ConfigSource::ClaudeDesktop,
        ] {
            let shadow = generate_shadow_config(&source, gateway_url);
            let parsed: serde_json::Value = serde_json::from_str(&shadow).unwrap();

            assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
            assert_eq!(
                parsed["mcpServers"]["mantra-gateway"]["url"],
                gateway_url
            );
        }
    }

    // ===== Task 6: 导入执行器测试 =====

    #[test]
    fn test_import_executor_basic() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test-mcp".to_string()]),
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: vec!["test-service".to_string()],
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert!(result.errors.is_empty());

        // 验证服务已创建
        let service = db.get_mcp_service_by_name("test-service").unwrap();
        assert!(service.is_some());
        assert_eq!(service.unwrap().command, "npx");
    }

    #[test]
    fn test_import_executor_skip_service() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "skipped-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(), // 不包含任何服务
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 0);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_keep() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing.clone()),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Keep,
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 应该保持原服务不变
        let service = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(service.command, "old-command");
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_replace() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Replace(0),
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 服务应该被替换
        let service = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(service.command, "new-command");
        assert_eq!(result.imported_count, 1);
    }

    #[test]
    fn test_import_executor_conflict_resolution_rename() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建已存在的服务
        let existing = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "conflict-service".to_string(),
                transport_type: Default::default(),
                command: "old-command".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: vec![ServiceConflict {
                name: "conflict-service".to_string(),
                existing: Some(existing),
                candidates: vec![DetectedService {
                    name: "conflict-service".to_string(),
                    transport_type: Default::default(),
                    command: "new-command".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/test/config.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                }],
            }],
            new_services: Vec::new(),
            env_vars_needed: Vec::new(),
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: Vec::new(),
            conflict_resolutions: HashMap::from([(
                "conflict-service".to_string(),
                ConflictResolution::Rename("renamed-service".to_string()),
            )]),
            env_var_values: HashMap::new(),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        // 原服务应该保留
        let original = db.get_mcp_service_by_name("conflict-service").unwrap().unwrap();
        assert_eq!(original.command, "old-command");

        // 新服务应该以新名称创建
        let renamed = db.get_mcp_service_by_name("renamed-service").unwrap().unwrap();
        assert_eq!(renamed.command, "new-command");

        assert_eq!(result.imported_count, 1);
    }

    #[test]
    fn test_import_executor_with_env_vars() {
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        let preview = ImportPreview {
            configs: Vec::new(),
            conflicts: Vec::new(),
            new_services: vec![DetectedService {
                name: "api-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: Some(HashMap::from([("API_KEY".to_string(), "$API_KEY".to_string())])),
                url: None,
                headers: None,
                source_file: PathBuf::from("/test/config.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            env_vars_needed: vec!["API_KEY".to_string()],
            total_services: 1,
        };

        let request = ImportRequest {
            services_to_import: vec!["api-service".to_string()],
            conflict_resolutions: HashMap::new(),
            env_var_values: HashMap::from([("API_KEY".to_string(), "secret-key-123".to_string())]),
            enable_shadow_mode: false,
            gateway_url: None,
            gateway_token: None,
        };

        let executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.execute(&preview, &request).unwrap();

        assert_eq!(result.imported_count, 1);
        assert!(result.errors.is_empty());

        // 验证环境变量已存储
        assert!(db.env_variable_exists("API_KEY").unwrap());
    }

    // ===== 回滚功能测试 =====

    #[test]
    fn test_rollback_from_backups() {
        let temp_dir = TempDir::new().unwrap();
        let original_file = temp_dir.path().join("config.json");
        let backup_file = temp_dir.path().join("config.json.mantra-backup");

        fs::write(&original_file, "modified content").unwrap();
        fs::write(&backup_file, "original content").unwrap();

        let restored = rollback_from_backups(&[backup_file]).unwrap();

        assert_eq!(restored, 1);
        let content = fs::read_to_string(&original_file).unwrap();
        assert_eq!(content, "original content");
    }

    // ===== Story 11.19: 智能接管预览引擎测试 =====

    #[test]
    fn test_config_equals_identical() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/test/.mcp.json".to_string()),
            source_adapter_id: Some("claude".to_string()),
            source_scope: Some("project".to_string()),
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let detected = DetectedService {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        };

        assert!(super::config_equals(&existing, &detected));
    }

    #[test]
    fn test_config_equals_different_command() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let detected = DetectedService {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "uvx".to_string(), // Different command
            args: None,
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        };

        assert!(!super::config_equals(&existing, &detected));
    }

    #[test]
    fn test_config_equals_different_args() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "old-package".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let detected = DetectedService {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "new-package".to_string()]), // Different args
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        };

        assert!(!super::config_equals(&existing, &detected));
    }

    #[test]
    fn test_classify_for_merge_auto_create() {
        use crate::models::mcp::McpTransportType;

        let candidates = vec![DetectedService {
            name: "new-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        }];

        let classification = super::classify_for_merge("new-service", &candidates, None);
        assert_eq!(classification, MergeClassification::AutoCreate);
    }

    #[test]
    fn test_classify_for_merge_auto_skip() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let candidates = vec![DetectedService {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        }];

        let classification = super::classify_for_merge("git-mcp", &candidates, Some(&existing));
        assert_eq!(classification, MergeClassification::AutoSkip);
    }

    #[test]
    fn test_classify_for_merge_needs_decision_config_diff() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "old-package".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let candidates = vec![DetectedService {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "new-package".to_string()]), // Different
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        }];

        let classification = super::classify_for_merge("git-mcp", &candidates, Some(&existing));
        assert_eq!(classification, MergeClassification::NeedsDecision);
    }

    #[test]
    fn test_classify_for_merge_needs_decision_multiple_candidates() {
        use crate::models::mcp::McpTransportType;

        let candidates = vec![
            DetectedService {
                name: "git-mcp".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/project/.mcp.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            },
            DetectedService {
                name: "git-mcp".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: Some(vec!["--verbose".to_string()]),
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("~/.cursor/mcp.json"),
                adapter_id: "cursor".to_string(),
                scope: Some(ConfigScope::User),
            },
        ];

        let classification = super::classify_for_merge("git-mcp", &candidates, None);
        assert_eq!(classification, MergeClassification::NeedsDecision);
    }

    #[test]
    fn test_compute_config_diff() {
        use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};

        let existing = McpService {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["old-arg".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let detected = DetectedService {
            name: "test-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "uvx".to_string(), // Different command
            args: Some(vec!["new-arg".to_string()]), // Different args
            env: None,
            url: None,
            headers: None,
            source_file: PathBuf::from("/test/.mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        };

        let diffs = super::compute_config_diff(&existing, &detected);
        assert_eq!(diffs.len(), 2);

        let command_diff = diffs.iter().find(|d| d.field == "command").unwrap();
        assert_eq!(command_diff.existing_value, Some("npx".to_string()));
        assert_eq!(command_diff.new_value, Some("uvx".to_string()));

        let args_diff = diffs.iter().find(|d| d.field == "args").unwrap();
        assert_eq!(args_diff.existing_value, Some("old-arg".to_string()));
        assert_eq!(args_diff.new_value, Some("new-arg".to_string()));
    }

    #[test]
    fn test_generate_smart_takeover_preview_auto_create() {
        let db = Database::new_in_memory().unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/project/.mcp.json"),
            scope: Some(ConfigScope::Project),
            services: vec![DetectedService {
                name: "new-service".to_string(),
                transport_type: crate::models::mcp::McpTransportType::Stdio,
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/project/.mcp.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            parse_errors: Vec::new(),
        }];

        let preview = super::generate_smart_takeover_preview(&configs, &db, "/project").unwrap();

        assert_eq!(preview.auto_create.len(), 1);
        assert_eq!(preview.auto_skip.len(), 0);
        assert_eq!(preview.needs_decision.len(), 0);
        assert_eq!(preview.auto_create[0].service_name, "new-service");
        assert!(preview.can_auto_execute());
    }

    #[test]
    fn test_generate_smart_takeover_preview_auto_skip() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        // Create existing service
        db.create_mcp_service(&CreateMcpServiceRequest {
            name: "existing-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "test-mcp".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/test/.mcp.json".to_string()),
        })
        .unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/project/.mcp.json"),
            scope: Some(ConfigScope::Project),
            services: vec![DetectedService {
                name: "existing-service".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test-mcp".to_string()]), // Same config
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/project/.mcp.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            parse_errors: Vec::new(),
        }];

        let preview = super::generate_smart_takeover_preview(&configs, &db, "/project").unwrap();

        assert_eq!(preview.auto_create.len(), 0);
        assert_eq!(preview.auto_skip.len(), 1);
        assert_eq!(preview.needs_decision.len(), 0);
        assert_eq!(preview.auto_skip[0].service_name, "existing-service");
        assert!(preview.can_auto_execute());
    }

    #[test]
    fn test_generate_smart_takeover_preview_needs_decision() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        // Create existing service with different config
        db.create_mcp_service(&CreateMcpServiceRequest {
            name: "conflict-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "old-package".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/test/.mcp.json".to_string()),
        })
        .unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/project/.mcp.json"),
            scope: Some(ConfigScope::Project),
            services: vec![DetectedService {
                name: "conflict-service".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "new-package".to_string()]), // Different config
                env: None,
                url: None,
                headers: None,
                source_file: PathBuf::from("/project/.mcp.json"),
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            }],
            parse_errors: Vec::new(),
        }];

        let preview = super::generate_smart_takeover_preview(&configs, &db, "/project").unwrap();

        assert_eq!(preview.auto_create.len(), 0);
        assert_eq!(preview.auto_skip.len(), 0);
        assert_eq!(preview.needs_decision.len(), 1);
        assert_eq!(preview.needs_decision[0].service_name, "conflict-service");
        assert_eq!(
            preview.needs_decision[0].conflict_type,
            crate::models::mcp::ConflictType::ConfigDiff
        );
        assert!(!preview.needs_decision[0].diff_details.is_empty());
        assert!(!preview.can_auto_execute());
    }

    #[test]
    fn test_generate_smart_takeover_preview_filters_gateway_service() {
        let db = Database::new_in_memory().unwrap();

        let configs = vec![DetectedConfig {
            adapter_id: "claude".to_string(),
            path: PathBuf::from("/project/.mcp.json"),
            scope: Some(ConfigScope::Project),
            services: vec![
                DetectedService {
                    name: "mantra-gateway".to_string(),
                    transport_type: crate::models::mcp::McpTransportType::Http,
                    command: String::new(),
                    args: None,
                    env: None,
                    url: Some("http://127.0.0.1:8080/mcp".to_string()),
                    headers: None,
                    source_file: PathBuf::from("/project/.mcp.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                },
                DetectedService {
                    name: "real-service".to_string(),
                    transport_type: crate::models::mcp::McpTransportType::Stdio,
                    command: "npx".to_string(),
                    args: None,
                    env: None,
                    url: None,
                    headers: None,
                    source_file: PathBuf::from("/project/.mcp.json"),
                    adapter_id: "claude".to_string(),
                    scope: Some(ConfigScope::Project),
                },
            ],
            parse_errors: Vec::new(),
        }];

        let preview = super::generate_smart_takeover_preview(&configs, &db, "/project").unwrap();

        // mantra-gateway 应被过滤，只剩 real-service
        assert_eq!(preview.total_services, 1);
        assert_eq!(preview.auto_create.len(), 1);
        assert_eq!(preview.auto_create[0].service_name, "real-service");
    }

    #[test]
    fn test_takeover_preview_stats() {
        use crate::models::mcp::TakeoverPreview;

        let preview = TakeoverPreview {
            project_path: "/project".to_string(),
            auto_create: vec![],
            auto_skip: vec![],
            needs_decision: vec![],
            env_vars_needed: vec![],
            total_services: 0,
        };

        assert_eq!(preview.get_stats(), (0, 0, 0));
        assert!(preview.can_auto_execute());
        assert!(!preview.has_conflicts());
    }

    // ===== Task 3: 合并执行引擎测试 =====

    #[test]
    fn test_smart_takeover_result_empty() {
        let result = SmartTakeoverResult::empty();
        assert_eq!(result.created_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.updated_count, 0);
        assert_eq!(result.renamed_count, 0);
        assert!(result.errors.is_empty());
        assert!(result.created_service_ids.is_empty());
        assert!(result.takeover_backup_ids.is_empty());
        assert!(result.takeover_config_paths.is_empty());
        assert!(!result.gateway_running);
        assert!(result.is_success());
    }

    #[test]
    fn test_smart_takeover_result_is_success() {
        let mut result = SmartTakeoverResult::empty();
        assert!(result.is_success());

        result.errors.push("Some error".to_string());
        assert!(!result.is_success());
    }

    #[test]
    fn test_create_mcp_service_with_source() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        let request = CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "test-package".to_string()]),
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: Some("/project/.mcp.json".to_string()),
        };

        let service = db
            .create_mcp_service_with_source(&request, Some("claude"), Some("project"))
            .unwrap();

        assert_eq!(service.name, "test-service");
        assert_eq!(service.command, "npx");
        assert_eq!(service.source_adapter_id, Some("claude".to_string()));
        assert_eq!(service.source_scope, Some("project".to_string()));
    }

    #[test]
    fn test_link_service_to_project_with_detection() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        // Create a valid project first
        let (project, _) = db.get_or_create_project("/test/project").unwrap();
        let project_id = &project.id;

        // Create a service
        let service = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // Link with detection info
        let link = db
            .link_service_to_project_with_detection(
                project_id,
                &service.id,
                None,
                Some("cursor"),
                Some("/project/.cursor/mcp.json"),
            )
            .unwrap();

        assert_eq!(link.project_id, *project_id);
        assert_eq!(link.service_id, service.id);
        assert_eq!(link.detected_adapter_id, Some("cursor".to_string()));
        assert_eq!(
            link.detected_config_path,
            Some("/project/.cursor/mcp.json".to_string())
        );

        // Verify via get_project_service_link
        let retrieved = db.get_project_service_link(project_id, &service.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.detected_adapter_id, Some("cursor".to_string()));
    }

    #[test]
    fn test_get_project_service_links() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        // Create a valid project first
        let (project, _) = db.get_or_create_project("/test/project").unwrap();
        let project_id = &project.id;

        // Create services
        let service1 = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "service-1".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let service2 = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "service-2".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "uvx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // Link services
        db.link_service_to_project(project_id, &service1.id, None)
            .unwrap();
        db.link_service_to_project_with_detection(
            project_id,
            &service2.id,
            None,
            Some("cursor"),
            Some("/project/.cursor/mcp.json"),
        )
        .unwrap();

        // Get all links
        let links = db.get_project_service_links(project_id).unwrap();
        assert_eq!(links.len(), 2);

        // Verify detection info is preserved
        let link2 = links.iter().find(|l| l.service_id == service2.id).unwrap();
        assert_eq!(link2.detected_adapter_id, Some("cursor".to_string()));
        assert_eq!(
            link2.detected_config_path,
            Some("/project/.cursor/mcp.json".to_string())
        );
    }

    #[test]
    fn test_is_service_linked() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let db = Database::new_in_memory().unwrap();

        // Create a valid project first
        let (project, _) = db.get_or_create_project("/test/project").unwrap();
        let project_id = &project.id;

        // Create a service
        let service = db
            .create_mcp_service(&CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: McpTransportType::Stdio,
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // Before linking
        let is_linked = super::is_service_linked(&db, project_id, &service.id).unwrap();
        assert!(!is_linked);

        // After linking
        db.link_service_to_project(project_id, &service.id, None)
            .unwrap();
        let is_linked = super::is_service_linked(&db, project_id, &service.id).unwrap();
        assert!(is_linked);
    }

    #[test]
    fn test_has_scope_conflict_true() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::McpTransportType;

        let candidates = vec![
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test".to_string()]),
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/mcp.json"), // project level (no /.)
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            },
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test".to_string()]),
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("~/.claude.json"), // user level (starts with ~)
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::User),
            },
        ];

        assert!(super::has_scope_conflict(&candidates));
    }

    #[test]
    fn test_has_scope_conflict_false_same_scope() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::McpTransportType;

        let candidates = vec![
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test".to_string()]),
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/mcp.json"), // project level
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            },
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: Some(vec!["-y".to_string(), "test".to_string()]),
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/cursor-mcp.json"), // also project level
                adapter_id: "cursor".to_string(),
                scope: Some(ConfigScope::Project),
            },
        ];

        assert!(!super::has_scope_conflict(&candidates));
    }

    #[test]
    fn test_has_scope_conflict_false_single_candidate() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::McpTransportType;

        let candidates = vec![DetectedService {
            name: "test-service".to_string(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            transport_type: McpTransportType::Stdio,
            source_file: PathBuf::from("/project/mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        }];

        assert!(!super::has_scope_conflict(&candidates));
    }

    #[test]
    fn test_determine_conflict_type_scope_conflict() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::{McpTransportType, ConflictType};

        let candidates = vec![
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/mcp.json"), // project level
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            },
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("~/.claude.json"), // user level
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::User),
            },
        ];

        let conflict_type = super::determine_conflict_type(&candidates, None);
        assert_eq!(conflict_type, ConflictType::ScopeConflict);
    }

    #[test]
    fn test_determine_conflict_type_multi_source() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::{McpTransportType, ConflictType};

        let candidates = vec![
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/mcp.json"), // project level
                adapter_id: "claude".to_string(),
                scope: Some(ConfigScope::Project),
            },
            DetectedService {
                name: "test-service".to_string(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                transport_type: McpTransportType::Stdio,
                source_file: PathBuf::from("/project/cursor-mcp.json"), // also project level, different adapter
                adapter_id: "cursor".to_string(),
                scope: Some(ConfigScope::Project),
            },
        ];

        let conflict_type = super::determine_conflict_type(&candidates, None);
        assert_eq!(conflict_type, ConflictType::MultiSource);
    }

    #[test]
    fn test_determine_conflict_type_config_diff() {
        use std::path::PathBuf;
        use crate::services::mcp_config::DetectedService;
        use crate::services::mcp_adapters::ConfigScope;
        use crate::models::mcp::{McpTransportType, ConflictType};

        let candidates = vec![DetectedService {
            name: "test-service".to_string(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            transport_type: McpTransportType::Stdio,
            source_file: PathBuf::from("/project/mcp.json"),
            adapter_id: "claude".to_string(),
            scope: Some(ConfigScope::Project),
        }];

        let conflict_type = super::determine_conflict_type(&candidates, None);
        assert_eq!(conflict_type, ConflictType::ConfigDiff);
    }

    // ===== Story 11.20: 工具检测测试 =====

    #[test]
    fn test_detect_installed_tools_returns_all_tools() {
        // 测试 detect_installed_tools 返回所有 4 个工具
        let result = super::detect_installed_tools();

        assert_eq!(result.total_count, 4);
        assert_eq!(result.tools.len(), 4);

        // 验证所有工具类型都在结果中
        let adapter_ids: Vec<&str> = result.tools.iter().map(|t| t.adapter_id.as_str()).collect();
        assert!(adapter_ids.contains(&"claude"));
        assert!(adapter_ids.contains(&"cursor"));
        assert!(adapter_ids.contains(&"codex"));
        assert!(adapter_ids.contains(&"gemini"));
    }

    #[test]
    fn test_detect_installed_tools_has_correct_display_names() {
        let result = super::detect_installed_tools();

        for tool in &result.tools {
            match tool.adapter_id.as_str() {
                "claude" => assert_eq!(tool.display_name, "Claude Code"),
                "cursor" => assert_eq!(tool.display_name, "Cursor"),
                "codex" => assert_eq!(tool.display_name, "Codex"),
                "gemini" => assert_eq!(tool.display_name, "Gemini CLI"),
                _ => panic!("Unknown adapter_id: {}", tool.adapter_id),
            }
        }
    }

    #[test]
    fn test_detect_installed_tools_user_config_paths() {
        let result = super::detect_installed_tools();

        for tool in &result.tools {
            let path_str = tool.user_config_path.to_string_lossy();
            match tool.adapter_id.as_str() {
                "claude" => assert!(path_str.ends_with(".claude.json")),
                "cursor" => assert!(path_str.ends_with(".cursor/mcp.json") || path_str.ends_with(".cursor\\mcp.json")),
                "codex" => assert!(path_str.ends_with(".codex/config.toml") || path_str.ends_with(".codex\\config.toml")),
                "gemini" => assert!(path_str.ends_with(".gemini/settings.json") || path_str.ends_with(".gemini\\settings.json")),
                _ => panic!("Unknown adapter_id: {}", tool.adapter_id),
            }
        }
    }

    #[test]
    fn test_detect_installed_tools_installed_count_matches() {
        let result = super::detect_installed_tools();

        let actual_installed_count = result.tools.iter().filter(|t| t.installed).count();
        assert_eq!(result.installed_count, actual_installed_count);
    }

    #[test]
    fn test_detect_installed_tools_consistency() {
        let result = super::detect_installed_tools();

        for tool in &result.tools {
            // installed 应该与 user_config_exists 一致
            assert_eq!(tool.installed, tool.user_config_exists);
        }
    }

    #[test]
    fn test_tool_type_all_returns_four_tools() {
        use crate::models::mcp::ToolType;

        let all_tools = ToolType::all();
        assert_eq!(all_tools.len(), 4);

        // 验证包含所有类型
        assert!(all_tools.contains(&ToolType::ClaudeCode));
        assert!(all_tools.contains(&ToolType::Cursor));
        assert!(all_tools.contains(&ToolType::Codex));
        assert!(all_tools.contains(&ToolType::GeminiCli));
    }

    // ===== Story 11.20: 全 Scope 扫描测试 =====

    #[test]
    fn test_scan_all_tool_configs_returns_all_tools() {
        let temp_dir = TempDir::new().unwrap();
        let result = super::scan_all_tool_configs(temp_dir.path());

        // 应该返回所有 4 个工具的结果
        assert_eq!(result.tools.len(), 4);

        // 验证所有工具类型都在结果中
        let adapter_ids: Vec<&str> = result.tools.iter().map(|t| t.adapter_id.as_str()).collect();
        assert!(adapter_ids.contains(&"claude"));
        assert!(adapter_ids.contains(&"cursor"));
        assert!(adapter_ids.contains(&"codex"));
        assert!(adapter_ids.contains(&"gemini"));
    }

    #[test]
    fn test_scan_all_tool_configs_with_project_config() {
        let temp_dir = TempDir::new().unwrap();

        // 创建 Claude Code 项目级配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "git-mcp": {
                        "command": "npx",
                        "args": ["-y", "@anthropic/git-mcp"]
                    },
                    "postgres-mcp": {
                        "command": "uvx",
                        "args": ["mcp-server-postgres"]
                    }
                }
            }"#,
        )
        .unwrap();

        let result = super::scan_all_tool_configs(temp_dir.path());

        // 查找 Claude Code 的结果
        let claude = result.tools.iter().find(|t| t.adapter_id == "claude").unwrap();

        // 验证 project_scope 有配置
        assert!(claude.project_scope.is_some());
        let project_scope = claude.project_scope.as_ref().unwrap();
        assert!(project_scope.exists);
        assert_eq!(project_scope.service_count, 2);
        assert!(project_scope.service_names.contains(&"git-mcp".to_string()));
        assert!(project_scope.service_names.contains(&"postgres-mcp".to_string()));

        // 验证总服务数量
        assert_eq!(claude.total_service_count, 2);
    }

    #[test]
    fn test_scan_all_tool_configs_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let result = super::scan_all_tool_configs(temp_dir.path());

        // 空项目的 project_scope 应该没有服务
        // 注意：user_scope 可能有服务（来自用户实际的配置文件）
        for tool in &result.tools {
            if let Some(ref project_scope) = tool.project_scope {
                // 项目级配置应该不存在或没有服务
                assert!(
                    !project_scope.exists || project_scope.service_count == 0,
                    "Tool {} has unexpected project scope services: {:?}",
                    tool.adapter_id,
                    project_scope.service_names
                );
            }
        }
    }

    #[test]
    fn test_scan_all_tool_configs_project_path_stored() {
        let temp_dir = TempDir::new().unwrap();
        let result = super::scan_all_tool_configs(temp_dir.path());

        assert_eq!(
            result.project_path,
            temp_dir.path().to_string_lossy().to_string()
        );
    }

    #[test]
    fn test_scan_all_tool_configs_cursor_project() {
        let temp_dir = TempDir::new().unwrap();

        // 创建 Cursor 项目级配置
        let cursor_dir = temp_dir.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        let cursor_config = cursor_dir.join("mcp.json");
        fs::write(
            &cursor_config,
            r#"{
                "mcpServers": {
                    "filesystem": {
                        "command": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-filesystem"]
                    }
                }
            }"#,
        )
        .unwrap();

        let result = super::scan_all_tool_configs(temp_dir.path());

        // 查找 Cursor 的结果
        let cursor = result.tools.iter().find(|t| t.adapter_id == "cursor").unwrap();

        // 验证 project_scope 有配置
        assert!(cursor.project_scope.is_some());
        let project_scope = cursor.project_scope.as_ref().unwrap();
        assert!(project_scope.exists);
        assert_eq!(project_scope.service_count, 1);
        assert!(project_scope.service_names.contains(&"filesystem".to_string()));
    }

    #[test]
    fn test_scan_all_tool_configs_filters_gateway() {
        let temp_dir = TempDir::new().unwrap();

        // 创建包含 gateway 服务的配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "git-mcp": {
                        "command": "npx",
                        "args": ["-y", "@anthropic/git-mcp"]
                    },
                    "mantra-gateway": {
                        "url": "http://127.0.0.1:8080/mcp"
                    }
                }
            }"#,
        )
        .unwrap();

        let result = super::scan_all_tool_configs(temp_dir.path());

        // 查找 Claude Code 的结果
        let claude = result.tools.iter().find(|t| t.adapter_id == "claude").unwrap();
        let project_scope = claude.project_scope.as_ref().unwrap();

        // mantra-gateway 应该被过滤掉
        assert_eq!(project_scope.service_count, 1);
        assert!(project_scope.service_names.contains(&"git-mcp".to_string()));
        assert!(!project_scope.service_names.contains(&"mantra-gateway".to_string()));
    }

    #[test]
    fn test_scan_all_tool_configs_statistics() {
        let temp_dir = TempDir::new().unwrap();

        // 创建 Claude Code 项目级配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "service1": {"command": "cmd1"},
                    "service2": {"command": "cmd2"},
                    "service3": {"command": "cmd3"}
                }
            }"#,
        )
        .unwrap();

        // 创建 Cursor 项目级配置
        let cursor_dir = temp_dir.path().join(".cursor");
        fs::create_dir(&cursor_dir).unwrap();
        let cursor_config = cursor_dir.join("mcp.json");
        fs::write(
            &cursor_config,
            r#"{
                "mcpServers": {
                    "cursor-service": {"command": "cmd"}
                }
            }"#,
        )
        .unwrap();

        let result = super::scan_all_tool_configs(temp_dir.path());

        // 验证项目级服务数量
        // 注意：total_service_count 可能包含用户级配置中的服务
        let project_service_count: usize = result.tools.iter()
            .filter_map(|t| t.project_scope.as_ref())
            .map(|s| s.service_count)
            .sum();
        assert_eq!(project_service_count, 4);

        // 验证有项目级配置的工具数量
        let tools_with_project_config: usize = result.tools.iter()
            .filter(|t| t.project_scope.as_ref().map_or(false, |s| s.exists && s.service_count > 0))
            .count();
        assert_eq!(tools_with_project_config, 2);
    }

    // ===== Story 11.20: 全工具接管预览测试 =====

    #[test]
    fn test_generate_full_tool_takeover_preview_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let result = super::generate_full_tool_takeover_preview(temp_dir.path(), &db).unwrap();

        // 验证返回所有工具
        assert_eq!(result.tools.len(), 4);

        // 空项目应该没有冲突
        assert!(result.can_auto_execute);
        assert_eq!(result.total_conflict_count, 0);

        // 项目级配置应该没有服务
        for tool in &result.tools {
            if let Some(ref project_scope) = tool.project_scope_preview {
                assert!(
                    !project_scope.exists || project_scope.service_count == 0,
                    "Tool {} has unexpected project scope services",
                    tool.adapter_id
                );
            }
        }
    }

    #[test]
    fn test_generate_full_tool_takeover_preview_with_services() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建 Claude Code 项目级配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "git-mcp": {
                        "command": "npx",
                        "args": ["-y", "@anthropic/git-mcp"]
                    },
                    "postgres-mcp": {
                        "command": "uvx",
                        "args": ["mcp-server-postgres"]
                    }
                }
            }"#,
        )
        .unwrap();

        let result = super::generate_full_tool_takeover_preview(temp_dir.path(), &db).unwrap();

        // 查找 Claude Code 的预览
        let claude = result.tools.iter().find(|t| t.adapter_id == "claude").unwrap();

        // 验证 project_scope_preview
        assert!(claude.project_scope_preview.is_some());
        let project_preview = claude.project_scope_preview.as_ref().unwrap();
        assert!(project_preview.exists);
        assert_eq!(project_preview.service_count, 2);

        // 新服务应该在 auto_create 中
        assert_eq!(project_preview.auto_create.len(), 2);
        assert_eq!(project_preview.auto_skip.len(), 0);
        assert_eq!(project_preview.needs_decision.len(), 0);
    }

    #[test]
    fn test_generate_full_tool_takeover_preview_with_existing_service() {
        use crate::models::mcp::{CreateMcpServiceRequest, McpServiceSource, McpTransportType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 先创建一个全局池中的服务
        db.create_mcp_service(&CreateMcpServiceRequest {
            name: "git-mcp".to_string(),
            transport_type: McpTransportType::Stdio,
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
            url: None,
            env: None,
            headers: None,
            source: McpServiceSource::Imported,
            source_file: None,
        })
        .unwrap();

        // 创建项目配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "git-mcp": {
                        "command": "npx",
                        "args": ["-y", "@anthropic/git-mcp"]
                    },
                    "new-service": {
                        "command": "new-cmd"
                    }
                }
            }"#,
        )
        .unwrap();

        let result = super::generate_full_tool_takeover_preview(temp_dir.path(), &db).unwrap();

        // 查找 Claude Code 的预览
        let claude = result.tools.iter().find(|t| t.adapter_id == "claude").unwrap();
        let project_preview = claude.project_scope_preview.as_ref().unwrap();

        // git-mcp 应该在 auto_skip 中（已存在）
        // new-service 应该在 auto_create 中
        assert_eq!(project_preview.auto_skip.len(), 1);
        assert_eq!(project_preview.auto_create.len(), 1);

        let skip_names: Vec<&str> = project_preview.auto_skip.iter()
            .map(|s| s.service_name.as_str())
            .collect();
        assert!(skip_names.contains(&"git-mcp"));

        let create_names: Vec<&str> = project_preview.auto_create.iter()
            .map(|s| s.service_name.as_str())
            .collect();
        assert!(create_names.contains(&"new-service"));
    }

    #[test]
    fn test_generate_full_tool_takeover_preview_installed_selection() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let result = super::generate_full_tool_takeover_preview(temp_dir.path(), &db).unwrap();

        // 已安装的工具应该默认选中
        for tool in &result.tools {
            assert_eq!(tool.selected, tool.installed);
        }
    }

    #[test]
    fn test_generate_full_tool_takeover_preview_stats() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建配置
        let mcp_json = temp_dir.path().join(".mcp.json");
        fs::write(
            &mcp_json,
            r#"{
                "mcpServers": {
                    "service1": {"command": "cmd1"},
                    "service2": {"command": "cmd2"}
                }
            }"#,
        )
        .unwrap();

        let result = super::generate_full_tool_takeover_preview(temp_dir.path(), &db).unwrap();

        // 验证项目级服务数量
        let claude = result.tools.iter().find(|t| t.adapter_id == "claude").unwrap();
        assert_eq!(claude.total_service_count, 2);

        // 验证无冲突
        assert!(result.can_auto_execute);
        assert_eq!(result.total_conflict_count, 0);
    }

    // ===== Story 11.21: Local Scope 备份和恢复测试 =====

    #[test]
    fn test_local_scope_backup_path_generation() {
        use super::executor::ImportExecutor;

        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);
        let executor = ImportExecutor::new(&db, &env_manager);

        // 测试备份路径生成
        let project_path = "/home/user/projects/my-project";
        let backup_path = executor.generate_local_scope_backup_path_for_test(project_path);

        // 验证路径格式
        assert!(backup_path.to_string_lossy().contains(".mantra"));
        assert!(backup_path.to_string_lossy().contains("local-scope"));
        assert!(backup_path.to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn test_local_scope_takeover_and_restore() {
        use crate::models::mcp::{TakeoverScope, TakeoverStatus, ToolType};
        use super::executor::ImportExecutor;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建模拟的 ~/.claude.json 配置文件
        let claude_config = temp_dir.path().join(".claude.json");
        let project_path = "/home/user/my-project";
        let config_content = format!(r#"{{
            "mcpServers": {{
                "global-service": {{"command": "global"}}
            }},
            "projects": {{
                "{}": {{
                    "mcpServers": {{
                        "local-service-1": {{"command": "local1"}},
                        "local-service-2": {{"command": "local2"}}
                    }}
                }}
            }}
        }}"#, project_path);
        fs::write(&claude_config, &config_content).unwrap();

        // 执行 Local Scope 接管
        let mut executor = ImportExecutor::new(&db, &env_manager);
        let backup_id = executor
            .apply_local_scope_takeover(&claude_config, project_path)
            .unwrap();

        // 验证备份记录已创建
        let backup = db.get_takeover_backup_by_id(&backup_id).unwrap().unwrap();
        assert_eq!(backup.tool_type, ToolType::ClaudeCode);
        assert_eq!(backup.scope, TakeoverScope::Local);
        assert_eq!(backup.status, TakeoverStatus::Active);
        assert_eq!(
            backup.project_path.unwrap().to_string_lossy(),
            project_path
        );

        // 验证备份文件存在
        assert!(backup.backup_path.exists());

        // 验证备份内容
        let backup_content = fs::read_to_string(&backup.backup_path).unwrap();
        let backup_json: serde_json::Value = serde_json::from_str(&backup_content).unwrap();
        assert!(backup_json.get("local-service-1").is_some());
        assert!(backup_json.get("local-service-2").is_some());

        // 验证配置文件中的 mcpServers 已被清空
        let updated_config = fs::read_to_string(&claude_config).unwrap();
        let updated_json: serde_json::Value = serde_json::from_str(&updated_config).unwrap();
        let project_mcp = &updated_json["projects"][project_path]["mcpServers"];
        assert!(project_mcp.is_null() || project_mcp.as_object().map_or(false, |o| o.is_empty()));

        // 验证全局 mcpServers 未受影响
        assert!(updated_json["mcpServers"]["global-service"].is_object());

        // 执行恢复
        let restored_backup = super::restore_local_scope_takeover(&db, &backup_id).unwrap();
        assert_eq!(restored_backup.status, TakeoverStatus::Restored);

        // 验证配置文件已恢复
        let restored_config = fs::read_to_string(&claude_config).unwrap();
        let restored_json: serde_json::Value = serde_json::from_str(&restored_config).unwrap();
        let restored_mcp = &restored_json["projects"][project_path]["mcpServers"];
        assert!(restored_mcp["local-service-1"].is_object());
        assert!(restored_mcp["local-service-2"].is_object());
    }

    #[test]
    fn test_local_scope_takeover_idempotent() {
        use super::executor::ImportExecutor;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建配置文件
        let claude_config = temp_dir.path().join(".claude.json");
        let project_path = "/home/user/my-project";
        let config_content = format!(r#"{{
            "projects": {{
                "{}": {{
                    "mcpServers": {{
                        "test-service": {{"command": "test"}}
                    }}
                }}
            }}
        }}"#, project_path);
        fs::write(&claude_config, &config_content).unwrap();

        // 第一次接管
        let mut executor = ImportExecutor::new(&db, &env_manager);
        let backup_id_1 = executor
            .apply_local_scope_takeover(&claude_config, project_path)
            .unwrap();

        // 第二次接管（应该返回相同的 backup_id）
        let backup_id_2 = executor
            .apply_local_scope_takeover(&claude_config, project_path)
            .unwrap();

        assert_eq!(backup_id_1, backup_id_2);
    }

    #[test]
    fn test_local_scope_takeover_no_mcp_servers_error() {
        use super::executor::ImportExecutor;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();
        let env_manager = crate::services::EnvManager::new(&[0u8; 32]);

        // 创建没有 mcpServers 的配置
        let claude_config = temp_dir.path().join(".claude.json");
        let project_path = "/home/user/my-project";
        let config_content = format!(r#"{{
            "projects": {{
                "{}": {{
                    "allowedTools": ["tool1"]
                }}
            }}
        }}"#, project_path);
        fs::write(&claude_config, &config_content).unwrap();

        // 接管应该失败
        let mut executor = ImportExecutor::new(&db, &env_manager);
        let result = executor.apply_local_scope_takeover(&claude_config, project_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No mcpServers found"));
    }

    #[test]
    fn test_restore_all_local_scope_takeovers() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, TakeoverStatus, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建模拟的配置文件
        let claude_config = temp_dir.path().join(".claude.json");
        let config_content = r#"{
            "projects": {
                "/project-a": { "mcpServers": {} },
                "/project-b": { "mcpServers": {} }
            }
        }"#;
        fs::write(&claude_config, config_content).unwrap();

        // 创建多个 local scope 备份记录
        for (i, project) in ["/project-a", "/project-b"].iter().enumerate() {
            let backup_file = temp_dir.path().join(format!("backup-{}.json", i));
            fs::write(&backup_file, r#"{"service": {"command": "test"}}"#).unwrap();

            let backup = TakeoverBackup::new_with_scope(
                ToolType::ClaudeCode,
                claude_config.clone(),
                backup_file,
                TakeoverScope::Local,
                Some(PathBuf::from(project)),
            );
            db.create_takeover_backup(&backup).unwrap();
        }

        // 验证有 2 个活跃的 local scope 备份
        let active_backups = db.get_active_local_scope_takeovers().unwrap();
        assert_eq!(active_backups.len(), 2);

        // 恢复所有
        let (restored_count, errors) = super::restore_all_local_scope_takeovers(&db).unwrap();
        assert_eq!(restored_count, 2);
        assert!(errors.is_empty());

        // 验证所有备份都已恢复
        let active_backups_after = db.get_active_local_scope_takeovers().unwrap();
        assert!(active_backups_after.is_empty());
    }

    // ===== Story 11.22 Task 4: 原子恢复重构测试 =====

    #[test]
    fn test_atomic_restore_success() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, TakeoverStatus, ToolType};
        use crate::services::atomic_fs;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建原始配置文件（被接管后的内容）
        let original_path = temp_dir.path().join("config.json");
        let takeover_content = r#"{"mcpServers": {"gateway": {"url": "http://localhost:3000"}}}"#;
        fs::write(&original_path, takeover_content).unwrap();

        // 创建备份文件（接管前的原始内容）
        let backup_content = r#"{"mcpServers": {"my-tool": {"command": "npx", "args": ["my-tool"]}}}"#;
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        fs::write(&backup_path, backup_content).unwrap();

        // 计算备份文件 hash
        let backup_hash = atomic_fs::calculate_file_hash(&backup_path).unwrap();

        // 创建备份记录（带 hash）
        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            backup_hash.clone(),
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 执行恢复
        let restored = super::restore_mcp_takeover(&db, &backup_id).unwrap();

        // 验证恢复状态
        assert_eq!(restored.status, TakeoverStatus::Restored);

        // 验证原始文件内容已恢复
        let restored_content = fs::read_to_string(&original_path).unwrap();
        assert_eq!(restored_content, backup_content);
    }

    #[test]
    fn test_atomic_restore_hash_verification_failure() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建配置和备份文件
        let original_path = temp_dir.path().join("config.json");
        fs::write(&original_path, "takeover content").unwrap();

        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        fs::write(&backup_path, "backup content").unwrap();

        // 创建备份记录，使用错误的 hash
        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            "wrong_hash_value".to_string(),
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 恢复应失败（hash 不匹配）
        let result = super::restore_mcp_takeover(&db, &backup_id);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("integrity check failed"));

        // 验证原始文件未被修改
        let content = fs::read_to_string(&original_path).unwrap();
        assert_eq!(content, "takeover content");
    }

    #[test]
    fn test_atomic_restore_backup_file_missing() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        fs::write(&original_path, "current content").unwrap();

        // 备份文件不存在
        let backup_path = temp_dir.path().join("nonexistent.mantra-backup");

        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path,
            TakeoverScope::User,
            None,
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 恢复应失败
        let result = super::restore_mcp_takeover(&db, &backup_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Backup file not found"));

        // 原始文件未被修改
        let content = fs::read_to_string(&original_path).unwrap();
        assert_eq!(content, "current content");
    }

    #[test]
    fn test_atomic_restore_already_restored() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        fs::write(&original_path, "current").unwrap();
        fs::write(&backup_path, "backup").unwrap();

        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 第一次恢复成功
        super::restore_mcp_takeover(&db, &backup_id).unwrap();

        // 第二次恢复应失败（已恢复）
        let result = super::restore_mcp_takeover(&db, &backup_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already restored"));
    }

    #[test]
    fn test_atomic_restore_without_hash() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, TakeoverStatus, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 旧备份记录没有 hash（兼容性测试）
        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        let backup_content = "original backup content";
        fs::write(&original_path, "takeover content").unwrap();
        fs::write(&backup_path, backup_content).unwrap();

        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
        );
        // backup_hash 默认为 None
        assert!(backup.backup_hash.is_none());

        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 恢复应该成功（跳过 hash 验证）
        let restored = super::restore_mcp_takeover(&db, &backup_id).unwrap();
        assert_eq!(restored.status, TakeoverStatus::Restored);

        // 验证内容已恢复
        let restored_content = fs::read_to_string(&original_path).unwrap();
        assert_eq!(restored_content, backup_content);
    }

    #[test]
    fn test_atomic_restore_preserves_original_on_failure() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_content = "important config that must not be corrupted";
        let original_path = temp_dir.path().join("config.json");
        fs::write(&original_path, original_content).unwrap();

        // 备份文件存在但 hash 不匹配
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        fs::write(&backup_path, "backup content").unwrap();

        let backup = TakeoverBackup::new_with_hash(
            ToolType::Cursor,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            "definitely_wrong_hash".to_string(),
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 恢复失败
        let result = super::restore_mcp_takeover(&db, &backup_id);
        assert!(result.is_err());

        // 原始文件内容必须完全不变
        let content_after = fs::read_to_string(&original_path).unwrap();
        assert_eq!(content_after, original_content);

        // 备份记录状态仍然是 Active（未被标记为 restored）
        let backup_after = db.get_takeover_backup_by_id(&backup_id).unwrap().unwrap();
        assert_eq!(backup_after.status, crate::models::mcp::TakeoverStatus::Active);
    }

    #[test]
    fn test_atomic_restore_by_tool_type() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, TakeoverStatus, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("settings.json");
        let backup_path = temp_dir.path().join("settings.json.mantra-backup");
        let backup_content = "original cursor config";
        fs::write(&original_path, "takeover content").unwrap();
        fs::write(&backup_path, backup_content).unwrap();

        let backup = TakeoverBackup::new_with_scope(
            ToolType::Cursor,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
        );
        db.create_takeover_backup(&backup).unwrap();

        // 按工具类型恢复
        let result = super::restore_mcp_takeover_by_tool(&db, &ToolType::Cursor).unwrap();
        assert!(result.is_some());
        let restored = result.unwrap();
        assert_eq!(restored.status, TakeoverStatus::Restored);

        // 验证文件内容
        let content = fs::read_to_string(&original_path).unwrap();
        assert_eq!(content, backup_content);
    }

    // ===== Story 11.22 Task 7: 备份完整性检查测试 =====

    #[test]
    fn test_list_takeover_backups_with_integrity_all_valid() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};
        use crate::services::atomic_fs;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建有效的备份
        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        let backup_content = "original content";

        fs::write(&original_path, "current content").unwrap();
        fs::write(&backup_path, backup_content).unwrap();

        let backup_hash = atomic_fs::calculate_file_hash(&backup_path).unwrap();

        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            backup_hash.clone(),
        );
        db.create_takeover_backup(&backup).unwrap();

        // 获取完整性列表
        let results = super::list_takeover_backups_with_integrity(&db).unwrap();

        assert_eq!(results.len(), 1);
        let item = &results[0];
        assert!(item.backup_file_exists);
        assert!(item.original_file_exists);
        assert_eq!(item.hash_valid, Some(true));
    }

    #[test]
    fn test_list_takeover_backups_with_integrity_missing_backup() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("missing.mantra-backup");

        fs::write(&original_path, "content").unwrap();
        // 备份文件不创建

        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            "some_hash".to_string(),
        );
        db.create_takeover_backup(&backup).unwrap();

        let results = super::list_takeover_backups_with_integrity(&db).unwrap();

        assert_eq!(results.len(), 1);
        let item = &results[0];
        assert!(!item.backup_file_exists);
        assert!(item.original_file_exists);
        assert_eq!(item.hash_valid, Some(false)); // 文件不存在，hash 验证失败
    }

    #[test]
    fn test_list_takeover_backups_with_integrity_hash_mismatch() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");

        fs::write(&original_path, "current").unwrap();
        fs::write(&backup_path, "backup content").unwrap();

        // 使用错误的 hash
        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
            "wrong_hash".to_string(),
        );
        db.create_takeover_backup(&backup).unwrap();

        let results = super::list_takeover_backups_with_integrity(&db).unwrap();

        assert_eq!(results.len(), 1);
        let item = &results[0];
        assert!(item.backup_file_exists);
        assert_eq!(item.hash_valid, Some(false)); // hash 不匹配
    }

    #[test]
    fn test_list_takeover_backups_with_integrity_no_hash() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");

        fs::write(&original_path, "current").unwrap();
        fs::write(&backup_path, "backup").unwrap();

        // 不设置 hash（旧备份兼容）
        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            original_path.clone(),
            backup_path.clone(),
            TakeoverScope::User,
            None,
        );
        db.create_takeover_backup(&backup).unwrap();

        let results = super::list_takeover_backups_with_integrity(&db).unwrap();

        assert_eq!(results.len(), 1);
        let item = &results[0];
        assert!(item.backup_file_exists);
        assert!(item.hash_valid.is_none()); // 无 hash 记录
    }

    #[test]
    fn test_delete_invalid_backups_removes_missing_files() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建一个有效备份和一个无效备份
        let valid_original = temp_dir.path().join("valid.json");
        let valid_backup = temp_dir.path().join("valid.json.mantra-backup");
        fs::write(&valid_original, "current").unwrap();
        fs::write(&valid_backup, "backup").unwrap();

        let valid = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            valid_original,
            valid_backup,
            TakeoverScope::User,
            None,
        );
        db.create_takeover_backup(&valid).unwrap();

        // 无效备份（文件不存在）
        let invalid_original = temp_dir.path().join("invalid.json");
        let invalid_backup = temp_dir.path().join("missing.mantra-backup");
        fs::write(&invalid_original, "current").unwrap();
        // 不创建 invalid_backup 文件

        let invalid = TakeoverBackup::new_with_scope(
            ToolType::Cursor,
            invalid_original,
            invalid_backup,
            TakeoverScope::User,
            None,
        );
        let invalid_id = invalid.id.clone();
        db.create_takeover_backup(&invalid).unwrap();

        // 删除无效备份
        let deleted = super::delete_invalid_backups(&db).unwrap();

        assert_eq!(deleted, 1);

        // 验证无效记录已删除
        assert!(db.get_takeover_backup_by_id(&invalid_id).unwrap().is_none());

        // 验证有效记录仍存在
        let remaining = super::list_takeover_backups_with_integrity(&db).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_delete_invalid_backups_removes_hash_mismatch() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");

        fs::write(&original_path, "current").unwrap();
        fs::write(&backup_path, "backup content").unwrap();

        // hash 不匹配的备份
        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path,
            backup_path,
            TakeoverScope::User,
            None,
            "wrong_hash".to_string(),
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 删除无效备份
        let deleted = super::delete_invalid_backups(&db).unwrap();

        assert_eq!(deleted, 1);
        assert!(db.get_takeover_backup_by_id(&backup_id).unwrap().is_none());
    }

    #[test]
    fn test_delete_invalid_backups_keeps_valid_with_hash() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};
        use crate::services::atomic_fs;

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        let original_path = temp_dir.path().join("config.json");
        let backup_path = temp_dir.path().join("config.json.mantra-backup");
        let backup_content = "backup content";

        fs::write(&original_path, "current").unwrap();
        fs::write(&backup_path, backup_content).unwrap();

        let backup_hash = atomic_fs::calculate_file_hash(&backup_path).unwrap();

        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            original_path,
            backup_path,
            TakeoverScope::User,
            None,
            backup_hash,
        );
        let backup_id = backup.id.clone();
        db.create_takeover_backup(&backup).unwrap();

        // 删除无效备份
        let deleted = super::delete_invalid_backups(&db).unwrap();

        assert_eq!(deleted, 0); // 没有无效备份
        assert!(db.get_takeover_backup_by_id(&backup_id).unwrap().is_some());
    }

// ===== Story 11.23: 备份版本管理测试 =====

    #[test]
    fn test_cleanup_old_backups_keeps_recent() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建 7 个备份
        for i in 0..7 {
            let backup_path = temp_dir.path().join(format!("backup_{}.json", i));
            let original_path = temp_dir.path().join("config.json");

            fs::write(&backup_path, format!("backup content {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::ClaudeCode,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();

            // 添加小延迟确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 验证有 7 个备份
        let backups = db.get_takeover_backups(None).unwrap();
        assert_eq!(backups.len(), 7);

        // 清理，保留 5 个
        let result = super::cleanup_old_backups(
            &db,
            &ToolType::ClaudeCode,
            &TakeoverScope::User,
            None,
            5,
        ).unwrap();

        assert_eq!(result.deleted_count, 2);

        // 验证只剩 5 个
        let remaining = db.get_takeover_backups(None).unwrap();
        assert_eq!(remaining.len(), 5);
    }

    #[test]
    fn test_cleanup_old_backups_handles_different_groups() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 为 ClaudeCode User scope 创建 3 个备份
        for i in 0..3 {
            let backup_path = temp_dir.path().join(format!("claude_backup_{}.json", i));
            let original_path = temp_dir.path().join("claude_config.json");

            fs::write(&backup_path, format!("claude backup {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::ClaudeCode,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();
        }

        // 为 Cursor User scope 创建 3 个备份
        for i in 0..3 {
            let backup_path = temp_dir.path().join(format!("cursor_backup_{}.json", i));
            let original_path = temp_dir.path().join("cursor_config.json");

            fs::write(&backup_path, format!("cursor backup {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::Cursor,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();
        }

        // 清理 ClaudeCode，保留 1 个
        let result = super::cleanup_old_backups(
            &db,
            &ToolType::ClaudeCode,
            &TakeoverScope::User,
            None,
            1,
        ).unwrap();

        assert_eq!(result.deleted_count, 2);

        // Cursor 应该不受影响
        let cursor_backups = db.get_backups_by_tool_scope(
            &ToolType::Cursor,
            &TakeoverScope::User,
            None,
        ).unwrap();
        assert_eq!(cursor_backups.len(), 3);
    }

    #[test]
    fn test_cleanup_all_old_backups() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 为两个工具各创建 4 个备份
        for tool in [ToolType::ClaudeCode, ToolType::Cursor] {
            for i in 0..4 {
                let backup_path = temp_dir.path().join(format!("{}_{}.json", tool.as_str(), i));
                let original_path = temp_dir.path().join(format!("{}_config.json", tool.as_str()));

                fs::write(&backup_path, format!("{} backup {}", tool.as_str(), i)).unwrap();
                fs::write(&original_path, "original").unwrap();

                let backup = TakeoverBackup::new(
                    tool.clone(),
                    original_path,
                    backup_path,
                );
                db.create_takeover_backup(&backup).unwrap();
            }
        }

        // 批量清理，每组保留 2 个
        let result = super::cleanup_all_old_backups(&db, 2).unwrap();

        assert_eq!(result.groups_processed, 2);
        assert_eq!(result.total_deleted, 4); // 每组删除 2 个

        // 验证每组只剩 2 个
        for tool in [ToolType::ClaudeCode, ToolType::Cursor] {
            let backups = db.get_backups_by_tool_scope(&tool, &TakeoverScope::User, None).unwrap();
            assert_eq!(backups.len(), 2);
        }
    }

    #[test]
    fn test_list_backups_with_version() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建 3 个备份
        for i in 0..3 {
            let backup_path = temp_dir.path().join(format!("backup_{}.json", i));
            let original_path = temp_dir.path().join("config.json");

            fs::write(&backup_path, format!("backup content {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::ClaudeCode,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let backups_with_version = super::list_backups_with_version(&db).unwrap();

        assert_eq!(backups_with_version.len(), 3);

        // 验证版本序号
        for backup in &backups_with_version {
            assert!(backup.version_index >= 1);
            assert!(backup.version_index <= 3);
            assert_eq!(backup.total_versions, 3);
        }
    }

    #[test]
    fn test_get_backup_stats() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建一些备份
        for i in 0..3 {
            let backup_path = temp_dir.path().join(format!("backup_{}.json", i));
            let original_path = temp_dir.path().join("config.json");

            fs::write(&backup_path, format!("backup content {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::ClaudeCode,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();
        }

        let stats = db.get_backup_stats().unwrap();

        assert_eq!(stats.total_count, 3);
        assert!(stats.total_size > 0);
        assert_eq!(stats.groups.len(), 1);
        assert_eq!(stats.groups[0].tool, ToolType::ClaudeCode);
        assert_eq!(stats.groups[0].count, 3);
    }

    // Task 2.4: 自动清理集成测试
    // 注意：完整的集成测试需要模拟 ImportExecutor，这里验证清理逻辑本身的正确性
    #[test]
    fn test_cleanup_preserves_newest_backups() {
        use crate::models::mcp::{TakeoverBackup, TakeoverScope, ToolType};

        let temp_dir = TempDir::new().unwrap();
        let db = Database::new_in_memory().unwrap();

        // 创建 10 个备份，模拟多次接管操作
        for i in 0..10 {
            let backup_path = temp_dir.path().join(format!("backup_{}.json", i));
            let original_path = temp_dir.path().join("config.json");

            fs::write(&backup_path, format!("backup content {}", i)).unwrap();
            fs::write(&original_path, "original").unwrap();

            let backup = TakeoverBackup::new(
                ToolType::ClaudeCode,
                original_path,
                backup_path,
            );
            db.create_takeover_backup(&backup).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 模拟自动清理（保留最新 5 个）
        let result = super::cleanup_old_backups(
            &db,
            &ToolType::ClaudeCode,
            &TakeoverScope::User,
            None,
            5,
        ).unwrap();

        // 验证清理结果
        assert_eq!(result.deleted_count, 5);

        // 验证只保留了 5 个最新的
        let remaining = db.get_backups_by_tool_scope(
            &ToolType::ClaudeCode,
            &TakeoverScope::User,
            None,
        ).unwrap();
        assert_eq!(remaining.len(), 5);

        // 验证保留的是最新的（按时间倒序，第一个应该是最新的）
        for i in 0..remaining.len() - 1 {
            assert!(remaining[i].taken_over_at >= remaining[i + 1].taken_over_at);
        }
    }
