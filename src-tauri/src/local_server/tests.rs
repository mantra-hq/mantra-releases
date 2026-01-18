//! 本地 HTTP Server 模块测试

use std::time::Duration;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::local_server::{LocalServer, LocalServerConfig, ServerManager};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_server_start_and_stop() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 使用随机端口避免冲突
        let handle = server.start(Some(19900)).await;
        assert!(handle.is_ok());

        let handle = handle.unwrap();
        assert_eq!(handle.port(), 19900);

        // 关闭 Server
        handle.shutdown();

        // 等待关闭完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 端口应该可用
        assert!(LocalServer::check_port_available(19900).await);
    }

    #[tokio::test]
    async fn test_server_port_validation() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 特权端口应该被拒绝
        let result = server.start(Some(80)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_manager_lifecycle() {
        let dir = tempdir().unwrap();

        // 配置使用测试端口
        let config = LocalServerConfig {
            local_api_port: 19901,
        };
        config.save(dir.path()).unwrap();

        let mut manager = ServerManager::new(dir.path().to_path_buf());

        // 初始状态：未运行
        assert!(!manager.is_running());

        // 启动
        let result = manager.start().await;
        assert!(result.is_ok());
        assert!(manager.is_running());
        assert_eq!(manager.current_port(), 19901);

        // 重复启动应该成功（幂等）
        let result = manager.start().await;
        assert!(result.is_ok());

        // 停止
        manager.stop();
        assert!(!manager.is_running());

        // 等待关闭完成
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_server_manager_restart_with_new_port() {
        let dir = tempdir().unwrap();

        // 初始配置
        let config = LocalServerConfig {
            local_api_port: 19902,
        };
        config.save(dir.path()).unwrap();

        let mut manager = ServerManager::new(dir.path().to_path_buf());

        // 启动
        manager.start().await.unwrap();
        assert_eq!(manager.current_port(), 19902);

        // 使用新端口重启
        manager.restart(Some(19903)).await.unwrap();
        assert_eq!(manager.current_port(), 19903);

        // 配置应该已更新
        let loaded_config = LocalServerConfig::load(dir.path());
        assert_eq!(loaded_config.local_api_port, 19903);

        manager.stop();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19904)).await.unwrap();

        // 等待 Server 启动
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试健康检查端点
        let client = reqwest::Client::new();
        let response = client
            .get("http://127.0.0.1:19904/api/health")
            .send()
            .await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "mantra-client");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_privacy_check_endpoint_allow() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19905)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试正常文本（无敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19905/api/privacy/check")
            .json(&serde_json::json!({
                "prompt": "Hello, World!"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "allow");
        assert!(body.get("matches").is_none());

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_privacy_check_endpoint_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19906)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试包含 API Key 的文本
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19906/api/privacy/check")
            .json(&serde_json::json!({
                "prompt": "My API key is sk-1234567890abcdefghij1234"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");
        assert!(body["matches"].is_array());
        assert!(body["message"].is_string());

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_privacy_check_with_context() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19907)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试带上下文的请求
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19907/api/privacy/check")
            .json(&serde_json::json!({
                "prompt": "Normal text",
                "context": {
                    "tool": "claude-code",
                    "timestamp": "2026-01-18T10:00:00Z"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "allow");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_port_available() {
        // 未使用的端口应该可用
        assert!(LocalServer::check_port_available(19950).await);

        // 占用端口
        let listener = tokio::net::TcpListener::bind("127.0.0.1:19951").await.unwrap();

        // 被占用的端口应该不可用
        assert!(!LocalServer::check_port_available(19951).await);

        drop(listener);

        // 释放后应该可用
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(LocalServer::check_port_available(19951).await);
    }
}
