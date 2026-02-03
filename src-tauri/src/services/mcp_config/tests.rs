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
        let backup_path = manager.backup(&test_file).unwrap();

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
        manager.backup(&test_file).unwrap();

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
            manager.backup(&test_file).unwrap();

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
            manager.backup(&test_file).unwrap();

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
        let backup_path = manager.backup(&test_file).unwrap();

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
