//! Gateway 模块集成测试
//!
//! Story 11.1: SSE Server 核心 - Task 8

use super::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// =====================================================================
// Task 8.1: 单元测试
// =====================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    // --- Error Tests ---

    #[test]
    fn test_gateway_error_display() {
        let err = GatewayError::InvalidToken;
        assert_eq!(err.to_string(), "无效的 Token");

        let err = GatewayError::PortInUse(8080);
        assert_eq!(err.to_string(), "端口 8080 已被占用");

        let err = GatewayError::SessionNotFound("abc123".to_string());
        assert_eq!(err.to_string(), "会话不存在: abc123");
    }

    #[test]
    fn test_gateway_error_json_rpc_codes() {
        assert_eq!(GatewayError::InvalidToken.json_rpc_code(), -32001);
        assert_eq!(GatewayError::MissingToken.json_rpc_code(), -32001);
        assert_eq!(
            GatewayError::SessionNotFound("test".to_string()).json_rpc_code(),
            -32002
        );
        assert_eq!(
            GatewayError::JsonRpcError("parse error".to_string()).json_rpc_code(),
            -32700
        );
        assert_eq!(
            GatewayError::Internal("error".to_string()).json_rpc_code(),
            -32603
        );
    }

    #[test]
    fn test_gateway_error_status_codes() {
        use axum::http::StatusCode;

        assert_eq!(GatewayError::InvalidToken.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(GatewayError::MissingToken.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            GatewayError::SessionNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            GatewayError::JsonRpcError("error".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            GatewayError::Internal("error".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // --- State Tests ---

    #[test]
    fn test_client_session_creation() {
        let session = ClientSession::new();
        assert!(!session.session_id.is_empty());
        assert!(session.message_endpoint.contains(&session.session_id));
        assert!(session.message_endpoint.starts_with("/message?session_id="));
    }

    #[test]
    fn test_gateway_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.port, 0);
        assert!(!config.auth_token.is_empty());
        assert!(!config.enabled);
        assert!(!config.auto_start);

        // Verify auth_token is valid UUID
        let parsed = uuid::Uuid::parse_str(&config.auth_token);
        assert!(parsed.is_ok(), "auth_token should be valid UUID");
    }

    #[test]
    fn test_gateway_state_session_management() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut state = GatewayState::new(config);

        // Initial state
        assert_eq!(state.active_connections(), 0);

        // Register session
        let session = state.register_session();
        assert_eq!(state.active_connections(), 1);

        // Get session
        let retrieved = state.get_session(&session.session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session.session_id);

        // Remove session
        let removed = state.remove_session(&session.session_id);
        assert!(removed.is_some());
        assert_eq!(state.active_connections(), 0);

        // Remove non-existent session
        let not_found = state.remove_session("non-existent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_token_validation() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "secret-token-12345".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = GatewayState::new(config);

        assert!(state.validate_token("secret-token-12345"));
        assert!(!state.validate_token("wrong-token"));
        assert!(!state.validate_token(""));
        assert!(!state.validate_token("secret-token")); // partial match
    }

    #[test]
    fn test_gateway_stats() {
        let stats = GatewayStats::new();

        assert_eq!(stats.get_total_connections(), 0);
        assert_eq!(stats.get_total_requests(), 0);

        stats.increment_connections();
        stats.increment_connections();
        stats.increment_requests();
        stats.increment_requests();
        stats.increment_requests();

        assert_eq!(stats.get_total_connections(), 2);
        assert_eq!(stats.get_total_requests(), 3);
    }

    // --- Auth Tests ---

    #[test]
    fn test_extract_token_from_query() {
        use super::auth::extract_token_from_query;

        assert_eq!(
            extract_token_from_query("/sse?token=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_token_from_query("/sse?session_id=xxx&token=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_token_from_query("/message?token=secret&session_id=123"),
            Some("secret".to_string())
        );
        assert_eq!(extract_token_from_query("/sse"), None);
        assert_eq!(extract_token_from_query("/sse?other=value"), None);
        assert_eq!(extract_token_from_query("/sse?token="), None);
    }
}

// =====================================================================
// Task 8.2-8.4: 集成测试
// =====================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    // --- Server Lifecycle Tests ---

    #[tokio::test]
    async fn test_server_start_and_stop() {
        let config = GatewayConfig {
            port: 0, // Auto-assign
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);

        let handle = server.start(None).await.expect("Server should start");
        let port = handle.port();
        
        // Port should be in valid range
        assert!(port > 0);

        // Port should be occupied
        assert!(!GatewayServer::check_port_available(port).await);

        // Shutdown
        handle.shutdown();

        // Wait for port release
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_server_manager_lifecycle() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut manager = GatewayServerManager::new(config);

        // Initial state
        assert!(!manager.is_running());

        // Start
        manager.start().await.expect("Manager should start");
        assert!(manager.is_running());

        let port = manager.current_port();
        assert!(port > 0);

        // Double start should be idempotent
        manager.start().await.expect("Double start should succeed");
        assert!(manager.is_running());
        assert_eq!(manager.current_port(), port);

        // Stop
        manager.stop();
        assert!(!manager.is_running());

        // Double stop should be idempotent
        manager.stop();
        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_server_port_auto_assignment() {
        let server1 = GatewayServer::with_defaults();
        let server2 = GatewayServer::with_defaults();

        let handle1 = server1.start(None).await.expect("Server 1 should start");
        let port1 = handle1.port();

        let handle2 = server2.start(None).await.expect("Server 2 should start");
        let port2 = handle2.port();

        // Ports should be different
        assert_ne!(port1, port2);

        handle1.shutdown();
        handle2.shutdown();

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_server_manager_restart() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let mut manager = GatewayServerManager::new(config);

        // Start
        manager.start().await.expect("Start should succeed");
        assert!(manager.is_running());

        // Restart (let OS assign new port to avoid timing issues with port release)
        manager.restart(None).await.expect("Restart should succeed");
        assert!(manager.is_running());

        // Cleanup
        manager.stop();
    }

    // --- Token Authentication Tests ---

    #[tokio::test]
    async fn test_auth_layer_validation() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "valid-secret-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = Arc::new(RwLock::new(GatewayState::new(config)));
        let auth = AuthLayer::new(state);

        assert!(auth.validate("valid-secret-token").await);
        assert!(!auth.validate("invalid-token").await);
        assert!(!auth.validate("").await);
    }

    // --- Concurrent Connection Tests (AC4) ---

    #[tokio::test]
    async fn test_concurrent_session_registration() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = Arc::new(RwLock::new(GatewayState::new(config)));

        // Register 50+ sessions concurrently
        let mut handles = Vec::new();
        for _ in 0..60 {
            let state_clone = state.clone();
            handles.push(tokio::spawn(async move {
                let mut state_guard = state_clone.write().await;
                state_guard.register_session()
            }));
        }

        // Wait for all registrations
        let sessions: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        // All 60 sessions should be registered
        assert_eq!(sessions.len(), 60);

        // Verify unique session IDs
        let unique_ids: std::collections::HashSet<_> = sessions.iter().map(|s| &s.session_id).collect();
        assert_eq!(unique_ids.len(), 60);

        // Verify state count
        let state_guard = state.read().await;
        assert_eq!(state_guard.active_connections(), 60);
    }

    #[tokio::test]
    async fn test_concurrent_token_validation() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "concurrent-test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = Arc::new(RwLock::new(GatewayState::new(config)));

        // Validate token concurrently
        let mut handles = Vec::new();
        for i in 0..100 {
            let state_clone = state.clone();
            let token = if i % 2 == 0 {
                "concurrent-test-token".to_string()
            } else {
                "invalid-token".to_string()
            };
            handles.push(tokio::spawn(async move {
                let state_guard = state_clone.read().await;
                state_guard.validate_token(&token)
            }));
        }

        let results: Vec<bool> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        // Half should be valid, half invalid
        let valid_count = results.iter().filter(|&&v| v).count();
        let invalid_count = results.iter().filter(|&&v| !v).count();
        assert_eq!(valid_count, 50);
        assert_eq!(invalid_count, 50);
    }
}

// =====================================================================
// HTTP Client Tests (requires running server)
// =====================================================================

#[cfg(test)]
mod http_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(None).await.expect("Server should start");
        let port = handle.port();

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Test health endpoint (no auth required)
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/health", port))
            .send()
            .await
            .expect("Health check should succeed");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "mantra-gateway");

        handle.shutdown();
    }

    #[tokio::test]
    async fn test_sse_endpoint_requires_auth() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "secret-token-123".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(None).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // Without token - should fail
        let response = client
            .get(format!("http://127.0.0.1:{}/sse", port))
            .send()
            .await
            .expect("Request should complete");

        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

        // With invalid token - should fail
        let response = client
            .get(format!("http://127.0.0.1:{}/sse?token=wrong", port))
            .send()
            .await
            .expect("Request should complete");

        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

        handle.shutdown();
    }

    #[tokio::test]
    async fn test_message_endpoint_requires_auth() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "secret-token-456".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(None).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // Without token - should fail
        let response = client
            .post(format!("http://127.0.0.1:{}/message?session_id=123", port))
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "ping"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

        handle.shutdown();
    }
}

// =====================================================================
// Story 11.17: MCP Aggregator Tests
// =====================================================================

#[cfg(test)]
mod aggregator_integration_tests {
    use super::*;
    use crate::gateway::aggregator::{McpAggregator, McpTool, McpResource, McpPrompt};

    // --- Task 9.1: 命名空间转换函数测试 ---

    #[test]
    fn test_tool_namespace_prefix_add() {
        let tool = McpTool::new(
            "service1-id",
            "service1",
            "read_file",
            None,
            Some("Read a file".to_string()),
            None,
            None,
        );
        // 验证工具名称包含服务前缀 (格式: service_name/tool_name)
        assert!(tool.name.starts_with("service1/"));
        assert_eq!(tool.name, "service1/read_file");
        assert_eq!(tool.original_name, "read_file");
    }

    #[test]
    fn test_tool_namespace_prefix_parse() {
        // 有命名空间
        let result = McpAggregator::parse_tool_name("git_mcp/list_commits");
        assert!(result.is_ok());
        let (service, tool) = result.unwrap();
        assert_eq!(service, "git_mcp");
        assert_eq!(tool, "list_commits");

        // 斜杠在工具名中
        let result = McpAggregator::parse_tool_name("service/my/tool");
        assert!(result.is_ok());
        let (service, tool) = result.unwrap();
        assert_eq!(service, "service");
        assert_eq!(tool, "my/tool");

        // 无命名空间
        let result = McpAggregator::parse_tool_name("read_file");
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_uri_namespace() {
        let resource = McpResource::new(
            "fs_service_id",
            "fs_service",
            "file:///home/user/file.txt",
            Some("User file".to_string()),
            None,
            None,
        );
        // 验证 URI 包含服务前缀 (格式: service_name:::original_uri)
        assert!(resource.uri.starts_with("fs_service:::"));
        assert_eq!(resource.original_uri, "file:///home/user/file.txt");

        // 解析带前缀的 URI
        let (service, original) = McpResource::parse_prefixed_uri(&resource.uri).unwrap();
        assert_eq!(service, "fs_service");
        assert_eq!(original, "file:///home/user/file.txt");
    }

    #[test]
    fn test_resource_uri_namespace_preserves_https_scheme() {
        let resource = McpResource::new(
            "api_service_id",
            "api_service",
            "https://example.com/resource",
            Some("Remote resource".to_string()),
            None,
            None,
        );
        // 验证 https:// scheme 被完整保留
        assert!(resource.uri.starts_with("api_service:::"));
        assert_eq!(resource.original_uri, "https://example.com/resource");

        let (service, original) = McpResource::parse_prefixed_uri(&resource.uri).unwrap();
        assert_eq!(service, "api_service");
        assert_eq!(original, "https://example.com/resource");
    }

    #[test]
    fn test_prompt_namespace() {
        let prompt = McpPrompt::new(
            "code_service_id",
            "code_service",
            "explain_code",
            Some("Explain code".to_string()),
            None,
        );
        // 提示名称包含服务前缀 (格式: service_name/prompt_name)
        assert!(prompt.name.starts_with("code_service/"));
        assert_eq!(prompt.name, "code_service/explain_code");
        assert_eq!(prompt.original_name, "explain_code");
    }

    // --- Task 9.2: Tool Policy 过滤逻辑测试 ---
    // Tool Policy 的核心测试在 models/mcp.rs 中
    // 这里测试 Aggregator 与 Tool Policy 的集成

    #[tokio::test]
    async fn test_aggregator_list_tools_empty() {
        let aggregator = McpAggregator::new(vec![]);
        let tools = aggregator.list_tools(None, None).await;
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_aggregator_list_resources_empty() {
        let aggregator = McpAggregator::new(vec![]);
        let resources = aggregator.list_resources(None).await;
        assert!(resources.is_empty());
    }

    #[tokio::test]
    async fn test_aggregator_list_prompts_empty() {
        let aggregator = McpAggregator::new(vec![]);
        let prompts = aggregator.list_prompts(None).await;
        assert!(prompts.is_empty());
    }

    // --- Task 9.3 & 9.4: 集成测试 - HTTP 端点测试 ---
    // 这些测试需要实际的 MCP 服务运行，作为 E2E 测试的一部分

    #[tokio::test]
    async fn test_mcp_endpoint_tools_list_no_aggregator() {
        // 测试在没有 aggregator 时，tools/list 返回空列表
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        // 使用 Some(0) 让 OS 自动分配端口
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // 发送 tools/list 请求
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");

        // 验证返回空工具列表
        assert!(body.get("result").is_some());
        let result = body.get("result").unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert!(tools.is_empty());

        handle.shutdown();
    }

    #[tokio::test]
    async fn test_mcp_endpoint_resources_list_no_aggregator() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "resources/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let result = body.get("result").unwrap();
        let resources = result.get("resources").unwrap().as_array().unwrap();
        assert!(resources.is_empty());

        handle.shutdown();
    }

    #[tokio::test]
    async fn test_mcp_endpoint_prompts_list_no_aggregator() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "prompts/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let result = body.get("result").unwrap();
        let prompts = result.get("prompts").unwrap().as_array().unwrap();
        assert!(prompts.is_empty());

        handle.shutdown();
    }

    #[tokio::test]
    async fn test_mcp_endpoint_tools_call_unknown_tool() {
        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::new(config);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // 调用不存在的工具
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "nonexistent_service/some_tool",
                    "arguments": {}
                }
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");

        // 应该返回错误（服务未找到）
        assert!(body.get("error").is_some());

        handle.shutdown();
    }
}

// =====================================================================
// Story 11.9 Phase 2: Tool Policy Gateway 集成测试
// =====================================================================

#[cfg(test)]
mod policy_integration_tests {
    use super::*;
    use crate::gateway::aggregator::{McpAggregator, McpTool, ServiceCache, ServiceCapabilities};
    use crate::gateway::policy::{PolicyResolver, SharedPolicyResolver};
    use crate::models::mcp::ToolPolicy;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::time::Duration;

    /// 用于测试的 MockPolicyResolver
    struct MockPolicyResolver {
        policies: HashMap<(Option<String>, String), ToolPolicy>,
    }

    impl MockPolicyResolver {
        fn new() -> Self {
            Self {
                policies: HashMap::new(),
            }
        }

        fn add_policy(
            &mut self,
            project_id: Option<String>,
            service_id: String,
            policy: ToolPolicy,
        ) {
            self.policies.insert((project_id, service_id), policy);
        }
    }

    #[async_trait]
    impl PolicyResolver for MockPolicyResolver {
        async fn get_policy(&self, project_id: Option<&str>, service_id: &str) -> ToolPolicy {
            let key = (project_id.map(|s| s.to_string()), service_id.to_string());
            self.policies
                .get(&key)
                .cloned()
                // 如果没有项目级匹配，回退到全局
                .or_else(|| {
                    let global_key = (None, service_id.to_string());
                    self.policies.get(&global_key).cloned()
                })
                .unwrap_or_default()
        }

        async fn get_policies(
            &self,
            project_id: Option<&str>,
            service_ids: &[String],
        ) -> HashMap<String, ToolPolicy> {
            let mut result = HashMap::new();
            for service_id in service_ids {
                let policy = self.get_policy(project_id, service_id).await;
                result.insert(service_id.clone(), policy);
            }
            result
        }
    }

    /// 创建带有工具数据的 Aggregator
    async fn create_test_aggregator() -> Arc<McpAggregator> {
        let aggregator = McpAggregator::new(vec![]);

        {
            let mut cache = aggregator.cache.write().await;
            cache.insert(
                "svc-a".to_string(),
                ServiceCache {
                    service_id: "svc-a".to_string(),
                    service_name: "service-alpha".to_string(),
                    capabilities: ServiceCapabilities::default(),
                    tools: vec![
                        McpTool::new("svc-a", "service-alpha", "read_file", None, None, None, None),
                        McpTool::new("svc-a", "service-alpha", "write_file", None, None, None, None),
                    ],
                    resources: vec![],
                    prompts: vec![],
                    initialized: true,
                    last_updated: Some(chrono::Utc::now()),
                    error: None,
                },
            );
            cache.insert(
                "svc-b".to_string(),
                ServiceCache {
                    service_id: "svc-b".to_string(),
                    service_name: "service-beta".to_string(),
                    capabilities: ServiceCapabilities::default(),
                    tools: vec![
                        McpTool::new("svc-b", "service-beta", "list_dir", None, None, None, None),
                    ],
                    resources: vec![],
                    prompts: vec![],
                    initialized: true,
                    last_updated: Some(chrono::Utc::now()),
                    error: None,
                },
            );
        }

        Arc::new(aggregator)
    }

    /// 测试: Aggregator 有工具，但无 PolicyResolver 时返回所有工具
    #[tokio::test]
    async fn test_tools_list_with_aggregator_no_policy() {
        let aggregator = create_test_aggregator().await;

        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::with_aggregator(config, aggregator);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let tools = body["result"]["tools"].as_array().unwrap();

        // 无 PolicyResolver → 返回全部 3 个工具
        assert_eq!(tools.len(), 3);

        handle.shutdown();
    }

    /// 测试: PolicyResolver DenyAll 某服务时，该服务工具被过滤
    #[tokio::test]
    async fn test_tools_list_with_policy_deny_all() {
        let aggregator = create_test_aggregator().await;

        let mut mock_resolver = MockPolicyResolver::new();
        // 服务 A 全局 DenyAll - 使用 custom 配置一个不存在的工具来模拟 deny all
        mock_resolver.add_policy(
            None,
            "svc-a".to_string(),
            ToolPolicy::custom(vec!["__none__".to_string()]),
        );

        let resolver: SharedPolicyResolver = Arc::new(mock_resolver);

        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server =
            GatewayServer::with_aggregator_and_policy(config, aggregator, resolver);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let tools = body["result"]["tools"].as_array().unwrap();

        // 服务 A (2 个工具) 被 DenyAll，只剩服务 B 的 1 个工具
        assert_eq!(tools.len(), 1);
        let tool_name = tools[0]["name"].as_str().unwrap();
        assert!(tool_name.contains("list_dir"));

        handle.shutdown();
    }

    /// 测试: Custom Policy 只允许部分工具
    #[tokio::test]
    async fn test_tools_list_with_custom_policy() {
        let aggregator = create_test_aggregator().await;

        let mut mock_resolver = MockPolicyResolver::new();
        // 服务 A 只允许 read_file
        mock_resolver.add_policy(
            None,
            "svc-a".to_string(),
            ToolPolicy::custom(vec!["read_file".to_string()]),
        );

        let resolver: SharedPolicyResolver = Arc::new(mock_resolver);

        let config = GatewayConfig {
            port: 0,
            auth_token: "test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server =
            GatewayServer::with_aggregator_and_policy(config, aggregator, resolver);
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let tools = body["result"]["tools"].as_array().unwrap();

        // 服务 A: read_file 允许，write_file 被过滤 → 1
        // 服务 B: AllowAll (默认) → 1
        // 总计 2 个工具
        assert_eq!(tools.len(), 2);

        // 验证 write_file 不在返回列表中
        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.iter().all(|name| !name.contains("write_file")));
        assert!(tool_names.iter().any(|name| name.contains("read_file")));
        assert!(tool_names.iter().any(|name| name.contains("list_dir")));

        handle.shutdown();
    }
}

// =====================================================================
// Story 11.9 Phase 2: E2E 测试 - 完整配置 → 生效流程
// =====================================================================

#[cfg(test)]
mod policy_e2e_tests {
    use super::*;
    use crate::gateway::aggregator::{McpAggregator, McpTool, ServiceCache, ServiceCapabilities};
    use crate::gateway::policy::StoragePolicyResolver;
    use crate::models::mcp::{
        CreateMcpServiceRequest, McpServiceSource, ToolPolicy,
    };
    use crate::storage::Database;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// E2E 测试: 在数据库中配置 Tool Policy，然后通过 HTTP 端点验证过滤生效
    ///
    /// 流程:
    /// 1. 创建 in-memory 数据库
    /// 2. 创建 MCP 服务，设置全局 Tool Policy
    /// 3. 创建 Aggregator 并填充缓存
    /// 4. 创建 StoragePolicyResolver (使用真实数据库)
    /// 5. 启动 Gateway Server
    /// 6. 通过 HTTP 请求 tools/list
    /// 7. 验证返回结果符合 Policy 配置
    #[tokio::test]
    async fn test_e2e_global_policy_filters_tools() {
        // 1. 创建数据库和服务
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));

        let service_id = {
            let db_guard = db.lock().unwrap();
            let request = CreateMcpServiceRequest {
                name: "test-mcp".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service = db_guard.create_mcp_service(&request).unwrap();

            // 设置全局 Tool Policy: 只允许 read_file
            let policy = ToolPolicy::custom(vec!["read_file".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&policy))
                .unwrap();

            service.id
        };

        // 2. 创建 Aggregator 并填充工具缓存
        let aggregator = McpAggregator::new(vec![]);
        {
            let mut cache = aggregator.cache.write().await;
            cache.insert(
                service_id.clone(),
                ServiceCache {
                    service_id: service_id.clone(),
                    service_name: "test-mcp".to_string(),
                    capabilities: ServiceCapabilities::default(),
                    tools: vec![
                        McpTool::new(&service_id, "test-mcp", "read_file", None, None, None, None),
                        McpTool::new(&service_id, "test-mcp", "write_file", None, None, None, None),
                        McpTool::new(&service_id, "test-mcp", "delete_file", None, None, None, None),
                    ],
                    resources: vec![],
                    prompts: vec![],
                    initialized: true,
                    last_updated: Some(chrono::Utc::now()),
                    error: None,
                },
            );
        }
        let shared_aggregator = Arc::new(aggregator);

        // 3. 创建 StoragePolicyResolver (使用真实数据库)
        let resolver = StoragePolicyResolver::new(db.clone());
        let shared_resolver: super::policy::SharedPolicyResolver = Arc::new(resolver);

        // 4. 启动 Gateway
        let config = GatewayConfig {
            port: 0,
            auth_token: "e2e-test-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::with_aggregator_and_policy(
            config,
            shared_aggregator,
            shared_resolver,
        );
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        // 5. 请求 tools/list
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer e2e-test-token")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let tools = body["result"]["tools"].as_array().unwrap();

        // 6. 验证: 3 个工具中只有 read_file 被返回
        assert_eq!(tools.len(), 1);
        let tool_name = tools[0]["name"].as_str().unwrap();
        assert!(tool_name.contains("read_file"));

        handle.shutdown();
    }

    /// E2E 测试: 项目级 Policy 覆盖全局 Policy
    #[tokio::test]
    async fn test_e2e_project_policy_overrides_global() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));

        let service_id = {
            let db_guard = db.lock().unwrap();
            let request = CreateMcpServiceRequest {
                name: "test-mcp-2".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: McpServiceSource::Manual,
                source_file: None,
            };
            let service = db_guard.create_mcp_service(&request).unwrap();

            // 全局 Policy: DenyAll - 使用 custom 配置一个不存在的工具来模拟 deny all
            let global_policy = ToolPolicy::custom(vec!["__none__".to_string()]);
            db_guard
                .update_service_default_policy(&service.id, Some(&global_policy))
                .unwrap();

            service.id
        };

        // 创建 Aggregator
        let aggregator = McpAggregator::new(vec![]);
        {
            let mut cache = aggregator.cache.write().await;
            cache.insert(
                service_id.clone(),
                ServiceCache {
                    service_id: service_id.clone(),
                    service_name: "test-mcp-2".to_string(),
                    capabilities: ServiceCapabilities::default(),
                    tools: vec![
                        McpTool::new(&service_id, "test-mcp-2", "tool_a", None, None, None, None),
                        McpTool::new(&service_id, "test-mcp-2", "tool_b", None, None, None, None),
                    ],
                    resources: vec![],
                    prompts: vec![],
                    initialized: true,
                    last_updated: Some(chrono::Utc::now()),
                    error: None,
                },
            );
        }
        let shared_aggregator = Arc::new(aggregator);

        // StoragePolicyResolver
        let resolver = StoragePolicyResolver::new(db.clone());
        let shared_resolver: super::policy::SharedPolicyResolver = Arc::new(resolver);

        // 启动 Gateway
        let config = GatewayConfig {
            port: 0,
            auth_token: "e2e-test-token-2".to_string(),
            enabled: true,
            auto_start: false,
        };
        let server = GatewayServer::with_aggregator_and_policy(
            config,
            shared_aggregator,
            shared_resolver,
        );
        let handle = server.start(Some(0)).await.expect("Server should start");
        let port = handle.port();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();

        // 无项目上下文: 全局 DenyAll，工具应为空
        let response = client
            .post(format!("http://127.0.0.1:{}/mcp", port))
            .header("Authorization", "Bearer e2e-test-token-2")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await
            .expect("Request should complete");

        assert!(response.status().is_success());
        let body: serde_json::Value = response.json().await.expect("Should parse JSON");
        let tools = body["result"]["tools"].as_array().unwrap();

        // 全局 DenyAll: 所有工具被过滤
        assert_eq!(tools.len(), 0);

        handle.shutdown();
    }
}
