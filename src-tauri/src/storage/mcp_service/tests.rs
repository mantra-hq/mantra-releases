use super::*;

fn create_test_request() -> CreateMcpServiceRequest {
    CreateMcpServiceRequest {
        name: "git-mcp".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]),
        env: Some(serde_json::json!({"DEBUG": "true"})),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    }
}

#[test]
fn test_create_mcp_service() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let service = db.create_mcp_service(&request).unwrap();

    assert!(!service.id.is_empty());
    assert_eq!(service.name, "git-mcp");
    assert_eq!(service.command, "npx");
    assert_eq!(service.args, Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]));
    assert_eq!(service.source, McpServiceSource::Manual);
    assert!(service.enabled);
}

#[test]
fn test_get_mcp_service() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();
    let fetched = db.get_mcp_service(&created.id).unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, created.name);
}

#[test]
fn test_get_mcp_service_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.get_mcp_service("non-existent-id");
    assert!(result.is_err());
}

#[test]
fn test_get_mcp_service_by_name() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    db.create_mcp_service(&request).unwrap();
    let fetched = db.get_mcp_service_by_name("git-mcp").unwrap();

    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "git-mcp");
}

#[test]
fn test_get_mcp_service_by_name_not_found() {
    let db = Database::new_in_memory().unwrap();

    let fetched = db.get_mcp_service_by_name("non-existent").unwrap();
    assert!(fetched.is_none());
}

#[test]
fn test_list_mcp_services() {
    let db = Database::new_in_memory().unwrap();

    // Create multiple services
    let request1 = CreateMcpServiceRequest {
        name: "alpha-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let request2 = CreateMcpServiceRequest {
        name: "beta-service".to_string(),
        transport_type: Default::default(),
        command: "uvx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some("/home/user/.mcp.json".to_string()),
    };

    db.create_mcp_service(&request1).unwrap();
    db.create_mcp_service(&request2).unwrap();

    let services = db.list_mcp_services().unwrap();

    assert_eq!(services.len(), 2);
    // Should be sorted by name
    assert_eq!(services[0].name, "alpha-service");
    assert_eq!(services[1].name, "beta-service");
}

#[test]
fn test_list_mcp_services_by_source() {
    let db = Database::new_in_memory().unwrap();

    let request1 = CreateMcpServiceRequest {
        name: "manual-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let request2 = CreateMcpServiceRequest {
        name: "imported-service".to_string(),
        transport_type: Default::default(),
        command: "uvx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some("/home/user/.mcp.json".to_string()),
    };

    db.create_mcp_service(&request1).unwrap();
    db.create_mcp_service(&request2).unwrap();

    let manual_services = db.list_mcp_services_by_source(&McpServiceSource::Manual).unwrap();
    let imported_services = db.list_mcp_services_by_source(&McpServiceSource::Imported).unwrap();

    assert_eq!(manual_services.len(), 1);
    assert_eq!(manual_services[0].name, "manual-service");

    assert_eq!(imported_services.len(), 1);
    assert_eq!(imported_services[0].name, "imported-service");
}

#[test]
fn test_update_mcp_service() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();

    let update = UpdateMcpServiceRequest {
        name: Some("updated-name".to_string()),
        command: Some("uvx".to_string()),
        ..Default::default()
    };

    let updated = db.update_mcp_service(&created.id, &update).unwrap();

    assert_eq!(updated.name, "updated-name");
    assert_eq!(updated.command, "uvx");
    // Args should be preserved
    assert_eq!(updated.args, created.args);
}

#[test]
fn test_update_mcp_service_partial() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();

    // Only update name
    let update = UpdateMcpServiceRequest {
        name: Some("new-name".to_string()),
        ..Default::default()
    };

    let updated = db.update_mcp_service(&created.id, &update).unwrap();

    assert_eq!(updated.name, "new-name");
    assert_eq!(updated.command, created.command);
    assert_eq!(updated.args, created.args);
    assert_eq!(updated.env, created.env);
}

#[test]
fn test_delete_mcp_service() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();

    db.delete_mcp_service(&created.id).unwrap();

    let result = db.get_mcp_service(&created.id);
    assert!(result.is_err());
}

#[test]
fn test_delete_mcp_service_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.delete_mcp_service("non-existent-id");
    assert!(result.is_err());
}

#[test]
fn test_toggle_mcp_service() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();
    assert!(created.enabled);

    // Disable
    let disabled = db.toggle_mcp_service(&created.id, false).unwrap();
    assert!(!disabled.enabled);

    // Re-enable
    let enabled = db.toggle_mcp_service(&created.id, true).unwrap();
    assert!(enabled.enabled);
}

#[test]
fn test_toggle_mcp_service_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.toggle_mcp_service("non-existent-id", true);
    assert!(result.is_err());
}

#[test]
fn test_mcp_service_with_env_variables() {
    let db = Database::new_in_memory().unwrap();

    let request = CreateMcpServiceRequest {
        name: "openai-mcp".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: Some(vec!["-y".to_string(), "@anthropic/openai-mcp".to_string()]),
        env: Some(serde_json::json!({
            "OPENAI_API_KEY": "$OPENAI_API_KEY",
            "DEBUG": "true"
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Imported,
        source_file: Some("/home/user/.claude/mcp.json".to_string()),
    };

    let service = db.create_mcp_service(&request).unwrap();

    assert_eq!(service.env, Some(serde_json::json!({
        "OPENAI_API_KEY": "$OPENAI_API_KEY",
        "DEBUG": "true"
    })));
}

#[test]
fn test_mcp_service_timestamps() {
    let db = Database::new_in_memory().unwrap();
    let request = create_test_request();

    let created = db.create_mcp_service(&request).unwrap();

    // created_at and updated_at should be set
    assert!(!created.created_at.is_empty());
    assert!(!created.updated_at.is_empty());
    assert_eq!(created.created_at, created.updated_at);

    // After update, updated_at should change
    std::thread::sleep(std::time::Duration::from_millis(10));
    let update = UpdateMcpServiceRequest {
        name: Some("new-name".to_string()),
        ..Default::default()
    };
    let updated = db.update_mcp_service(&created.id, &update).unwrap();

    assert_eq!(updated.created_at, created.created_at);
    assert_ne!(updated.updated_at, created.updated_at);
}

// ===== Task 4: 项目关联存储测试 =====

fn create_test_project(db: &Database, id: &str, name: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    // 使用唯一的 cwd 路径避免 UNIQUE 约束冲突
    let cwd = format!("/path/to/{}", id);
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
            [id, name, &cwd, &now],
        )
        .unwrap();
}

#[test]
fn test_link_service_to_project() {
    let db = Database::new_in_memory().unwrap();

    // Create project and service
    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    // Link them
    let link = db.link_service_to_project("proj1", &service.id, None).unwrap();

    assert_eq!(link.project_id, "proj1");
    assert_eq!(link.service_id, service.id);
    assert!(link.config_override.is_none());
}

#[test]
fn test_link_service_to_project_with_override() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    let override_config = serde_json::json!({"args": ["--custom-arg"]});
    let link = db
        .link_service_to_project("proj1", &service.id, Some(&override_config))
        .unwrap();

    assert_eq!(link.config_override, Some(override_config));
}

#[test]
fn test_unlink_service_from_project() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    db.link_service_to_project("proj1", &service.id, None).unwrap();
    db.unlink_service_from_project("proj1", &service.id).unwrap();

    // Verify link is removed
    let link = db.get_project_service_link("proj1", &service.id).unwrap();
    assert!(link.is_none());
}

#[test]
fn test_unlink_service_from_project_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.unlink_service_from_project("proj1", "svc1");
    assert!(result.is_err());
}

#[test]
fn test_get_project_services() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");

    let service1 = db
        .create_mcp_service(&CreateMcpServiceRequest {
            name: "alpha-service".to_string(),
            transport_type: Default::default(),
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
            name: "beta-service".to_string(),
            transport_type: Default::default(),
            command: "uvx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    // Link both services
    db.link_service_to_project("proj1", &service1.id, None).unwrap();
    let override_config = serde_json::json!({"args": ["--verbose"]});
    db.link_service_to_project("proj1", &service2.id, Some(&override_config))
        .unwrap();

    // Get project services
    let services = db.get_project_services("proj1").unwrap();

    assert_eq!(services.len(), 2);
    // Should be sorted by name
    assert_eq!(services[0].service.name, "alpha-service");
    assert!(services[0].config_override.is_none());
    assert_eq!(services[1].service.name, "beta-service");
    assert_eq!(services[1].config_override, Some(override_config));
}

#[test]
fn test_get_project_services_empty() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");

    let services = db.get_project_services("proj1").unwrap();
    assert!(services.is_empty());
}

#[test]
fn test_get_service_projects() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    create_test_project(&db, "proj2", "Project 2");

    let service = db.create_mcp_service(&create_test_request()).unwrap();

    // Link service to both projects
    db.link_service_to_project("proj1", &service.id, None).unwrap();
    db.link_service_to_project("proj2", &service.id, None).unwrap();

    // Get service projects
    let project_ids = db.get_service_projects(&service.id).unwrap();

    assert_eq!(project_ids.len(), 2);
    assert!(project_ids.contains(&"proj1".to_string()));
    assert!(project_ids.contains(&"proj2".to_string()));
}

#[test]
fn test_get_service_projects_empty() {
    let db = Database::new_in_memory().unwrap();

    let service = db.create_mcp_service(&create_test_request()).unwrap();

    let project_ids = db.get_service_projects(&service.id).unwrap();
    assert!(project_ids.is_empty());
}

#[test]
fn test_update_project_service_override() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    // Link without override
    db.link_service_to_project("proj1", &service.id, None).unwrap();

    // Update with override
    let override_config = serde_json::json!({"args": ["--new-arg"]});
    db.update_project_service_override("proj1", &service.id, Some(&override_config))
        .unwrap();

    // Verify
    let link = db.get_project_service_link("proj1", &service.id).unwrap().unwrap();
    assert_eq!(link.config_override, Some(override_config));
}

#[test]
fn test_update_project_service_override_to_none() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    let override_config = serde_json::json!({"args": ["--arg"]});
    db.link_service_to_project("proj1", &service.id, Some(&override_config))
        .unwrap();

    // Clear override
    db.update_project_service_override("proj1", &service.id, None)
        .unwrap();

    // Verify
    let link = db.get_project_service_link("proj1", &service.id).unwrap().unwrap();
    assert!(link.config_override.is_none());
}

#[test]
fn test_update_project_service_override_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.update_project_service_override("proj1", "svc1", None);
    assert!(result.is_err());
}

#[test]
fn test_get_project_service_link() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    db.link_service_to_project("proj1", &service.id, None).unwrap();

    let link = db.get_project_service_link("proj1", &service.id).unwrap();
    assert!(link.is_some());
    assert_eq!(link.unwrap().project_id, "proj1");
}

#[test]
fn test_get_project_service_link_not_found() {
    let db = Database::new_in_memory().unwrap();

    let link = db.get_project_service_link("proj1", "svc1").unwrap();
    assert!(link.is_none());
}

#[test]
fn test_cascade_delete_removes_links() {
    let db = Database::new_in_memory().unwrap();

    create_test_project(&db, "proj1", "Project 1");
    let service = db.create_mcp_service(&create_test_request()).unwrap();

    db.link_service_to_project("proj1", &service.id, None).unwrap();

    // Delete service
    db.delete_mcp_service(&service.id).unwrap();

    // Link should be removed due to CASCADE
    let link = db.get_project_service_link("proj1", &service.id).unwrap();
    assert!(link.is_none());
}

// ===== Story 11.4: 受影响服务查询测试 =====

#[test]
fn test_find_services_using_env_var_simple_format() {
    let db = Database::new_in_memory().unwrap();

    // 创建使用 $OPENAI_API_KEY 的服务
    let request = CreateMcpServiceRequest {
        name: "openai-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "OPENAI_API_KEY": "$OPENAI_API_KEY",
            "DEBUG": "true"
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request).unwrap();

    // 创建不使用该变量的服务
    let request2 = CreateMcpServiceRequest {
        name: "other-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "DEBUG": "true"
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request2).unwrap();

    let affected = db.find_services_using_env_var("OPENAI_API_KEY").unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].name, "openai-service");
}

#[test]
fn test_find_services_using_env_var_braced_format() {
    let db = Database::new_in_memory().unwrap();

    // 创建使用 ${ANTHROPIC_API_KEY} 的服务
    let request = CreateMcpServiceRequest {
        name: "anthropic-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "API_KEY": "${ANTHROPIC_API_KEY}",
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request).unwrap();

    let affected = db.find_services_using_env_var("ANTHROPIC_API_KEY").unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].name, "anthropic-service");
}

#[test]
fn test_find_services_using_env_var_multiple_services() {
    let db = Database::new_in_memory().unwrap();

    // 创建多个使用同一变量的服务
    for name in ["service-a", "service-b", "service-c"] {
        let request = CreateMcpServiceRequest {
            name: name.to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "API_KEY": "$SHARED_KEY",
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
        };
        db.create_mcp_service(&request).unwrap();
    }

    let affected = db.find_services_using_env_var("SHARED_KEY").unwrap();
    assert_eq!(affected.len(), 3);
    // 应该按名称排序
    assert_eq!(affected[0].name, "service-a");
    assert_eq!(affected[1].name, "service-b");
    assert_eq!(affected[2].name, "service-c");
}

#[test]
fn test_find_services_using_env_var_no_match() {
    let db = Database::new_in_memory().unwrap();

    let request = CreateMcpServiceRequest {
        name: "some-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "OTHER_VAR": "$OTHER_VAR",
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request).unwrap();

    let affected = db.find_services_using_env_var("NONEXISTENT_VAR").unwrap();
    assert!(affected.is_empty());
}

#[test]
fn test_find_services_using_env_var_no_env() {
    let db = Database::new_in_memory().unwrap();

    // 创建没有 env 字段的服务
    let request = CreateMcpServiceRequest {
        name: "no-env-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request).unwrap();

    let affected = db.find_services_using_env_var("ANY_VAR").unwrap();
    assert!(affected.is_empty());
}

#[test]
fn test_find_services_using_env_var_no_substring_false_positive() {
    let db = Database::new_in_memory().unwrap();

    // 创建使用 $OPENAI_API_KEY 的服务（包含 API_KEY 子串）
    let request1 = CreateMcpServiceRequest {
        name: "openai-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "KEY": "$OPENAI_API_KEY",
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request1).unwrap();

    // 创建使用 $API_KEY 的服务
    let request2 = CreateMcpServiceRequest {
        name: "api-key-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: Some(serde_json::json!({
            "KEY": "$API_KEY",
        })),
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    db.create_mcp_service(&request2).unwrap();

    // 搜索 API_KEY 应该只返回 api-key-service，而不是 openai-service
    let affected = db.find_services_using_env_var("API_KEY").unwrap();
    assert_eq!(affected.len(), 1, "Should only match exact variable name, not substring");
    assert_eq!(affected[0].name, "api-key-service");

    // 搜索 OPENAI_API_KEY 应该只返回 openai-service
    let affected = db.find_services_using_env_var("OPENAI_API_KEY").unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0].name, "openai-service");
}

// ===== Story 11.15: 接管备份存储测试 =====

fn create_test_backup(tool_type: ToolType) -> TakeoverBackup {
    TakeoverBackup::new(
        tool_type,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.claude.json.mantra-backup.20260201"),
    )
}

#[test]
fn test_create_takeover_backup() {
    let db = Database::new_in_memory().unwrap();
    let backup = create_test_backup(ToolType::ClaudeCode);

    db.create_takeover_backup(&backup).unwrap();

    // Verify it was created
    let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap();
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.id, backup.id);
    assert_eq!(fetched.tool_type, ToolType::ClaudeCode);
    assert_eq!(fetched.status, TakeoverStatus::Active);
}

#[test]
fn test_get_takeover_backups_all() {
    let db = Database::new_in_memory().unwrap();

    // Create multiple backups
    let backup1 = create_test_backup(ToolType::ClaudeCode);
    let backup2 = TakeoverBackup::new(
        ToolType::Cursor,
        PathBuf::from("/home/user/.cursor/mcp.json"),
        PathBuf::from("/home/user/.cursor/mcp.json.backup"),
    );

    db.create_takeover_backup(&backup1).unwrap();
    db.create_takeover_backup(&backup2).unwrap();

    let backups = db.get_takeover_backups(None).unwrap();
    assert_eq!(backups.len(), 2);
}

#[test]
fn test_get_takeover_backups_by_status() {
    let db = Database::new_in_memory().unwrap();

    // Create active backup
    let backup1 = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&backup1).unwrap();

    // Create and restore another backup
    let backup2 = TakeoverBackup::new(
        ToolType::Cursor,
        PathBuf::from("/home/user/.cursor/mcp.json"),
        PathBuf::from("/home/user/.cursor/mcp.json.backup"),
    );
    db.create_takeover_backup(&backup2).unwrap();
    db.update_backup_status_restored(&backup2.id).unwrap();

    // Filter by active
    let active = db.get_takeover_backups(Some(TakeoverStatus::Active)).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].tool_type, ToolType::ClaudeCode);

    // Filter by restored
    let restored = db.get_takeover_backups(Some(TakeoverStatus::Restored)).unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].tool_type, ToolType::Cursor);
}

#[test]
fn test_get_active_takeover_by_tool() {
    let db = Database::new_in_memory().unwrap();

    // No backup exists
    let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
    assert!(result.is_none());

    // Create backup
    let backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&backup).unwrap();

    let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, backup.id);

    // Different tool type should return None
    let result = db.get_active_takeover_by_tool(&ToolType::Cursor).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_active_takeover_excludes_restored() {
    let db = Database::new_in_memory().unwrap();

    let backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&backup).unwrap();
    db.update_backup_status_restored(&backup.id).unwrap();

    // Should not return restored backups
    let result = db.get_active_takeover_by_tool(&ToolType::ClaudeCode).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_update_backup_status_restored() {
    let db = Database::new_in_memory().unwrap();

    let backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&backup).unwrap();

    // Update status
    db.update_backup_status_restored(&backup.id).unwrap();

    // Verify
    let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap().unwrap();
    assert_eq!(fetched.status, TakeoverStatus::Restored);
    assert!(fetched.restored_at.is_some());
}

#[test]
fn test_update_backup_status_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.update_backup_status_restored("nonexistent-id");
    assert!(result.is_err());
}

#[test]
fn test_delete_takeover_backup() {
    let db = Database::new_in_memory().unwrap();

    let backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&backup).unwrap();

    db.delete_takeover_backup(&backup.id).unwrap();

    let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap();
    assert!(fetched.is_none());
}

#[test]
fn test_takeover_backup_preserves_paths() {
    let db = Database::new_in_memory().unwrap();

    let original = PathBuf::from("/home/user/.claude.json");
    let backup_path = PathBuf::from("/home/user/.claude.json.mantra-backup.20260201");

    let backup = TakeoverBackup::new(
        ToolType::ClaudeCode,
        original.clone(),
        backup_path.clone(),
    );
    db.create_takeover_backup(&backup).unwrap();

    let fetched = db.get_takeover_backup_by_id(&backup.id).unwrap().unwrap();
    assert_eq!(fetched.original_path, original);
    assert_eq!(fetched.backup_path, backup_path);
}

// ===== Story 11.16: 接管作用域存储测试 =====

fn create_test_project_backup(tool_type: ToolType, project_path: &str) -> TakeoverBackup {
    TakeoverBackup::new_with_scope(
        tool_type,
        PathBuf::from(format!("{}/.mcp.json", project_path)),
        PathBuf::from(format!("{}/.mcp.json.backup", project_path)),
        TakeoverScope::Project,
        Some(PathBuf::from(project_path)),
    )
}

#[test]
fn test_create_takeover_backup_with_scope() {
    let db = Database::new_in_memory().unwrap();

    // 用户级备份
    let user_backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&user_backup).unwrap();

    let fetched = db.get_takeover_backup_by_id(&user_backup.id).unwrap().unwrap();
    assert_eq!(fetched.scope, TakeoverScope::User);
    assert!(fetched.project_path.is_none());

    // 项目级备份
    let project_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
    db.create_takeover_backup(&project_backup).unwrap();

    let fetched = db.get_takeover_backup_by_id(&project_backup.id).unwrap().unwrap();
    assert_eq!(fetched.scope, TakeoverScope::Project);
    assert_eq!(fetched.project_path, Some(PathBuf::from("/home/user/project")));
}

#[test]
fn test_get_active_takeover_by_tool_and_scope_user() {
    let db = Database::new_in_memory().unwrap();

    // 创建用户级和项目级备份
    let user_backup = create_test_backup(ToolType::ClaudeCode);
    let project_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");

    db.create_takeover_backup(&user_backup).unwrap();
    db.create_takeover_backup(&project_backup).unwrap();

    // 按用户级作用域查询
    let result = db
        .get_active_takeover_by_tool_and_scope(&ToolType::ClaudeCode, &TakeoverScope::User, None)
        .unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().scope, TakeoverScope::User);
}

#[test]
fn test_get_active_takeover_by_tool_and_scope_project() {
    let db = Database::new_in_memory().unwrap();

    // 创建多个项目的备份
    let project1_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project1");
    let project2_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project2");

    db.create_takeover_backup(&project1_backup).unwrap();
    db.create_takeover_backup(&project2_backup).unwrap();

    // 按项目级作用域查询
    let result = db
        .get_active_takeover_by_tool_and_scope(
            &ToolType::ClaudeCode,
            &TakeoverScope::Project,
            Some("/home/user/project1"),
        )
        .unwrap();

    assert!(result.is_some());
    let found = result.unwrap();
    assert_eq!(found.scope, TakeoverScope::Project);
    assert_eq!(found.project_path, Some(PathBuf::from("/home/user/project1")));
}

#[test]
fn test_get_active_takeover_by_tool_and_scope_not_found() {
    let db = Database::new_in_memory().unwrap();

    // 创建用户级备份
    let user_backup = create_test_backup(ToolType::ClaudeCode);
    db.create_takeover_backup(&user_backup).unwrap();

    // 查询不存在的项目级备份
    let result = db
        .get_active_takeover_by_tool_and_scope(
            &ToolType::ClaudeCode,
            &TakeoverScope::Project,
            Some("/nonexistent"),
        )
        .unwrap();

    assert!(result.is_none());
}

#[test]
fn test_get_active_takeover_by_original_path() {
    let db = Database::new_in_memory().unwrap();

    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/project/.mcp.json"),
        PathBuf::from("/project/.mcp.json.mantra-backup.20260203"),
        TakeoverScope::Project,
        Some(PathBuf::from("/project")),
    );
    db.create_takeover_backup(&backup).unwrap();

    // 查询存在的路径
    let result = db
        .get_active_takeover_by_original_path("/project/.mcp.json")
        .unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, backup.id);

    // 查询不存在的路径
    let result = db
        .get_active_takeover_by_original_path("/other/.mcp.json")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_active_takeover_by_original_path_excludes_restored() {
    let db = Database::new_in_memory().unwrap();

    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/project/.mcp.json"),
        PathBuf::from("/project/.mcp.json.mantra-backup.20260203"),
        TakeoverScope::Project,
        Some(PathBuf::from("/project")),
    );
    db.create_takeover_backup(&backup).unwrap();

    // 标记为已恢复
    db.update_backup_status_restored(&backup.id).unwrap();

    // 查询应返回 None（已恢复的不算活跃）
    let result = db
        .get_active_takeover_by_original_path("/project/.mcp.json")
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_active_takeovers_by_project() {
    let db = Database::new_in_memory().unwrap();

    // 创建同一个项目的多个工具备份
    let claude_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
    let cursor_backup = TakeoverBackup::new_with_scope(
        ToolType::Cursor,
        PathBuf::from("/home/user/project/.mcp.json"),
        PathBuf::from("/home/user/project/.mcp.json.cursor-backup"),
        TakeoverScope::Project,
        Some(PathBuf::from("/home/user/project")),
    );

    // 不同项目的备份
    let other_backup = create_test_project_backup(ToolType::ClaudeCode, "/home/user/other");

    db.create_takeover_backup(&claude_backup).unwrap();
    db.create_takeover_backup(&cursor_backup).unwrap();
    db.create_takeover_backup(&other_backup).unwrap();

    // 查询特定项目的备份
    let backups = db.get_active_takeovers_by_project("/home/user/project").unwrap();

    assert_eq!(backups.len(), 2);
    assert!(backups.iter().all(|b| b.project_path == Some(PathBuf::from("/home/user/project"))));
}

#[test]
fn test_get_active_takeovers_by_project_excludes_restored() {
    let db = Database::new_in_memory().unwrap();

    // 创建两个备份，其中一个已恢复
    let backup1 = create_test_project_backup(ToolType::ClaudeCode, "/home/user/project");
    let backup2 = create_test_project_backup(ToolType::Cursor, "/home/user/project");

    db.create_takeover_backup(&backup1).unwrap();
    db.create_takeover_backup(&backup2).unwrap();
    db.update_backup_status_restored(&backup2.id).unwrap();

    // 只应该返回活跃的备份
    let backups = db.get_active_takeovers_by_project("/home/user/project").unwrap();

    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].id, backup1.id);
}

#[test]
fn test_takeover_backup_scope_in_get_backups() {
    let db = Database::new_in_memory().unwrap();

    let user_backup = create_test_backup(ToolType::ClaudeCode);
    let project_backup = create_test_project_backup(ToolType::Cursor, "/home/user/project");

    db.create_takeover_backup(&user_backup).unwrap();
    db.create_takeover_backup(&project_backup).unwrap();

    // get_takeover_backups 应该返回正确的 scope
    let backups = db.get_takeover_backups(None).unwrap();

    assert_eq!(backups.len(), 2);
    let user_found = backups.iter().find(|b| b.id == user_backup.id).unwrap();
    let project_found = backups.iter().find(|b| b.id == project_backup.id).unwrap();

    assert_eq!(user_found.scope, TakeoverScope::User);
    assert_eq!(project_found.scope, TakeoverScope::Project);
    assert_eq!(project_found.project_path, Some(PathBuf::from("/home/user/project")));
}

// ===== Story 11.9 Phase 2: Default Tool Policy Tests =====

#[test]
fn test_get_service_default_policy_none() {
    let db = Database::new_in_memory().unwrap();

    // 创建服务（无默认策略）
    let request = CreateMcpServiceRequest {
        name: "test-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let service = db.create_mcp_service(&request).unwrap();

    // 获取默认策略应返回 AllowAll
    let policy = db.get_service_default_policy(&service.id).unwrap();
    assert!(policy.is_allow_all());
}

#[test]
fn test_update_service_default_policy() {
    let db = Database::new_in_memory().unwrap();

    // 创建服务
    let request = CreateMcpServiceRequest {
        name: "test-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let service = db.create_mcp_service(&request).unwrap();

    // 更新默认策略为 Custom（仅允许特定工具）
    let policy = ToolPolicy::custom(vec!["read_file".to_string()]);
    let updated = db.update_service_default_policy(&service.id, Some(&policy)).unwrap();
    assert!(updated.default_tool_policy.is_some());

    let retrieved_policy = db.get_service_default_policy(&service.id).unwrap();
    assert!(retrieved_policy.is_custom());
}

#[test]
fn test_update_service_default_policy_custom() {
    let db = Database::new_in_memory().unwrap();

    // 创建服务
    let request = CreateMcpServiceRequest {
        name: "test-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let service = db.create_mcp_service(&request).unwrap();

    // 更新为 Custom 策略
    let policy = ToolPolicy::custom(vec!["read_file".to_string(), "list_commits".to_string()]);
    db.update_service_default_policy(&service.id, Some(&policy)).unwrap();

    let retrieved = db.get_service_default_policy(&service.id).unwrap();
    assert!(retrieved.is_custom());
    let allowed = retrieved.allowed_tools.unwrap();
    assert!(allowed.contains(&"read_file".to_string()));
    assert!(allowed.contains(&"list_commits".to_string()));
}

#[test]
fn test_update_service_default_policy_clear() {
    let db = Database::new_in_memory().unwrap();

    // 创建服务
    let request = CreateMcpServiceRequest {
        name: "test-service".to_string(),
        transport_type: Default::default(),
        command: "npx".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
    };
    let service = db.create_mcp_service(&request).unwrap();

    // 先设置 Custom 策略
    let policy = ToolPolicy::custom(vec!["some_tool".to_string()]);
    db.update_service_default_policy(&service.id, Some(&policy)).unwrap();

    // 然后清除策略
    let updated = db.update_service_default_policy(&service.id, None).unwrap();
    assert!(updated.default_tool_policy.is_none());

    // 获取策略应返回默认值（AllowAll）
    let retrieved = db.get_service_default_policy(&service.id).unwrap();
    assert!(retrieved.is_allow_all());
}

#[test]
fn test_update_service_default_policy_not_found() {
    let db = Database::new_in_memory().unwrap();

    let policy = ToolPolicy::default();
    let result = db.update_service_default_policy("nonexistent-id", Some(&policy));
    assert!(result.is_err());
}

// ===== Story 11.21: Local Scope 存储测试 =====

#[test]
fn test_create_takeover_backup_local_scope() {
    let db = Database::new_in_memory().unwrap();

    let backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.mantra/backups/project-a-local.backup"),
        TakeoverScope::Local,
        Some(PathBuf::from("/home/user/project-a")),
    );

    let result = db.create_takeover_backup(&backup);
    assert!(result.is_ok(), "Should successfully create local scope backup: {:?}", result.err());

    // 验证保存的数据
    let retrieved = db.get_takeover_backup_by_id(&backup.id).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.scope, TakeoverScope::Local);
    assert_eq!(retrieved.project_path, Some(PathBuf::from("/home/user/project-a")));
    assert!(retrieved.is_local_level());
}

#[test]
fn test_get_active_takeover_by_tool_and_scope_local() {
    let db = Database::new_in_memory().unwrap();

    // 创建 user scope 备份
    let user_backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.mantra/backups/user.backup"),
        TakeoverScope::User,
        None,
    );
    db.create_takeover_backup(&user_backup).unwrap();

    // 创建 local scope 备份
    let local_backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.mantra/backups/local-a.backup"),
        TakeoverScope::Local,
        Some(PathBuf::from("/home/user/project-a")),
    );
    db.create_takeover_backup(&local_backup).unwrap();

    // 查询 local scope
    let result = db.get_active_takeover_by_tool_and_scope(
        &ToolType::ClaudeCode,
        &TakeoverScope::Local,
        Some("/home/user/project-a"),
    ).unwrap();

    assert!(result.is_some());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.id, local_backup.id);
    assert_eq!(retrieved.scope, TakeoverScope::Local);

    // 查询不存在的 local scope 项目
    let not_found = db.get_active_takeover_by_tool_and_scope(
        &ToolType::ClaudeCode,
        &TakeoverScope::Local,
        Some("/home/user/project-nonexistent"),
    ).unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_get_active_local_scope_takeovers() {
    let db = Database::new_in_memory().unwrap();

    // 创建多个 local scope 备份
    for (i, project) in ["project-a", "project-b", "project-c"].iter().enumerate() {
        let backup = TakeoverBackup::new_with_scope(
            ToolType::ClaudeCode,
            PathBuf::from("/home/user/.claude.json"),
            PathBuf::from(format!("/home/user/.mantra/backups/{}.backup", project)),
            TakeoverScope::Local,
            Some(PathBuf::from(format!("/home/user/{}", project))),
        );
        db.create_takeover_backup(&backup).unwrap();

        // project-c 设为 restored
        if i == 2 {
            db.update_backup_status_restored(&backup.id).unwrap();
        }
    }

    // 创建一个 user scope 备份（应被排除）
    let user_backup = TakeoverBackup::new_with_scope(
        ToolType::ClaudeCode,
        PathBuf::from("/home/user/.claude.json"),
        PathBuf::from("/home/user/.mantra/backups/user.backup"),
        TakeoverScope::User,
        None,
    );
    db.create_takeover_backup(&user_backup).unwrap();

    // 查询所有活跃的 local scope 备份
    let local_backups = db.get_active_local_scope_takeovers().unwrap();

    // 应该只有 2 个（project-c 已恢复，user scope 不算）
    assert_eq!(local_backups.len(), 2);

    // 验证按 project_path 排序
    assert!(local_backups[0].project_path.as_ref().unwrap().to_string_lossy().contains("project-a"));
    assert!(local_backups[1].project_path.as_ref().unwrap().to_string_lossy().contains("project-b"));

    // 验证都是 local scope
    for backup in &local_backups {
        assert!(backup.is_local_level());
        assert_eq!(backup.tool_type, ToolType::ClaudeCode);
    }
}
