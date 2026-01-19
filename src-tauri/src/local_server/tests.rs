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

    // Story 3.12: PreToolUse Hook 支持测试

    #[tokio::test]
    async fn test_pretooluse_webfetch_allow() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19960)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse WebFetch 请求（无敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19960/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "WebFetch",
                "tool_input": {
                    "url": "https://example.com/api",
                    "prompt": "Analyze this API documentation"
                },
                "context": {
                    "tool": "claude-code"
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
    async fn test_pretooluse_webfetch_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19961)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse WebFetch 请求（包含 API Key）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19961/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "WebFetch",
                "tool_input": {
                    "url": "https://example.com/api",
                    "prompt": "Use this API key: sk-1234567890abcdefghij1234"
                },
                "context": {
                    "tool": "claude-code"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");
        assert!(body["matches"].is_array());
        assert!(body["message"].as_str().unwrap().contains("PreToolUse"));

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pretooluse_websearch_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19962)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse WebSearch 请求（包含敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19962/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "WebSearch",
                "tool_input": {
                    "query": "how to use API key sk-1234567890abcdefghij1234"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pretooluse_bash_network_command_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19963)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse Bash 网络命令（包含敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19963/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": {
                    "command": "curl -H 'Authorization: Bearer sk-1234567890abcdefghij1234' https://api.example.com"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pretooluse_bash_local_command_allow() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19964)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse Bash 本地命令（即使包含敏感信息也应该放行）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19964/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": {
                    "command": "echo 'API_KEY=sk-1234567890abcdefghij1234' > .env"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        // 本地命令不检测，直接放行
        assert_eq!(body["action"], "allow");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pretooluse_task_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19965)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse Task 请求（包含敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19965/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "Task",
                "tool_input": {
                    "prompt": "Analyze this code with API key sk-1234567890abcdefghij1234",
                    "description": "Code analysis"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pretooluse_mcp_tool_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19966)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 PreToolUse MCP 工具请求（包含敏感信息）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19966/api/privacy/check")
            .json(&serde_json::json!({
                "hook_event": "PreToolUse",
                "tool_name": "mcp__github__create_issue",
                "tool_input": {
                    "title": "Bug report",
                    "body": "Found issue with API key sk-1234567890abcdefghij1234",
                    "repo": "owner/repo"
                }
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Story 3.11: /api/privacy/check-files 端点测试

    #[tokio::test]
    async fn test_check_files_empty_list() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19970)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试空文件列表应直接放行
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19970/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [],
                "tool_name": "Read"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "allow");
        assert!(body["findings"].as_array().unwrap().is_empty());

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_safe_file_allow() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建一个安全的测试文件
        let test_file = dir.path().join("safe.txt");
        std::fs::write(&test_file, "Hello, World! This is safe content.").unwrap();

        let handle = server.start(Some(19971)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19971/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [test_file.to_str().unwrap()],
                "tool_name": "Read"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "allow");
        assert!(body["findings"].as_array().unwrap().is_empty());

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_sensitive_env_block() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建一个包含敏感数据的 .env 文件
        let env_file = dir.path().join(".env");
        std::fs::write(&env_file, "OPENAI_API_KEY=sk-1234567890abcdefghij1234\nDATABASE_URL=postgres://user:pass@localhost/db").unwrap();

        let handle = server.start(Some(19972)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19972/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [env_file.to_str().unwrap()],
                "tool_name": "Read"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");
        assert!(!body["findings"].as_array().unwrap().is_empty());
        assert!(body["message"].is_string());

        // 检查 findings 包含文件路径信息
        let findings = body["findings"].as_array().unwrap();
        assert!(findings.iter().any(|f| f["file_path"].as_str().unwrap().contains(".env")));

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_multiple_files() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建多个测试文件
        let safe_file = dir.path().join("safe.txt");
        std::fs::write(&safe_file, "This is safe content.").unwrap();

        let env_file = dir.path().join(".env");
        std::fs::write(&env_file, "SECRET_KEY=sk-aaaaaaaaaaaaaaaaaaaaaaaa").unwrap();

        let config_file = dir.path().join("config.json");
        std::fs::write(&config_file, r#"{"api_key": "sk-bbbbbbbbbbbbbbbbbbbbbbbb"}"#).unwrap();

        let handle = server.start(Some(19973)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19973/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [
                    safe_file.to_str().unwrap(),
                    env_file.to_str().unwrap(),
                    config_file.to_str().unwrap()
                ],
                "tool_name": "Grep"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        // 应该检测到多个敏感数据
        let findings = body["findings"].as_array().unwrap();
        assert!(findings.len() >= 2);

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_nonexistent_file() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19974)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试不存在的文件应被跳过（不阻止操作）
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19974/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": ["/nonexistent/path/file.txt"],
                "tool_name": "Read"
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
    async fn test_check_files_directory_skipped() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        let handle = server.start(Some(19975)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试目录路径应被跳过
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19975/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [dir.path().to_str().unwrap()],
                "tool_name": "Grep"
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
    async fn test_check_files_finding_includes_line_number() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建包含多行内容的文件，敏感数据在第3行
        let env_file = dir.path().join(".env");
        std::fs::write(&env_file, "# Comment\nNAME=test\nAPI_KEY=sk-1234567890abcdefghij1234\nOTHER=value").unwrap();

        let handle = server.start(Some(19976)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19976/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [env_file.to_str().unwrap()],
                "tool_name": "Read"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        // 检查 finding 包含行号
        let findings = body["findings"].as_array().unwrap();
        assert!(!findings.is_empty());
        
        let finding = &findings[0];
        assert!(finding["line_number"].as_u64().unwrap() > 0);
        assert!(finding["rule_id"].is_string());
        assert!(finding["severity"].is_string());
        assert!(finding["preview"].is_string());

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_with_bash_tool() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建敏感文件
        let env_file = dir.path().join(".env");
        std::fs::write(&env_file, "TOKEN=sk-1234567890abcdefghij1234").unwrap();

        let handle = server.start(Some(19977)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 Bash 工具名
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19977/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [env_file.to_str().unwrap()],
                "tool_name": "Bash"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_check_files_with_edit_tool() {
        let dir = tempdir().unwrap();
        let server = LocalServer::new(dir.path().to_path_buf());

        // 创建敏感文件
        let env_file = dir.path().join("secrets.json");
        std::fs::write(&env_file, r#"{"key": "sk-1234567890abcdefghij1234"}"#).unwrap();

        let handle = server.start(Some(19978)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 测试 Edit 工具名
        let client = reqwest::Client::new();
        let response = client
            .post("http://127.0.0.1:19978/api/privacy/check-files")
            .json(&serde_json::json!({
                "file_paths": [env_file.to_str().unwrap()],
                "tool_name": "Edit"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["action"], "block");

        handle.shutdown();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
