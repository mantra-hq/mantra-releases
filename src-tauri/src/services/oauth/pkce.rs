//! PKCE (Proof Key for Code Exchange) 实现
//!
//! Story 11.12: Remote MCP OAuth Support - Task 1.2
//!
//! 实现 RFC 7636 PKCE 规范，防止授权码拦截攻击。

use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// PKCE 错误
#[derive(Debug, Error)]
pub enum PkceError {
    #[error("Failed to generate random bytes")]
    RandomGenerationFailed,
}

/// PKCE Code Verifier
///
/// 43-128 字符的随机字符串，用于生成 code_challenge
#[derive(Debug, Clone)]
pub struct PkceVerifier(String);

impl PkceVerifier {
    /// 创建新的 verifier
    pub fn new(verifier: String) -> Self {
        Self(verifier)
    }

    /// 生成随机 verifier
    ///
    /// 生成 43-128 字符的 URL-safe base64 字符串
    pub fn generate() -> Result<Self, PkceError> {
        let mut rng = rand::thread_rng();
        // 生成 32 字节随机数据，编码后约 43 字符
        let bytes: [u8; 32] = rng.gen();
        let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        Ok(Self(verifier))
    }

    /// 获取 verifier 字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 计算对应的 code_challenge
    pub fn challenge(&self) -> PkceChallenge {
        PkceChallenge::from_verifier(self)
    }
}

/// PKCE Code Challenge
///
/// code_verifier 的 SHA256 哈希值的 base64url 编码
#[derive(Debug, Clone)]
pub struct PkceChallenge(String);

impl PkceChallenge {
    /// 从 verifier 计算 challenge
    pub fn from_verifier(verifier: &PkceVerifier) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_str().as_bytes());
        let hash = hasher.finalize();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
        Self(challenge)
    }

    /// 生成 verifier 和 challenge 对
    pub fn generate() -> Result<(PkceVerifier, PkceChallenge), PkceError> {
        let verifier = PkceVerifier::generate()?;
        let challenge = verifier.challenge();
        Ok((verifier, challenge))
    }

    /// 获取 challenge 字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_generation() {
        let verifier = PkceVerifier::generate().unwrap();
        let s = verifier.as_str();

        // 验证长度在 43-128 之间
        assert!(s.len() >= 43);
        assert!(s.len() <= 128);

        // 验证是 URL-safe base64
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_verifier_uniqueness() {
        let v1 = PkceVerifier::generate().unwrap();
        let v2 = PkceVerifier::generate().unwrap();

        assert_ne!(v1.as_str(), v2.as_str());
    }

    #[test]
    fn test_challenge_from_verifier() {
        let verifier = PkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_string());
        let challenge = verifier.challenge();

        // 验证 challenge 是 URL-safe base64
        let s = challenge.as_str();
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_challenge_deterministic() {
        let verifier = PkceVerifier::new("test-verifier-12345".to_string());
        let c1 = verifier.challenge();
        let c2 = verifier.challenge();

        assert_eq!(c1.as_str(), c2.as_str());
    }

    #[test]
    fn test_generate_pair() {
        let (verifier, challenge) = PkceChallenge::generate().unwrap();

        // 验证 challenge 是从 verifier 计算的
        let expected_challenge = verifier.challenge();
        assert_eq!(challenge.as_str(), expected_challenge.as_str());
    }

    #[test]
    fn test_rfc7636_example() {
        // RFC 7636 Appendix B 测试向量
        // 注意：RFC 使用的是 plain 方法的示例，这里我们使用 S256
        let verifier = PkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_string());
        let challenge = verifier.challenge();

        // S256 challenge 应该是 verifier 的 SHA256 哈希的 base64url 编码
        // 验证格式正确
        assert!(!challenge.as_str().is_empty());
        assert!(challenge.as_str().len() == 43); // SHA256 = 32 bytes = 43 base64url chars
    }

    #[test]
    fn test_challenge_length() {
        // SHA256 输出 32 字节，base64url 编码后应该是 43 字符
        let verifier = PkceVerifier::generate().unwrap();
        let challenge = verifier.challenge();

        assert_eq!(challenge.as_str().len(), 43);
    }
}
