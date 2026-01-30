//! Token 认证中间件
//!
//! Story 11.1: SSE Server 核心 - Task 3
//!
//! 实现 Axum 中间件，支持 URL Query 参数认证 (`?token=xxx`)

use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{GatewayError, JsonRpcErrorObject, JsonRpcErrorResponse};
use super::state::GatewayState;

/// Token 查询参数
#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

/// 认证层结构体
#[derive(Clone)]
pub struct AuthLayer {
    state: Arc<RwLock<GatewayState>>,
}

impl AuthLayer {
    /// 创建新的认证层
    pub fn new(state: Arc<RwLock<GatewayState>>) -> Self {
        Self { state }
    }

    /// 验证 Token
    pub async fn validate(&self, token: &str) -> bool {
        let state = self.state.read().await;
        state.validate_token(token)
    }
}

/// 认证中间件函数
///
/// 检查请求中的 `token` 查询参数是否有效
pub async fn auth_middleware(
    State(state): State<Arc<RwLock<GatewayState>>>,
    Query(query): Query<TokenQuery>,
    request: Request,
    next: Next,
) -> Response {
    // 检查 Token 是否存在
    let token = match query.token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return unauthorized_response(GatewayError::MissingToken);
        }
    };

    // 验证 Token
    let is_valid = {
        let state_guard = state.read().await;
        state_guard.validate_token(&token)
    };

    if !is_valid {
        return unauthorized_response(GatewayError::InvalidToken);
    }

    // Token 有效，继续处理请求
    next.run(request).await
}

/// 生成 401 Unauthorized 响应
fn unauthorized_response(error: GatewayError) -> Response {
    let response = JsonRpcErrorResponse {
        jsonrpc: "2.0",
        id: None,
        error: JsonRpcErrorObject {
            code: error.json_rpc_code(),
            message: error.to_string(),
            data: None,
        },
    };

    (StatusCode::UNAUTHORIZED, Json(response)).into_response()
}

/// 提取 Token 从请求中（用于日志或统计）
#[allow(dead_code)]
pub fn extract_token_from_query(uri: &str) -> Option<String> {
    let parts: Vec<&str> = uri.split('?').collect();
    if parts.len() < 2 {
        return None;
    }

    for param in parts[1].split('&') {
        let kv: Vec<&str> = param.split('=').collect();
        if kv.len() == 2 && kv[0] == "token" {
            let token = kv[1].to_string();
            // 返回 None 如果 token 为空
            if token.is_empty() {
                return None;
            }
            return Some(token);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::state::GatewayConfig;

    #[tokio::test]
    async fn test_auth_layer_validate() {
        let config = GatewayConfig {
            port: 8080,
            auth_token: "valid-token".to_string(),
            enabled: true,
            auto_start: false,
        };
        let state = Arc::new(RwLock::new(GatewayState::new(config)));
        let auth = AuthLayer::new(state);

        assert!(auth.validate("valid-token").await);
        assert!(!auth.validate("invalid-token").await);
    }

    #[test]
    fn test_extract_token_from_query() {
        assert_eq!(
            extract_token_from_query("/sse?token=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_token_from_query("/sse?session_id=xxx&token=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_token_from_query("/sse"), None);
        assert_eq!(extract_token_from_query("/sse?other=value"), None);
    }
}
