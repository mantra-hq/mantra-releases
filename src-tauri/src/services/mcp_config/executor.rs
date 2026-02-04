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
use super::takeover::cleanup_old_backups;
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

    /// 执行配置接管 (Story 11.15, 11.22)
    ///
    /// 接管配置文件（使用原子操作 Story 11.22）：
    /// 1. 原子备份原始文件（tempfile + hash verify + rename）
    /// 2. 原子写入 Gateway 配置
    /// 3. 将备份记录（含 hash）存储到数据库
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
        use crate::services::atomic_fs;

        // 1. 检查该配置文件是否已有活跃的接管记录（按 original_path 判断）
        let path_str = path.to_string_lossy().to_string();
        if let Ok(Some(existing)) = self.db.get_active_takeover_by_original_path(&path_str) {
            // 已经接管过，更新配置文件但返回已有记录的 ID（不创建新备份记录）
            self.update_gateway_config(path, adapter_id, gateway_url, gateway_token)?;
            return Ok(existing.id);
        }

        // 2. 生成备份文件路径
        let backup_path = self.generate_backup_path(path);

        // 3. 原子备份原文件（仅当文件存在）(Story 11.22)
        let backup_hash = if path.exists() {
            let hash = atomic_fs::atomic_copy(path, &backup_path).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to atomic backup file: {}", e))
            })?;
            Some(hash)
        } else {
            None
        };

        // 4. 读取原始内容（如果存在）
        let original_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 5. 使用适配器生成新配置
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

        // 6. 原子写入新配置 (Story 11.22)
        atomic_fs::atomic_write_str(path, &new_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to atomic write config file: {}", e))
        })?;

        // Story 11.23: 保存清理用的 project_path 字符串（在所有权转移前）
        let project_path_str_for_cleanup = project_path.as_ref().map(|p| p.to_string_lossy().to_string());

        // 7. 创建备份记录并存储到数据库 (Story 11.16, 11.22: 含 hash)
        let backup = if let Some(hash) = backup_hash {
            TakeoverBackup::new_with_hash(
                tool_type.clone(),
                path.to_path_buf(),
                backup_path,
                scope.clone(),
                project_path,
                hash,
            )
        } else {
            TakeoverBackup::new_with_scope(
                tool_type.clone(),
                path.to_path_buf(),
                backup_path,
                scope.clone(),
                project_path,
            )
        };
        let backup_id = backup.id.clone();
        self.db.create_takeover_backup(&backup)?;

        // 8.5. Story 11.23: 自动清理旧备份（清理失败不影响备份结果）
        if let Err(e) = cleanup_old_backups(self.db, &tool_type, &scope, project_path_str_for_cleanup.as_deref(), 5) {
            eprintln!("[Backup] Warning: Failed to cleanup old backups: {}", e);
        }

        // 9. 添加到备份管理器（用于可能的回滚）
        self.backup_manager.add_backup_path(path.to_path_buf());

        Ok(backup_id)
    }

    /// 更新 Gateway 配置（不创建备份）(Story 11.22)
    ///
    /// 用于已有活跃接管记录时更新配置文件内容
    fn update_gateway_config(
        &self,
        path: &Path,
        adapter_id: &str,
        gateway_url: &str,
        gateway_token: Option<&str>,
    ) -> Result<(), StorageError> {
        use crate::services::atomic_fs;
        use crate::services::mcp_adapters::GATEWAY_SERVICE_NAME;

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

        // 原子写入新配置 (Story 11.22)
        atomic_fs::atomic_write_str(path, &new_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to atomic write config file: {}", e))
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

    // ===== Story 11.21: Local Scope 备份接管 =====

    /// 执行 Local Scope 配置接管 (Story 11.21 - Task 4)
    ///
    /// Local Scope 是 Claude Code 特有的功能，配置存储在 ~/.claude.json 的
    /// `projects.{path}.mcpServers` 中。接管流程：
    ///
    /// 1. 提取指定项目的 mcpServers JSON 片段
    /// 2. 保存到独立的备份文件
    /// 3. 创建 TakeoverBackup 记录（scope=Local）
    /// 4. 清空该项目的 mcpServers（保留项目配置的其他字段）
    ///
    /// # Arguments
    /// * `user_config_path` - ~/.claude.json 的路径
    /// * `project_path` - 要接管的项目路径（projects 的 key）
    ///
    /// # Returns
    /// 创建的备份 ID
    pub(super) fn apply_local_scope_takeover(
        &mut self,
        user_config_path: &Path,
        project_path: &str,
    ) -> Result<String, StorageError> {
        use crate::services::atomic_fs;
        use crate::services::mcp_adapters::ClaudeAdapter;

        let adapter = ClaudeAdapter;

        // 1. 检查是否已有该项目的 local scope 备份
        if let Ok(Some(existing)) = self.db.get_active_takeover_by_tool_and_scope(
            &ToolType::ClaudeCode,
            &TakeoverScope::Local,
            Some(project_path),
        ) {
            // 已存在活跃的 local scope 备份，直接返回已有记录的 ID
            return Ok(existing.id);
        }

        // 2. 读取配置文件内容
        let config_content = if user_config_path.exists() {
            fs::read_to_string(user_config_path).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to read config file: {}", e))
            })?
        } else {
            return Err(StorageError::InvalidInput(format!(
                "Config file not found: {:?}",
                user_config_path
            )));
        };

        // 3. 提取该项目的 mcpServers 用于备份
        let backup_mcp_servers = adapter
            .extract_local_scope_backup(&config_content, project_path)
            .map_err(|e| {
                StorageError::InvalidInput(format!("Failed to extract local scope backup: {}", e))
            })?;

        // 如果 mcpServers 为空，无需备份
        if backup_mcp_servers == serde_json::json!({}) {
            return Err(StorageError::InvalidInput(format!(
                "No mcpServers found for project: {}",
                project_path
            )));
        }

        // 4. 生成备份文件路径
        //    格式: ~/.mantra/backups/local-scope/<project-hash>.json
        let backup_path = self.generate_local_scope_backup_path(project_path);

        // 5. 保存备份内容（JSON 片段）- 使用原子写入 (Story 11.22)
        let backup_content = serde_json::to_string_pretty(&backup_mcp_servers).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to serialize backup content: {}", e))
        })?;
        let backup_hash = atomic_fs::atomic_write_str(&backup_path, &backup_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to atomic write backup file: {}", e))
        })?;

        // 6. 清空该项目的 mcpServers
        let new_content = adapter
            .clear_local_scope_for_project(&config_content, project_path)
            .map_err(|e| {
                StorageError::InvalidInput(format!("Failed to clear local scope: {}", e))
            })?;

        // 7. 原子写回配置文件 (Story 11.22)
        atomic_fs::atomic_write_str(user_config_path, &new_content).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to atomic write config file: {}", e))
        })?;

        // 8. 创建备份记录并存储到数据库 (含 hash)
        let backup = TakeoverBackup::new_with_hash(
            ToolType::ClaudeCode,
            user_config_path.to_path_buf(),
            backup_path.clone(),
            TakeoverScope::Local,
            Some(PathBuf::from(project_path)),
            backup_hash,
        );
        let backup_id = backup.id.clone();
        self.db.create_takeover_backup(&backup)?;

        // 8.5. Story 11.23: 自动清理旧备份（清理失败不影响备份结果）
        if let Err(e) = cleanup_old_backups(self.db, &ToolType::ClaudeCode, &TakeoverScope::Local, Some(project_path), 5) {
            eprintln!("[Backup] Warning: Failed to cleanup old backups for local scope: {}", e);
        }

        // 9. 添加到备份管理器（用于可能的回滚）
        self.backup_manager.add_backup_path(backup_path);

        Ok(backup_id)
    }

    /// 批量执行 Local Scope 接管 (Story 11.21 - Task 5)
    ///
    /// 接管 Claude Code ~/.claude.json 中所有 local scope 项目
    ///
    /// # Arguments
    /// * `user_config_path` - ~/.claude.json 的路径
    /// * `project_paths` - 要接管的项目路径列表
    ///
    /// # Returns
    /// (成功的备份 ID 列表, 失败的项目路径和错误)
    pub(super) fn apply_all_local_scope_takeovers(
        &mut self,
        user_config_path: &Path,
        project_paths: &[String],
    ) -> (Vec<String>, Vec<(String, String)>) {
        let mut backup_ids = Vec::new();
        let mut errors = Vec::new();

        for project_path in project_paths {
            match self.apply_local_scope_takeover(user_config_path, project_path) {
                Ok(backup_id) => backup_ids.push(backup_id),
                Err(e) => errors.push((project_path.clone(), e.to_string())),
            }
        }

        (backup_ids, errors)
    }

    /// 生成 Local Scope 备份文件路径
    ///
    /// 格式: ~/.mantra/backups/local-scope/<project-hash>-<timestamp>.json
    fn generate_local_scope_backup_path(&self, project_path: &str) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // 生成项目路径的 hash（避免路径中的特殊字符）
        let mut hasher = DefaultHasher::new();
        project_path.hash(&mut hasher);
        let hash = hasher.finish();

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{:x}-{}.json", hash, timestamp);

        // 备份目录: ~/.mantra/backups/local-scope/
        let backup_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mantra")
            .join("backups")
            .join("local-scope");

        backup_dir.join(backup_name)
    }

    /// 测试辅助方法：暴露 generate_local_scope_backup_path 用于测试
    #[cfg(test)]
    pub fn generate_local_scope_backup_path_for_test(&self, project_path: &str) -> PathBuf {
        self.generate_local_scope_backup_path(project_path)
    }
}
