//! HTTP 请求转发器
//!
//! Story 11.12: Remote MCP OAuth Support - Task 3
//!
//! 负责将请求转发到远程 HTTP MCP 服务，并自动注入 OAuth Token。

use std::collections::HashMap;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::services::oauth::{OAuthConfig, OAuthError, OAuthManager};

/// HTTP 转发错误
#[derive(Debug, Error)]
pub enum ForwarderError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("OAuth error: {0}")]
    OAuthError(#[from] OAuthError),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Service not configured: {0}")]
    ServiceNotConfigured(String),

    #[error("Token injection failed: {0}")]
    TokenInjectionFailed(String),
}

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthType {
    /// 无认证
    None,
    /// OAuth 2.0
    OAuth {
        /// OAuth 配置
        config: OAuthConfig,
    },
    /// Bearer Token (静态)
    BearerToken {
        /// Token 值或环境变量引用
        token: String,
    },
    /// API Key
    ApiKey {
        /// Header 名称
        header_name: String,
        /// Key 值或环境变量引用
        key: String,
    },
    /// Basic Auth
    BasicAuth {
        /// 用户名
        username: String,
        /// 密码
        password: String,
    },
}

/// 远程服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteServiceConfig {
    /// 服务 ID
    pub service_id: String,
    /// 服务名称
    pub name: String,
    /// 基础 URL
    pub base_url: String,
    /// 认证类型
    pub auth: AuthType,
    /// 自定义 Headers
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,
}

/// HTTP 请求转发器
pub struct HttpForwarder {
    /// HTTP 客户端
    http_client: reqwest::Client,
    /// OAuth Manager
    oauth_manager: Arc<OAuthManager>,
}

impl HttpForwarder {
    /// 创建新的转发器
    pub fn new(oauth_manager: Arc<OAuthManager>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            oauth_manager,
        }
    }

    /// 转发 JSON-RPC 请求到远程服务
    ///
    /// # Arguments
    /// * `config` - 远程服务配置
    /// * `request` - JSON-RPC 请求
    ///
    /// # Returns
    /// JSON-RPC 响应
    pub async fn forward_request(
        &self,
        config: &RemoteServiceConfig,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, ForwarderError> {
        // 构建请求 URL
        let url = format!("{}/message", config.base_url.trim_end_matches('/'));

        // 构建 Headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // 注入认证 Header
        self.inject_auth_header(&mut headers, config).await?;

        // 添加自定义 Headers
        for (key, value) in &config.custom_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::try_from(key.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                headers.insert(name, val);
            }
        }

        // 发送请求
        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(request)
            .send()
            .await
            .map_err(|e| ForwarderError::NetworkError(e.to_string()))?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ForwarderError::InvalidResponse(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        // 解析响应
        let json_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ForwarderError::InvalidResponse(e.to_string()))?;

        Ok(json_response)
    }

    /// 注入认证 Header
    async fn inject_auth_header(
        &self,
        headers: &mut HeaderMap,
        config: &RemoteServiceConfig,
    ) -> Result<(), ForwarderError> {
        match &config.auth {
            AuthType::None => {
                // 无需认证
            }
            AuthType::OAuth { config: oauth_config } => {
                // 获取有效的 OAuth Token
                let token = self
                    .oauth_manager
                    .get_valid_token(oauth_config, &config.service_id)
                    .await?;

                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::try_from(format!("Bearer {}", token))
                        .map_err(|_| ForwarderError::TokenInjectionFailed("Invalid token".to_string()))?,
                );
            }
            AuthType::BearerToken { token } => {
                // 解析环境变量引用
                let resolved_token = self.resolve_env_var(token);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::try_from(format!("Bearer {}", resolved_token))
                        .map_err(|_| ForwarderError::TokenInjectionFailed("Invalid token".to_string()))?,
                );
            }
            AuthType::ApiKey { header_name, key } => {
                let resolved_key = self.resolve_env_var(key);
                if let (Ok(name), Ok(val)) = (
                    HeaderName::try_from(header_name.as_str()),
                    HeaderValue::try_from(resolved_key.as_str()),
                ) {
                    headers.insert(name, val);
                }
            }
            AuthType::BasicAuth { username, password } => {
                let resolved_username = self.resolve_env_var(username);
                let resolved_password = self.resolve_env_var(password);
                let credentials = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{}:{}", resolved_username, resolved_password),
                );
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::try_from(format!("Basic {}", credentials))
                        .map_err(|_| ForwarderError::TokenInjectionFailed("Invalid credentials".to_string()))?,
                );
            }
        }

        Ok(())
    }

    /// 解析环境变量引用
    ///
    /// 支持格式: `${env:VAR_NAME}`
    fn resolve_env_var(&self, value: &str) -> String {
        if value.starts_with("${env:") && value.ends_with('}') {
            let var_name = &value[6..value.len() - 1];
            std::env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    /// 检查服务是否需要 OAuth 认证
    pub fn requires_oauth(config: &RemoteServiceConfig) -> bool {
        matches!(config.auth, AuthType::OAuth { .. })
    }

    /// 获取服务的 OAuth 配置
    pub fn get_oauth_config(config: &RemoteServiceConfig) -> Option<&OAuthConfig> {
        match &config.auth {
            AuthType::OAuth { config } => Some(config),
            _ => None,
        }
    }
}

/// 带自动重试的请求转发
pub struct RetryingForwarder {
    forwarder: HttpForwarder,
    max_retries: u32,
}

impl RetryingForwarder {
    /// 创建带重试的转发器
    pub fn new(oauth_manager: Arc<OAuthManager>, max_retries: u32) -> Self {
        Self {
            forwarder: HttpForwarder::new(oauth_manager),
            max_retries,
        }
    }

    /// 转发请求，自动处理 Token 刷新
    ///
    /// 如果收到 401 响应且服务使用 OAuth，自动刷新 Token 并重试
    pub async fn forward_with_retry(
        &self,
        config: &RemoteServiceConfig,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, ForwarderError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.forwarder.forward_request(config, request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // 检查是否是认证错误
                    if let ForwarderError::InvalidResponse(ref msg) = e {
                        if msg.contains("401") && attempt < self.max_retries {
                            // 尝试刷新 Token
                            if let AuthType::OAuth { config: oauth_config } = &config.auth {
                                if let Err(refresh_err) = self
                                    .forwarder
                                    .oauth_manager
                                    .refresh_token(oauth_config, &config.service_id)
                                    .await
                                {
                                    last_error = Some(ForwarderError::OAuthError(refresh_err));
                                    continue;
                                }
                                // Token 刷新成功，重试请求
                                continue;
                            }
                        }
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ForwarderError::NetworkError("Max retries exceeded".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::oauth::token_store::InMemoryTokenStore;

    fn create_test_forwarder() -> HttpForwarder {
        let token_store = Arc::new(InMemoryTokenStore::new());
        let oauth_manager = Arc::new(OAuthManager::new(token_store));
        HttpForwarder::new(oauth_manager)
    }

    #[test]
    fn test_resolve_env_var_plain() {
        let forwarder = create_test_forwarder();
        let result = forwarder.resolve_env_var("plain-value");
        assert_eq!(result, "plain-value");
    }

    #[test]
    fn test_resolve_env_var_with_env() {
        std::env::set_var("TEST_VAR_123", "secret-value");
        let forwarder = create_test_forwarder();
        let result = forwarder.resolve_env_var("${env:TEST_VAR_123}");
        assert_eq!(result, "secret-value");
        std::env::remove_var("TEST_VAR_123");
    }

    #[test]
    fn test_resolve_env_var_missing() {
        let forwarder = create_test_forwarder();
        let result = forwarder.resolve_env_var("${env:NONEXISTENT_VAR_XYZ}");
        // 如果环境变量不存在，返回原始值
        assert_eq!(result, "${env:NONEXISTENT_VAR_XYZ}");
    }

    #[test]
    fn test_requires_oauth() {
        let oauth_config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::OAuth {
                config: OAuthConfig {
                    service_id: "test".to_string(),
                    client_id: "client".to_string(),
                    client_secret: None,
                    authorization_url: "https://example.com/auth".to_string(),
                    token_url: "https://example.com/token".to_string(),
                    revoke_url: None,
                    scopes: vec![],
                    callback_port: 0,
                },
            },
            custom_headers: HashMap::new(),
        };

        assert!(HttpForwarder::requires_oauth(&oauth_config));

        let bearer_config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::BearerToken {
                token: "token".to_string(),
            },
            custom_headers: HashMap::new(),
        };

        assert!(!HttpForwarder::requires_oauth(&bearer_config));
    }

    #[test]
    fn test_get_oauth_config() {
        let oauth_config = OAuthConfig {
            service_id: "test".to_string(),
            client_id: "client".to_string(),
            client_secret: None,
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            revoke_url: None,
            scopes: vec!["read".to_string()],
            callback_port: 0,
        };

        let config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::OAuth {
                config: oauth_config.clone(),
            },
            custom_headers: HashMap::new(),
        };

        let result = HttpForwarder::get_oauth_config(&config);
        assert!(result.is_some());
        assert_eq!(result.unwrap().client_id, "client");
    }

    #[tokio::test]
    async fn test_inject_auth_header_none() {
        let forwarder = create_test_forwarder();
        let mut headers = HeaderMap::new();

        let config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::None,
            custom_headers: HashMap::new(),
        };

        forwarder.inject_auth_header(&mut headers, &config).await.unwrap();
        assert!(!headers.contains_key(AUTHORIZATION));
    }

    #[tokio::test]
    async fn test_inject_auth_header_bearer() {
        let forwarder = create_test_forwarder();
        let mut headers = HeaderMap::new();

        let config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::BearerToken {
                token: "my-secret-token".to_string(),
            },
            custom_headers: HashMap::new(),
        };

        forwarder.inject_auth_header(&mut headers, &config).await.unwrap();
        assert!(headers.contains_key(AUTHORIZATION));
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer my-secret-token"
        );
    }

    #[tokio::test]
    async fn test_inject_auth_header_api_key() {
        let forwarder = create_test_forwarder();
        let mut headers = HeaderMap::new();

        let config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::ApiKey {
                header_name: "X-API-Key".to_string(),
                key: "my-api-key".to_string(),
            },
            custom_headers: HashMap::new(),
        };

        forwarder.inject_auth_header(&mut headers, &config).await.unwrap();
        assert!(headers.contains_key("x-api-key"));
        assert_eq!(
            headers.get("x-api-key").unwrap().to_str().unwrap(),
            "my-api-key"
        );
    }

    #[tokio::test]
    async fn test_inject_auth_header_basic() {
        let forwarder = create_test_forwarder();
        let mut headers = HeaderMap::new();

        let config = RemoteServiceConfig {
            service_id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com".to_string(),
            auth: AuthType::BasicAuth {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            custom_headers: HashMap::new(),
        };

        forwarder.inject_auth_header(&mut headers, &config).await.unwrap();
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_value = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert!(auth_value.starts_with("Basic "));

        // 验证 base64 编码
        let encoded = &auth_value[6..];
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "user:pass");
    }
}
