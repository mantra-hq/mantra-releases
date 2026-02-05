use super::*;

// ===== McpSession 测试 =====

#[test]
fn test_mcp_session_new() {
    let session = McpSession::new();
    assert!(!session.session_id.is_empty());
    assert!(!session.internal_id.is_empty());
    assert_eq!(session.protocol_version, "2025-03-26");
    assert!(!session.initialized);
    assert!(session.work_dir.is_none());
    assert!(session.project_context.is_none());
}

#[test]
fn test_mcp_session_uuid_format() {
    let session = McpSession::new();
    // UUID v4 格式：xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    assert_eq!(session.session_id.len(), 36);
    assert!(Uuid::parse_str(&session.session_id).is_ok());
}

#[test]
fn test_mcp_session_touch() {
    let mut session = McpSession::new();
    let initial_time = session.last_active;

    // 等待一小段时间
    std::thread::sleep(std::time::Duration::from_millis(10));

    session.touch();
    assert!(session.last_active > initial_time);
}

#[test]
fn test_mcp_session_mark_initialized() {
    let mut session = McpSession::new();
    assert!(!session.initialized);

    session.mark_initialized();
    assert!(session.initialized);
}

#[test]
fn test_mcp_session_expiry() {
    let mut session = McpSession::with_timeout(0); // 0 分钟超时
    session.last_active = Utc::now() - Duration::minutes(1);

    assert!(session.is_expired());
}

#[test]
fn test_mcp_session_not_expired() {
    let session = McpSession::new();
    assert!(!session.is_expired());
}

#[test]
fn test_mcp_session_with_timeout() {
    let session = McpSession::with_timeout(60);
    assert_eq!(session.timeout_minutes, 60);
}

// ===== McpSessionStore 测试 =====

#[test]
fn test_session_store_create() {
    let mut store = McpSessionStore::new();
    let session = store.create_session();

    assert!(!session.session_id.is_empty());
    assert_eq!(store.active_count(), 1);
}

#[test]
fn test_session_store_get() {
    let mut store = McpSessionStore::new();
    let session = store.create_session();
    let session_id = session.session_id.clone();

    let retrieved = store.get_session(&session_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().session_id, session_id);
}

#[test]
fn test_session_store_get_expired() {
    let mut store = McpSessionStore::with_timeout(0);
    let session = store.create_session();
    let session_id = session.session_id.clone();

    // 手动设置过期
    if let Some(s) = store.sessions.get_mut(&session_id) {
        s.last_active = Utc::now() - Duration::minutes(1);
    }

    // 过期会话应该返回 None
    assert!(store.get_session(&session_id).is_none());
}

#[test]
fn test_session_store_remove() {
    let mut store = McpSessionStore::new();
    let session = store.create_session();
    let session_id = session.session_id.clone();

    let removed = store.remove_session(&session_id);
    assert!(removed.is_some());
    assert!(store.get_session(&session_id).is_none());
    assert_eq!(store.active_count(), 0);
}

#[test]
fn test_session_store_is_valid() {
    let mut store = McpSessionStore::new();
    let session = store.create_session();
    let session_id = session.session_id.clone();

    assert!(store.is_session_valid(&session_id));
    assert!(!store.is_session_valid("invalid-session-id"));
}

#[test]
fn test_session_store_cleanup_expired() {
    let mut store = McpSessionStore::new(); // 使用默认的 30 分钟超时

    // 创建两个会话
    let id1 = store.create_session().session_id.clone();
    let id2 = store.create_session().session_id.clone();

    // 手动将第一个会话的 timeout 设为 0 并标记为过期
    if let Some(s) = store.sessions.get_mut(&id1) {
        s.timeout_minutes = 0;
        s.last_active = Utc::now() - Duration::minutes(1);
    }

    // 清理过期
    let cleaned = store.cleanup_expired();
    assert_eq!(cleaned, 1);

    // 只有 session2 应该存在
    assert!(store.sessions.get(&id1).is_none());
    assert!(store.sessions.get(&id2).is_some());
}

// ===== SessionErrorResponse 测试 =====

#[test]
fn test_session_error_not_found() {
    let response = SessionErrorResponse::not_found("test-session-id");
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.id.is_none());
    assert_eq!(response.error.code, -32002);
    assert!(response.error.message.contains("test-session-id"));
    assert!(response.error.message.contains("reinitialize"));
}

#[test]
fn test_session_error_missing() {
    let response = SessionErrorResponse::missing_session_id();
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.id.is_none());
    assert_eq!(response.error.code, -32003);
    assert!(response.error.message.contains("Missing"));
}

#[test]
fn test_session_error_serialization() {
    let response = SessionErrorResponse::not_found("abc-123");
    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":null"));
    assert!(json.contains("-32002"));
}

// ===== Helper 函数测试 =====

#[test]
fn test_create_session_id_header() {
    let (name, value) = create_session_id_header("test-session-123");
    assert_eq!(name.as_str(), "mcp-session-id");
    assert_eq!(value.to_str().unwrap(), "test-session-123");
}

#[test]
fn test_extract_session_id() {
    use axum::http::Request as HttpRequest;

    let request = HttpRequest::builder()
        .header(MCP_SESSION_ID_HEADER, "my-session-id")
        .body(Body::empty())
        .unwrap();

    let session_id = extract_session_id(&request);
    assert_eq!(session_id, Some("my-session-id".to_string()));
}

#[test]
fn test_extract_session_id_missing() {
    use axum::http::Request as HttpRequest;

    let request = HttpRequest::builder().body(Body::empty()).unwrap();

    let session_id = extract_session_id(&request);
    assert!(session_id.is_none());
}

// ===== Story 11.26: Roots Capability 测试 =====

#[test]
fn test_mcp_session_roots_capability_defaults() {
    let session = McpSession::new();
    assert!(!session.supports_roots);
    assert!(!session.roots_list_changed);
    assert!(session.pending_roots_request_id.is_none());
    assert!(session.roots_paths.is_empty());
    assert!(!session.roots_request_timed_out);
}

#[test]
fn test_mcp_session_set_roots_capability() {
    let mut session = McpSession::new();
    session.set_roots_capability(true, true);
    assert!(session.supports_roots);
    assert!(session.roots_list_changed);
}

#[test]
fn test_mcp_session_set_roots_capability_without_list_changed() {
    let mut session = McpSession::new();
    session.set_roots_capability(true, false);
    assert!(session.supports_roots);
    assert!(!session.roots_list_changed);
}

#[test]
fn test_mcp_session_set_roots_paths() {
    let mut session = McpSession::new();
    let paths = vec![
        std::path::PathBuf::from("/home/user/project1"),
        std::path::PathBuf::from("/home/user/project2"),
    ];
    session.set_roots_paths(paths.clone());
    assert_eq!(session.roots_paths.len(), 2);
    assert_eq!(session.roots_paths[0], std::path::PathBuf::from("/home/user/project1"));
    assert_eq!(session.roots_paths[1], std::path::PathBuf::from("/home/user/project2"));
}

// ===== Story 11.26: ServerToClientManager 测试 =====

#[tokio::test]
async fn test_s2c_manager_register_channel() {
    let manager = ServerToClientManager::new();
    let mut rx = manager.register_channel("session-1", 16).await;

    // 发送消息应该成功
    let result = manager.send_to_client("session-1", "hello".to_string()).await;
    assert!(result.is_ok());

    // 接收消息
    let msg = rx.recv().await;
    assert_eq!(msg, Some("hello".to_string()));
}

#[tokio::test]
async fn test_s2c_manager_send_to_nonexistent_session() {
    let manager = ServerToClientManager::new();

    // 发送到不存在的会话应该失败
    let result = manager.send_to_client("nonexistent", "hello".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No SSE channel"));
}

#[tokio::test]
async fn test_s2c_manager_unregister_channel() {
    let manager = ServerToClientManager::new();
    let _rx = manager.register_channel("session-1", 16).await;

    // 注销后应该无法发送
    manager.unregister_channel("session-1").await;
    let result = manager.send_to_client("session-1", "hello".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_s2c_manager_handle_client_response() {
    let manager = ServerToClientManager::new();
    let _rx = manager.register_channel("session-1", 16).await;

    // 先创建一个 pending request
    let manager_clone = std::sync::Arc::new(manager);
    let manager_for_spawn = manager_clone.clone();

    // 在后台发送请求
    let handle = tokio::spawn(async move {
        manager_for_spawn.send_request_and_wait(
            "session-1",
            "req-1",
            r#"{"jsonrpc":"2.0","id":"req-1","method":"roots/list"}"#.to_string(),
            5,
        ).await
    });

    // 等待一小段时间让请求注册
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 发送响应
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-1",
        "result": {
            "roots": [
                {"uri": "file:///home/user/project", "name": "project"}
            ]
        }
    });
    let matched = manager_clone.handle_client_response("req-1", response).await;
    assert!(matched);

    // 验证请求收到了响应
    let result = handle.await.unwrap();
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.get("result").is_some());
}

#[tokio::test]
async fn test_s2c_manager_request_timeout() {
    let manager = ServerToClientManager::new();
    let _rx = manager.register_channel("session-1", 16).await;

    // 发送请求但不响应，应该超时
    let result = manager.send_request_and_wait(
        "session-1",
        "req-timeout",
        r#"{"jsonrpc":"2.0","id":"req-timeout","method":"roots/list"}"#.to_string(),
        1, // 1秒超时
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timed out"));
}

#[tokio::test]
async fn test_s2c_manager_handle_unmatched_response() {
    let manager = ServerToClientManager::new();

    // 没有 pending request 时，handle_client_response 应返回 false
    let response = serde_json::json!({"jsonrpc": "2.0", "id": "unknown", "result": {}});
    let matched = manager.handle_client_response("unknown", response).await;
    assert!(!matched);
}
