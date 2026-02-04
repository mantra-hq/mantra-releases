//! 备份管理器
//!
//! RAII 模式管理配置文件备份，确保在出错时自动回滚

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::services::atomic_fs;

/// 备份条目
struct BackupEntry {
    original_path: PathBuf,
    backup_path: PathBuf,
}

/// 备份管理器
///
/// 使用 RAII 模式管理备份文件，确保在出错时自动回滚
pub struct BackupManager {
    backups: Vec<BackupEntry>,
    committed: bool,
}

impl BackupManager {
    /// 创建新的备份管理器
    pub fn new() -> Self {
        Self {
            backups: Vec::new(),
            committed: false,
        }
    }

    /// 原子备份文件 (Story 11.22)
    ///
    /// # Arguments
    /// * `path` - 要备份的文件路径
    ///
    /// # Returns
    /// (备份文件的路径, 备份文件的 SHA256 hash)
    pub fn backup(&mut self, path: &Path) -> io::Result<(PathBuf, String)> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {:?}", path),
            ));
        }

        // 构建备份文件名
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let backup_extension = if extension.is_empty() {
            "mantra-backup".to_string()
        } else {
            format!("{}.mantra-backup", extension)
        };

        let mut backup_path = path.with_extension(&backup_extension);

        // 如果备份已存在，添加时间戳
        if backup_path.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let timestamped_extension = format!("{}.{}", backup_extension, timestamp);
            backup_path = path.with_extension(timestamped_extension);
        }

        // 原子复制文件 (Story 11.22)
        let hash = atomic_fs::atomic_copy(path, &backup_path)?;

        self.backups.push(BackupEntry {
            original_path: path.to_path_buf(),
            backup_path: backup_path.clone(),
        });

        Ok((backup_path, hash))
    }

    /// 标记备份成功，不再需要回滚
    pub fn commit(&mut self) {
        self.committed = true;
    }

    /// 手动回滚所有备份 (Story 11.22: 使用原子操作)
    pub fn rollback(&self) -> io::Result<()> {
        for entry in &self.backups {
            if entry.backup_path.exists() {
                atomic_fs::atomic_copy(&entry.backup_path, &entry.original_path)?;
            }
        }
        Ok(())
    }

    /// 获取所有备份文件路径
    pub fn backup_paths(&self) -> Vec<PathBuf> {
        self.backups.iter().map(|e| e.backup_path.clone()).collect()
    }

    /// 添加已存在的备份路径（用于追踪外部创建的备份）
    ///
    /// Story 11.15: 用于 apply_takeover 手动创建备份后的追踪
    pub fn add_backup_path(&mut self, original_path: PathBuf) {
        // 不实际创建备份，只是记录路径用于可能的回滚
        // 注意：这里我们不知道备份路径，所以只记录原始路径
        self.backups.push(BackupEntry {
            original_path: original_path.clone(),
            backup_path: original_path, // 占位，实际备份由 apply_takeover 处理
        });
    }

    /// 清理备份文件
    pub fn cleanup(&self) -> io::Result<()> {
        for entry in &self.backups {
            if entry.backup_path.exists() {
                fs::remove_file(&entry.backup_path)?;
            }
        }
        Ok(())
    }
}

impl Drop for BackupManager {
    fn drop(&mut self) {
        if !self.committed {
            // 自动回滚
            let _ = self.rollback();
        }
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 从备份文件回滚 (Story 11.22: 使用原子操作)
///
/// # Arguments
/// * `backup_files` - 备份文件路径列表
///
/// # Returns
/// 成功恢复的文件数量
pub fn rollback_from_backups(backup_files: &[PathBuf]) -> io::Result<usize> {
    let mut restored = 0;

    for backup_path in backup_files {
        if backup_path.exists() {
            // 从备份路径推断原始路径
            let original_path = backup_path
                .to_string_lossy()
                .replace(".mantra-backup", "")
                // 移除时间戳后缀（如果有）
                .split(".mantra-backup.")
                .next()
                .map(PathBuf::from);

            if let Some(original) = original_path {
                // 原子恢复原始文件 (Story 11.22)
                atomic_fs::atomic_copy(backup_path, &original)?;
                restored += 1;
            }
        }
    }

    Ok(restored)
}
