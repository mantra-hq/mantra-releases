//! 本地 Server 配置模块
//!
//! 管理端口配置，支持从配置文件读取和保存。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 默认端口号
pub const DEFAULT_PORT: u16 = 19836;

/// 配置文件名
const CONFIG_FILENAME: &str = "settings.yaml";

/// 本地 Server 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// 本地 API 端口
    #[serde(default = "default_port")]
    pub local_api_port: u16,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Default for LocalServerConfig {
    fn default() -> Self {
        Self {
            local_api_port: DEFAULT_PORT,
        }
    }
}

impl LocalServerConfig {
    /// 从配置目录加载配置
    ///
    /// # Arguments
    /// * `config_dir` - 配置目录路径 (app_data_dir)
    ///
    /// # Returns
    /// 配置对象，如果文件不存在则返回默认配置
    pub fn load(config_dir: &Path) -> Self {
        let config_path = config_dir.join(CONFIG_FILENAME);

        if !config_path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&config_path) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 保存配置到配置目录
    ///
    /// # Arguments
    /// * `config_dir` - 配置目录路径 (app_data_dir)
    pub fn save(&self, config_dir: &Path) -> Result<(), String> {
        let config_path = config_dir.join(CONFIG_FILENAME);

        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// 验证端口是否有效
    ///
    /// 端口必须在 1024-65535 范围内
    pub fn validate_port(port: u16) -> Result<(), String> {
        if port < 1024 {
            return Err("Port must be >= 1024 (non-privileged ports)".to_string());
        }
        Ok(())
    }

    /// 获取配置文件的完整路径（供 Hook 读取）
    pub fn get_config_path(config_dir: &Path) -> std::path::PathBuf {
        config_dir.join(CONFIG_FILENAME)
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = LocalServerConfig::default();
        assert_eq!(config.local_api_port, DEFAULT_PORT);
    }

    #[test]
    fn test_load_nonexistent_config() {
        let dir = tempdir().unwrap();
        let config = LocalServerConfig::load(dir.path());
        assert_eq!(config.local_api_port, DEFAULT_PORT);
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let config = LocalServerConfig {
            local_api_port: 12345,
        };

        config.save(dir.path()).unwrap();

        let loaded = LocalServerConfig::load(dir.path());
        assert_eq!(loaded.local_api_port, 12345);
    }

    #[test]
    fn test_validate_port() {
        assert!(LocalServerConfig::validate_port(1024).is_ok());
        assert!(LocalServerConfig::validate_port(19836).is_ok());
        assert!(LocalServerConfig::validate_port(65535).is_ok());
        assert!(LocalServerConfig::validate_port(1023).is_err());
        assert!(LocalServerConfig::validate_port(80).is_err());
    }
}
