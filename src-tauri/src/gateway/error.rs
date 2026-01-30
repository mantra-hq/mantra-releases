//! Gateway 模块错误类型定义
//!
//! Story 11.1: SSE Server 核心

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Gateway 模块错误枚举
#[derive(Error, Debug)]
pub enum GatewayError {
    /// 认证失败
    #[error("认证失败: {0}")]
    Unauthorized(String),

    /// Token 无效
    #[error("无效的 Token")]
    InvalidToken,

    /// Token 缺失
    #[error("缺少认证 Token")]
    MissingToken,

    /// 会话未找到
    #[error("会话不存在: {0}")]
    SessionNotFound(String),

    /// 服务器启动失败
    #[error("服务器启动失败: {0}")]
    StartupError(String),

    /// 端口被占用
    #[error("端口 {0} 已被占用")]
    PortInUse(u16),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// JSON-RPC 解析错误
    #[error("JSON-RPC 解析错误: {0}")]
    JsonRpcError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// JSON-RPC 错误响应
#[derive(Debug, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: &'static str,
    pub id: Option<serde_json::Value>,
    pub error: JsonRpcErrorObject,
}

/// JSON-RPC 错误对象
#[derive(Debug, Serialize)]
pub struct JsonRpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl GatewayError {
    /// 获取对应的 JSON-RPC 错误码
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            Self::InvalidToken | Self::MissingToken | Self::Unauthorized(_) => -32001, // 认证错误
            Self::SessionNotFound(_) => -32002,                                         // 会话错误
            Self::JsonRpcError(_) => -32700,                                            // 解析错误
            Self::ConfigError(_) => -32603,                                             // 内部错误
            Self::Internal(_) | Self::DatabaseError(_) => -32603,                       // 内部错误
            Self::StartupError(_) | Self::PortInUse(_) => -32000,                       // 服务器错误
        }
    }

    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidToken | Self::MissingToken | Self::Unauthorized(_) => {
                StatusCode::UNAUTHORIZED
            }
            Self::SessionNotFound(_) => StatusCode::NOT_FOUND,
            Self::JsonRpcError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = JsonRpcErrorResponse {
            jsonrpc: "2.0",
            id: None,
            error: JsonRpcErrorObject {
                code: self.json_rpc_code(),
                message: self.to_string(),
                data: None,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<rusqlite::Error> for GatewayError {
    fn from(err: rusqlite::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for GatewayError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonRpcError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(GatewayError::InvalidToken.json_rpc_code(), -32001);
        assert_eq!(GatewayError::MissingToken.json_rpc_code(), -32001);
        assert_eq!(
            GatewayError::SessionNotFound("test".to_string()).json_rpc_code(),
            -32002
        );
        assert_eq!(
            GatewayError::JsonRpcError("test".to_string()).json_rpc_code(),
            -32700
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(GatewayError::InvalidToken.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            GatewayError::SessionNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            GatewayError::Internal("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_display() {
        let err = GatewayError::Unauthorized("token expired".to_string());
        assert_eq!(err.to_string(), "认证失败: token expired");

        let err = GatewayError::PortInUse(8080);
        assert_eq!(err.to_string(), "端口 8080 已被占用");
    }
}
