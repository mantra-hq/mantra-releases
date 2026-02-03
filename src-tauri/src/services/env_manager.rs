//! 环境变量管理器
//!
//! Story 11.2: MCP 服务数据模型 - Task 5
//! Story 11.4: 环境变量管理 - Task 2 (变量注入逻辑)
//!
//! 提供环境变量的加密存储、解密读取和变量注入功能

use std::collections::HashMap;

use regex::Regex;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use rusqlite::params;

use crate::models::mcp::{EnvVariable, McpService};
use crate::storage::{Database, StorageError};

/// 环境变量管理器
///
/// 使用 AES-256-GCM 加密存储敏感环境变量
pub struct EnvManager {
    key: LessSafeKey,
}

impl EnvManager {
    /// 从密钥材料创建管理器
    ///
    /// # Arguments
    /// * `key_bytes` - 32 字节的密钥材料
    ///
    /// # Panics
    /// 如果密钥材料无效则 panic
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key_bytes)
            .expect("Invalid key material for AES-256-GCM");
        Self {
            key: LessSafeKey::new(unbound_key),
        }
    }

    /// 从机器唯一标识派生密钥
    ///
    /// 使用机器 ID 和应用标识符派生一个稳定的加密密钥
    pub fn from_machine_id() -> Self {
        // 获取机器唯一标识
        let machine_id = Self::get_machine_id();
        
        // 使用简单的 HKDF-like 派生（实际生产环境应使用 ring::hkdf）
        let mut key_bytes = [0u8; 32];
        let salt = b"mantra-env-manager-v1";
        
        // 简单的密钥派生：将 machine_id 和 salt 混合
        for (i, byte) in machine_id.bytes().chain(salt.iter().copied()).enumerate() {
            key_bytes[i % 32] ^= byte;
        }
        
        // 额外的混合以增加熵
        for i in 0..32 {
            key_bytes[i] = key_bytes[i].wrapping_add(key_bytes[(i + 17) % 32]);
        }
        
        Self::new(&key_bytes)
    }

    /// 获取机器唯一标识
    fn get_machine_id() -> String {
        // 尝试从多个来源获取机器 ID
        // 1. 环境变量（用于测试）
        if let Ok(id) = std::env::var("MANTRA_MACHINE_ID") {
            return id;
        }

        // 2. 尝试读取系统机器 ID
        #[cfg(target_os = "linux")]
        {
            if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
                return id.trim().to_string();
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 使用 IOPlatformUUID
            if let Ok(output) = std::process::Command::new("ioreg")
                .args(["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("IOPlatformUUID")) {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return uuid.to_string();
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 使用 MachineGuid
            if let Ok(output) = std::process::Command::new("reg")
                .args([
                    "query",
                    "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Cryptography",
                    "/v",
                    "MachineGuid",
                ])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("MachineGuid")) {
                    if let Some(guid) = line.split_whitespace().last() {
                        return guid.to_string();
                    }
                }
            }
        }

        // 3. 回退：使用用户名 + 主机名
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "localhost".to_string());

        format!("{}-{}", username, hostname)
    }

    /// 加密值
    ///
    /// # Arguments
    /// * `plaintext` - 要加密的明文
    ///
    /// # Returns
    /// 加密后的字节数组（nonce + ciphertext + tag）
    pub fn encrypt(&self, plaintext: &str) -> Vec<u8> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .expect("Failed to generate random nonce");

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();

        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .expect("Encryption failed");

        // 返回 nonce + ciphertext + tag
        [nonce_bytes.to_vec(), in_out].concat()
    }

    /// 解密值
    ///
    /// # Arguments
    /// * `ciphertext` - 加密后的字节数组
    ///
    /// # Returns
    /// 解密后的明文
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String, &'static str> {
        if ciphertext.len() < NONCE_LEN {
            return Err("Invalid ciphertext: too short");
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_LEN);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes
                .try_into()
                .map_err(|_| "Invalid nonce length")?,
        );

        let mut in_out = encrypted.to_vec();
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| "Decryption failed")?;

        String::from_utf8(plaintext.to_vec()).map_err(|_| "Invalid UTF-8")
    }
}

/// 生成脱敏显示
///
/// # Arguments
/// * `value` - 原始值
///
/// # Returns
/// 脱敏后的值，如 "sk-****...****xyz"
pub fn mask_value(value: &str) -> String {
    if value.len() <= 8 {
        "*".repeat(value.len())
    } else {
        let prefix = &value[..4];
        let suffix = &value[value.len() - 4..];
        format!("{}****{}", prefix, suffix)
    }
}

// ===== Story 11.4: 变量注入逻辑 (Task 2) =====

/// 解析环境变量引用
///
/// 将字符串中的环境变量引用替换为实际值
///
/// # 支持格式
/// - `$VAR_NAME` - 简单格式
/// - `${VAR_NAME}` - 带花括号格式（支持与其他文本连接）
///
/// # Arguments
/// * `value` - 包含变量引用的字符串
/// * `db` - 数据库连接
/// * `env_manager` - 环境变量管理器
///
/// # Returns
/// 解析后的字符串，变量引用被替换为实际值
///
/// # Note
/// 如果变量不存在，保留原始引用不变
pub fn resolve_env_references(
    value: &str,
    db: &Database,
    env_manager: &EnvManager,
) -> Result<String, StorageError> {
    // 正则：匹配 ${VAR_NAME} 或 $VAR_NAME
    // 先匹配带花括号的格式，再匹配简单格式
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}|\$([A-Z_][A-Z0-9_]*)").unwrap();

    let mut result = value.to_string();
    let mut replacements: Vec<(String, String)> = Vec::new();

    for cap in re.captures_iter(value) {
        // 获取变量名（可能在第一个或第二个捕获组）
        let var_name = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());

        if let Some(var_name) = var_name {
            if let Some(decrypted) = db.get_env_variable(env_manager, var_name)? {
                let full_match = cap.get(0).unwrap().as_str();
                replacements.push((full_match.to_string(), decrypted));
            }
        }
    }

    // 应用替换（从后向前避免索引问题）
    for (pattern, replacement) in replacements {
        result = result.replace(&pattern, &replacement);
    }

    Ok(result)
}

/// 为 MCP 服务构建环境变量
///
/// 解析服务配置中的环境变量引用，构建可用于子进程的环境变量映射
///
/// # Arguments
/// * `service` - MCP 服务配置
/// * `db` - 数据库连接
/// * `env_manager` - 环境变量管理器
///
/// # Returns
/// 环境变量映射（key -> value）
pub fn build_mcp_env(
    service: &McpService,
    db: &Database,
    env_manager: &EnvManager,
) -> Result<HashMap<String, String>, StorageError> {
    let mut env = HashMap::new();

    if let Some(env_config) = &service.env {
        if let Some(obj) = env_config.as_object() {
            for (key, value) in obj {
                let resolved = if let Some(s) = value.as_str() {
                    resolve_env_references(s, db, env_manager)?
                } else {
                    // 非字符串值直接转为字符串
                    value.to_string()
                };
                env.insert(key.clone(), resolved);
            }
        }
    }

    Ok(env)
}

/// 提取字符串中引用的所有环境变量名
///
/// # Arguments
/// * `value` - 包含变量引用的字符串
///
/// # Returns
/// 引用的环境变量名列表
pub fn extract_env_var_names(value: &str) -> Vec<String> {
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}|\$([A-Z_][A-Z0-9_]*)").unwrap();
    let mut names = Vec::new();

    for cap in re.captures_iter(value) {
        if let Some(var_name) = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str()) {
            if !names.contains(&var_name.to_string()) {
                names.push(var_name.to_string());
            }
        }
    }

    names
}

// ===== 数据库存储操作 =====

impl Database {
    /// 设置环境变量（加密存储）
    ///
    /// 如果变量已存在则更新，否则创建新变量
    ///
    /// # Arguments
    /// * `env_manager` - 环境变量管理器
    /// * `name` - 变量名称
    /// * `value` - 变量值（明文）
    /// * `description` - 变量描述
    ///
    /// # Returns
    /// 创建/更新的环境变量（值已脱敏）
    pub fn set_env_variable(
        &self,
        env_manager: &EnvManager,
        name: &str,
        value: &str,
        description: Option<&str>,
    ) -> Result<EnvVariable, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();
        let encrypted_value = env_manager.encrypt(value);
        let masked_value = mask_value(value);

        // 尝试更新现有变量
        let affected = self.connection().execute(
            r#"UPDATE env_variables SET encrypted_value = ?1, description = ?2, updated_at = ?3
               WHERE name = ?4"#,
            params![&encrypted_value, description, &now, name],
        )?;

        if affected == 0 {
            // 创建新变量
            let id = uuid::Uuid::new_v4().to_string();
            self.connection().execute(
                r#"INSERT INTO env_variables (id, name, encrypted_value, description, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?5)"#,
                params![&id, name, &encrypted_value, description, &now],
            )?;

            Ok(EnvVariable {
                id,
                name: name.to_string(),
                masked_value,
                description: description.map(|s| s.to_string()),
                created_at: now.clone(),
                updated_at: now,
            })
        } else {
            // 获取更新后的变量
            self.get_env_variable_by_name(name)?
                .ok_or_else(|| StorageError::NotFound(format!("Env variable not found: {}", name)))
        }
    }

    /// 获取环境变量（解密）
    ///
    /// # Arguments
    /// * `env_manager` - 环境变量管理器
    /// * `name` - 变量名称
    ///
    /// # Returns
    /// 解密后的变量值
    pub fn get_env_variable(
        &self,
        env_manager: &EnvManager,
        name: &str,
    ) -> Result<Option<String>, StorageError> {
        let result = self.connection().query_row(
            "SELECT encrypted_value FROM env_variables WHERE name = ?1",
            [name],
            |row| {
                let encrypted: Vec<u8> = row.get(0)?;
                Ok(encrypted)
            },
        );

        match result {
            Ok(encrypted) => {
                let decrypted = env_manager
                    .decrypt(&encrypted)
                    .map_err(|e| StorageError::InvalidInput(e.to_string()))?;
                Ok(Some(decrypted))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// 获取环境变量元信息（值已脱敏）
    ///
    /// # Arguments
    /// * `name` - 变量名称
    ///
    /// # Returns
    /// 环境变量元信息
    fn get_env_variable_by_name(&self, name: &str) -> Result<Option<EnvVariable>, StorageError> {
        let result = self.connection().query_row(
            r#"SELECT id, name, encrypted_value, description, created_at, updated_at
               FROM env_variables WHERE name = ?1"#,
            [name],
            |row| {
                let encrypted: Vec<u8> = row.get(2)?;
                // 我们无法在这里解密，所以使用占位符
                // 实际的脱敏值需要在设置时保存或重新计算
                let masked = format!("****({} bytes)", encrypted.len());

                Ok(EnvVariable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    masked_value: masked,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(var) => Ok(Some(var)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// 列出所有环境变量（值已脱敏）
    ///
    /// # Returns
    /// 所有环境变量列表（值已脱敏）
    pub fn list_env_variables(&self) -> Result<Vec<EnvVariable>, StorageError> {
        let mut stmt = self.connection().prepare(
            r#"SELECT id, name, encrypted_value, description, created_at, updated_at
               FROM env_variables ORDER BY name ASC"#,
        )?;

        let variables = stmt
            .query_map([], |row| {
                let encrypted: Vec<u8> = row.get(2)?;
                let masked = format!("****({} bytes)", encrypted.len());

                Ok(EnvVariable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    masked_value: masked,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(variables)
    }

    /// 删除环境变量
    ///
    /// # Arguments
    /// * `name` - 变量名称
    pub fn delete_env_variable(&self, name: &str) -> Result<(), StorageError> {
        let affected = self.connection().execute(
            "DELETE FROM env_variables WHERE name = ?1",
            [name],
        )?;

        if affected == 0 {
            return Err(StorageError::NotFound(format!(
                "Env variable not found: {}",
                name
            )));
        }

        Ok(())
    }

    /// 检查环境变量是否存在
    ///
    /// # Arguments
    /// * `name` - 变量名称
    ///
    /// # Returns
    /// 是否存在
    pub fn env_variable_exists(&self, name: &str) -> Result<bool, StorageError> {
        let count: i32 = self.connection().query_row(
            "SELECT COUNT(*) FROM env_variables WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_env_manager() -> EnvManager {
        let key_bytes: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        EnvManager::new(&key_bytes)
    }

    #[test]
    fn test_encrypt_decrypt() {
        let manager = create_test_env_manager();
        let plaintext = "sk-1234567890abcdef";

        let encrypted = manager.encrypt(plaintext);
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let manager = create_test_env_manager();
        let plaintext = "test-value";

        let encrypted1 = manager.encrypt(plaintext);
        let encrypted2 = manager.encrypt(plaintext);

        // 由于使用随机 nonce，每次加密结果应该不同
        assert_ne!(encrypted1, encrypted2);

        // 但解密结果应该相同
        assert_eq!(manager.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(manager.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_invalid_ciphertext() {
        let manager = create_test_env_manager();

        // 太短
        let result = manager.decrypt(&[0u8; 5]);
        assert!(result.is_err());

        // 无效数据
        let result = manager.decrypt(&[0u8; 32]);
        assert!(result.is_err());
    }

    #[test]
    fn test_mask_value_short() {
        assert_eq!(mask_value("abc"), "***");
        assert_eq!(mask_value("12345678"), "********");
    }

    #[test]
    fn test_mask_value_long() {
        assert_eq!(mask_value("sk-1234567890abcdef"), "sk-1****cdef");
        assert_eq!(mask_value("AKIAIOSFODNN7EXAMPLE"), "AKIA****MPLE");
    }

    #[test]
    fn test_set_env_variable() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        let var = db
            .set_env_variable(&manager, "API_KEY", "sk-secret123", Some("Test API Key"))
            .unwrap();

        assert_eq!(var.name, "API_KEY");
        assert!(var.masked_value.contains("****"));
        assert_eq!(var.description, Some("Test API Key".to_string()));
    }

    #[test]
    fn test_set_env_variable_update() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        // Create
        db.set_env_variable(&manager, "API_KEY", "old-value", None)
            .unwrap();

        // Update
        db.set_env_variable(&manager, "API_KEY", "new-value", Some("Updated"))
            .unwrap();

        // Verify
        let value = db.get_env_variable(&manager, "API_KEY").unwrap().unwrap();
        assert_eq!(value, "new-value");
    }

    #[test]
    fn test_get_env_variable() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "SECRET", "my-secret-value", None)
            .unwrap();

        let value = db.get_env_variable(&manager, "SECRET").unwrap().unwrap();
        assert_eq!(value, "my-secret-value");
    }

    #[test]
    fn test_get_env_variable_not_found() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        let value = db.get_env_variable(&manager, "NONEXISTENT").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_list_env_variables() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "ALPHA_KEY", "value1", None)
            .unwrap();
        db.set_env_variable(&manager, "BETA_KEY", "value2", Some("Beta"))
            .unwrap();

        let vars = db.list_env_variables().unwrap();

        assert_eq!(vars.len(), 2);
        // Should be sorted by name
        assert_eq!(vars[0].name, "ALPHA_KEY");
        assert_eq!(vars[1].name, "BETA_KEY");
        // Values should be masked
        assert!(vars[0].masked_value.contains("****"));
        assert!(vars[1].masked_value.contains("****"));
    }

    #[test]
    fn test_delete_env_variable() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "TO_DELETE", "value", None)
            .unwrap();
        db.delete_env_variable("TO_DELETE").unwrap();

        let value = db.get_env_variable(&manager, "TO_DELETE").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_delete_env_variable_not_found() {
        let db = Database::new_in_memory().unwrap();

        let result = db.delete_env_variable("NONEXISTENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_env_variable_exists() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        assert!(!db.env_variable_exists("MY_VAR").unwrap());

        db.set_env_variable(&manager, "MY_VAR", "value", None)
            .unwrap();

        assert!(db.env_variable_exists("MY_VAR").unwrap());
    }

    #[test]
    fn test_env_manager_from_machine_id() {
        // 设置测试环境变量
        std::env::set_var("MANTRA_MACHINE_ID", "test-machine-123");

        let manager = EnvManager::from_machine_id();
        let plaintext = "test-encryption";

        let encrypted = manager.encrypt(plaintext);
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);

        // 清理
        std::env::remove_var("MANTRA_MACHINE_ID");
    }

    #[test]
    fn test_encrypt_empty_string() {
        let manager = create_test_env_manager();

        let encrypted = manager.encrypt("");
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_unicode() {
        let manager = create_test_env_manager();
        let plaintext = "密钥🔑测试";

        let encrypted = manager.encrypt(plaintext);
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_long_value() {
        let manager = create_test_env_manager();
        let plaintext = "a".repeat(10000);

        let encrypted = manager.encrypt(&plaintext);
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    // ===== Story 11.4: 变量注入逻辑测试 =====

    #[test]
    fn test_resolve_env_references_simple_format() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        // 设置环境变量
        db.set_env_variable(&manager, "API_KEY", "sk-secret123", None)
            .unwrap();

        // 解析简单格式
        let result = resolve_env_references("$API_KEY", &db, &manager).unwrap();
        assert_eq!(result, "sk-secret123");
    }

    #[test]
    fn test_resolve_env_references_braced_format() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "BASE_URL", "https://api.example.com", None)
            .unwrap();

        // 解析带花括号格式
        let result = resolve_env_references("${BASE_URL}/v1/chat", &db, &manager).unwrap();
        assert_eq!(result, "https://api.example.com/v1/chat");
    }

    #[test]
    fn test_resolve_env_references_multiple_vars() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "HOST", "localhost", None)
            .unwrap();
        db.set_env_variable(&manager, "PORT", "8080", None)
            .unwrap();

        let result = resolve_env_references("http://$HOST:$PORT", &db, &manager).unwrap();
        assert_eq!(result, "http://localhost:8080");
    }

    #[test]
    fn test_resolve_env_references_mixed_formats() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        db.set_env_variable(&manager, "USER", "admin", None)
            .unwrap();
        db.set_env_variable(&manager, "PASS", "secret", None)
            .unwrap();

        let result =
            resolve_env_references("${USER}:$PASS@server", &db, &manager).unwrap();
        assert_eq!(result, "admin:secret@server");
    }

    #[test]
    fn test_resolve_env_references_missing_var() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        // 变量不存在时保留原始引用
        let result = resolve_env_references("$NONEXISTENT_VAR", &db, &manager).unwrap();
        assert_eq!(result, "$NONEXISTENT_VAR");
    }

    #[test]
    fn test_resolve_env_references_no_vars() {
        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        // 没有变量引用的字符串保持不变
        let result = resolve_env_references("plain text", &db, &manager).unwrap();
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_build_mcp_env() {
        use crate::models::mcp::{McpService, McpServiceSource};

        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        // 设置环境变量
        db.set_env_variable(&manager, "OPENAI_API_KEY", "sk-openai-key", None)
            .unwrap();

        // 创建 MCP 服务
        let service = McpService {
            id: "test-id".to_string(),
            name: "openai-mcp".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: Some(serde_json::json!({
                "OPENAI_API_KEY": "$OPENAI_API_KEY",
                "DEBUG": "true"
            })),
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let env = build_mcp_env(&service, &db, &manager).unwrap();

        assert_eq!(env.get("OPENAI_API_KEY"), Some(&"sk-openai-key".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_build_mcp_env_no_env() {
        use crate::models::mcp::{McpService, McpServiceSource};

        let db = Database::new_in_memory().unwrap();
        let manager = create_test_env_manager();

        let service = McpService {
            id: "test-id".to_string(),
            name: "simple-mcp".to_string(),
            transport_type: Default::default(),
            command: "npx".to_string(),
            args: None,
            env: None,
            url: None,
            headers: None,
            source: McpServiceSource::Manual,
            source_file: None,
            source_adapter_id: None,
            source_scope: None,
            enabled: true,
            created_at: "2026-01-30T00:00:00Z".to_string(),
            updated_at: "2026-01-30T00:00:00Z".to_string(),
            default_tool_policy: None,
        };

        let env = build_mcp_env(&service, &db, &manager).unwrap();
        assert!(env.is_empty());
    }

    #[test]
    fn test_extract_env_var_names() {
        let names = extract_env_var_names("$API_KEY and ${BASE_URL}/path");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"API_KEY".to_string()));
        assert!(names.contains(&"BASE_URL".to_string()));
    }

    #[test]
    fn test_extract_env_var_names_duplicates() {
        let names = extract_env_var_names("$KEY $KEY ${KEY}");
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "KEY");
    }

    #[test]
    fn test_extract_env_var_names_no_vars() {
        let names = extract_env_var_names("no variables here");
        assert!(names.is_empty());
    }
}
