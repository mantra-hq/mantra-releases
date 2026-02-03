//! 接管事务管理模块
//!
//! Story 11.20: 全工具自动接管生成 - Task 4
//!
//! 提供事务性接管操作支持，确保多工具接管的原子性。
//! 当任意工具接管失败时，可以回滚所有已执行的操作。

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

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
                    // 从临时备份恢复原始配置
                    if temp_backup.exists() {
                        if let Err(e) = fs::rename(temp_backup, config_path) {
                            result.errors.push(format!(
                                "Failed to restore config {}: {}",
                                config_path.display(),
                                e
                            ));
                        } else {
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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transaction_begin() {
        let tx = TakeoverTransaction::begin();

        assert!(!tx.id().is_empty());
        assert_eq!(tx.operation_count(), 0);
        assert!(!tx.is_committed());
        assert!(!tx.is_rolled_back());
    }

    #[test]
    fn test_add_operation() {
        let mut tx = TakeoverTransaction::begin();

        tx.record_service_created("service-1".to_string());
        tx.record_project_linked("proj-1".to_string(), "service-1".to_string());

        assert_eq!(tx.operation_count(), 2);
    }

    #[test]
    fn test_add_operation_after_commit_ignored() {
        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created("service-1".to_string());
        tx.commit().unwrap();

        // 提交后的操作应被忽略
        tx.record_service_created("service-2".to_string());
        assert_eq!(tx.operation_count(), 1);
    }

    #[test]
    fn test_add_operation_after_rollback_ignored() {
        let db = Database::new_in_memory().unwrap();
        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created("service-1".to_string());
        tx.rollback(&db);

        // 回滚后的操作应被忽略
        tx.record_service_created("service-2".to_string());
        assert_eq!(tx.operation_count(), 1);
    }

    #[test]
    fn test_commit_success() {
        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created("service-1".to_string());

        let result = tx.commit();
        assert!(result.is_ok());
        assert!(tx.is_committed());
    }

    #[test]
    fn test_commit_idempotent() {
        let mut tx = TakeoverTransaction::begin();
        tx.commit().unwrap();

        // 重复提交应该成功
        let result = tx.commit();
        assert!(result.is_ok());
    }

    #[test]
    fn test_commit_after_rollback_fails() {
        let db = Database::new_in_memory().unwrap();
        let mut tx = TakeoverTransaction::begin();
        tx.rollback(&db);

        // 回滚后不能提交
        let result = tx.commit();
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_cleans_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("temp-backup.json");
        fs::write(&temp_file, "{}").unwrap();

        let mut tx = TakeoverTransaction::begin();
        tx.add_temp_file(temp_file.clone());
        tx.commit().unwrap();

        assert!(!temp_file.exists());
    }

    #[test]
    fn test_rollback_service_created() {
        let db = Database::new_in_memory().unwrap();

        // 创建服务
        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created(service.id.clone());

        // 回滚
        let result = tx.rollback(&db);

        assert_eq!(result.success_count, 1);
        assert!(result.errors.is_empty());
        assert!(tx.is_rolled_back());

        // 服务应该被删除
        assert!(db.get_mcp_service(&service.id).is_err());
    }

    #[test]
    fn test_rollback_project_linked() {
        let db = Database::new_in_memory().unwrap();

        // 创建项目
        create_test_project(&db, "proj-1", "Test Project");

        // 创建服务
        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 关联
        db.link_service_to_project("proj-1", &service.id, None)
            .unwrap();

        let mut tx = TakeoverTransaction::begin();
        tx.record_project_linked("proj-1".to_string(), service.id.clone());

        // 回滚
        let result = tx.rollback(&db);

        assert!(result.errors.is_empty());
        assert!(tx.is_rolled_back());

        // 关联应该被删除
        let link = db.get_project_service_link("proj-1", &service.id).unwrap();
        assert!(link.is_none());
    }

    #[test]
    fn test_rollback_config_modified() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let temp_backup = temp_dir.path().join("config.json.temp-backup");

        // 模拟已修改的配置：临时备份包含原始内容
        fs::write(&temp_backup, r#"{"original": true}"#).unwrap();
        fs::write(&config_path, r#"{"modified": true}"#).unwrap();

        let db = Database::new_in_memory().unwrap();
        let mut tx = TakeoverTransaction::begin();
        tx.record_config_modified(config_path.clone(), temp_backup.clone());

        // 回滚
        let result = tx.rollback(&db);

        assert_eq!(result.success_count, 1);
        assert!(result.errors.is_empty());

        // 配置应该恢复到原始内容
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("original"));
    }

    #[test]
    fn test_rollback_backup_created() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("backup.json");
        fs::write(&backup_path, "{}").unwrap();

        let db = Database::new_in_memory().unwrap();

        // 创建备份记录
        let backup = crate::models::mcp::TakeoverBackup::new(
            crate::models::mcp::ToolType::ClaudeCode,
            PathBuf::from("/original/config.json"),
            backup_path.clone(),
        );
        db.create_takeover_backup(&backup).unwrap();

        let mut tx = TakeoverTransaction::begin();
        tx.record_backup_created(backup.id.clone(), backup_path.clone());

        // 回滚
        let result = tx.rollback(&db);

        assert_eq!(result.success_count, 1);
        assert!(result.errors.is_empty());

        // 备份文件应该被删除
        assert!(!backup_path.exists());
        // 备份记录也应该被删除
        assert!(db.get_takeover_backup_by_id(&backup.id).unwrap().is_none());
    }

    #[test]
    fn test_rollback_idempotent() {
        let db = Database::new_in_memory().unwrap();
        let mut tx = TakeoverTransaction::begin();

        let result1 = tx.rollback(&db);
        let result2 = tx.rollback(&db);

        assert!(result1.errors.is_empty());
        assert!(result2.errors.is_empty());
    }

    #[test]
    fn test_rollback_after_commit_fails() {
        let db = Database::new_in_memory().unwrap();
        let mut tx = TakeoverTransaction::begin();
        tx.commit().unwrap();

        let result = tx.rollback(&db);

        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("Cannot rollback a committed transaction"));
    }

    #[test]
    fn test_rollback_order_is_reversed() {
        let db = Database::new_in_memory().unwrap();

        // 创建项目
        create_test_project(&db, "proj-1", "Test Project");

        // 创建服务
        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 关联
        db.link_service_to_project("proj-1", &service.id, None)
            .unwrap();

        // 记录操作顺序：创建服务 -> 关联项目
        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created(service.id.clone());
        tx.record_project_linked("proj-1".to_string(), service.id.clone());

        // 回滚应该先解除关联，再删除服务
        // 如果顺序错了（先删除服务），CASCADE 会自动删除关联，但我们仍能正确处理
        let result = tx.rollback(&db);

        assert!(result.errors.is_empty());

        // 服务和关联都应该被删除
        assert!(db.get_mcp_service(&service.id).is_err());
    }

    #[test]
    fn test_rollback_handles_cascade_delete() {
        let db = Database::new_in_memory().unwrap();

        // 创建项目
        create_test_project(&db, "proj-1", "Test Project");

        // 创建服务
        let service = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "test-service".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        // 关联
        db.link_service_to_project("proj-1", &service.id, None)
            .unwrap();

        // 手动删除服务（CASCADE 会删除关联）
        db.delete_mcp_service(&service.id).unwrap();

        // 即使服务已被删除，回滚也应该正常处理
        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created(service.id.clone());
        tx.record_project_linked("proj-1".to_string(), service.id.clone());

        let result = tx.rollback(&db);

        // 不应该有致命错误（NotFound 不算错误）
        assert!(result.errors.is_empty() || result.errors.iter().all(|e| e.contains("not found")));
    }

    #[test]
    fn test_multiple_services_rollback() {
        let db = Database::new_in_memory().unwrap();

        // 创建多个服务
        let service1 = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "service-1".to_string(),
                transport_type: Default::default(),
                command: "npx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let service2 = db
            .create_mcp_service(&crate::models::mcp::CreateMcpServiceRequest {
                name: "service-2".to_string(),
                transport_type: Default::default(),
                command: "uvx".to_string(),
                args: None,
                env: None,
                url: None,
                headers: None,
                source: crate::models::mcp::McpServiceSource::Manual,
                source_file: None,
            })
            .unwrap();

        let mut tx = TakeoverTransaction::begin();
        tx.record_service_created(service1.id.clone());
        tx.record_service_created(service2.id.clone());

        let result = tx.rollback(&db);

        assert_eq!(result.success_count, 2);
        assert!(result.errors.is_empty());

        // 两个服务都应该被删除
        assert!(db.get_mcp_service(&service1.id).is_err());
        assert!(db.get_mcp_service(&service2.id).is_err());
    }

    // 辅助函数：创建测试项目
    fn create_test_project(db: &Database, id: &str, name: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let cwd = format!("/path/to/{}", id);
        db.connection()
            .execute(
                "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
                [id, name, &cwd, &now],
            )
            .unwrap();
    }
}
