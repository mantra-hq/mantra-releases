//! 导入执行器
//!
//! Story 11.3: 配置导入与接管
//! Story 11.15: MCP 接管流程重构
//! Story 11.16: 项目级配置接管

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::backup::BackupManager;
use super::parsers::generate_shadow_config;
use super::types::*;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpServiceSource, TakeoverBackup, TakeoverScope, ToolType,
};
use crate::services::mcp_adapters::{ConfigScope, GatewayInjectionConfig, ToolAdapterRegistry};
use crate::services::EnvManager;
use crate::storage::{Database, StorageError};

/// 导入执行器
pub struct ImportExecutor<'a> {
    db: &'a Database,
    env_manager: &'a EnvManager,
    pub(super) backup_manager: BackupManager,
}

impl<'a> ImportExecutor<'a> {
    /// 创建导入执行器
    pub fn new(db: &'a Database, env_manager: &'a EnvManager) -> Self {
        Self {
            db,
            env_manager,
            backup_manager: BackupManager::new(),
        }
    }

    /// 执行导入
    ///
    /// # Arguments
    /// * `preview` - 导入预览
    /// * `request` - 导入请求
    ///
    /// # Returns
    /// 导入结果
    pub fn execute(
        mut self,
        preview: &ImportPreview,
        request: &ImportRequest,
    ) -> Result<ImportResult, StorageError> {
        let mut imported_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        let mut imported_service_ids = Vec::new();
        let mut shadow_configs = Vec::new();

        // 1. 存储环境变量
        for (name, value) in &request.env_var_values {
            if let Err(e) = self.db.set_env_variable(
                self.env_manager,
                name,
                value,
                Some("Imported from MCP config"),
            ) {
                errors.push(format!("Failed to set env var {}: {}", name, e));
            }
        }

        // 2. 导入新服务
        for service in &preview.new_services {
            if request.services_to_import.contains(&service.name) {
                match self.import_service(service) {
                    Ok(id) => {
                        imported_count += 1;
                        imported_service_ids.push(id);
                    }
                    Err(e) => {
                        errors.push(format!("Failed to import {}: {}", service.name, e));
                    }
                }
            } else {
                skipped_count += 1;
            }
        }

        // 3. 处理冲突
        for conflict in &preview.conflicts {
            if let Some(resolution) = request.conflict_resolutions.get(&conflict.name) {
                match self.resolve_conflict(conflict, resolution) {
                    Ok(Some(id)) => {
                        imported_count += 1;
                        imported_service_ids.push(id);
                    }
                    Ok(None) => {
                        skipped_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!(
                            "Failed to resolve conflict for {}: {}",
                            conflict.name, e
                        ));
                    }
                }
            } else {
                skipped_count += 1;
            }
        }

        // 4. 强制接管所有检测到的工具配置 (Story 11.15, 11.16)
        let mut takeover_backup_ids = Vec::new();

        // 只有当有需要接管的配置时才需要 gateway_url
        let has_configs_to_takeover = preview.configs.iter().any(|c| !c.services.is_empty());
        if has_configs_to_takeover {
            if let Some(gateway_url) = &request.gateway_url {
                let gateway_token = request.gateway_token.as_deref();

                // Story 11.16: 遍历每个配置文件，根据其 scope 进行接管
                for config in &preview.configs {
                    if config.services.is_empty() {
                        continue;
                    }

                    if let Some(tool_type) = ToolType::from_adapter_id(&config.adapter_id) {
                        // 确定 scope 和 project_path
                        let (scope, project_path) = match &config.scope {
                            Some(ConfigScope::Project) => {
                                let proj_path = config.path.parent().and_then(|p| {
                                    let path_str = p.to_string_lossy();
                                    if path_str.contains(".cursor")
                                        || path_str.contains(".codex")
                                        || path_str.contains(".gemini")
                                    {
                                        p.parent().map(|pp| pp.to_path_buf())
                                    } else {
                                        Some(p.to_path_buf())
                                    }
                                });
                                (TakeoverScope::Project, proj_path)
                            }
                            _ => (TakeoverScope::User, None),
                        };

                        // 使用实际检测到的配置文件路径
                        let config_path = &config.path;

                        match self.apply_takeover(
                            config_path,
                            &config.adapter_id,
                            gateway_url,
                            gateway_token,
                            &tool_type,
                            scope,
                            project_path,
                        ) {
                            Ok(backup_id) => {
                                shadow_configs.push(config_path.clone());
                                takeover_backup_ids.push(backup_id);
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "Failed to takeover {} config at {:?}: {}",
                                    tool_type.display_name(),
                                    config_path,
                                    e
                                ));
                            }
                        }
                    }
                }
            } else {
                errors.push("Gateway URL required for MCP import with configurations".to_string());
            }
        }

        // 5. 提交备份（成功则不回滚）
        if errors.is_empty() {
            self.backup_manager.commit();
        }

        Ok(ImportResult {
            imported_count,
            skipped_count,
            backup_files: self.backup_manager.backup_paths(),
            shadow_configs,
            errors,
            imported_service_ids,
            takeover_backup_ids,
        })
    }

    /// 导入单个服务
    fn import_service(&self, service: &DetectedService) -> Result<String, StorageError> {
        let request = CreateMcpServiceRequest {
            name: service.name.clone(),
            transport_type: service.transport_type.clone(),
            command: service.command.clone(),
            args: service.args.clone(),
            env: service
                .env
                .as_ref()
                .map(|e| serde_json::to_value(e).unwrap()),
            url: service.url.clone(),
            headers: service.headers.clone(),
            source: McpServiceSource::Imported,
            source_file: Some(service.source_file.to_string_lossy().to_string()),
        };
        let created = self.db.create_mcp_service(&request)?;
        Ok(created.id)
    }

    /// 解决冲突
    fn resolve_conflict(
        &self,
        conflict: &ServiceConflict,
        resolution: &ConflictResolution,
    ) -> Result<Option<String>, StorageError> {
        match resolution {
            ConflictResolution::Keep => {
                // 保留现有，不导入
                Ok(None)
            }
            ConflictResolution::Replace(idx) => {
                // 替换现有服务
                if let Some(candidate) = conflict.candidates.get(*idx) {
                    if let Some(existing) = &conflict.existing {
                        // 删除现有服务
                        self.db.delete_mcp_service(&existing.id)?;
                    }
                    // 导入新服务
                    let id = self.import_service(candidate)?;
                    Ok(Some(id))
                } else {
                    Err(StorageError::InvalidInput(format!(
                        "Invalid candidate index: {}",
                        idx
                    )))
                }
            }
            ConflictResolution::Rename(new_name) => {
                // 使用新名称导入第一个候选
                if let Some(candidate) = conflict.candidates.first() {
                    let mut renamed = candidate.clone();
                    renamed.name = new_name.clone();
                    let id = self.import_service(&renamed)?;
                    Ok(Some(id))
                } else {
                    Err(StorageError::InvalidInput(
                        "No candidates to rename".to_string(),
                    ))
                }
            }
            ConflictResolution::Skip => {
                // 跳过
                Ok(None)
            }
        }
    }

    /// 应用影子模式 (旧版，向后兼容)
    #[allow(deprecated, dead_code)]
    fn apply_shadow_mode(
        &mut self,
        path: &Path,
        source: &ConfigSource,
        gateway_url: &str,
    ) -> io::Result<()> {
        // 备份原文件
        self.backup_manager.backup(path)?;

        // 生成影子配置
        let shadow_content = generate_shadow_config(source, gateway_url);

        // 写入影子配置
        fs::write(path, shadow_content)?;

        Ok(())
    }

    /// 应用影子模式 (新版，使用适配器架构)
    ///
    /// Story 11.8: 使用 HTTP Transport + Authorization Header
    ///
    /// Note: Story 11.15 后使用 apply_takeover() 替代
    #[allow(dead_code)]
    fn apply_shadow_mode_v2(
        &mut self,
        path: &Path,
        adapter_id: &str,
        gateway_url: &str,
        gateway_token: Option<&str>,
    ) -> io::Result<()> {
        // 备份原文件
        self.backup_manager.backup(path)?;

        // 读取原始内容
        let original_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 使用新的适配器架构生成影子配置
        let registry = ToolAdapterRegistry::new();
        let token = gateway_token.unwrap_or("");
        let shadow_content = if let Some(adapter) = registry.get(adapter_id) {
            let config = GatewayInjectionConfig::new(gateway_url, token);
            adapter
                .inject_gateway(&original_content, &config)
                .unwrap_or_else(|_| original_content.clone())
        } else {
            // 回退到旧的生成方式
            serde_json::json!({
                "mcpServers": {
                    "mantra-gateway": {
                        "url": gateway_url
                    }
                }
            })
            .to_string()
        };

        // 写入影子配置
        fs::write(path, shadow_content)?;

        Ok(())
    }

    /// 执行配置接管 (Story 11.15)
    ///
    /// 接管配置文件：
    /// 1. 备份原始文件
    /// 2. 写入 Gateway 配置
    /// 3. 将备份记录存储到数据库
    pub(super) fn apply_takeover(
        &mut self,
        path: &Path,
        adapter_id: &str,
        gateway_url: &str,
        gateway_token: Option<&str>,
        tool_type: &ToolType,
        scope: TakeoverScope,
        project_path: Option<PathBuf>,
    ) -> Result<String, StorageError> {
        // 1. 检查该配置文件是否已有活跃的接管记录（按 original_path 判断）
        let path_str = path.to_string_lossy().to_string();
        if let Ok(Some(existing)) = self.db.get_active_takeover_by_original_path(&path_str) {
            // 已经接管过，更新配置文件但返回已有记录的 ID（不创建新备份记录）
            self.update_gateway_config(path, adapter_id, gateway_url, gateway_token)?;
            return Ok(existing.id);
        }

        // 2. 生成备份文件路径
        let backup_path = self.generate_backup_path(path);

        // 3. 备份原文件（仅当文件存在）
        if path.exists() {
            fs::copy(path, &backup_path).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to backup file: {}", e))
            })?;
        }

        // 4. 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to create directory: {}", e))
            })?;
        }

        // 5. 读取原始内容（如果存在）
        let original_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 6. 使用适配器生成新配置
        let registry = ToolAdapterRegistry::new();
        let token = gateway_token.unwrap_or("");
        let new_content = if let Some(adapter) = registry.get(adapter_id) {
            let config = GatewayInjectionConfig::new(gateway_url, token);
            adapter.inject_gateway(&original_content, &config).map_err(
                |e| StorageError::InvalidInput(format!("Failed to inject gateway: {}", e)),
            )?
        } else {
            // 回退到默认 JSON 格式
            serde_json::json!({
                "mcpServers": {
                    "mantra-gateway": {
                        "url": gateway_url,
                        "headers": {
                            "Authorization": format!("Bearer {}", token)
                        }
                    }
                }
            })
            .to_string()
        };

        // 7. 写入新配置
        fs::write(path, new_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to write config file: {}", e))
        })?;

        // 8. 创建备份记录并存储到数据库 (Story 11.16: 使用 new_with_scope)
        let backup = TakeoverBackup::new_with_scope(
            tool_type.clone(),
            path.to_path_buf(),
            backup_path,
            scope,
            project_path,
        );
        let backup_id = backup.id.clone();
        self.db.create_takeover_backup(&backup)?;

        // 9. 添加到备份管理器（用于可能的回滚）
        self.backup_manager.add_backup_path(path.to_path_buf());

        Ok(backup_id)
    }

    /// 更新 Gateway 配置（不创建备份）
    ///
    /// 用于已有活跃接管记录时更新配置文件内容
    fn update_gateway_config(
        &self,
        path: &Path,
        adapter_id: &str,
        gateway_url: &str,
        gateway_token: Option<&str>,
    ) -> Result<(), StorageError> {
        use crate::services::mcp_adapters::GATEWAY_SERVICE_NAME;

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to create directory: {}", e))
            })?;
        }

        // 读取原始内容（如果存在）
        let original_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 使用适配器生成新配置
        let registry = ToolAdapterRegistry::new();
        let token = gateway_token.unwrap_or("");
        let new_content = if let Some(adapter) = registry.get(adapter_id) {
            let config = GatewayInjectionConfig::new(gateway_url, token);
            adapter.inject_gateway(&original_content, &config).map_err(
                |e| StorageError::InvalidInput(format!("Failed to inject gateway: {}", e)),
            )?
        } else {
            // 回退到默认 JSON 格式
            serde_json::json!({
                "mcpServers": {
                    GATEWAY_SERVICE_NAME: {
                        "url": gateway_url,
                        "headers": {
                            "Authorization": format!("Bearer {}", token)
                        }
                    }
                }
            })
            .to_string()
        };

        // 写入新配置
        fs::write(path, new_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// 生成备份文件路径
    ///
    /// 格式: <原路径>.mantra-backup.<时间戳>
    fn generate_backup_path(&self, path: &Path) -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.mantra-backup.{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            timestamp
        );
        path.parent().unwrap_or(Path::new(".")).join(backup_name)
    }
}
