//! OAuth 回调服务器
//!
//! Story 11.12: Remote MCP OAuth Support - Task 1.3
//!
//! 临时 HTTP 服务器，用于接收 OAuth 回调。

use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, watch, Mutex};

/// 回调参数
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    /// Authorization code
    pub code: Option<String>,
    /// State 参数
    pub state: Option<String>,
    /// 错误代码
    pub error: Option<String>,
    /// 错误描述
    pub error_description: Option<String>,
}

/// 回调结果
#[derive(Debug, Clone)]
pub enum CallbackResult {
    /// 成功，包含 code 和 state
    Success { code: String, state: String },
    /// 授权被拒绝
    Denied { error: String, description: String },
}

/// 回调服务器状态
struct CallbackState {
    /// 结果发送器
    result_tx: Mutex<Option<oneshot::Sender<CallbackResult>>>,
}

/// 回调服务器句柄
pub struct CallbackServerHandle {
    /// 服务器端口
    port: u16,
    /// 关闭信号发送器
    shutdown_tx: Option<watch::Sender<bool>>,
    /// 结果接收器
    result_rx: Option<oneshot::Receiver<CallbackResult>>,
}

impl CallbackServerHandle {
    /// 获取端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 等待回调结果
    ///
    /// # Arguments
    /// * `timeout` - 超时时间
    ///
    /// # Returns
    /// 回调结果，如果超时返回 None
    pub async fn wait_for_callback(
        mut self,
        timeout: std::time::Duration,
    ) -> Option<CallbackResult> {
        let result_rx = self.result_rx.take()?;

        let result = tokio::time::timeout(timeout, result_rx).await.ok()?.ok();

        // 关闭服务器
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }

        result
    }

    /// 关闭服务器
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
    }
}

impl Drop for CallbackServerHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
    }
}

/// 回调服务器
pub struct CallbackServer;

impl CallbackServer {
    /// 启动回调服务器
    ///
    /// # Arguments
    /// * `port` - 端口号，0 表示自动分配
    ///
    /// # Returns
    /// 服务器句柄
    pub async fn start(port: u16) -> Result<CallbackServerHandle, String> {
        // 创建结果通道
        let (result_tx, result_rx) = oneshot::channel();

        // 创建状态
        let state = Arc::new(CallbackState {
            result_tx: Mutex::new(Some(result_tx)),
        });

        // 创建路由
        let app = Router::new()
            .route("/oauth/callback", get(callback_handler))
            .with_state(state);

        // 绑定端口
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind port: {}", e))?;

        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local address: {}", e))?
            .port();

        // 创建关闭信号
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        // 启动服务器
        tokio::spawn(async move {
            let graceful = axum::serve(listener, app).with_graceful_shutdown(async move {
                loop {
                    shutdown_rx.changed().await.ok();
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            });

            let _ = graceful.await;
        });

        Ok(CallbackServerHandle {
            port: actual_port,
            shutdown_tx: Some(shutdown_tx),
            result_rx: Some(result_rx),
        })
    }
}

/// 回调处理器
async fn callback_handler(
    axum::extract::State(state): axum::extract::State<Arc<CallbackState>>,
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    let result = if let Some(error) = params.error {
        // 授权被拒绝
        CallbackResult::Denied {
            error,
            description: params.error_description.unwrap_or_default(),
        }
    } else if let (Some(code), Some(state_param)) = (params.code, params.state) {
        // 授权成功
        CallbackResult::Success {
            code,
            state: state_param,
        }
    } else {
        // 缺少必要参数
        CallbackResult::Denied {
            error: "invalid_request".to_string(),
            description: "Missing code or state parameter".to_string(),
        }
    };

    // 发送结果
    let mut tx = state.result_tx.lock().await;
    if let Some(sender) = tx.take() {
        let _ = sender.send(result.clone());
    }

    // 返回 HTML 页面
    match result {
        CallbackResult::Success { .. } => Html(SUCCESS_HTML.to_string()),
        CallbackResult::Denied { error, description } => {
            Html(ERROR_HTML.replace("{error}", &error).replace("{description}", &description))
        }
    }
}

/// 成功页面 HTML
const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authorization Successful - Mantra</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #09090b 0%, #18181b 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #fafafa;
        }
        .container {
            text-align: center;
            padding: 3rem;
            background: rgba(24, 24, 27, 0.8);
            border-radius: 1rem;
            border: 1px solid rgba(63, 63, 70, 0.5);
            max-width: 400px;
        }
        .icon {
            width: 80px;
            height: 80px;
            margin: 0 auto 1.5rem;
            background: linear-gradient(135deg, #10b981 0%, #059669 100%);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .icon svg {
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
        }
        h1 {
            font-size: 1.5rem;
            margin-bottom: 0.75rem;
            color: #10b981;
        }
        p {
            color: #a1a1aa;
            line-height: 1.6;
        }
        .hint {
            margin-top: 1.5rem;
            font-size: 0.875rem;
            color: #71717a;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
        </div>
        <h1>Authorization Successful</h1>
        <p>Your account has been connected successfully. You can close this window and return to Mantra.</p>
        <p class="hint">This window will close automatically...</p>
    </div>
    <script>
        setTimeout(() => window.close(), 3000);
    </script>
</body>
</html>"#;

/// 错误页面 HTML 模板
const ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authorization Failed - Mantra</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #09090b 0%, #18181b 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #fafafa;
        }
        .container {
            text-align: center;
            padding: 3rem;
            background: rgba(24, 24, 27, 0.8);
            border-radius: 1rem;
            border: 1px solid rgba(63, 63, 70, 0.5);
            max-width: 400px;
        }
        .icon {
            width: 80px;
            height: 80px;
            margin: 0 auto 1.5rem;
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .icon svg {
            width: 40px;
            height: 40px;
            stroke: white;
            stroke-width: 3;
            fill: none;
        }
        h1 {
            font-size: 1.5rem;
            margin-bottom: 0.75rem;
            color: #ef4444;
        }
        p {
            color: #a1a1aa;
            line-height: 1.6;
        }
        .error-code {
            margin-top: 1rem;
            padding: 0.75rem;
            background: rgba(239, 68, 68, 0.1);
            border-radius: 0.5rem;
            font-family: monospace;
            font-size: 0.875rem;
            color: #fca5a5;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">
            <svg viewBox="0 0 24 24">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
        </div>
        <h1>Authorization Failed</h1>
        <p>{description}</p>
        <div class="error-code">Error: {error}</div>
    </div>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_server_dynamic_port() {
        let handle = CallbackServer::start(0).await.unwrap();
        assert!(handle.port() > 0);
        handle.shutdown();
    }

    #[tokio::test]
    async fn test_start_server_specific_port() {
        // 使用一个不太可能被占用的端口
        let handle = CallbackServer::start(39777).await.unwrap();
        assert_eq!(handle.port(), 39777);
        handle.shutdown();
    }

    #[tokio::test]
    async fn test_callback_success() {
        let handle = CallbackServer::start(0).await.unwrap();
        let port = handle.port();

        // 模拟 OAuth 回调
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "http://localhost:{}/oauth/callback?code=test-code&state=test-state",
                port
            ))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        // 验证结果
        let result = handle
            .wait_for_callback(std::time::Duration::from_secs(5))
            .await;

        assert!(result.is_some());
        match result.unwrap() {
            CallbackResult::Success { code, state } => {
                assert_eq!(code, "test-code");
                assert_eq!(state, "test-state");
            }
            _ => panic!("Expected Success result"),
        }
    }

    #[tokio::test]
    async fn test_callback_error() {
        let handle = CallbackServer::start(0).await.unwrap();
        let port = handle.port();

        // 模拟 OAuth 错误回调
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "http://localhost:{}/oauth/callback?error=access_denied&error_description=User%20denied%20access",
                port
            ))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        // 验证结果
        let result = handle
            .wait_for_callback(std::time::Duration::from_secs(5))
            .await;

        assert!(result.is_some());
        match result.unwrap() {
            CallbackResult::Denied { error, description } => {
                assert_eq!(error, "access_denied");
                assert_eq!(description, "User denied access");
            }
            _ => panic!("Expected Denied result"),
        }
    }

    #[tokio::test]
    async fn test_callback_missing_params() {
        let handle = CallbackServer::start(0).await.unwrap();
        let port = handle.port();

        // 缺少参数的回调
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://localhost:{}/oauth/callback", port))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        // 验证结果
        let result = handle
            .wait_for_callback(std::time::Duration::from_secs(5))
            .await;

        assert!(result.is_some());
        match result.unwrap() {
            CallbackResult::Denied { error, .. } => {
                assert_eq!(error, "invalid_request");
            }
            _ => panic!("Expected Denied result"),
        }
    }

    #[tokio::test]
    async fn test_callback_timeout() {
        let handle = CallbackServer::start(0).await.unwrap();

        // 不发送回调，等待超时
        let result = handle
            .wait_for_callback(std::time::Duration::from_millis(100))
            .await;

        assert!(result.is_none());
    }
}
