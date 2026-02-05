use super::*;
use super::methods::{handle_initialize, handle_tools_call, handle_tools_list};
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

// ===== Story 11.26: MCP Roots Capability 测试 =====

#[tokio::test]
async fn test_handle_initialize_detects_roots_capability() {
    let (app_state, session_id) = create_test_app_state_with_session();

    // MCP 标准 initialize 请求，声明 roots capability
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "roots": {
                    "listChanged": true
                }
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 roots capability 已记录
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(session.supports_roots);
    assert!(session.roots_list_changed);
}

#[tokio::test]
async fn test_handle_initialize_no_roots_capability() {
    let (app_state, session_id) = create_test_app_state_with_session();

    // MCP initialize 请求，没有 roots capability
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 roots capability 未设置
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(!session.supports_roots);
    assert!(!session.roots_list_changed);
}

#[tokio::test]
async fn test_handle_initialize_roots_without_list_changed() {
    let (app_state, session_id) = create_test_app_state_with_session();

    // MCP initialize 请求，有 roots capability 但没有 listChanged
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "roots": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());

    // 验证 roots capability 已设置，但 listChanged 为 false
    let state_guard = app_state.state.read().await;
    let session = state_guard.get_session(&session_id).unwrap();
    assert!(session.supports_roots);
    assert!(!session.roots_list_changed);
}

#[tokio::test]
async fn test_handle_initialize_response_format() {
    let (app_state, session_id) = create_test_app_state_with_session();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {}
        })),
    };

    let response = handle_initialize(&app_state, &session_id, &request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());

    let result = response.result.unwrap();

    // 验证响应格式
    assert_eq!(result.get("protocolVersion").unwrap().as_str().unwrap(), "2025-03-26");
    assert!(result.get("capabilities").is_some());
    assert!(result.get("serverInfo").is_some());

    // 验证 server capabilities
    let caps = result.get("capabilities").unwrap();
    assert!(caps.get("tools").is_some());
    assert!(caps.get("resources").is_some());
    assert!(caps.get("prompts").is_some());
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
