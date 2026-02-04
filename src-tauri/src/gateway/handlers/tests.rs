use super::*;
use super::methods::{handle_initialize, handle_tools_call, handle_tools_list};
use std::path::PathBuf;
use crate::models::mcp::ToolPolicy;
use crate::gateway::state::{GatewayState, GatewayStats};

#[test]
fn test_json_rpc_response_success() {
    let response = JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({"result": "ok"}));
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    assert_eq!(response.jsonrpc, "2.0");
}

#[test]
fn test_json_rpc_response_error() {
    let response = JsonRpcResponse::error(Some(serde_json::json!(1)), -32600, "Test error".to_string());
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Test error");
}

#[test]
fn test_json_rpc_method_not_found() {
    let response = JsonRpcResponse::method_not_found(Some(serde_json::json!(1)));
    let error = response.error.unwrap();
    assert_eq!(error.code, -32601);
}

#[test]
fn test_json_rpc_parse_error() {
    let response = JsonRpcResponse::parse_error();
    assert!(response.id.is_none());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32700);
}

// ===== Story 11.5: 上下文路由测试 =====

#[test]
fn test_parse_work_dir_from_root_uri() {
    let params = serde_json::json!({
        "rootUri": "file:///home/user/projects/mantra"
    });

    let result = parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_from_workspace_folders() {
    let params = serde_json::json!({
        "workspaceFolders": [
            {
                "uri": "file:///home/user/projects/mantra",
                "name": "mantra"
            }
        ]
    });

    let result = parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_workspace_folders_priority() {
    // workspaceFolders 应该优先于 rootUri
    let params = serde_json::json!({
        "rootUri": "file:///other/path",
        "workspaceFolders": [
            {
                "uri": "file:///home/user/projects/mantra",
                "name": "mantra"
            }
        ]
    });

    let result = parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_from_root_path() {
    let params = serde_json::json!({
        "rootPath": "/home/user/projects/mantra"
    });

    let result = parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_no_params() {
    let params = serde_json::json!({});

    let result = parse_work_dir_from_params(&params);
    assert!(result.is_none());
}

#[test]
fn test_uri_to_path_unix() {
    let result = uri_to_path("file:///home/user/projects");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects"));
}

#[test]
fn test_uri_to_path_with_spaces() {
    let result = uri_to_path("file:///home/user/my%20projects");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/my projects"));
}

#[test]
fn test_uri_to_path_invalid() {
    let result = uri_to_path("http://example.com");
    assert!(result.is_none());
}

// ===== Story 11.5: tools/call 参数验证测试 =====

/// 创建测试用的 GatewayAppState
fn create_test_app_state() -> GatewayAppState {
    let state = Arc::new(RwLock::new(GatewayState::with_defaults()));
    let stats = Arc::new(GatewayStats::new());
    GatewayAppState::new(state, stats)
}

/// 创建带有已注册会话的测试 GatewayAppState
fn create_test_app_state_with_session() -> (GatewayAppState, String) {
    let mut gateway_state = GatewayState::with_defaults();
    let session = gateway_state.register_session();
    let session_id = session.session_id.clone();

    let state = Arc::new(RwLock::new(gateway_state));
    let stats = Arc::new(GatewayStats::new());
    let app_state = GatewayAppState::new(state, stats);
    (app_state, session_id)
}

#[tokio::test]
async fn test_handle_tools_call_missing_params() {
    let app_state = create_test_app_state();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: None, // 缺少 params
    };

    let response = handle_tools_call(&app_state, "test-session", &request).await;
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602); // Invalid params
    assert!(error.message.contains("Missing params"));
}

#[tokio::test]
async fn test_handle_tools_call_missing_tool_name() {
    let app_state = create_test_app_state();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "arguments": {}
        })), // 缺少 name
    };

    let response = handle_tools_call(&app_state, "test-session", &request).await;
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Missing tool name"));
}

#[tokio::test]
async fn test_handle_tools_call_invalid_tool_name_format() {
    let app_state = create_test_app_state();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "invalid_tool_name_without_slash",
            "arguments": {}
        })),
    };

    let response = handle_tools_call(&app_state, "test-session", &request).await;
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid tool name format"));
}

#[tokio::test]
async fn test_handle_tools_call_service_not_found() {
    let app_state = create_test_app_state();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "nonexistent_service/tool_name",
            "arguments": {"key": "value"}
        })),
    };

    let response = handle_tools_call(&app_state, "test-session", &request).await;
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    // Story 11.17: 当没有 aggregator 时返回 -32603 (Internal error)
    assert_eq!(error.code, -32603);
    // 测试 app_state 没有 aggregator，所以返回 "not initialized" 错误
    assert!(error.message.contains("not initialized") || error.message.contains("Aggregator"));
}

// ===== Story 11.5: tools/list 测试 =====

#[tokio::test]
async fn test_handle_tools_list_returns_empty_list() {
    let app_state = create_test_app_state();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = handle_tools_list(&app_state, "test-session", &request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();
    assert!(tools.is_empty());
}

// ===== Story 11.5: initialize 测试 =====

#[tokio::test]
async fn test_handle_initialize_stores_work_dir() {
    let (app_state, session_id) = create_test_app_state_with_session();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "rootUri": "file:///home/user/projects/test"
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 work_dir 已存储
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(session.work_dir.is_some());
    assert_eq!(
        session.work_dir.as_ref().unwrap(),
        &PathBuf::from("/home/user/projects/test")
    );
}

#[tokio::test]
async fn test_handle_initialize_no_work_dir() {
    let (app_state, session_id) = create_test_app_state_with_session();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "capabilities": {}
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 work_dir 为 None
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(session.work_dir.is_none());
}

#[tokio::test]
async fn test_handle_initialize_with_workspace_folders() {
    let (app_state, session_id) = create_test_app_state_with_session();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "workspaceFolders": [
                {
                    "uri": "file:///home/user/workspace/project1",
                    "name": "project1"
                },
                {
                    "uri": "file:///home/user/workspace/project2",
                    "name": "project2"
                }
            ]
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 work_dir 使用第一个 workspace folder
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(session.work_dir.is_some());
    assert_eq!(
        session.work_dir.as_ref().unwrap(),
        &PathBuf::from("/home/user/workspace/project1")
    );
}

// ===== Story 11.18: 简化 Tool Policy 拦截测试 =====

#[test]
fn test_is_tool_blocked_allow_all() {
    let policy = ToolPolicy::allow_all();

    // 全选模式下，所有工具都被允许
    assert!(!is_tool_blocked("read_file", &policy));
    assert!(!is_tool_blocked("write_file", &policy));
}

#[test]
fn test_is_tool_blocked_custom() {
    let policy = ToolPolicy::custom(vec!["read_file".to_string()]);

    // 部分选模式下，只有 allowedTools 中的工具被允许
    assert!(!is_tool_blocked("read_file", &policy));
    assert!(is_tool_blocked("write_file", &policy));
}

#[test]
fn test_is_tool_blocked_inherit() {
    let policy = ToolPolicy::inherit();

    // 继承模式下，默认允许（实际继承由 PolicyResolver 处理）
    assert!(!is_tool_blocked("read_file", &policy));
    assert!(!is_tool_blocked("write_file", &policy));
}

#[test]
fn test_tool_blocked_error_response() {
    let response = tool_blocked_error(Some(serde_json::json!(1)), "git-mcp/write_file");
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32601);
    assert!(error.message.contains("Tool not found"));
    assert!(error.message.contains("git-mcp/write_file"));
}
