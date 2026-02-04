//! 接管事务管理模块
//!
//! Story 11.20: 全工具自动接管生成 - Task 4
//! Story 11.22: 原子性备份恢复机制 - Task 5
//!
//! 提供事务性接管操作支持，确保多工具接管的原子性。
//! 当任意工具接管失败时，可以回滚所有已执行的操作。
//! 回滚使用原子文件操作确保配置文件不会损坏 (Story 11.22)。

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::services::atomic_fs;
use crate::storage::StorageError;
use crate::storage::Database;

/// 接管操作类型
///
/// Story 11.20: 事务管理 - 记录每个可回滚的操作
#[derive(Debug, Clone)]
pub enum TakeoverOperation {
    /// 创建了备份文件
    BackupCreated {
        /// 备份记录 ID
        backup_id: String,
        /// 备份文件路径
        backup_path: PathBuf,
    },
    /// 修改了配置文件
    ConfigModified {
        /// 原始配置文件路径
        config_path: PathBuf,
        /// 临时备份路径（用于回滚恢复）
        temp_backup: PathBuf,
    },
    /// 创建了 MCP 服务
    ServiceCreated {
        /// 服务 ID
        service_id: String,
    },
    /// 项目关联了服务
    ProjectLinked {
        /// 项目 ID
        project_id: String,
        /// 服务 ID
        service_id: String,
    },
    /// 创建了 Local Scope 备份 (Story 11.21)
    LocalScopeBackupCreated {
        /// 备份记录 ID
        backup_id: String,
    },
}

/// 接管事务
///
/// Story 11.20: 全工具自动接管生成 - AC 5
///
/// 管理多工具接管的事务状态，支持：
/// - 记录所有操作
/// - 提交成功的事务
/// - 回滚失败的事务
#[derive(Debug)]
pub struct TakeoverTransaction {
    /// 事务 ID
    id: String,
    /// 已执行的操作列表（按执行顺序）
    operations: Vec<TakeoverOperation>,
    /// 临时文件列表（提交时清理）
    temp_files: Vec<PathBuf>,
    /// 是否已提交
    committed: bool,
    /// 是否已回滚
    rolled_back: bool,
}

/// 事务回滚结果
#[derive(Debug, Default)]
pub struct RollbackResult {
    /// 成功回滚的操作数
    pub success_count: usize,
    /// 回滚失败的错误列表
    pub errors: Vec<String>,
}

impl TakeoverTransaction {
    /// 创建新事务（begin）
    ///
    /// # Returns
    /// 新的事务实例
    pub fn begin() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operations: Vec::new(),
            temp_files: Vec::new(),
            committed: false,
            rolled_back: false,
        }
    }

    /// 获取事务 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取已记录的操作数量
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// 检查事务是否已提交
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 检查事务是否已回滚
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }

    /// 记录操作
    ///
    /// # Arguments
    /// * `operation` - 要记录的操作
    pub fn add_operation(&mut self, operation: TakeoverOperation) {
        if !self.committed && !self.rolled_back {
            self.operations.push(operation);
        }
    }

    /// 记录创建的备份
    pub fn record_backup_created(&mut self, backup_id: String, backup_path: PathBuf) {
        self.add_operation(TakeoverOperation::BackupCreated {
            backup_id,
            backup_path,
        });
    }

    /// 记录修改的配置文件
    pub fn record_config_modified(&mut self, config_path: PathBuf, temp_backup: PathBuf) {
        // 同时记录临时备份文件，提交时清理
        self.temp_files.push(temp_backup.clone());
        self.add_operation(TakeoverOperation::ConfigModified {
            config_path,
            temp_backup,
        });
    }

    /// 记录创建的服务
    pub fn record_service_created(&mut self, service_id: String) {
        self.add_operation(TakeoverOperation::ServiceCreated { service_id });
    }

    /// 记录项目关联
    pub fn record_project_linked(&mut self, project_id: String, service_id: String) {
        self.add_operation(TakeoverOperation::ProjectLinked {
            project_id,
            service_id,
        });
    }

    /// 记录 Local Scope 备份 (Story 11.21)
    pub fn record_local_scope_backup(&mut self, backup_id: String) {
        self.add_operation(TakeoverOperation::LocalScopeBackupCreated { backup_id });
    }

    /// 添加临时文件（提交时清理）
    pub fn add_temp_file(&mut self, path: PathBuf) {
        self.temp_files.push(path);
    }

    /// 提交事务
    ///
    /// 清理所有临时文件，标记事务为已提交
    pub fn commit(&mut self) -> Result<(), StorageError> {
        if self.rolled_back {
            return Err(StorageError::Conflict(
                "Cannot commit a rolled back transaction".to_string(),
            ));
        }

        if self.committed {
            return Ok(()); // 幂等操作
        }

        // 清理临时文件
        for temp_file in &self.temp_files {
            let _ = fs::remove_file(temp_file); // 忽略删除失败
        }
        self.temp_files.clear();

        self.committed = true;
        Ok(())
    }

    /// 回滚事务
    ///
    /// 按操作的逆序撤销所有已执行的操作
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    ///
    /// # Returns
    /// 回滚结果，包含成功数和错误列表
    pub fn rollback(&mut self, db: &Database) -> RollbackResult {
        if self.committed {
            return RollbackResult {
                success_count: 0,
                errors: vec!["Cannot rollback a committed transaction".to_string()],
            };
        }

        if self.rolled_back {
            return RollbackResult {
                success_count: self.operations.len(),
                errors: vec![],
            }; // 幂等操作
        }

        let mut result = RollbackResult::default();

        // 用于跟踪已删除的服务（避免重复删除）
        let mut deleted_services: HashSet<String> = HashSet::new();

        // 按逆序回滚操作
        for op in self.operations.iter().rev() {
            match op {
                TakeoverOperation::BackupCreated {
                    backup_id,
                    backup_path,
                } => {
                    // 删除备份文件
                    if backup_path.exists() {
                        if let Err(e) = fs::remove_file(backup_path) {
                            result.errors.push(format!(
                                "Failed to remove backup file {}: {}",
                                backup_path.display(),
                                e
                            ));
                        }
                    }
                    // 删除备份记录
                    if let Err(e) = db.delete_takeover_backup(backup_id) {
                        result.errors.push(format!(
                            "Failed to delete backup record {}: {}",
                            backup_id, e
                        ));
                    } else {
                        result.success_count += 1;
                    }
                }

                TakeoverOperation::ConfigModified {
                    config_path,
                    temp_backup,
                } => {
                    // 从临时备份原子恢复原始配置 (Story 11.22)
                    if temp_backup.exists() {
                        // 使用原子复制确保回滚过程中配置文件不会损坏
                        if let Err(e) = atomic_fs::atomic_copy(temp_backup, config_path) {
                            result.errors.push(format!(
                                "Failed to atomically restore config {}: {}",
                                config_path.display(),
                                e
                            ));
                        } else {
                            // 清理临时备份文件
                            let _ = fs::remove_file(temp_backup);
                            result.success_count += 1;
                        }
                    } else {
                        // 临时备份不存在，尝试删除配置文件（如果是新创建的）
                        if config_path.exists() {
                            let _ = fs::remove_file(config_path);
                        }
                        result.success_count += 1;
                    }
                }

                TakeoverOperation::ServiceCreated { service_id } => {
                    // 删除创建的服务
                    if !deleted_services.contains(service_id) {
                        if let Err(e) = db.delete_mcp_service(service_id) {
                            result.errors.push(format!(
                                "Failed to delete service {}: {}",
                                service_id, e
                            ));
                        } else {
                            deleted_services.insert(service_id.clone());
                            result.success_count += 1;
                        }
                    } else {
                        result.success_count += 1; // 已删除，视为成功
                    }
                }

                TakeoverOperation::ProjectLinked {
                    project_id,
                    service_id,
                } => {
                    // 解除项目关联
                    // 注意：如果服务已被删除（CASCADE），关联也会被删除
                    if !deleted_services.contains(service_id) {
                        if let Err(e) = db.unlink_service_from_project(project_id, service_id) {
                            // 如果关联不存在（可能已被 CASCADE 删除），不算错误
                            if !matches!(e, StorageError::NotFound(_)) {
                                result.errors.push(format!(
                                    "Failed to unlink service {} from project {}: {}",
                                    service_id, project_id, e
                                ));
                            }
                        }
                    }
                    result.success_count += 1;
                }

                TakeoverOperation::LocalScopeBackupCreated { backup_id } => {
                    // Story 11.21: 回滚 Local Scope 备份
                    // 恢复备份并删除备份记录
                    if let Err(e) = crate::services::mcp_config::restore_local_scope_takeover(db, backup_id) {
                        result.errors.push(format!(
                            "Failed to restore local scope backup {}: {}",
                            backup_id, e
                        ));
                    } else {
                        result.success_count += 1;
                    }
                }
            }
        }

        // 清理临时文件
        for temp_file in &self.temp_files {
            let _ = fs::remove_file(temp_file);
        }
        self.temp_files.clear();

        self.rolled_back = true;
        result
    }

    /// 获取所有操作的引用
    pub fn operations(&self) -> &[TakeoverOperation] {
        &self.operations
    }
}

#[cfg(test)]
mod tests;
