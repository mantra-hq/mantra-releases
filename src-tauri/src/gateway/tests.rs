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
        let original_port = manager.current_port();

        // Restart with same port
        manager.restart(Some(original_port)).await.expect("Restart should succeed");
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
