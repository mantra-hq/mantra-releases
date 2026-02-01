//! Origin 验证中间件
//!
//! Story 11.14: MCP Streamable HTTP 规范合规 - Task 1
//!
//! 实现 MCP Streamable HTTP 规范要求的 Origin Header 验证。
//! 这是一个 MUST 要求，用于防止 DNS rebinding 攻击。
//!
//! ## 规范要求
//! - 服务端必须验证 Origin Header 以防止 DNS rebinding 攻击
//! - Origin 存在但无效时，必须返回 HTTP 403 Forbidden
//! - 如果请求不包含 Origin Header，允许请求通过（兼容命令行工具如 curl）

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// 允许的 Origin 列表（精确匹配）
const ALLOWED_ORIGINS: &[&str] = &["tauri://localhost"];

/// 允许的 Origin 前缀（支持任意端口）
const ALLOWED_ORIGIN_PREFIXES: &[&str] = &["http://localhost", "http://127.0.0.1"];

/// Origin 验证错误响应
#[derive(Debug, Serialize)]
pub struct OriginErrorResponse {
    pub jsonrpc: &'static str,
    pub id: Option<()>,
    pub error: OriginError,
}

/// Origin 验证错误对象
#[derive(Debug, Serialize)]
pub struct OriginError {
    pub code: i32,
    pub message: String,
}

impl Default for OriginErrorResponse {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0",
            id: None,
            error: OriginError {
                code: -32001,
                message: "Forbidden: Invalid origin".to_string(),
            },
        }
    }
}

/// Origin 验证配置
#[derive(Debug, Clone)]
pub struct OriginValidatorConfig {
    /// 额外允许的精确匹配 Origin 列表
    pub allowed_origins: Vec<String>,
    /// 额外允许的 Origin 前缀列表
    pub allowed_prefixes: Vec<String>,
    /// 是否允许空 Origin（兼容命令行工具）
    pub allow_missing_origin: bool,
}

impl Default for OriginValidatorConfig {
    fn default() -> Self {
        Self {
            allowed_origins: Vec::new(),
            allowed_prefixes: Vec::new(),
            allow_missing_origin: true,
        }
    }
}

/// 验证 Origin Header
///
/// 根据 MCP Streamable HTTP 规范：
/// - 服务端必须验证 Origin Header 以防止 DNS rebinding 攻击
/// - Origin 存在但无效时，返回 false
/// - 如果请求不包含 Origin Header，返回 true（兼容命令行工具）
///
/// # Arguments
/// * `origin` - 请求的 Origin Header 值，None 表示没有 Origin
/// * `config` - 可选的自定义配置
///
/// # Returns
/// `true` 如果 Origin 有效或缺失，`false` 如果 Origin 无效
pub fn validate_origin(origin: Option<&str>, config: Option<&OriginValidatorConfig>) -> bool {
    match origin {
        // 没有 Origin Header - 允许（兼容 curl 等命令行工具）
        None => config.map(|c| c.allow_missing_origin).unwrap_or(true),

        // 空 Origin - 某些本地工具可能发送空 Origin
        Some("") => config.map(|c| c.allow_missing_origin).unwrap_or(true),

        // 有 Origin Header - 验证是否在白名单中
        Some(origin) => {
            // 检查内置精确匹配列表
            if ALLOWED_ORIGINS.contains(&origin) {
                return true;
            }

            // 检查内置前缀匹配列表（支持任意端口）
            for prefix in ALLOWED_ORIGIN_PREFIXES {
                if origin.starts_with(prefix) {
                    // 验证格式：prefix:port 或 prefix（无端口）
                    let remaining = &origin[prefix.len()..];
                    if remaining.is_empty() || remaining.starts_with(':') {
                        return true;
                    }
                }
            }

            // 检查自定义配置
            if let Some(cfg) = config {
                // 检查自定义精确匹配
                if cfg.allowed_origins.iter().any(|o| o == origin) {
                    return true;
                }

                // 检查自定义前缀匹配
                for prefix in &cfg.allowed_prefixes {
                    if origin.starts_with(prefix) {
                        return true;
                    }
                }
            }

            // 不在白名单中
            false
        }
    }
}

/// Origin 验证中间件
///
/// MCP Streamable HTTP 规范 MUST 要求：
/// - 验证 Origin Header 以防止 DNS rebinding 攻击
/// - Origin 存在但无效时，返回 HTTP 403 Forbidden
/// - 响应体为空 id 的 JSON-RPC 错误
///
/// 默认白名单：
/// - `tauri://localhost` (Tauri 应用)
/// - `http://localhost:*` (本地开发)
/// - `http://127.0.0.1:*` (本地开发)
/// - 空 Origin（某些本地工具不发送 Origin）
pub async fn origin_middleware(request: Request, next: Next) -> Response {
    origin_middleware_with_config(request, next, None).await
}

/// 带配置的 Origin 验证中间件
///
/// 允许自定义 Origin 白名单
pub async fn origin_middleware_with_config(
    request: Request,
    next: Next,
    config: Option<&OriginValidatorConfig>,
) -> Response {
    // 提取 Origin Header
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());

    // 验证 Origin
    if !validate_origin(origin, config) {
        // 返回 HTTP 403 Forbidden + JSON-RPC 错误
        return forbidden_response();
    }

    // Origin 有效或缺失，继续处理请求
    next.run(request).await
}

/// 生成 403 Forbidden 响应
///
/// 返回 JSON-RPC 格式的错误响应，符合 MCP 规范
fn forbidden_response() -> Response {
    let response = OriginErrorResponse::default();
    (StatusCode::FORBIDDEN, Json(response)).into_response()
}

/// 从请求中提取 Origin Header（用于日志或统计）
#[allow(dead_code)]
pub fn extract_origin(request: &Request<Body>) -> Option<String> {
    request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== validate_origin 函数测试 =====

    #[test]
    fn test_validate_origin_tauri_localhost() {
        // Tauri 应用的 Origin 应该通过
        assert!(validate_origin(Some("tauri://localhost"), None));
    }

    #[test]
    fn test_validate_origin_http_localhost_no_port() {
        // localhost 无端口
        assert!(validate_origin(Some("http://localhost"), None));
    }

    #[test]
    fn test_validate_origin_http_localhost_with_port() {
        // localhost 带端口
        assert!(validate_origin(Some("http://localhost:3000"), None));
        assert!(validate_origin(Some("http://localhost:8080"), None));
        assert!(validate_origin(Some("http://localhost:39600"), None));
    }

    #[test]
    fn test_validate_origin_http_127_no_port() {
        // 127.0.0.1 无端口
        assert!(validate_origin(Some("http://127.0.0.1"), None));
    }

    #[test]
    fn test_validate_origin_http_127_with_port() {
        // 127.0.0.1 带端口
        assert!(validate_origin(Some("http://127.0.0.1:3000"), None));
        assert!(validate_origin(Some("http://127.0.0.1:8080"), None));
        assert!(validate_origin(Some("http://127.0.0.1:39600"), None));
    }

    #[test]
    fn test_validate_origin_missing() {
        // 没有 Origin Header - 允许（兼容 curl）
        assert!(validate_origin(None, None));
    }

    #[test]
    fn test_validate_origin_empty() {
        // 空 Origin - 允许
        assert!(validate_origin(Some(""), None));
    }

    #[test]
    fn test_validate_origin_invalid_external() {
        // 外部 Origin - 拒绝
        assert!(!validate_origin(Some("https://example.com"), None));
        assert!(!validate_origin(Some("http://evil.com"), None));
        assert!(!validate_origin(Some("https://attacker.com:8080"), None));
    }

    #[test]
    fn test_validate_origin_invalid_https_localhost() {
        // HTTPS localhost - 不在默认白名单中
        assert!(!validate_origin(Some("https://localhost"), None));
        assert!(!validate_origin(Some("https://localhost:3000"), None));
    }

    #[test]
    fn test_validate_origin_invalid_other_local() {
        // 其他本地 IP - 不在默认白名单中
        assert!(!validate_origin(Some("http://192.168.1.1"), None));
        assert!(!validate_origin(Some("http://10.0.0.1"), None));
    }

    #[test]
    fn test_validate_origin_invalid_prefix_attack() {
        // 前缀攻击 - 确保不会被绕过
        assert!(!validate_origin(Some("http://localhost.evil.com"), None));
        assert!(!validate_origin(Some("http://127.0.0.1.evil.com"), None));
    }

    // ===== 自定义配置测试 =====

    #[test]
    fn test_validate_origin_custom_exact_match() {
        let config = OriginValidatorConfig {
            allowed_origins: vec!["https://custom.local".to_string()],
            allowed_prefixes: vec![],
            allow_missing_origin: true,
        };

        assert!(validate_origin(Some("https://custom.local"), Some(&config)));
        assert!(!validate_origin(Some("https://other.local"), Some(&config)));
    }

    #[test]
    fn test_validate_origin_custom_prefix() {
        let config = OriginValidatorConfig {
            allowed_origins: vec![],
            allowed_prefixes: vec!["https://192.168.".to_string()],
            allow_missing_origin: true,
        };

        assert!(validate_origin(Some("https://192.168.1.1"), Some(&config)));
        assert!(validate_origin(
            Some("https://192.168.0.100:8080"),
            Some(&config)
        ));
        assert!(!validate_origin(Some("https://10.0.0.1"), Some(&config)));
    }

    #[test]
    fn test_validate_origin_disallow_missing() {
        let config = OriginValidatorConfig {
            allowed_origins: vec![],
            allowed_prefixes: vec![],
            allow_missing_origin: false,
        };

        // 禁止空 Origin
        assert!(!validate_origin(None, Some(&config)));
        assert!(!validate_origin(Some(""), Some(&config)));

        // 但有效 Origin 仍然通过
        assert!(validate_origin(Some("http://localhost:3000"), Some(&config)));
    }

    // ===== OriginErrorResponse 测试 =====

    #[test]
    fn test_origin_error_response_default() {
        let response = OriginErrorResponse::default();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none());
        assert_eq!(response.error.code, -32001);
        assert!(response.error.message.contains("Forbidden"));
    }

    #[test]
    fn test_origin_error_response_serialization() {
        let response = OriginErrorResponse::default();
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":null"));
        assert!(json.contains("-32001"));
        assert!(json.contains("Forbidden"));
    }

    // ===== OriginValidatorConfig 测试 =====

    #[test]
    fn test_origin_validator_config_default() {
        let config = OriginValidatorConfig::default();
        assert!(config.allowed_origins.is_empty());
        assert!(config.allowed_prefixes.is_empty());
        assert!(config.allow_missing_origin);
    }
}
