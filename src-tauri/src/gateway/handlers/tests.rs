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

// ===== Story 11.28: MCP 严格模式服务过滤测试 =====

use crate::gateway::aggregator::{McpAggregator, McpTool, ServiceCache, ServiceCapabilities};
use crate::gateway::project_services_query::ProjectServicesQueryClient;
use crate::models::mcp::{McpService, McpServiceSource, McpTransportType};
use std::path::PathBuf;

/// 创建带有项目上下文和项目服务客户端的测试 GatewayAppState
///
/// Story 11.28 Task 5: AC5 测试辅助函数
async fn create_test_app_state_with_project_context_and_services(
    project_id: &str,
) -> (GatewayAppState, String, tokio::sync::mpsc::Receiver<crate::gateway::project_services_query::PendingProjectServicesQuery>) {
    // 1. 创建 Gateway State 和 Session
    let mut gateway_state = GatewayState::with_defaults();
    let session = gateway_state.register_session();
    let session_id = session.session_id.clone();

    // 2. 设置项目上下文
    if let Some(s) = gateway_state.get_session_mut(&session_id) {
        s.set_auto_context(
            project_id.to_string(),
            "Test Project".to_string(),
            PathBuf::from("/test/path"),
        );
    }

    let state = Arc::new(RwLock::new(gateway_state));
    let stats = Arc::new(GatewayStats::new());

    // 3. 创建项目服务查询客户端
    let (ps_client, ps_rx) = ProjectServicesQueryClient::new(16);

    // 4. 创建 MCP 服务配置
    let test_service = McpService {
        id: "svc-test".to_string(),
        name: "test-service".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "echo".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: McpServiceSource::Manual,
        source_file: None,
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        default_tool_policy: None,
    };

    // 5. 创建 Aggregator（传入服务配置）
    let aggregator = Arc::new(McpAggregator::new(vec![test_service]));

    // 6. 手动填充缓存（模拟初始化后的状态）
    {
        let mut cache_guard = aggregator.cache.write().await;
        cache_guard.insert(
            "svc-test".to_string(),
            ServiceCache {
                service_id: "svc-test".to_string(),
                service_name: "test-service".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-test", "test-service", "my_tool", None, Some("Test tool".to_string()), None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 7. 创建 AppState
    let mut app_state = GatewayAppState::new(state, stats);
    app_state.aggregator = Some(aggregator);
    app_state.project_services_client = Some(Arc::new(ps_client));

    (app_state, session_id, ps_rx)
}

/// Story 11.28 AC5: 测试 tools/call 严格模式阻断
///
/// 当 session 有项目上下文时，调用未关联服务的工具应被阻断
#[tokio::test]
async fn test_handle_tools_call_blocked_by_strict_mode() {
    use crate::gateway::project_services_query::ProjectServicesQueryResponse;

    let (app_state, session_id, mut ps_rx) = create_test_app_state_with_project_context_and_services(
        "proj-123",
    ).await;

    // 启动模拟的项目服务查询服务 - 返回空服务列表（项目未关联任何服务）
    tokio::spawn(async move {
        while let Some(pending) = ps_rx.recv().await {
            let response = ProjectServicesQueryResponse {
                request_id: pending.request.request_id,
                service_ids: vec![], // 空列表 - 项目没有关联任何服务
            };
            let _ = pending.response_tx.send(response);
        }
    });

    // 构造 tools/call 请求
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "test-service/my_tool",
            "arguments": {}
        })),
    };

    // 调用 handle_tools_call
    let response = handle_tools_call(&app_state, &session_id, &request).await;

    // 验证返回错误
    assert!(response.error.is_some(), "应该返回错误");
    let error = response.error.unwrap();
    assert_eq!(error.code, -32601, "错误码应为 -32601");
    assert!(
        error.message.contains("not available in current project context"),
        "错误消息应包含 'not available in current project context'，实际: {}",
        error.message
    );
}

/// Story 11.28 AC5: 测试 tools/call 严格模式允许关联服务
///
/// 当服务在项目关联列表中时，调用应被放行（到 aggregator，此测试验证不被严格模式阻断）
#[tokio::test]
async fn test_handle_tools_call_allowed_by_strict_mode() {
    use crate::gateway::project_services_query::ProjectServicesQueryResponse;

    let (app_state, session_id, mut ps_rx) = create_test_app_state_with_project_context_and_services(
        "proj-123",
    ).await;

    // 启动模拟的项目服务查询服务 - 返回包含 svc-test 的服务列表
    tokio::spawn(async move {
        while let Some(pending) = ps_rx.recv().await {
            let response = ProjectServicesQueryResponse {
                request_id: pending.request.request_id,
                service_ids: vec!["svc-test".to_string()], // 包含目标服务
            };
            let _ = pending.response_tx.send(response);
        }
    });

    // 构造 tools/call 请求
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "test-service/my_tool",
            "arguments": {}
        })),
    };

    // 调用 handle_tools_call
    let response = handle_tools_call(&app_state, &session_id, &request).await;

    // 验证没有严格模式阻断错误
    // 注意：由于没有真正的 MCP 服务运行，会返回其他错误（如连接失败）
    // 但不应该是 "not available in current project context" 错误
    if let Some(error) = &response.error {
        assert!(
            !error.message.contains("not available in current project context"),
            "不应该被严格模式阻断，实际错误: {}",
            error.message
        );
    }
    // 如果没有错误（不太可能因为没有真正的服务），测试也通过
}
