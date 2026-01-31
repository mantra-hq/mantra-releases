//! OAuth 2.0 认证模块
//!
//! Story 11.12: Remote MCP OAuth Support
//!
//! 提供 OAuth 2.0 Authorization Code + PKCE 流程支持，
//! 用于远程 MCP 服务（如 Google Drive, Slack, GitHub）的认证。

pub mod callback_server;
mod discovery;
mod pkce;
pub mod token_store;

pub use callback_server::{CallbackResult, CallbackServer, CallbackServerHandle};
pub use discovery::{OAuthDiscovery, OAuthEndpoints};
pub use pkce::{PkceChallenge, PkceVerifier};
pub use token_store::{InMemoryTokenStore, OAuthToken, SecureTokenStore, TokenStoreError};

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

/// OAuth 错误类型
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("PKCE generation failed: {0}")]
    PkceError(String),

    #[error("Callback server error: {0}")]
    CallbackServerError(String),

    #[error("Token exchange failed: {0}")]
    TokenExchangeError(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshError(String),

    #[error("Token storage error: {0}")]
    StorageError(#[from] TokenStoreError),

    #[error("Discovery failed: {0}")]
    DiscoveryError(String),

    #[error("State mismatch: expected {expected}, got {actual}")]
    StateMismatch { expected: String, actual: String },

    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Token expired and refresh failed")]
    TokenExpiredNoRefresh,

    #[error("Service not found: {0}")]
    ServiceNotFound(String),
}

/// OAuth 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// 服务 ID
    pub service_id: String,
    /// Client ID
    pub client_id: String,
    /// Client Secret (可选，用于 confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Authorization URL
    pub authorization_url: String,
    /// Token URL
    pub token_url: String,
    /// Revoke URL (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoke_url: Option<String>,
    /// 请求的 scopes
    pub scopes: Vec<String>,
    /// 回调端口 (0 表示动态分配)
    #[serde(default)]
    pub callback_port: u16,
}

/// OAuth 流程状态
#[derive(Debug, Clone)]
pub struct OAuthFlowState {
    /// 服务 ID
    pub service_id: String,
    /// PKCE verifier
    pub pkce_verifier: PkceVerifier,
    /// State 参数 (防 CSRF)
    pub state: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 回调端口
    pub callback_port: u16,
}

/// OAuth 服务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OAuthStatus {
    /// 未连接
    Disconnected,
    /// 已连接
    Connected,
    /// Token 已过期
    Expired,
    /// 授权流程进行中
    Pending,
}

/// OAuth 服务状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthServiceStatus {
    /// 服务 ID
    pub service_id: String,
    /// 连接状态
    pub status: OAuthStatus,
    /// Token 过期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// 已授权的 scopes
    pub scopes: Vec<String>,
    /// 最后刷新时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refreshed: Option<DateTime<Utc>>,
}

/// OAuth Manager
///
/// 管理 OAuth 2.0 认证流程，包括：
/// - 启动授权流程 (Authorization Code + PKCE)
/// - 处理回调并交换 Token
/// - Token 存储和刷新
pub struct OAuthManager {
    /// Token 存储
    token_store: Arc<dyn SecureTokenStore>,
    /// 进行中的授权流程
    pending_flows: RwLock<HashMap<String, OAuthFlowState>>,
    /// HTTP 客户端
    http_client: reqwest::Client,
}

impl OAuthManager {
    /// 创建新的 OAuth Manager
    pub fn new(token_store: Arc<dyn SecureTokenStore>) -> Self {
        Self {
            token_store,
            pending_flows: RwLock::new(HashMap::new()),
            http_client: reqwest::Client::new(),
        }
    }

    /// 启动 OAuth 授权流程
    ///
    /// 1. 生成 PKCE challenge
    /// 2. 生成 state 参数
    /// 3. 启动回调服务器
    /// 4. 返回授权 URL
    ///
    /// # Arguments
    /// * `config` - OAuth 配置
    ///
    /// # Returns
    /// 授权 URL 和回调服务器句柄
    pub async fn start_flow(
        &self,
        config: &OAuthConfig,
    ) -> Result<(String, CallbackServerHandle), OAuthError> {
        // 生成 PKCE
        let (verifier, challenge) = PkceChallenge::generate()
            .map_err(|e| OAuthError::PkceError(e.to_string()))?;

        // 生成 state
        let state = Self::generate_state();

        // 确定回调端口
        let callback_port = if config.callback_port > 0 {
            config.callback_port
        } else {
            0 // 动态分配
        };

        // 启动回调服务器
        let callback_handle = CallbackServer::start(callback_port)
            .await
            .map_err(|e| OAuthError::CallbackServerError(e.to_string()))?;

        let actual_port = callback_handle.port();

        // 构建授权 URL
        let redirect_uri = format!("http://localhost:{}/oauth/callback", actual_port);
        let auth_url = self.build_authorization_url(config, &challenge, &state, &redirect_uri);

        // 保存流程状态
        let flow_state = OAuthFlowState {
            service_id: config.service_id.clone(),
            pkce_verifier: verifier,
            state: state.clone(),
            created_at: Utc::now(),
            callback_port: actual_port,
        };

        {
            let mut flows = self.pending_flows.write().await;
            flows.insert(state, flow_state);
        }

        Ok((auth_url, callback_handle))
    }

    /// 处理 OAuth 回调
    ///
    /// 1. 验证 state
    /// 2. 使用 authorization code 交换 token
    /// 3. 存储 token
    ///
    /// # Arguments
    /// * `config` - OAuth 配置
    /// * `code` - Authorization code
    /// * `state` - State 参数
    /// * `callback_port` - 回调端口
    pub async fn handle_callback(
        &self,
        config: &OAuthConfig,
        code: &str,
        state: &str,
        callback_port: u16,
    ) -> Result<OAuthToken, OAuthError> {
        // 验证 state 并获取流程状态
        let flow_state = {
            let mut flows = self.pending_flows.write().await;
            flows
                .remove(state)
                .ok_or_else(|| OAuthError::StateMismatch {
                    expected: "valid state".to_string(),
                    actual: state.to_string(),
                })?
        };

        // 验证 state 匹配
        if flow_state.state != state {
            return Err(OAuthError::StateMismatch {
                expected: flow_state.state,
                actual: state.to_string(),
            });
        }

        // 交换 token
        let redirect_uri = format!("http://localhost:{}/oauth/callback", callback_port);
        let token = self
            .exchange_code(config, code, &flow_state.pkce_verifier, &redirect_uri)
            .await?;

        // 存储 token
        self.token_store.store(&token).await?;

        Ok(token)
    }

    /// 使用 authorization code 交换 token
    async fn exchange_code(
        &self,
        config: &OAuthConfig,
        code: &str,
        verifier: &PkceVerifier,
        redirect_uri: &str,
    ) -> Result<OAuthToken, OAuthError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", redirect_uri);
        params.insert("client_id", &config.client_id);
        params.insert("code_verifier", verifier.as_str());

        let mut request = self.http_client.post(&config.token_url).form(&params);

        // 如果有 client_secret，添加到请求
        if let Some(ref secret) = config.client_secret {
            request = request.basic_auth(&config.client_id, Some(secret));
        }

        let response = request
            .send()
            .await
            .map_err(|e| OAuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenExchangeError(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| OAuthError::TokenExchangeError(e.to_string()))?;

        Ok(OAuthToken {
            service_id: config.service_id.clone(),
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            token_type: token_response.token_type.unwrap_or_else(|| "Bearer".to_string()),
            expires_at: token_response
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs as i64)),
            scopes: config.scopes.clone(),
            created_at: Utc::now(),
        })
    }

    /// 刷新 access token
    ///
    /// # Arguments
    /// * `config` - OAuth 配置
    /// * `service_id` - 服务 ID
    pub async fn refresh_token(
        &self,
        config: &OAuthConfig,
        service_id: &str,
    ) -> Result<OAuthToken, OAuthError> {
        let existing_token = self
            .token_store
            .get(service_id)
            .await?
            .ok_or_else(|| OAuthError::ServiceNotFound(service_id.to_string()))?;

        let refresh_token = existing_token
            .refresh_token
            .as_ref()
            .ok_or(OAuthError::TokenExpiredNoRefresh)?;

        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token.as_str());
        params.insert("client_id", config.client_id.as_str());

        let mut request = self.http_client.post(&config.token_url).form(&params);

        if let Some(ref secret) = config.client_secret {
            request = request.basic_auth(&config.client_id, Some(secret));
        }

        let response = request
            .send()
            .await
            .map_err(|e| OAuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenRefreshError(format!(
                "Token refresh failed: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| OAuthError::TokenRefreshError(e.to_string()))?;

        let new_token = OAuthToken {
            service_id: service_id.to_string(),
            access_token: token_response.access_token,
            refresh_token: token_response
                .refresh_token
                .or(existing_token.refresh_token),
            token_type: token_response
                .token_type
                .unwrap_or_else(|| "Bearer".to_string()),
            expires_at: token_response
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs as i64)),
            scopes: existing_token.scopes,
            created_at: Utc::now(),
        };

        self.token_store.store(&new_token).await?;

        Ok(new_token)
    }

    /// 获取有效的 access token
    ///
    /// 如果 token 已过期，自动刷新
    ///
    /// # Arguments
    /// * `config` - OAuth 配置
    /// * `service_id` - 服务 ID
    pub async fn get_valid_token(
        &self,
        config: &OAuthConfig,
        service_id: &str,
    ) -> Result<String, OAuthError> {
        let token = self
            .token_store
            .get(service_id)
            .await?
            .ok_or_else(|| OAuthError::ServiceNotFound(service_id.to_string()))?;

        // 检查是否过期（提前 60 秒刷新）
        if let Some(expires_at) = token.expires_at {
            if expires_at < Utc::now() + Duration::seconds(60) {
                // Token 即将过期，尝试刷新
                let new_token = self.refresh_token(config, service_id).await?;
                return Ok(new_token.access_token);
            }
        }

        Ok(token.access_token)
    }

    /// 获取服务的 OAuth 状态
    pub async fn get_status(&self, service_id: &str) -> Result<OAuthServiceStatus, OAuthError> {
        // 检查是否有进行中的流程
        {
            let flows = self.pending_flows.read().await;
            for flow in flows.values() {
                if flow.service_id == service_id {
                    return Ok(OAuthServiceStatus {
                        service_id: service_id.to_string(),
                        status: OAuthStatus::Pending,
                        expires_at: None,
                        scopes: vec![],
                        last_refreshed: None,
                    });
                }
            }
        }

        // 检查是否有存储的 token
        match self.token_store.get(service_id).await? {
            Some(token) => {
                let status = if let Some(expires_at) = token.expires_at {
                    if expires_at < Utc::now() {
                        OAuthStatus::Expired
                    } else {
                        OAuthStatus::Connected
                    }
                } else {
                    OAuthStatus::Connected
                };

                Ok(OAuthServiceStatus {
                    service_id: service_id.to_string(),
                    status,
                    expires_at: token.expires_at,
                    scopes: token.scopes,
                    last_refreshed: Some(token.created_at),
                })
            }
            None => Ok(OAuthServiceStatus {
                service_id: service_id.to_string(),
                status: OAuthStatus::Disconnected,
                expires_at: None,
                scopes: vec![],
                last_refreshed: None,
            }),
        }
    }

    /// 断开服务连接（撤销授权）
    pub async fn disconnect(
        &self,
        config: &OAuthConfig,
        service_id: &str,
    ) -> Result<(), OAuthError> {
        // 获取 token
        if let Some(token) = self.token_store.get(service_id).await? {
            // 如果有 revoke URL，调用撤销接口
            if let Some(ref revoke_url) = config.revoke_url {
                let _ = self
                    .http_client
                    .post(revoke_url)
                    .form(&[("token", &token.access_token)])
                    .send()
                    .await;
                // 忽略撤销失败，继续删除本地 token
            }
        }

        // 删除本地 token
        self.token_store.delete(service_id).await?;

        Ok(())
    }

    /// 构建授权 URL
    fn build_authorization_url(
        &self,
        config: &OAuthConfig,
        challenge: &PkceChallenge,
        state: &str,
        redirect_uri: &str,
    ) -> String {
        let scopes = config.scopes.join(" ");
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            config.authorization_url,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state),
            urlencoding::encode(challenge.as_str())
        )
    }

    /// 生成随机 state 参数
    fn generate_state() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
    }

    /// 清理过期的流程状态
    pub async fn cleanup_expired_flows(&self) {
        let mut flows = self.pending_flows.write().await;
        let now = Utc::now();
        let timeout = Duration::minutes(10);

        flows.retain(|_, flow| now - flow.created_at < timeout);
    }
}

/// Token 响应结构
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::oauth::token_store::InMemoryTokenStore;

    #[test]
    fn test_generate_state() {
        let state1 = OAuthManager::generate_state();
        let state2 = OAuthManager::generate_state();

        // State 应该是 URL-safe base64
        assert!(!state1.is_empty());
        assert!(!state2.is_empty());
        // 两次生成的 state 应该不同
        assert_ne!(state1, state2);
    }

    #[tokio::test]
    async fn test_get_status_disconnected() {
        let store = Arc::new(InMemoryTokenStore::new());
        let manager = OAuthManager::new(store);

        let status = manager.get_status("test-service").await.unwrap();
        assert_eq!(status.status, OAuthStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_get_status_connected() {
        let store = Arc::new(InMemoryTokenStore::new());
        let token = OAuthToken {
            service_id: "test-service".to_string(),
            access_token: "test-token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            scopes: vec!["read".to_string()],
            created_at: Utc::now(),
        };
        store.store(&token).await.unwrap();

        let manager = OAuthManager::new(store);
        let status = manager.get_status("test-service").await.unwrap();

        assert_eq!(status.status, OAuthStatus::Connected);
        assert_eq!(status.scopes, vec!["read".to_string()]);
    }

    #[tokio::test]
    async fn test_get_status_expired() {
        let store = Arc::new(InMemoryTokenStore::new());
        let token = OAuthToken {
            service_id: "test-service".to_string(),
            access_token: "test-token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(Utc::now() - Duration::hours(1)), // 已过期
            scopes: vec!["read".to_string()],
            created_at: Utc::now() - Duration::hours(2),
        };
        store.store(&token).await.unwrap();

        let manager = OAuthManager::new(store);
        let status = manager.get_status("test-service").await.unwrap();

        assert_eq!(status.status, OAuthStatus::Expired);
    }

    #[tokio::test]
    async fn test_cleanup_expired_flows() {
        let store = Arc::new(InMemoryTokenStore::new());
        let manager = OAuthManager::new(store);

        // 添加一个过期的流程
        {
            let mut flows = manager.pending_flows.write().await;
            flows.insert(
                "old-state".to_string(),
                OAuthFlowState {
                    service_id: "test".to_string(),
                    pkce_verifier: PkceVerifier::new("test-verifier".to_string()),
                    state: "old-state".to_string(),
                    created_at: Utc::now() - Duration::hours(1), // 1 小时前
                    callback_port: 7777,
                },
            );
        }

        manager.cleanup_expired_flows().await;

        let flows = manager.pending_flows.read().await;
        assert!(flows.is_empty());
    }
}
