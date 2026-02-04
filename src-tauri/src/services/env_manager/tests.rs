use super::*;

fn create_test_env_manager() -> EnvManager {
    let key_bytes: [u8; 32] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
        0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
        0x1c, 0x1d, 0x1e, 0x1f,
    ];
    EnvManager::new(&key_bytes)
}

#[test]
fn test_encrypt_decrypt() {
    let manager = create_test_env_manager();
    let plaintext = "sk-1234567890abcdef";

    let encrypted = manager.encrypt(plaintext);
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_produces_different_ciphertext() {
    let manager = create_test_env_manager();
    let plaintext = "test-value";

    let encrypted1 = manager.encrypt(plaintext);
    let encrypted2 = manager.encrypt(plaintext);

    // 由于使用随机 nonce，每次加密结果应该不同
    assert_ne!(encrypted1, encrypted2);

    // 但解密结果应该相同
    assert_eq!(manager.decrypt(&encrypted1).unwrap(), plaintext);
    assert_eq!(manager.decrypt(&encrypted2).unwrap(), plaintext);
}

#[test]
fn test_decrypt_invalid_ciphertext() {
    let manager = create_test_env_manager();

    // 太短
    let result = manager.decrypt(&[0u8; 5]);
    assert!(result.is_err());

    // 无效数据
    let result = manager.decrypt(&[0u8; 32]);
    assert!(result.is_err());
}

#[test]
fn test_mask_value_short() {
    assert_eq!(mask_value("abc"), "***");
    assert_eq!(mask_value("12345678"), "********");
}

#[test]
fn test_mask_value_long() {
    assert_eq!(mask_value("sk-1234567890abcdef"), "sk-1****cdef");
    assert_eq!(mask_value("AKIAIOSFODNN7EXAMPLE"), "AKIA****MPLE");
}

#[test]
fn test_set_env_variable() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    let var = db
        .set_env_variable(&manager, "API_KEY", "sk-secret123", Some("Test API Key"))
        .unwrap();

    assert_eq!(var.name, "API_KEY");
    assert!(var.masked_value.contains("****"));
    assert_eq!(var.description, Some("Test API Key".to_string()));
}

#[test]
fn test_set_env_variable_update() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    // Create
    db.set_env_variable(&manager, "API_KEY", "old-value", None)
        .unwrap();

    // Update
    db.set_env_variable(&manager, "API_KEY", "new-value", Some("Updated"))
        .unwrap();

    // Verify
    let value = db.get_env_variable(&manager, "API_KEY").unwrap().unwrap();
    assert_eq!(value, "new-value");
}

#[test]
fn test_get_env_variable() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "SECRET", "my-secret-value", None)
        .unwrap();

    let value = db.get_env_variable(&manager, "SECRET").unwrap().unwrap();
    assert_eq!(value, "my-secret-value");
}

#[test]
fn test_get_env_variable_not_found() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    let value = db.get_env_variable(&manager, "NONEXISTENT").unwrap();
    assert!(value.is_none());
}

#[test]
fn test_list_env_variables() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "ALPHA_KEY", "value1", None)
        .unwrap();
    db.set_env_variable(&manager, "BETA_KEY", "value2", Some("Beta"))
        .unwrap();

    let vars = db.list_env_variables().unwrap();

    assert_eq!(vars.len(), 2);
    // Should be sorted by name
    assert_eq!(vars[0].name, "ALPHA_KEY");
    assert_eq!(vars[1].name, "BETA_KEY");
    // Values should be masked
    assert!(vars[0].masked_value.contains("****"));
    assert!(vars[1].masked_value.contains("****"));
}

#[test]
fn test_delete_env_variable() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "TO_DELETE", "value", None)
        .unwrap();
    db.delete_env_variable("TO_DELETE").unwrap();

    let value = db.get_env_variable(&manager, "TO_DELETE").unwrap();
    assert!(value.is_none());
}

#[test]
fn test_delete_env_variable_not_found() {
    let db = Database::new_in_memory().unwrap();

    let result = db.delete_env_variable("NONEXISTENT");
    assert!(result.is_err());
}

#[test]
fn test_env_variable_exists() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    assert!(!db.env_variable_exists("MY_VAR").unwrap());

    db.set_env_variable(&manager, "MY_VAR", "value", None)
        .unwrap();

    assert!(db.env_variable_exists("MY_VAR").unwrap());
}

#[test]
fn test_env_manager_from_machine_id() {
    // 设置测试环境变量
    std::env::set_var("MANTRA_MACHINE_ID", "test-machine-123");

    let manager = EnvManager::from_machine_id();
    let plaintext = "test-encryption";

    let encrypted = manager.encrypt(plaintext);
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);

    // 清理
    std::env::remove_var("MANTRA_MACHINE_ID");
}

#[test]
fn test_encrypt_empty_string() {
    let manager = create_test_env_manager();

    let encrypted = manager.encrypt("");
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_unicode() {
    let manager = create_test_env_manager();
    let plaintext = "密钥🔑测试";

    let encrypted = manager.encrypt(plaintext);
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_long_value() {
    let manager = create_test_env_manager();
    let plaintext = "a".repeat(10000);

    let encrypted = manager.encrypt(&plaintext);
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

// ===== Story 11.4: 变量注入逻辑测试 =====

#[test]
fn test_resolve_env_references_simple_format() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    // 设置环境变量
    db.set_env_variable(&manager, "API_KEY", "sk-secret123", None)
        .unwrap();

    // 解析简单格式
    let result = resolve_env_references("$API_KEY", &db, &manager).unwrap();
    assert_eq!(result, "sk-secret123");
}

#[test]
fn test_resolve_env_references_braced_format() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "BASE_URL", "https://api.example.com", None)
        .unwrap();

    // 解析带花括号格式
    let result = resolve_env_references("${BASE_URL}/v1/chat", &db, &manager).unwrap();
    assert_eq!(result, "https://api.example.com/v1/chat");
}

#[test]
fn test_resolve_env_references_multiple_vars() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "HOST", "localhost", None)
        .unwrap();
    db.set_env_variable(&manager, "PORT", "8080", None)
        .unwrap();

    let result = resolve_env_references("http://$HOST:$PORT", &db, &manager).unwrap();
    assert_eq!(result, "http://localhost:8080");
}

#[test]
fn test_resolve_env_references_mixed_formats() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    db.set_env_variable(&manager, "USER", "admin", None)
        .unwrap();
    db.set_env_variable(&manager, "PASS", "secret", None)
        .unwrap();

    let result =
        resolve_env_references("${USER}:$PASS@server", &db, &manager).unwrap();
    assert_eq!(result, "admin:secret@server");
}

#[test]
fn test_resolve_env_references_missing_var() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    // 变量不存在时保留原始引用
    let result = resolve_env_references("$NONEXISTENT_VAR", &db, &manager).unwrap();
    assert_eq!(result, "$NONEXISTENT_VAR");
}

#[test]
fn test_resolve_env_references_no_vars() {
    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    // 没有变量引用的字符串保持不变
    let result = resolve_env_references("plain text", &db, &manager).unwrap();
    assert_eq!(result, "plain text");
}

#[test]
fn test_build_mcp_env() {
    use crate::models::mcp::{McpService, McpServiceSource};

    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    // 设置环境变量
    db.set_env_variable(&manager, "OPENAI_API_KEY", "sk-openai-key", None)
        .unwrap();

    // 创建 MCP 服务
    let service = McpService {
        id: "test-id".to_string(),
        name: "openai-mcp".to_string(),
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
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: "2026-01-30T00:00:00Z".to_string(),
        updated_at: "2026-01-30T00:00:00Z".to_string(),
        default_tool_policy: None,
    };

    let env = build_mcp_env(&service, &db, &manager).unwrap();

    assert_eq!(env.get("OPENAI_API_KEY"), Some(&"sk-openai-key".to_string()));
    assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
}

#[test]
fn test_build_mcp_env_no_env() {
    use crate::models::mcp::{McpService, McpServiceSource};

    let db = Database::new_in_memory().unwrap();
    let manager = create_test_env_manager();

    let service = McpService {
        id: "test-id".to_string(),
        name: "simple-mcp".to_string(),
        transport_type: Default::default(),
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

    let env = build_mcp_env(&service, &db, &manager).unwrap();
    assert!(env.is_empty());
}

#[test]
fn test_extract_env_var_names() {
    let names = extract_env_var_names("$API_KEY and ${BASE_URL}/path");
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"API_KEY".to_string()));
    assert!(names.contains(&"BASE_URL".to_string()));
}

#[test]
fn test_extract_env_var_names_duplicates() {
    let names = extract_env_var_names("$KEY $KEY ${KEY}");
    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "KEY");
}

#[test]
fn test_extract_env_var_names_no_vars() {
    let names = extract_env_var_names("no variables here");
    assert!(names.is_empty());
}
