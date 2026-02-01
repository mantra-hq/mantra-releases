//! MCP Streamable HTTP 传输客户端
//!
//! 实现 MCP 2025-03-26 规范的 Streamable HTTP 传输协议。
//! 用于连接 HTTP 类型的远程 MCP 服务（如 DeepWiki）。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP 传输错误
#[derive(Debug, thiserror::Error)]
pub enum HttpTransportError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("JSON-RPC error: code={code}, message={message}")]
    JsonRpcError { code: i64, message: String },
}

impl From<reqwest::Error> for HttpTransportError {
    fn from(e: reqwest::Error) -> Self {
        HttpTransportError::ConnectionError(e.to_string())
    }
}

/// MCP Streamable HTTP 客户端
///
/// 按照 MCP 2025-03-26 规范实现 Streamable HTTP 传输：
/// - POST JSON-RPC 请求到 MCP 端点
/// - 支持 application/json 响应
/// - Session ID 管理
pub struct McpHttpClient {
    /// HTTP 客户端
    client: reqwest::Client,
    /// MCP 端点 URL
    endpoint: String,
    /// 自定义请求头
    custom_headers: HashMap<String, String>,
    /// Session ID（服务器在 initialize 响应中设置）
    session_id: Arc<RwLock<Option<String>>>,
}

impl McpHttpClient {
    /// 创建新的 HTTP 传输客户端
    pub fn new(
        endpoint: String,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            endpoint,
            custom_headers: custom_headers.unwrap_or_default(),
            session_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 发送 JSON-RPC 请求并获取响应
    ///
    /// 按照 MCP 规范：
    /// 1. POST JSON-RPC 消息到 MCP 端点
    /// 2. Accept: application/json, text/event-stream
    /// 3. 如果有 Session ID，附加 Mcp-Session-Id 头
    pub async fn send_request(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, HttpTransportError> {
        let mut req_builder = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&request);

        // 添加自定义请求头
        for (key, value) in &self.custom_headers {
            req_builder = req_builder.header(key, value);
        }

        // 添加 Session ID（如有）
        {
            let session_id = self.session_id.read().await;
            if let Some(ref id) = *session_id {
                req_builder = req_builder.header("Mcp-Session-Id", id);
            }
        }

        let response = req_builder.send().await?;

        // 检查 HTTP 状态码
        let status = response.status();
        if !status.is_success() {
            // 特殊处理 404：Session 过期
            if status.as_u16() == 404 {
                let mut session_id = self.session_id.write().await;
                *session_id = None;
            }
            let body = response.text().await.unwrap_or_default();
            return Err(HttpTransportError::RequestFailed(format!(
                "HTTP {} - {}",
                status, body
            )));
        }

        // 保存 Session ID（如果响应中包含）
        if let Some(session_id_header) = response.headers().get("mcp-session-id") {
            if let Ok(id) = session_id_header.to_str() {
                let mut session_id = self.session_id.write().await;
                *session_id = Some(id.to_string());
            }
        }

        // 检查 Content-Type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.contains("text/event-stream") {
            // SSE 流式响应：读取完整内容并提取 JSON-RPC 响应
            let body = response.text().await?;
            self.parse_sse_response(&body)
        } else {
            // JSON 响应：直接解析
            let body: serde_json::Value = response.json().await?;
            Ok(body)
        }
    }

    /// 从 SSE 响应中提取 JSON-RPC 响应
    fn parse_sse_response(
        &self,
        body: &str,
    ) -> Result<serde_json::Value, HttpTransportError> {
        // SSE 格式：每个事件以 "data: " 开头，事件之间用空行分隔
        for line in body.lines() {
            let line = line.trim();
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    // 找到 JSON-RPC 响应（有 "result" 或 "error" 字段）
                    if json.get("result").is_some() || json.get("error").is_some() {
                        return Ok(json);
                    }
                }
            }
        }

        Err(HttpTransportError::InvalidResponse(
            "No JSON-RPC response found in SSE stream".to_string(),
        ))
    }

    /// 发送 initialize 请求
    pub async fn initialize(&self) -> Result<serde_json::Value, HttpTransportError> {
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "mantra-inspector",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        self.send_request(init_request).await
    }

    /// 发送 notifications/initialized 通知
    pub async fn send_initialized(&self) -> Result<(), HttpTransportError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let mut req_builder = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&notification);

        // 添加自定义请求头
        for (key, value) in &self.custom_headers {
            req_builder = req_builder.header(key, value);
        }

        // 添加 Session ID（如有）
        {
            let session_id = self.session_id.read().await;
            if let Some(ref id) = *session_id {
                req_builder = req_builder.header("Mcp-Session-Id", id);
            }
        }

        let response = req_builder.send().await?;
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(HttpTransportError::RequestFailed(format!(
                "Initialized notification failed: {}",
                body
            )));
        }

        Ok(())
    }

    /// 获取工具列表
    pub async fn list_tools(&self) -> Result<serde_json::Value, HttpTransportError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        self.send_request(request).await
    }

    /// 获取资源列表
    pub async fn list_resources(&self) -> Result<serde_json::Value, HttpTransportError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list",
            "params": {}
        });

        self.send_request(request).await
    }

    /// 调用工具
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, HttpTransportError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        self.send_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_http_client() {
        let client = McpHttpClient::new(
            "https://mcp.deepwiki.com/mcp".to_string(),
            None,
        );
        assert_eq!(client.endpoint, "https://mcp.deepwiki.com/mcp");
        assert!(client.custom_headers.is_empty());
    }

    #[test]
    fn test_create_http_client_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test".to_string());

        let client = McpHttpClient::new(
            "https://example.com/mcp".to_string(),
            Some(headers),
        );
        assert_eq!(client.custom_headers.len(), 1);
        assert_eq!(
            client.custom_headers.get("Authorization"),
            Some(&"Bearer test".to_string())
        );
    }

    #[test]
    fn test_parse_sse_response() {
        let client = McpHttpClient::new("https://example.com/mcp".to_string(), None);

        let sse_body = r#"data: {"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"test"}]}}

"#;
        let result = client.parse_sse_response(sse_body).unwrap();
        assert!(result.get("result").is_some());
    }

    #[test]
    fn test_parse_sse_response_no_result() {
        let client = McpHttpClient::new("https://example.com/mcp".to_string(), None);

        let sse_body = "data: {\"jsonrpc\":\"2.0\",\"method\":\"ping\"}\n\n";
        let result = client.parse_sse_response(sse_body);
        assert!(result.is_err());
    }
}
