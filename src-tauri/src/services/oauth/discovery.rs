//! OAuth Discovery
//!
//! Story 11.12: Remote MCP OAuth Support - Task 4
//!
//! 实现 OAuth 2.0 Authorization Server Metadata (RFC 8414) 发现。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Discovery 错误
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Discovery not supported")]
    NotSupported,
}

/// OAuth Endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthEndpoints {
    /// Authorization endpoint
    pub authorization_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// Revocation endpoint (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
    /// Userinfo endpoint (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,
    /// 支持的 scopes
    #[serde(default)]
    pub scopes_supported: Vec<String>,
    /// 支持的 response types
    #[serde(default)]
    pub response_types_supported: Vec<String>,
    /// 支持的 grant types
    #[serde(default)]
    pub grant_types_supported: Vec<String>,
    /// 支持的 code challenge methods
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
}

/// Authorization Server Metadata 响应
#[derive(Debug, Deserialize)]
struct AuthorizationServerMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    revocation_endpoint: Option<String>,
    #[serde(default)]
    userinfo_endpoint: Option<String>,
    #[serde(default)]
    scopes_supported: Vec<String>,
    #[serde(default)]
    response_types_supported: Vec<String>,
    #[serde(default)]
    grant_types_supported: Vec<String>,
    #[serde(default)]
    code_challenge_methods_supported: Vec<String>,
}

/// OAuth Discovery
pub struct OAuthDiscovery {
    http_client: reqwest::Client,
}

impl OAuthDiscovery {
    /// 创建新的 Discovery 实例
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// 使用自定义 HTTP 客户端创建
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            http_client: client,
        }
    }

    /// 从 issuer URL 发现 OAuth endpoints
    ///
    /// 尝试以下 well-known 路径:
    /// 1. `{issuer}/.well-known/oauth-authorization-server`
    /// 2. `{issuer}/.well-known/openid-configuration` (OIDC 兼容)
    ///
    /// # Arguments
    /// * `issuer` - OAuth issuer URL (例如 `https://accounts.google.com`)
    pub async fn discover(&self, issuer: &str) -> Result<OAuthEndpoints, DiscoveryError> {
        let issuer = issuer.trim_end_matches('/');

        // 尝试 OAuth 2.0 Authorization Server Metadata
        let oauth_url = format!("{}/.well-known/oauth-authorization-server", issuer);
        if let Ok(endpoints) = self.fetch_metadata(&oauth_url).await {
            return Ok(endpoints);
        }

        // 回退到 OpenID Connect Discovery
        let oidc_url = format!("{}/.well-known/openid-configuration", issuer);
        self.fetch_metadata(&oidc_url).await
    }

    /// 获取元数据
    async fn fetch_metadata(&self, url: &str) -> Result<OAuthEndpoints, DiscoveryError> {
        let response = self
            .http_client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| DiscoveryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DiscoveryError::NotSupported);
        }

        let metadata: AuthorizationServerMetadata = response
            .json()
            .await
            .map_err(|e| DiscoveryError::InvalidResponse(e.to_string()))?;

        Ok(OAuthEndpoints {
            authorization_endpoint: metadata.authorization_endpoint,
            token_endpoint: metadata.token_endpoint,
            revocation_endpoint: metadata.revocation_endpoint,
            userinfo_endpoint: metadata.userinfo_endpoint,
            scopes_supported: metadata.scopes_supported,
            response_types_supported: metadata.response_types_supported,
            grant_types_supported: metadata.grant_types_supported,
            code_challenge_methods_supported: metadata.code_challenge_methods_supported,
        })
    }

    /// 验证 endpoints 支持 PKCE
    pub fn supports_pkce(endpoints: &OAuthEndpoints) -> bool {
        if endpoints.code_challenge_methods_supported.is_empty() {
            // 如果没有声明，假设支持 (很多服务器不声明但支持)
            return true;
        }
        endpoints
            .code_challenge_methods_supported
            .iter()
            .any(|m| m == "S256")
    }

    /// 验证 endpoints 支持 authorization_code grant
    pub fn supports_authorization_code(endpoints: &OAuthEndpoints) -> bool {
        if endpoints.grant_types_supported.is_empty() {
            // 如果没有声明，假设支持
            return true;
        }
        endpoints
            .grant_types_supported
            .iter()
            .any(|g| g == "authorization_code")
    }
}

impl Default for OAuthDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// 常见 OAuth Provider 的预设配置
pub mod presets {
    use super::OAuthEndpoints;

    /// Google OAuth endpoints
    pub fn google() -> OAuthEndpoints {
        OAuthEndpoints {
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_endpoint: "https://oauth2.googleapis.com/token".to_string(),
            revocation_endpoint: Some("https://oauth2.googleapis.com/revoke".to_string()),
            userinfo_endpoint: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
            scopes_supported: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string()],
        }
    }

    /// GitHub OAuth endpoints
    pub fn github() -> OAuthEndpoints {
        OAuthEndpoints {
            authorization_endpoint: "https://github.com/login/oauth/authorize".to_string(),
            token_endpoint: "https://github.com/login/oauth/access_token".to_string(),
            revocation_endpoint: None,
            userinfo_endpoint: Some("https://api.github.com/user".to_string()),
            scopes_supported: vec![
                "repo".to_string(),
                "user".to_string(),
                "read:user".to_string(),
            ],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            code_challenge_methods_supported: vec!["S256".to_string()],
        }
    }

    /// Slack OAuth endpoints
    pub fn slack() -> OAuthEndpoints {
        OAuthEndpoints {
            authorization_endpoint: "https://slack.com/oauth/v2/authorize".to_string(),
            token_endpoint: "https://slack.com/api/oauth.v2.access".to_string(),
            revocation_endpoint: Some("https://slack.com/api/auth.revoke".to_string()),
            userinfo_endpoint: None,
            scopes_supported: vec![],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            code_challenge_methods_supported: vec![],
        }
    }

    /// Microsoft/Azure AD OAuth endpoints
    pub fn microsoft(tenant: &str) -> OAuthEndpoints {
        let base = format!("https://login.microsoftonline.com/{}", tenant);
        OAuthEndpoints {
            authorization_endpoint: format!("{}/oauth2/v2.0/authorize", base),
            token_endpoint: format!("{}/oauth2/v2.0/token", base),
            revocation_endpoint: None,
            userinfo_endpoint: Some("https://graph.microsoft.com/oidc/userinfo".to_string()),
            scopes_supported: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "offline_access".to_string(),
            ],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_pkce_with_s256() {
        let endpoints = OAuthEndpoints {
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            revocation_endpoint: None,
            userinfo_endpoint: None,
            scopes_supported: vec![],
            response_types_supported: vec![],
            grant_types_supported: vec![],
            code_challenge_methods_supported: vec!["S256".to_string(), "plain".to_string()],
        };

        assert!(OAuthDiscovery::supports_pkce(&endpoints));
    }

    #[test]
    fn test_supports_pkce_empty() {
        let endpoints = OAuthEndpoints {
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            revocation_endpoint: None,
            userinfo_endpoint: None,
            scopes_supported: vec![],
            response_types_supported: vec![],
            grant_types_supported: vec![],
            code_challenge_methods_supported: vec![],
        };

        // 空列表假设支持
        assert!(OAuthDiscovery::supports_pkce(&endpoints));
    }

    #[test]
    fn test_supports_pkce_only_plain() {
        let endpoints = OAuthEndpoints {
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            revocation_endpoint: None,
            userinfo_endpoint: None,
            scopes_supported: vec![],
            response_types_supported: vec![],
            grant_types_supported: vec![],
            code_challenge_methods_supported: vec!["plain".to_string()],
        };

        // 只有 plain，不支持 S256
        assert!(!OAuthDiscovery::supports_pkce(&endpoints));
    }

    #[test]
    fn test_supports_authorization_code() {
        let endpoints = OAuthEndpoints {
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            revocation_endpoint: None,
            userinfo_endpoint: None,
            scopes_supported: vec![],
            response_types_supported: vec![],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            code_challenge_methods_supported: vec![],
        };

        assert!(OAuthDiscovery::supports_authorization_code(&endpoints));
    }

    #[test]
    fn test_google_preset() {
        let endpoints = presets::google();
        assert!(endpoints.authorization_endpoint.contains("google.com"));
        assert!(endpoints.token_endpoint.contains("googleapis.com"));
        assert!(OAuthDiscovery::supports_pkce(&endpoints));
        assert!(OAuthDiscovery::supports_authorization_code(&endpoints));
    }

    #[test]
    fn test_github_preset() {
        let endpoints = presets::github();
        assert!(endpoints.authorization_endpoint.contains("github.com"));
        assert!(endpoints.token_endpoint.contains("github.com"));
    }

    #[test]
    fn test_slack_preset() {
        let endpoints = presets::slack();
        assert!(endpoints.authorization_endpoint.contains("slack.com"));
        assert!(endpoints.revocation_endpoint.is_some());
    }

    #[test]
    fn test_microsoft_preset() {
        let endpoints = presets::microsoft("common");
        assert!(endpoints
            .authorization_endpoint
            .contains("login.microsoftonline.com"));
        assert!(endpoints.authorization_endpoint.contains("common"));
    }
}
