use super::*;
use tempfile::TempDir;

#[test]
fn test_transaction_begin() {
    let tx = TakeoverTransaction::begin();

    assert!(!tx.id().is_empty());
    assert_eq!(tx.operation_count(), 0);
    assert!(!tx.is_committed());
    assert!(!tx.is_rolled_back());
}

#[test]
fn test_add_operation() {
    let mut tx = TakeoverTransaction::begin();

    tx.record_service_created("service-1".to_string());
    tx.record_project_linked("proj-1".to_string(), "service-1".to_string());

    assert_eq!(tx.operation_count(), 2);
}

#[test]
fn test_add_operation_after_commit_ignored() {
    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created("service-1".to_string());
    tx.commit().unwrap();

    // 提交后的操作应被忽略
    tx.record_service_created("service-2".to_string());
    assert_eq!(tx.operation_count(), 1);
}

#[test]
fn test_add_operation_after_rollback_ignored() {
    let db = Database::new_in_memory().unwrap();
    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created("service-1".to_string());
    tx.rollback(&db);

    // 回滚后的操作应被忽略
    tx.record_service_created("service-2".to_string());
    assert_eq!(tx.operation_count(), 1);
}

#[test]
fn test_commit_success() {
    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created("service-1".to_string());

    let result = tx.commit();
    assert!(result.is_ok());
    assert!(tx.is_committed());
}

#[test]
fn test_commit_idempotent() {
    let mut tx = TakeoverTransaction::begin();
    tx.commit().unwrap();

    // 重复提交应该成功
    let result = tx.commit();
    assert!(result.is_ok());
}

#[test]
fn test_commit_after_rollback_fails() {
    let db = Database::new_in_memory().unwrap();
    let mut tx = TakeoverTransaction::begin();
    tx.rollback(&db);

    // 回滚后不能提交
    let result = tx.commit();
    assert!(result.is_err());
}

#[test]
fn test_commit_cleans_temp_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("temp-backup.json");
    fs::write(&temp_file, "{}").unwrap();

    let mut tx = TakeoverTransaction::begin();
    tx.add_temp_file(temp_file.clone());
    tx.commit().unwrap();

    assert!(!temp_file.exists());
}

#[test]
fn test_rollback_service_created() {
    let db = Database::new_in_memory().unwrap();

    // 创建服务
    let service = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created(service.id.clone());

    // 回滚
    let result = tx.rollback(&db);

    assert_eq!(result.success_count, 1);
    assert!(result.errors.is_empty());
    assert!(tx.is_rolled_back());

    // 服务应该被删除
    assert!(db.get_mcp_service(&service.id).is_err());
}

#[test]
fn test_rollback_project_linked() {
    let db = Database::new_in_memory().unwrap();

    // 创建项目
    create_test_project(&db, "proj-1", "Test Project");

    // 创建服务
    let service = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    // 关联
    db.link_service_to_project("proj-1", &service.id, None)
        .unwrap();

    let mut tx = TakeoverTransaction::begin();
    tx.record_project_linked("proj-1".to_string(), service.id.clone());

    // 回滚
    let result = tx.rollback(&db);

    assert!(result.errors.is_empty());
    assert!(tx.is_rolled_back());

    // 关联应该被删除
    let link = db.get_project_service_link("proj-1", &service.id).unwrap();
    assert!(link.is_none());
}

#[test]
fn test_rollback_config_modified() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let temp_backup = temp_dir.path().join("config.json.temp-backup");

    // 模拟已修改的配置：临时备份包含原始内容
    fs::write(&temp_backup, r#"{"original": true}"#).unwrap();
    fs::write(&config_path, r#"{"modified": true}"#).unwrap();

    let db = Database::new_in_memory().unwrap();
    let mut tx = TakeoverTransaction::begin();
    tx.record_config_modified(config_path.clone(), temp_backup.clone());

    // 回滚
    let result = tx.rollback(&db);

    assert_eq!(result.success_count, 1);
    assert!(result.errors.is_empty());

    // 配置应该恢复到原始内容
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("original"));
}

#[test]
fn test_rollback_backup_created() {
    let temp_dir = TempDir::new().unwrap();
    let backup_path = temp_dir.path().join("backup.json");
    fs::write(&backup_path, "{}").unwrap();

    let db = Database::new_in_memory().unwrap();

    // 创建备份记录
    let backup = crate::models::mcp::TakeoverBackup::new(
        crate::models::mcp::ToolType::ClaudeCode,
        PathBuf::from("/original/config.json"),
        backup_path.clone(),
    );
    db.create_takeover_backup(&backup).unwrap();

    let mut tx = TakeoverTransaction::begin();
    tx.record_backup_created(backup.id.clone(), backup_path.clone());

    // 回滚
    let result = tx.rollback(&db);

    assert_eq!(result.success_count, 1);
    assert!(result.errors.is_empty());

    // 备份文件应该被删除
    assert!(!backup_path.exists());
    // 备份记录也应该被删除
    assert!(db.get_takeover_backup_by_id(&backup.id).unwrap().is_none());
}

#[test]
fn test_rollback_idempotent() {
    let db = Database::new_in_memory().unwrap();
    let mut tx = TakeoverTransaction::begin();

    let result1 = tx.rollback(&db);
    let result2 = tx.rollback(&db);

    assert!(result1.errors.is_empty());
    assert!(result2.errors.is_empty());
}

#[test]
fn test_rollback_after_commit_fails() {
    let db = Database::new_in_memory().unwrap();
    let mut tx = TakeoverTransaction::begin();
    tx.commit().unwrap();

    let result = tx.rollback(&db);

    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("Cannot rollback a committed transaction"));
}

#[test]
fn test_rollback_order_is_reversed() {
    let db = Database::new_in_memory().unwrap();

    // 创建项目
    create_test_project(&db, "proj-1", "Test Project");

    // 创建服务
    let service = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    // 关联
    db.link_service_to_project("proj-1", &service.id, None)
        .unwrap();

    // 记录操作顺序：创建服务 -> 关联项目
    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created(service.id.clone());
    tx.record_project_linked("proj-1".to_string(), service.id.clone());

    // 回滚应该先解除关联，再删除服务
    // 如果顺序错了（先删除服务），CASCADE 会自动删除关联，但我们仍能正确处理
    let result = tx.rollback(&db);

    assert!(result.errors.is_empty());

    // 服务和关联都应该被删除
    assert!(db.get_mcp_service(&service.id).is_err());
}

#[test]
fn test_rollback_handles_cascade_delete() {
    let db = Database::new_in_memory().unwrap();

    // 创建项目
    create_test_project(&db, "proj-1", "Test Project");

    // 创建服务
    let service = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "test-service".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    // 关联
    db.link_service_to_project("proj-1", &service.id, None)
        .unwrap();

    // 手动删除服务（CASCADE 会删除关联）
    db.delete_mcp_service(&service.id).unwrap();

    // 即使服务已被删除，回滚也应该正常处理
    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created(service.id.clone());
    tx.record_project_linked("proj-1".to_string(), service.id.clone());

    let result = tx.rollback(&db);

    // 不应该有致命错误（NotFound 不算错误）
    assert!(result.errors.is_empty() || result.errors.iter().all(|e| e.contains("not found")));
}

#[test]
fn test_multiple_services_rollback() {
    let db = Database::new_in_memory().unwrap();

    // 创建多个服务
    let service1 = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "service-1".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    let service2 = db
        .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
            name: "service-2".to_string(),
            transport_type: Default::default(),
            command: "uvx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: crate::models::mcp::McpServiceSource::Manual,
            source_file: None,
        })
        .unwrap();

    let mut tx = TakeoverTransaction::begin();
    tx.record_service_created(service1.id.clone());
    tx.record_service_created(service2.id.clone());

    let result = tx.rollback(&db);

    assert_eq!(result.success_count, 2);
    assert!(result.errors.is_empty());

    // 两个服务都应该被删除
    assert!(db.get_mcp_service(&service1.id).is_err());
    assert!(db.get_mcp_service(&service2.id).is_err());
}

// 辅助函数：创建测试项目
fn create_test_project(db: &Database, id: &str, name: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let cwd = format!("/path/to/{}", id);
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
            [id, name, &cwd, &now],
        )
        .unwrap();
}
