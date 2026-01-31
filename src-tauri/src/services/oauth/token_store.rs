//! Token 安全存储
//!
//! Story 11.12: Remote MCP OAuth Support - Task 2
//!
//! 提供 OAuth Token 的安全存储，支持：
//! - 系统 Keyring (首选)
//! - SQLite 加密存储 (回退)
//! - 内存存储 (测试用)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Token 存储错误
#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("Token not found: {0}")]
    NotFound(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Keyring error: {0}")]
    KeyringError(String),
}

/// OAuth Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// 服务 ID
    pub service_id: String,
    /// Access Token
    pub access_token: String,
    /// Refresh Token (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token 类型 (通常是 "Bearer")
    pub token_type: String,
    /// 过期时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// 已授权的 scopes
    pub scopes: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 安全 Token 存储 trait
#[async_trait]
pub trait SecureTokenStore: Send + Sync {
    /// 存储 token
    async fn store(&self, token: &OAuthToken) -> Result<(), TokenStoreError>;

    /// 获取 token
    async fn get(&self, service_id: &str) -> Result<Option<OAuthToken>, TokenStoreError>;

    /// 删除 token
    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError>;

    /// 列出所有服务 ID
    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError>;
}

/// 内存 Token 存储 (用于测试)
pub struct InMemoryTokenStore {
    tokens: RwLock<HashMap<String, OAuthToken>>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecureTokenStore for InMemoryTokenStore {
    async fn store(&self, token: &OAuthToken) -> Result<(), TokenStoreError> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.service_id.clone(), token.clone());
        Ok(())
    }

    async fn get(&self, service_id: &str) -> Result<Option<OAuthToken>, TokenStoreError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(service_id).cloned())
    }

    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError> {
        let mut tokens = self.tokens.write().await;
        tokens.remove(service_id);
        Ok(())
    }

    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.keys().cloned().collect())
    }
}

/// 加密 Token 存储
///
/// 使用 ChaCha20-Poly1305 加密 Token 数据
pub struct EncryptedTokenStore {
    /// 加密密钥
    key: LessSafeKey,
    /// 底层存储
    storage: Arc<dyn TokenStorage>,
}

/// 底层存储 trait
#[async_trait]
pub trait TokenStorage: Send + Sync {
    /// 存储加密数据
    async fn store_encrypted(
        &self,
        service_id: &str,
        data: &[u8],
    ) -> Result<(), TokenStoreError>;

    /// 获取加密数据
    async fn get_encrypted(&self, service_id: &str) -> Result<Option<Vec<u8>>, TokenStoreError>;

    /// 删除数据
    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError>;

    /// 列出所有服务 ID
    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError>;
}

impl EncryptedTokenStore {
    /// 创建新的加密存储
    ///
    /// # Arguments
    /// * `key_bytes` - 32 字节密钥
    /// * `storage` - 底层存储实现
    pub fn new(key_bytes: &[u8; 32], storage: Arc<dyn TokenStorage>) -> Result<Self, TokenStoreError> {
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, key_bytes)
            .map_err(|_| TokenStoreError::EncryptionError("Invalid key".to_string()))?;
        let key = LessSafeKey::new(unbound_key);

        Ok(Self { key, storage })
    }

    /// 从系统随机源生成密钥
    pub fn generate_key() -> Result<[u8; 32], TokenStoreError> {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key)
            .map_err(|_| TokenStoreError::EncryptionError("Failed to generate key".to_string()))?;
        Ok(key)
    }

    /// 加密数据
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, TokenStoreError> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| TokenStoreError::EncryptionError("Failed to generate nonce".to_string()))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| TokenStoreError::EncryptionError("Encryption failed".to_string()))?;

        // 返回 nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(in_out);
        Ok(result)
    }

    /// 解密数据
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, TokenStoreError> {
        if ciphertext.len() < 12 {
            return Err(TokenStoreError::DecryptionError(
                "Ciphertext too short".to_string(),
            ));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| TokenStoreError::DecryptionError("Invalid nonce".to_string()))?;
        let nonce = Nonce::assume_unique_for_key(nonce_array);

        let mut in_out = encrypted.to_vec();
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| TokenStoreError::DecryptionError("Decryption failed".to_string()))?;

        Ok(plaintext.to_vec())
    }
}

#[async_trait]
impl SecureTokenStore for EncryptedTokenStore {
    async fn store(&self, token: &OAuthToken) -> Result<(), TokenStoreError> {
        let json = serde_json::to_vec(token)
            .map_err(|e| TokenStoreError::SerializationError(e.to_string()))?;

        let encrypted = self.encrypt(&json)?;
        self.storage
            .store_encrypted(&token.service_id, &encrypted)
            .await
    }

    async fn get(&self, service_id: &str) -> Result<Option<OAuthToken>, TokenStoreError> {
        match self.storage.get_encrypted(service_id).await? {
            Some(encrypted) => {
                let decrypted = self.decrypt(&encrypted)?;
                let token: OAuthToken = serde_json::from_slice(&decrypted)
                    .map_err(|e| TokenStoreError::SerializationError(e.to_string()))?;
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError> {
        self.storage.delete(service_id).await
    }

    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError> {
        self.storage.list_services().await
    }
}

/// 内存底层存储 (用于测试)
pub struct InMemoryStorage {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenStorage for InMemoryStorage {
    async fn store_encrypted(
        &self,
        service_id: &str,
        data: &[u8],
    ) -> Result<(), TokenStoreError> {
        let mut storage = self.data.write().await;
        storage.insert(service_id.to_string(), data.to_vec());
        Ok(())
    }

    async fn get_encrypted(&self, service_id: &str) -> Result<Option<Vec<u8>>, TokenStoreError> {
        let storage = self.data.read().await;
        Ok(storage.get(service_id).cloned())
    }

    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError> {
        let mut storage = self.data.write().await;
        storage.remove(service_id);
        Ok(())
    }

    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError> {
        let storage = self.data.read().await;
        Ok(storage.keys().cloned().collect())
    }
}

/// Keyring Token 存储
///
/// 使用系统 Keyring 存储 Token
#[cfg(feature = "keyring")]
pub struct KeyringTokenStore {
    service_name: String,
}

#[cfg(feature = "keyring")]
impl KeyringTokenStore {
    /// 创建新的 Keyring 存储
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    fn get_entry(&self, service_id: &str) -> keyring::Entry {
        keyring::Entry::new(&self.service_name, service_id).unwrap()
    }
}

#[cfg(feature = "keyring")]
#[async_trait]
impl SecureTokenStore for KeyringTokenStore {
    async fn store(&self, token: &OAuthToken) -> Result<(), TokenStoreError> {
        let json = serde_json::to_string(token)
            .map_err(|e| TokenStoreError::SerializationError(e.to_string()))?;

        let entry = self.get_entry(&token.service_id);
        entry
            .set_password(&json)
            .map_err(|e| TokenStoreError::KeyringError(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, service_id: &str) -> Result<Option<OAuthToken>, TokenStoreError> {
        let entry = self.get_entry(service_id);
        match entry.get_password() {
            Ok(json) => {
                let token: OAuthToken = serde_json::from_str(&json)
                    .map_err(|e| TokenStoreError::SerializationError(e.to_string()))?;
                Ok(Some(token))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(TokenStoreError::KeyringError(e.to_string())),
        }
    }

    async fn delete(&self, service_id: &str) -> Result<(), TokenStoreError> {
        let entry = self.get_entry(service_id);
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // 不存在也算成功
            Err(e) => Err(TokenStoreError::KeyringError(e.to_string())),
        }
    }

    async fn list_services(&self) -> Result<Vec<String>, TokenStoreError> {
        // Keyring 不支持列出所有条目，需要配合其他存储记录服务列表
        // 这里返回空列表，实际使用时应该配合数据库记录
        Ok(vec![])
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryTokenStore::new();

        let token = OAuthToken {
            service_id: "test-service".to_string(),
            access_token: "access-123".to_string(),
            refresh_token: Some("refresh-456".to_string()),
            token_type: "Bearer".to_string(),
            expires_at: Some(Utc::now()),
            scopes: vec!["read".to_string(), "write".to_string()],
            created_at: Utc::now(),
        };

        // 存储
        store.store(&token).await.unwrap();

        // 获取
        let retrieved = store.get("test-service").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.access_token, "access-123");
        assert_eq!(retrieved.refresh_token, Some("refresh-456".to_string()));

        // 列出服务
        let services = store.list_services().await.unwrap();
        assert_eq!(services, vec!["test-service".to_string()]);

        // 删除
        store.delete("test-service").await.unwrap();
        let retrieved = store.get("test-service").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_encrypted_store() {
        let key = EncryptedTokenStore::generate_key().unwrap();
        let storage = Arc::new(InMemoryStorage::new());
        let store = EncryptedTokenStore::new(&key, storage).unwrap();

        let token = OAuthToken {
            service_id: "test-service".to_string(),
            access_token: "secret-access-token".to_string(),
            refresh_token: Some("secret-refresh-token".to_string()),
            token_type: "Bearer".to_string(),
            expires_at: Some(Utc::now()),
            scopes: vec!["read".to_string()],
            created_at: Utc::now(),
        };

        // 存储
        store.store(&token).await.unwrap();

        // 获取
        let retrieved = store.get("test-service").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.access_token, "secret-access-token");

        // 删除
        store.delete("test-service").await.unwrap();
        let retrieved = store.get("test-service").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_encrypted_store_wrong_key() {
        let key1 = EncryptedTokenStore::generate_key().unwrap();
        let key2 = EncryptedTokenStore::generate_key().unwrap();
        let storage = Arc::new(InMemoryStorage::new());

        let store1 = EncryptedTokenStore::new(&key1, storage.clone()).unwrap();
        let store2 = EncryptedTokenStore::new(&key2, storage).unwrap();

        let token = OAuthToken {
            service_id: "test-service".to_string(),
            access_token: "secret".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec![],
            created_at: Utc::now(),
        };

        // 使用 key1 存储
        store1.store(&token).await.unwrap();

        // 使用 key2 尝试获取应该失败
        let result = store2.get("test-service").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_key() {
        let key1 = EncryptedTokenStore::generate_key().unwrap();
        let key2 = EncryptedTokenStore::generate_key().unwrap();

        // 两次生成的密钥应该不同
        assert_ne!(key1, key2);
        // 密钥长度应该是 32 字节
        assert_eq!(key1.len(), 32);
    }

    #[tokio::test]
    async fn test_store_multiple_tokens() {
        let store = InMemoryTokenStore::new();

        for i in 0..5 {
            let token = OAuthToken {
                service_id: format!("service-{}", i),
                access_token: format!("token-{}", i),
                refresh_token: None,
                token_type: "Bearer".to_string(),
                expires_at: None,
                scopes: vec![],
                created_at: Utc::now(),
            };
            store.store(&token).await.unwrap();
        }

        let services = store.list_services().await.unwrap();
        assert_eq!(services.len(), 5);

        for i in 0..5 {
            let token = store.get(&format!("service-{}", i)).await.unwrap();
            assert!(token.is_some());
            assert_eq!(token.unwrap().access_token, format!("token-{}", i));
        }
    }
}
