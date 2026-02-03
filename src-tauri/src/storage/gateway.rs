//! Gateway 配置存储操作
//!
//! Story 11.1: SSE Server 核心 - Task 6
//!
//! 提供 gateway_config 表的 CRUD 操作

use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use super::database::Database;
use super::error::StorageError;

/// Gateway 配置数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfigRecord {
    pub id: i32,
    pub port: Option<i32>,
    pub auth_token: String,
    pub enabled: bool,
    pub auto_start: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Gateway 配置更新参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfigUpdate {
    pub port: Option<i32>,
    pub auth_token: Option<String>,
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
}

/// 从数据库行解析 GatewayConfigRecord
fn parse_gateway_config_row(row: &Row) -> rusqlite::Result<GatewayConfigRecord> {
    Ok(GatewayConfigRecord {
        id: row.get(0)?,
        port: row.get(1)?,
        auth_token: row.get(2)?,
        enabled: row.get::<_, i32>(3)? != 0,
        auto_start: row.get::<_, i32>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

impl Database {
    /// 获取 Gateway 配置
    ///
    /// 返回单例配置记录，如果不存在则创建默认配置
    pub fn get_gateway_config(&self) -> Result<GatewayConfigRecord, StorageError> {
        let result = self.connection().query_row(
            "SELECT id, port, auth_token, enabled, auto_start, created_at, updated_at 
             FROM gateway_config WHERE id = 1",
            [],
            parse_gateway_config_row,
        );

        match result {
            Ok(config) => Ok(config),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // 创建默认配置（默认启用自动启动）
                let auth_token = uuid::Uuid::new_v4().to_string();
                self.connection().execute(
                    "INSERT INTO gateway_config (id, auth_token, enabled, auto_start) VALUES (1, ?1, 1, 1)",
                    [&auth_token],
                )?;
                self.get_gateway_config()
            }
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    /// 更新 Gateway 配置
    ///
    /// 仅更新提供的字段，其他字段保持不变
    pub fn update_gateway_config(&self, update: &GatewayConfigUpdate) -> Result<GatewayConfigRecord, StorageError> {
        let now = chrono::Utc::now().to_rfc3339();

        // 获取当前配置
        let current = self.get_gateway_config()?;

        // 合并更新
        let port = update.port.or(current.port);
        let auth_token = update.auth_token.clone().unwrap_or(current.auth_token);
        let enabled = update.enabled.unwrap_or(current.enabled);
        let auto_start = update.auto_start.unwrap_or(current.auto_start);

        // 执行更新
        self.connection().execute(
            "UPDATE gateway_config SET port = ?1, auth_token = ?2, enabled = ?3, auto_start = ?4, updated_at = ?5 WHERE id = 1",
            params![port, &auth_token, enabled as i32, auto_start as i32, &now],
        )?;

        self.get_gateway_config()
    }

    /// 重新生成认证 Token
    ///
    /// 使用 UUID v4 生成新的 Token 并更新数据库
    pub fn regenerate_gateway_token(&self) -> Result<String, StorageError> {
        let new_token = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.connection().execute(
            "UPDATE gateway_config SET auth_token = ?1, updated_at = ?2 WHERE id = 1",
            params![&new_token, &now],
        )?;

        Ok(new_token)
    }

    /// 设置 Gateway 启用状态
    pub fn set_gateway_enabled(&self, enabled: bool) -> Result<(), StorageError> {
        // 确保配置记录存在
        self.get_gateway_config()?;
        
        let now = chrono::Utc::now().to_rfc3339();
        self.connection().execute(
            "UPDATE gateway_config SET enabled = ?1, updated_at = ?2 WHERE id = 1",
            params![enabled as i32, &now],
        )?;
        Ok(())
    }

    /// 设置 Gateway 端口
    pub fn set_gateway_port(&self, port: Option<i32>) -> Result<(), StorageError> {
        // 确保配置记录存在
        self.get_gateway_config()?;
        
        let now = chrono::Utc::now().to_rfc3339();
        self.connection().execute(
            "UPDATE gateway_config SET port = ?1, updated_at = ?2 WHERE id = 1",
            params![port, &now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gateway_config_creates_default() {
        let db = Database::new_in_memory().unwrap();
        let config = db.get_gateway_config().unwrap();

        assert_eq!(config.id, 1);
        assert!(!config.auth_token.is_empty());
        // 默认启用 Gateway 和自动启动
        assert!(config.enabled);
        assert!(config.auto_start);
    }

    #[test]
    fn test_update_gateway_config_port() {
        let db = Database::new_in_memory().unwrap();

        let update = GatewayConfigUpdate {
            port: Some(8080),
            ..Default::default()
        };

        let config = db.update_gateway_config(&update).unwrap();
        assert_eq!(config.port, Some(8080));
    }

    #[test]
    fn test_update_gateway_config_enabled() {
        let db = Database::new_in_memory().unwrap();

        let update = GatewayConfigUpdate {
            enabled: Some(true),
            ..Default::default()
        };

        let config = db.update_gateway_config(&update).unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn test_update_gateway_config_multiple_fields() {
        let db = Database::new_in_memory().unwrap();

        let update = GatewayConfigUpdate {
            port: Some(9000),
            enabled: Some(true),
            auto_start: Some(true),
            ..Default::default()
        };

        let config = db.update_gateway_config(&update).unwrap();
        assert_eq!(config.port, Some(9000));
        assert!(config.enabled);
        assert!(config.auto_start);
    }

    #[test]
    fn test_regenerate_gateway_token() {
        let db = Database::new_in_memory().unwrap();

        let original_config = db.get_gateway_config().unwrap();
        let new_token = db.regenerate_gateway_token().unwrap();

        assert_ne!(original_config.auth_token, new_token);
        
        let updated_config = db.get_gateway_config().unwrap();
        assert_eq!(updated_config.auth_token, new_token);
    }

    #[test]
    fn test_set_gateway_enabled() {
        let db = Database::new_in_memory().unwrap();

        db.set_gateway_enabled(true).unwrap();
        let config = db.get_gateway_config().unwrap();
        assert!(config.enabled);

        db.set_gateway_enabled(false).unwrap();
        let config = db.get_gateway_config().unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_set_gateway_port() {
        let db = Database::new_in_memory().unwrap();

        db.set_gateway_port(Some(39600)).unwrap();
        let config = db.get_gateway_config().unwrap();
        assert_eq!(config.port, Some(39600));

        db.set_gateway_port(None).unwrap();
        let config = db.get_gateway_config().unwrap();
        assert_eq!(config.port, None);
    }

    #[test]
    fn test_gateway_config_singleton() {
        let db = Database::new_in_memory().unwrap();

        // 尝试插入第二条记录应该失败（CHECK 约束）
        let result = db.connection().execute(
            "INSERT INTO gateway_config (id, auth_token) VALUES (2, 'test')",
            [],
        );

        assert!(result.is_err(), "Should not allow id != 1");
    }

    #[test]
    fn test_gateway_config_auth_token_uuid_format() {
        let db = Database::new_in_memory().unwrap();
        let config = db.get_gateway_config().unwrap();

        // 验证 auth_token 是有效的 UUID v4 格式
        let parsed = uuid::Uuid::parse_str(&config.auth_token);
        assert!(parsed.is_ok(), "auth_token should be valid UUID");
    }
}
