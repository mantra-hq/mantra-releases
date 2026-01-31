//! Token 认证中间件
//!
//! Story 11.1: SSE Server 核心 - Task 3
//! Story 11.8: MCP Gateway Architecture Refactor - Task 8
//!
//! 实现 Axum 中间件，支持:
//! - Authorization Header 认证 (`Authorization: Bearer xxx`) - 推荐
//! - URL Query 参数认证 (`?token=xxx`) - 向后兼容

use axum::{
    extract::{Query, Request, State},
    http::{header, StatusCode},
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

/// 从 Authorization Header 提取 Bearer Token
///
/// 支持格式: `Authorization: Bearer <token>`
fn extract_bearer_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            let value = value.trim();
            if value.to_lowercase().starts_with("bearer ") {
                let token = value[7..].trim().to_string();
                if token.is_empty() {
                    None
                } else {
                    Some(token)
                }
            } else {
                None
            }
        })
}

/// 认证中间件函数
///
/// 检查请求中的 Token 是否有效
/// 优先级: Authorization Header > Query Parameter
pub async fn auth_middleware(
    State(state): State<Arc<RwLock<GatewayState>>>,
    Query(query): Query<TokenQuery>,
    request: Request,
    next: Next,
) -> Response {
    // 优先从 Authorization Header 提取 Token
    let token = extract_bearer_token(&request)
        // 回退到 Query 参数（向后兼容）
        .or_else(|| query.token.filter(|t| !t.is_empty()));

    // 检查 Token 是否存在
    let token = match token {
        Some(t) => t,
        None => {
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
    use axum::http::Request as HttpRequest;

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

    #[test]
    fn test_extract_bearer_token_valid() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "Bearer my-secret-token")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(
            extract_bearer_token(&request),
            Some("my-secret-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_case_insensitive() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "bearer my-token")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(
            extract_bearer_token(&request),
            Some("my-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_with_extra_spaces() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "  Bearer   my-token  ")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(
            extract_bearer_token(&request),
            Some("my-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let request = HttpRequest::builder()
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(extract_bearer_token(&request), None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "Basic dXNlcjpwYXNz")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(extract_bearer_token(&request), None);
    }

    #[test]
    fn test_extract_bearer_token_empty_token() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "Bearer ")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(extract_bearer_token(&request), None);
    }

    #[test]
    fn test_extract_bearer_token_only_bearer() {
        let request = HttpRequest::builder()
            .header(header::AUTHORIZATION, "Bearer")
            .body(())
            .unwrap();
        let request = Request::from_parts(request.into_parts().0, axum::body::Body::empty());

        assert_eq!(extract_bearer_token(&request), None);
    }
}
