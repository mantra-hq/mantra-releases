//! 接管恢复和执行引擎
//!
//! Story 11.15: MCP 接管流程重构
//! Story 11.19: 智能接管执行引擎
//! Story 11.20: 全工具自动接管生成
//! Story 11.22: 原子性备份恢复机制

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::services::atomic_fs;

use super::executor::ImportExecutor;
use super::scanner::scan_mcp_configs;
use super::types::*;
use crate::models::mcp::{
    CreateMcpServiceRequest, McpServiceSource, TakeoverBackup, TakeoverDecision,
    TakeoverDecisionOption, TakeoverPreview, TakeoverScope, TakeoverStatus, ToolType,
    UpdateMcpServiceRequest,
};
use crate::services::mcp_adapters::{ConfigScope, GatewayInjectionConfig, ToolAdapterRegistry};
use crate::services::takeover_transaction::TakeoverTransaction;
use crate::services::EnvManager;
use crate::storage::{Database, StorageError};

// ===== Story 11.15: 接管恢复功能 =====

/// 恢复 MCP 配置接管 (Story 11.22: 原子性恢复)
///
/// Story 11.15: MCP 接管流程重构 - AC 5
/// Story 11.22: 原子性备份恢复机制 - AC 2
///
/// 从备份文件恢复原始配置，使用原子操作确保：
/// 1. 恢复前验证备份文件 hash 完整性
/// 2. 使用临时文件 + 原子重命名
/// 3. 失败时保持原文件不变
pub fn restore_mcp_takeover(
    db: &Database,
    backup_id: &str,
) -> Result<TakeoverBackup, StorageError> {
    // 1. 获取备份记录
    let backup = db
        .get_takeover_backup_by_id(backup_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Takeover backup not found: {}", backup_id)))?;

    // 2. 检查是否可以恢复
    if backup.status != TakeoverStatus::Active {
        return Err(StorageError::InvalidInput(format!(
            "Backup {} is already restored",
            backup_id
        )));
    }

    // 3. 检查备份文件是否存在
    if !backup.backup_path.exists() {
        return Err(StorageError::InvalidInput(format!(
            "Backup file not found: {:?}",
            backup.backup_path
        )));
    }

    // 4. 验证备份文件完整性 (Story 11.22 - AC2 step 1)
    if let Some(expected_hash) = &backup.backup_hash {
        let actual_hash = atomic_fs::calculate_file_hash(&backup.backup_path).map_err(|e| {
            StorageError::InvalidInput(format!(
                "Failed to calculate backup file hash: {}", e
            ))
        })?;
        if &actual_hash != expected_hash {
            return Err(StorageError::InvalidInput(format!(
                "Backup file integrity check failed: expected hash {}, got {}",
                expected_hash, actual_hash
            )));
        }
    }

    // 5. 原子恢复: 使用 atomic_copy 确保临时文件 + hash 验证 + 原子重命名
    //    如果失败，原文件保持不变 (Story 11.22 - AC2 steps 2-4)
    atomic_fs::atomic_copy(&backup.backup_path, &backup.original_path).map_err(|e| {
        StorageError::InvalidInput(format!("Failed to atomically restore file: {}", e))
    })?;

    // 6. 更新数据库记录状态
    db.update_backup_status_restored(backup_id)?;

    // 7. 返回更新后的备份记录
    db.get_takeover_backup_by_id(backup_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Backup not found after update: {}", backup_id)))
}

/// 恢复指定工具类型的 MCP 配置接管
///
/// Story 11.15: MCP 接管流程重构 - AC 5
pub fn restore_mcp_takeover_by_tool(
    db: &Database,
    tool_type: &ToolType,
) -> Result<Option<TakeoverBackup>, StorageError> {
    // 1. 获取该工具类型的活跃备份
    let backup = match db.get_active_takeover_by_tool(tool_type)? {
        Some(b) => b,
        None => return Ok(None),
    };

    // 2. 恢复
    let restored = restore_mcp_takeover(db, &backup.id)?;
    Ok(Some(restored))
}

/// 获取所有活跃的接管状态
///
/// Story 11.15: MCP 接管流程重构 - AC 5
pub fn get_takeover_status(db: &Database) -> Result<Vec<TakeoverBackup>, StorageError> {
    db.get_takeover_backups(Some(TakeoverStatus::Active))
}

/// 同步所有活跃接管配置中的 Gateway URL 和 Token
///
/// 当 Gateway 启动或重启后端口或 token 可能变化时调用此函数，
/// 更新所有活跃接管的配置文件，确保工具能够正确连接到 Gateway。
pub fn sync_active_takeovers(
    db: &Database,
    gateway_url: &str,
    gateway_token: &str,
) -> Result<SyncTakeoverResult, StorageError> {
    let mut result = SyncTakeoverResult {
        synced_count: 0,
        failed_count: 0,
        errors: Vec::new(),
    };

    // 获取所有活跃的接管记录
    let active_backups = db.get_takeover_backups(Some(TakeoverStatus::Active))?;

    if active_backups.is_empty() {
        return Ok(result);
    }

    let registry = ToolAdapterRegistry::new();
    let injection_config = GatewayInjectionConfig::new(gateway_url, gateway_token);

    for backup in active_backups {
        let adapter_id = backup.tool_type.to_adapter_id();

        // 获取对应的适配器
        let Some(adapter) = registry.get(adapter_id) else {
            result.failed_count += 1;
            result.errors.push(format!(
                "Unknown adapter for tool type: {}",
                backup.tool_type.display_name()
            ));
            continue;
        };

        // 读取当前配置文件内容
        let original_content = match fs::read_to_string(&backup.original_path) {
            Ok(content) => content,
            Err(e) => {
                result.failed_count += 1;
                result.errors.push(format!(
                    "Failed to read {}: {}",
                    backup.original_path.display(),
                    e
                ));
                continue;
            }
        };

        // 使用适配器注入新的 Gateway 配置
        let new_content = match adapter.inject_gateway(&original_content, &injection_config) {
            Ok(content) => content,
            Err(e) => {
                result.failed_count += 1;
                result.errors.push(format!(
                    "Failed to inject gateway for {}: {}",
                    backup.original_path.display(),
                    e
                ));
                continue;
            }
        };

        // 写回配置文件 (Story 11.22: 使用原子写入)
        if let Err(e) = atomic_fs::atomic_write_str(&backup.original_path, &new_content) {
            result.failed_count += 1;
            result.errors.push(format!(
                "Failed to atomic write {}: {}",
                backup.original_path.display(),
                e
            ));
            continue;
        }

        result.synced_count += 1;
    }

    Ok(result)
}

// ===== Story 11.19: 智能接管执行引擎 =====

/// 执行智能接管 (Story 11.19)
///
/// 根据预览结果和用户决策执行合并操作
pub fn execute_smart_takeover(
    preview: &TakeoverPreview,
    decisions: &[TakeoverDecision],
    project_id: &str,
    db: &Database,
    env_manager: &EnvManager,
    gateway_url: Option<&str>,
    gateway_token: Option<&str>,
    gateway_running: bool,
) -> Result<SmartTakeoverResult, StorageError> {
    let mut result = SmartTakeoverResult::empty();
    result.gateway_running = gateway_running;

    // 构建决策映射（service_name -> decision）
    let decision_map: HashMap<String, &TakeoverDecision> = decisions
        .iter()
        .map(|d| (d.service_name.clone(), d))
        .collect();

    // 扫描以获取完整的服务信息
    let scan_result = scan_mcp_configs(Some(Path::new(&preview.project_path)));
    let all_detected: HashMap<String, Vec<DetectedService>> = scan_result
        .configs
        .iter()
        .flat_map(|c| c.services.clone())
        .fold(HashMap::new(), |mut acc, service| {
            acc.entry(service.name.clone()).or_default().push(service);
            acc
        });

    // 1. 处理 auto_create 项：创建服务 + 关联项目
    for item in &preview.auto_create {
        if let Some(services) = all_detected.get(&item.service_name) {
            if let Some(detected) = services.first() {
                match create_and_link_service(detected, project_id, db, &item.scope) {
                    Ok(service_id) => {
                        result.created_count += 1;
                        result.created_service_ids.push(service_id);
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to create service '{}': {}",
                            item.service_name, e
                        ));
                    }
                }
            }
        }
    }

    // 2. 处理 auto_skip 项：仅关联项目
    for item in &preview.auto_skip {
        let service_id = &item.existing_service.id;

        // 检查是否已关联
        if !is_service_linked(db, project_id, service_id)? {
            if let Err(e) = db.link_service_to_project_with_detection(
                project_id,
                service_id,
                None,
                Some(&item.detected_adapter_id),
                Some(&item.detected_config_path),
            ) {
                result.errors.push(format!(
                    "Failed to link service '{}' to project: {}",
                    item.service_name, e
                ));
            } else {
                result.skipped_count += 1;
            }
        } else {
            result.skipped_count += 1;
        }
    }

    // 3. 处理 needs_decision 项：按用户决策执行
    for conflict in &preview.needs_decision {
        if let Some(decision) = decision_map.get(&conflict.service_name) {
            match &decision.decision {
                TakeoverDecisionOption::KeepExisting => {
                    if let Some(existing) = &conflict.existing_service {
                        if !is_service_linked(db, project_id, &existing.id)? {
                            let (adapter_id, config_path) = conflict
                                .candidates
                                .first()
                                .map(|c| (c.adapter_id.as_str(), c.config_path.as_str()))
                                .unwrap_or(("", ""));

                            if let Err(e) = db.link_service_to_project_with_detection(
                                project_id,
                                &existing.id,
                                None,
                                Some(adapter_id),
                                Some(config_path),
                            ) {
                                result.errors.push(format!(
                                    "Failed to link existing service '{}': {}",
                                    conflict.service_name, e
                                ));
                            }
                        }
                    }
                    result.skipped_count += 1;
                }
                TakeoverDecisionOption::UseNew => {
                    let candidate_idx = decision.selected_candidate_index.unwrap_or(0);
                    if let Some(candidate) = conflict.candidates.get(candidate_idx) {
                        if let Some(services) = all_detected.get(&conflict.service_name) {
                            if let Some(detected) = services
                                .iter()
                                .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                                .or_else(|| services.first())
                            {
                                if let Some(existing) = &conflict.existing_service {
                                    match update_service_from_detected(db, &existing.id, detected) {
                                        Ok(_) => {
                                            if !is_service_linked(db, project_id, &existing.id)? {
                                                let _ = db.link_service_to_project_with_detection(
                                                    project_id,
                                                    &existing.id,
                                                    None,
                                                    Some(&candidate.adapter_id),
                                                    Some(&candidate.config_path),
                                                );
                                            }
                                            result.updated_count += 1;
                                        }
                                        Err(e) => {
                                            result.errors.push(format!(
                                                "Failed to update service '{}': {}",
                                                conflict.service_name, e
                                            ));
                                        }
                                    }
                                } else {
                                    match create_and_link_service(
                                        detected,
                                        project_id,
                                        db,
                                        &candidate.scope,
                                    ) {
                                        Ok(service_id) => {
                                            result.created_count += 1;
                                            result.created_service_ids.push(service_id);
                                        }
                                        Err(e) => {
                                            result.errors.push(format!(
                                                "Failed to create service '{}': {}",
                                                conflict.service_name, e
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                TakeoverDecisionOption::KeepBoth => {
                    let candidate_idx = decision.selected_candidate_index.unwrap_or(0);
                    if let Some(candidate) = conflict.candidates.get(candidate_idx) {
                        if let Some(services) = all_detected.get(&conflict.service_name) {
                            if let Some(detected) = services
                                .iter()
                                .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                                .or_else(|| services.first())
                            {
                                let new_name =
                                    format!("{}-{}", conflict.service_name, candidate.adapter_id);
                                let mut renamed_detected = detected.clone();
                                renamed_detected.name = new_name;

                                match create_and_link_service(
                                    &renamed_detected,
                                    project_id,
                                    db,
                                    &candidate.scope,
                                ) {
                                    Ok(service_id) => {
                                        result.renamed_count += 1;
                                        result.created_service_ids.push(service_id);
                                    }
                                    Err(e) => {
                                        result.errors.push(format!(
                                            "Failed to create renamed service '{}': {}",
                                            conflict.service_name, e
                                        ));
                                    }
                                }

                                if let Some(existing) = &conflict.existing_service {
                                    if !is_service_linked(db, project_id, &existing.id)? {
                                        let _ = db.link_service_to_project_with_detection(
                                            project_id,
                                            &existing.id,
                                            None,
                                            Some(&candidate.adapter_id),
                                            Some(&candidate.config_path),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                TakeoverDecisionOption::UseProjectScope | TakeoverDecisionOption::UseUserScope => {
                    let target_scope = match &decision.decision {
                        TakeoverDecisionOption::UseProjectScope => TakeoverScope::Project,
                        TakeoverDecisionOption::UseUserScope => TakeoverScope::User,
                        _ => TakeoverScope::Project,
                    };

                    if let Some(candidate) = conflict
                        .candidates
                        .iter()
                        .find(|c| c.scope == target_scope)
                        .or_else(|| conflict.candidates.first())
                    {
                        if let Some(services) = all_detected.get(&conflict.service_name) {
                            if let Some(detected) = services
                                .iter()
                                .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                                .or_else(|| services.first())
                            {
                                match create_and_link_service(
                                    detected,
                                    project_id,
                                    db,
                                    &candidate.scope,
                                ) {
                                    Ok(service_id) => {
                                        result.created_count += 1;
                                        result.created_service_ids.push(service_id);
                                    }
                                    Err(e) => {
                                        result.errors.push(format!(
                                            "Failed to create service '{}': {}",
                                            conflict.service_name, e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            result.skipped_count += 1;
        }
    }

    // 4. 执行配置文件接管（如果提供了 gateway_url）
    // Story 11.15 PD-001: 统一写入用户级配置
    if let Some(url) = gateway_url {
        let scan_result = scan_mcp_configs(Some(Path::new(&preview.project_path)));

        // 收集需要接管的用户级配置（去重）
        let mut processed_user_configs: std::collections::HashSet<PathBuf> =
            std::collections::HashSet::new();

        for config in &scan_result.configs {
            if config.services.is_empty() {
                continue;
            }

            if let Some(tool_type) = ToolType::from_adapter_id(&config.adapter_id) {
                // Story 11.15 PD-001: 统一写入用户级配置
                let user_config_path = tool_type.resolve_config_path(db);

                // 避免重复接管同一个用户级配置
                if processed_user_configs.contains(&user_config_path) {
                    continue;
                }
                processed_user_configs.insert(user_config_path.clone());

                let (scope, project_path) = determine_takeover_scope(config);

                let mut executor = ImportExecutor::new(db, env_manager);

                match executor.apply_takeover(
                    &user_config_path,
                    &config.adapter_id,
                    url,
                    gateway_token,
                    &tool_type,
                    scope,
                    project_path,
                ) {
                    Ok(backup_id) => {
                        result.takeover_config_paths.push(user_config_path.clone());
                        result.takeover_backup_ids.push(backup_id);
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to takeover {} config at {:?}: {}",
                            tool_type.display_name(),
                            user_config_path,
                            e
                        ));
                    }
                }

                executor.backup_manager.commit();
            }
        }

        // 4.5 Story 11.25: 项目级配置清理
        // 在用户级 Gateway 注入成功后，清理项目级配置文件中的 MCP 服务定义
        // DD-018: 清理失败只记录警告，不回滚用户级配置的 Gateway 注入
        if !result.takeover_backup_ids.is_empty() {
            // 收集所有项目级配置
            let project_configs: Vec<_> = scan_result
                .configs
                .iter()
                .filter(|c| matches!(c.scope, Some(ConfigScope::Project)))
                .collect();

            for config in project_configs {
                if let Some(tool_type) = ToolType::from_adapter_id(&config.adapter_id) {
                    let project_root = ImportExecutor::get_project_root_from_config_path(&config.path);

                    let mut cleanup_executor = ImportExecutor::new(db, env_manager);
                    match cleanup_executor.apply_project_config_cleanup(
                        &config.path,
                        &config.adapter_id,
                        &tool_type,
                        project_root,
                    ) {
                        Ok(backup_id) => {
                            result.takeover_backup_ids.push(backup_id);
                        }
                        Err(e) => {
                            // DD-018: 清理失败只记录警告
                            result.errors.push(format!(
                                "[Warning] Failed to cleanup project config {:?}: {}",
                                config.path, e
                            ));
                        }
                    }
                    cleanup_executor.backup_manager.commit();
                }
            }
        }
    }

    Ok(result)
}

// ===== Story 11.20: 全工具自动接管生成 =====

/// 执行全工具自动接管（带事务支持）(Story 11.20 - Task 5)
///
/// 遍历所有检测到的工具配置，执行统一的接管操作。
/// 任意工具接管失败时，回滚所有已执行的操作。
pub fn execute_full_tool_takeover(
    preview: &TakeoverPreview,
    decisions: &[TakeoverDecision],
    project_id: &str,
    db: &Database,
    env_manager: &EnvManager,
    gateway_url: &str,
    gateway_token: Option<&str>,
    gateway_running: bool,
) -> FullTakeoverResult {
    let mut result = FullTakeoverResult::empty();
    result.gateway_running = gateway_running;

    // 创建事务
    let mut transaction = TakeoverTransaction::begin();

    // 构建决策映射
    let decision_map: HashMap<String, &TakeoverDecision> = decisions
        .iter()
        .map(|d| (d.service_name.clone(), d))
        .collect();

    // 扫描获取完整服务信息
    let scan_result = scan_mcp_configs(Some(Path::new(&preview.project_path)));
    let all_detected: HashMap<String, Vec<DetectedService>> = scan_result
        .configs
        .iter()
        .flat_map(|c| c.services.clone())
        .fold(HashMap::new(), |mut acc, service| {
            acc.entry(service.name.clone()).or_default().push(service);
            acc
        });

    // Phase 1: 处理服务创建和关联
    // 1.1 处理 auto_create 项
    for item in &preview.auto_create {
        if let Some(services) = all_detected.get(&item.service_name) {
            if let Some(detected) = services.first() {
                match create_and_link_service_transactional(
                    detected,
                    project_id,
                    db,
                    &item.scope,
                    &mut transaction,
                ) {
                    Ok(service_id) => {
                        result.stats.created_count += 1;
                        result.created_service_ids.push(service_id);
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to create service '{}': {}",
                            item.service_name, e
                        );
                        result.errors.push(error_msg);
                        result.success = false;
                    }
                }
            }
        }
    }

    // 1.2 处理 auto_skip 项
    for item in &preview.auto_skip {
        let service_id = &item.existing_service.id;

        match is_service_linked(db, project_id, service_id) {
            Ok(linked) => {
                if !linked {
                    match db.link_service_to_project_with_detection(
                        project_id,
                        service_id,
                        None,
                        Some(&item.detected_adapter_id),
                        Some(&item.detected_config_path),
                    ) {
                        Ok(_) => {
                            transaction
                                .record_project_linked(project_id.to_string(), service_id.to_string());
                            result.stats.skipped_count += 1;
                        }
                        Err(e) => {
                            result.warnings.push(format!(
                                "Failed to link service '{}': {}",
                                item.service_name, e
                            ));
                        }
                    }
                } else {
                    result.stats.skipped_count += 1;
                }
            }
            Err(e) => {
                result.warnings.push(format!(
                    "Failed to check service link '{}': {}",
                    item.service_name, e
                ));
            }
        }
    }

    // 1.3 处理 needs_decision 项
    for conflict in &preview.needs_decision {
        if let Some(decision) = decision_map.get(&conflict.service_name) {
            if let Err(e) = process_decision_transactional(
                conflict,
                decision,
                project_id,
                db,
                &all_detected,
                &mut transaction,
                &mut result,
            ) {
                result.errors.push(format!(
                    "Failed to process decision for '{}': {}",
                    conflict.service_name, e
                ));
                result.success = false;
            }
        } else {
            result.stats.skipped_count += 1;
        }
    }

    // 检查是否需要回滚（Phase 1 失败）
    if !result.success {
        let rollback_result = transaction.rollback(db);
        result.rolled_back = true;
        if !rollback_result.errors.is_empty() {
            result.errors.extend(
                rollback_result
                    .errors
                    .iter()
                    .map(|e| format!("Rollback: {}", e)),
            );
        }
        return result;
    }

    // Phase 2: 执行配置文件接管
    // Story 11.15 PD-001: 统一写入用户级配置
    let mut executor = ImportExecutor::new(db, env_manager);
    let mut processed_user_configs = std::collections::HashSet::new();

    for config in &scan_result.configs {
        if config.services.is_empty() {
            continue;
        }

        if let Some(tool_type) = ToolType::from_adapter_id(&config.adapter_id) {
            // Story 11.15 PD-001: 统一写入用户级配置
            // Story 13.1: 使用 resolve_config_path 支持用户自定义配置目录
            let user_config_path = tool_type.resolve_config_path(db);

            // 避免重复接管同一个用户级配置
            if processed_user_configs.contains(&user_config_path) {
                continue;
            }
            processed_user_configs.insert(user_config_path.clone());

            let (scope, project_path) = determine_takeover_scope(config);

            match executor.apply_takeover(
                &user_config_path,
                &config.adapter_id,
                gateway_url,
                gateway_token,
                &tool_type,
                scope,
                project_path.clone(),
            ) {
                Ok(backup_id) => {
                    transaction.record_backup_created(backup_id.clone(), user_config_path.clone());
                    result.takeover_config_paths.push(user_config_path.clone());
                    result.takeover_backup_ids.push(backup_id);
                    result.stats.takeover_count += 1;
                    result.stats.tool_count += 1;
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to takeover {} config at {:?}: {}",
                        tool_type.display_name(),
                        user_config_path,
                        e
                    );
                    result.errors.push(error_msg);
                    result.success = false;
                }
            }
        }
    }

    // 检查是否需要回滚（Phase 2 失败）
    if !result.success {
        let rollback_result = transaction.rollback(db);
        result.rolled_back = true;
        if !rollback_result.errors.is_empty() {
            result.errors.extend(
                rollback_result
                    .errors
                    .iter()
                    .map(|e| format!("Rollback: {}", e)),
            );
        }
        return result;
    }

    // Phase 2.5: 项目级配置清理 (Story 11.25)
    // DD-018: 清理失败只记录警告，不影响整体成功
    {
        // 收集所有项目级配置
        let project_configs: Vec<_> = scan_result
            .configs
            .iter()
            .filter(|c| matches!(c.scope, Some(ConfigScope::Project)))
            .collect();

        for config in project_configs {
            if let Some(tool_type) = ToolType::from_adapter_id(&config.adapter_id) {
                let project_root = ImportExecutor::get_project_root_from_config_path(&config.path);

                match executor.apply_project_config_cleanup(
                    &config.path,
                    &config.adapter_id,
                    &tool_type,
                    project_root,
                ) {
                    Ok(backup_id) => {
                        transaction.record_backup_created(backup_id.clone(), config.path.clone());
                        result.takeover_backup_ids.push(backup_id);
                    }
                    Err(e) => {
                        // DD-018: 清理失败只记录警告，不影响 success 状态
                        result.warnings.push(format!(
                            "Failed to cleanup project config {:?}: {}",
                            config.path, e
                        ));
                    }
                }
            }
        }
    }

    // Phase 3: 处理 Claude Code Local Scope (Story 11.21 - Task 5)
    // Local Scope 是 Claude Code 特有的功能，对应 ~/.claude.json 中的 projects.{path}.mcpServers
    let claude_user_config = ToolType::ClaudeCode.resolve_config_path(db);
    if claude_user_config.exists() {
        use crate::services::mcp_adapters::ClaudeAdapter;

        let adapter = ClaudeAdapter;
        if let Ok(config_content) = fs::read_to_string(&claude_user_config) {
            // 获取所有 local scope 项目
            if let Ok(local_projects) = adapter.list_local_scope_projects(&config_content) {
                let project_paths: Vec<String> = local_projects
                    .iter()
                    .filter(|p| p.service_count > 0) // 只处理有服务的项目
                    .map(|p| p.project_path.clone())
                    .collect();

                if !project_paths.is_empty() {
                    // 批量执行 Local Scope 接管
                    let (backup_ids, local_errors) =
                        executor.apply_all_local_scope_takeovers(&claude_user_config, &project_paths);

                    // 记录成功的备份
                    for backup_id in backup_ids {
                        transaction.record_local_scope_backup(backup_id.clone());
                        result.takeover_backup_ids.push(backup_id);
                    }

                    // 记录失败的错误（作为警告，不影响整体成功）
                    for (project_path, error) in local_errors {
                        result.warnings.push(format!(
                            "Local scope takeover failed for '{}': {}",
                            project_path, error
                        ));
                    }

                    // 更新统计
                    result.stats.local_scope_count = project_paths.len();
                }
            }
        }
    }

    // 成功：提交事务
    if let Err(e) = transaction.commit() {
        result.warnings.push(format!("Failed to commit transaction: {}", e));
    }

    // 提交备份管理器
    executor.backup_manager.commit();

    result
}

// ===== 内部辅助函数 =====

/// 创建服务并关联到项目 (Story 11.19)
fn create_and_link_service(
    detected: &DetectedService,
    project_id: &str,
    db: &Database,
    scope: &TakeoverScope,
) -> Result<String, StorageError> {
    let request = CreateMcpServiceRequest {
        name: detected.name.clone(),
        transport_type: detected.transport_type.clone(),
        command: detected.command.clone(),
        args: detected.args.clone(),
        env: detected
            .env
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap()),
        url: detected.url.clone(),
        headers: detected.headers.clone(),
        source: McpServiceSource::Imported,
        source_file: Some(detected.source_file.to_string_lossy().to_string()),
    };

    let service = db.create_mcp_service_with_source(
        &request,
        Some(&detected.adapter_id),
        Some(scope.as_str()),
    )?;

    db.link_service_to_project_with_detection(
        project_id,
        &service.id,
        None,
        Some(&detected.adapter_id),
        Some(&detected.source_file.to_string_lossy()),
    )?;

    Ok(service.id)
}

/// 创建服务并关联到项目（带事务记录）(Story 11.20)
fn create_and_link_service_transactional(
    detected: &DetectedService,
    project_id: &str,
    db: &Database,
    scope: &TakeoverScope,
    transaction: &mut TakeoverTransaction,
) -> Result<String, StorageError> {
    let request = CreateMcpServiceRequest {
        name: detected.name.clone(),
        transport_type: detected.transport_type.clone(),
        command: detected.command.clone(),
        args: detected.args.clone(),
        env: detected
            .env
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap()),
        url: detected.url.clone(),
        headers: detected.headers.clone(),
        source: McpServiceSource::Imported,
        source_file: Some(detected.source_file.to_string_lossy().to_string()),
    };

    let service = db.create_mcp_service_with_source(
        &request,
        Some(&detected.adapter_id),
        Some(scope.as_str()),
    )?;

    transaction.record_service_created(service.id.clone());

    db.link_service_to_project_with_detection(
        project_id,
        &service.id,
        None,
        Some(&detected.adapter_id),
        Some(&detected.source_file.to_string_lossy()),
    )?;

    transaction.record_project_linked(project_id.to_string(), service.id.clone());

    Ok(service.id)
}

/// 从检测到的服务更新现有服务 (Story 11.19)
fn update_service_from_detected(
    db: &Database,
    service_id: &str,
    detected: &DetectedService,
) -> Result<(), StorageError> {
    let update = UpdateMcpServiceRequest {
        name: Some(detected.name.clone()),
        transport_type: Some(detected.transport_type.clone()),
        command: Some(detected.command.clone()),
        args: detected.args.clone(),
        env: detected
            .env
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap()),
        url: detected.url.clone(),
        headers: detected.headers.clone(),
        enabled: Some(true),
    };

    db.update_mcp_service(service_id, &update)?;
    Ok(())
}

/// 检查服务是否已关联到项目 (Story 11.19)
pub(crate) fn is_service_linked(
    db: &Database,
    project_id: &str,
    service_id: &str,
) -> Result<bool, StorageError> {
    let links = db.get_project_service_links(project_id)?;
    Ok(links.iter().any(|l| l.service_id == service_id))
}

/// 处理用户决策（带事务记录）(Story 11.20)
fn process_decision_transactional(
    conflict: &crate::models::mcp::ConflictDetail,
    decision: &TakeoverDecision,
    project_id: &str,
    db: &Database,
    all_detected: &HashMap<String, Vec<DetectedService>>,
    transaction: &mut TakeoverTransaction,
    result: &mut FullTakeoverResult,
) -> Result<(), StorageError> {
    match &decision.decision {
        TakeoverDecisionOption::KeepExisting => {
            if let Some(existing) = &conflict.existing_service {
                if !is_service_linked(db, project_id, &existing.id)? {
                    let (adapter_id, config_path) = conflict
                        .candidates
                        .first()
                        .map(|c| (c.adapter_id.as_str(), c.config_path.as_str()))
                        .unwrap_or(("", ""));

                    db.link_service_to_project_with_detection(
                        project_id,
                        &existing.id,
                        None,
                        Some(adapter_id),
                        Some(config_path),
                    )?;
                    transaction.record_project_linked(project_id.to_string(), existing.id.clone());
                }
            }
            result.stats.skipped_count += 1;
        }

        TakeoverDecisionOption::UseNew => {
            let candidate_idx = decision.selected_candidate_index.unwrap_or(0);
            if let Some(candidate) = conflict.candidates.get(candidate_idx) {
                if let Some(services) = all_detected.get(&conflict.service_name) {
                    if let Some(detected) = services
                        .iter()
                        .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                        .or_else(|| services.first())
                    {
                        if let Some(existing) = &conflict.existing_service {
                            update_service_from_detected(db, &existing.id, detected)?;
                            if !is_service_linked(db, project_id, &existing.id)? {
                                db.link_service_to_project_with_detection(
                                    project_id,
                                    &existing.id,
                                    None,
                                    Some(&candidate.adapter_id),
                                    Some(&candidate.config_path),
                                )?;
                                transaction.record_project_linked(
                                    project_id.to_string(),
                                    existing.id.clone(),
                                );
                            }
                            result.stats.updated_count += 1;
                        } else {
                            let service_id = create_and_link_service_transactional(
                                detected,
                                project_id,
                                db,
                                &candidate.scope,
                                transaction,
                            )?;
                            result.stats.created_count += 1;
                            result.created_service_ids.push(service_id);
                        }
                    }
                }
            }
        }

        TakeoverDecisionOption::KeepBoth => {
            let candidate_idx = decision.selected_candidate_index.unwrap_or(0);
            if let Some(candidate) = conflict.candidates.get(candidate_idx) {
                if let Some(services) = all_detected.get(&conflict.service_name) {
                    if let Some(detected) = services
                        .iter()
                        .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                        .or_else(|| services.first())
                    {
                        let new_name =
                            format!("{}-{}", conflict.service_name, candidate.adapter_id);
                        let mut renamed_detected = detected.clone();
                        renamed_detected.name = new_name;

                        let service_id = create_and_link_service_transactional(
                            &renamed_detected,
                            project_id,
                            db,
                            &candidate.scope,
                            transaction,
                        )?;
                        result.stats.renamed_count += 1;
                        result.created_service_ids.push(service_id);

                        if let Some(existing) = &conflict.existing_service {
                            if !is_service_linked(db, project_id, &existing.id)? {
                                db.link_service_to_project_with_detection(
                                    project_id,
                                    &existing.id,
                                    None,
                                    Some(&candidate.adapter_id),
                                    Some(&candidate.config_path),
                                )?;
                                transaction.record_project_linked(
                                    project_id.to_string(),
                                    existing.id.clone(),
                                );
                            }
                        }
                    }
                }
            }
        }

        TakeoverDecisionOption::UseProjectScope | TakeoverDecisionOption::UseUserScope => {
            let target_scope = match &decision.decision {
                TakeoverDecisionOption::UseProjectScope => TakeoverScope::Project,
                TakeoverDecisionOption::UseUserScope => TakeoverScope::User,
                _ => TakeoverScope::Project,
            };

            if let Some(candidate) = conflict
                .candidates
                .iter()
                .find(|c| c.scope == target_scope)
                .or_else(|| conflict.candidates.first())
            {
                if let Some(services) = all_detected.get(&conflict.service_name) {
                    if let Some(detected) = services
                        .iter()
                        .find(|s| s.source_file.to_string_lossy() == candidate.config_path)
                        .or_else(|| services.first())
                    {
                        let service_id = create_and_link_service_transactional(
                            detected,
                            project_id,
                            db,
                            &candidate.scope,
                            transaction,
                        )?;
                        result.stats.created_count += 1;
                        result.created_service_ids.push(service_id);
                    }
                }
            }
        }
    }

    Ok(())
}

/// 确定接管作用域 (Story 11.20, 11.21)
/// Story 11.15 PD-001: 统一写入用户级配置
/// 无论来源是 Project 还是 User scope，实际写入的都是用户级配置
fn determine_takeover_scope(config: &DetectedConfig) -> (TakeoverScope, Option<PathBuf>) {
    match &config.scope {
        Some(ConfigScope::Project) => {
            // 记录来源项目路径（用于上下文路由），但 scope 为 User（因为写入用户级配置）
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
            // Story 11.15 PD-001: 虽然来源是项目配置，但实际写入用户级配置
            (TakeoverScope::User, proj_path)
        }
        Some(ConfigScope::Local) => {
            // Story 11.21: Local scope 不在此函数处理
            // Local scope 由专门的 apply_local_scope_takeover 处理
            (TakeoverScope::Local, None)
        }
        _ => (TakeoverScope::User, None),
    }
}

// ===== Story 11.21: Local Scope 恢复 =====

/// 恢复 Local Scope 接管 (Story 11.21 - Task 7, Story 11.22)
///
/// 从备份文件恢复指定项目的 mcpServers 配置到 ~/.claude.json
/// 使用原子操作和 hash 验证 (Story 11.22)
///
/// # Arguments
/// * `db` - 数据库
/// * `backup_id` - 备份记录 ID
///
/// # Returns
/// 更新后的备份记录
pub fn restore_local_scope_takeover(
    db: &Database,
    backup_id: &str,
) -> Result<TakeoverBackup, StorageError> {
    use crate::services::atomic_fs;
    use crate::services::mcp_adapters::ClaudeAdapter;

    // 1. 获取备份记录
    let backup = db
        .get_takeover_backup_by_id(backup_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Local scope backup not found: {}", backup_id)))?;

    // 2. 验证是 local scope 备份
    if backup.scope != TakeoverScope::Local {
        return Err(StorageError::InvalidInput(format!(
            "Backup {} is not a local scope backup (scope: {:?})",
            backup_id, backup.scope
        )));
    }

    // 3. 检查是否可以恢复
    if backup.status != TakeoverStatus::Active {
        return Err(StorageError::InvalidInput(format!(
            "Backup {} is already restored",
            backup_id
        )));
    }

    // 4. 获取项目路径
    let project_path = backup.project_path.as_ref()
        .ok_or_else(|| StorageError::InvalidInput(
            "Local scope backup missing project_path".to_string()
        ))?
        .to_string_lossy()
        .to_string();

    // 5. 验证备份文件完整性 (Story 11.22)
    if let Some(expected_hash) = &backup.backup_hash {
        let actual_hash = atomic_fs::calculate_file_hash(&backup.backup_path).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to calculate backup file hash: {}", e))
        })?;
        if &actual_hash != expected_hash {
            return Err(StorageError::InvalidInput(format!(
                "Backup file integrity check failed: expected hash {}, got {}",
                expected_hash, actual_hash
            )));
        }
    }

    // 6. 读取备份文件内容（mcpServers JSON 片段）
    let backup_content = fs::read_to_string(&backup.backup_path).map_err(|e| {
        StorageError::InvalidInput(format!("Failed to read backup file: {}", e))
    })?;
    let backup_mcp_servers: serde_json::Value = serde_json::from_str(&backup_content)
        .map_err(|e| {
            StorageError::InvalidInput(format!("Failed to parse backup content: {}", e))
        })?;

    // 7. 读取当前配置文件
    let current_content = if backup.original_path.exists() {
        fs::read_to_string(&backup.original_path).map_err(|e| {
            StorageError::InvalidInput(format!("Failed to read config file: {}", e))
        })?
    } else {
        return Err(StorageError::InvalidInput(format!(
            "Config file not found: {:?}",
            backup.original_path
        )));
    };

    // 8. 使用 ClaudeAdapter 恢复 mcpServers
    let adapter = ClaudeAdapter;
    let restored_content = adapter
        .restore_local_scope_mcp_servers(&current_content, &project_path, &backup_mcp_servers)
        .map_err(|e| {
            StorageError::InvalidInput(format!("Failed to restore local scope: {}", e))
        })?;

    // 9. 原子写回配置文件 (Story 11.22)
    atomic_fs::atomic_write_str(&backup.original_path, &restored_content).map_err(|e| {
        StorageError::InvalidInput(format!("Failed to atomic write config file: {}", e))
    })?;

    // 10. 更新数据库记录状态
    db.update_backup_status_restored(backup_id)?;

    // 11. 返回更新后的备份记录
    db.get_takeover_backup_by_id(backup_id)?
        .ok_or_else(|| StorageError::NotFound(format!("Backup not found after update: {}", backup_id)))
}

/// 恢复所有活跃的 Local Scope 接管 (Story 11.21 - Task 7)
///
/// # Returns
/// (成功恢复的数量, 失败的错误列表)
pub fn restore_all_local_scope_takeovers(
    db: &Database,
) -> Result<(usize, Vec<String>), StorageError> {
    let local_backups = db.get_active_local_scope_takeovers()?;

    let mut restored_count = 0;
    let mut errors = Vec::new();

    for backup in local_backups {
        match restore_local_scope_takeover(db, &backup.id) {
            Ok(_) => restored_count += 1,
            Err(e) => errors.push(format!(
                "Failed to restore local scope for {:?}: {}",
                backup.project_path, e
            )),
        }
    }

    Ok((restored_count, errors))
}

// ===== Story 11.22: 备份完整性检查 =====

/// 获取带完整性信息的活跃备份列表 (Story 11.22 - AC4)
///
/// 检查每个备份记录的文件存在状态和 hash 完整性
pub fn list_takeover_backups_with_integrity(
    db: &Database,
) -> Result<Vec<crate::models::mcp::TakeoverBackupIntegrity>, StorageError> {
    use crate::models::mcp::TakeoverBackupIntegrity;

    let backups = db.get_takeover_backups(Some(TakeoverStatus::Active))?;

    let results = backups
        .into_iter()
        .map(|backup| {
            let backup_file_exists = backup.backup_path.exists();
            let original_file_exists = backup.original_path.exists();

            // 验证 hash（如果有记录）
            let hash_valid = if backup_file_exists {
                backup.backup_hash.as_ref().map(|expected_hash| {
                    atomic_fs::calculate_file_hash(&backup.backup_path)
                        .map(|actual_hash| &actual_hash == expected_hash)
                        .unwrap_or(false)
                })
            } else {
                backup.backup_hash.as_ref().map(|_| false) // 文件不存在，hash 验证失败
            };

            TakeoverBackupIntegrity {
                backup,
                backup_file_exists,
                original_file_exists,
                hash_valid,
            }
        })
        .collect();

    Ok(results)
}

/// 删除无效的备份记录 (Story 11.22 - AC4)
///
/// 无效备份定义为：备份文件不存在或 hash 验证失败的记录
///
/// # Returns
/// 删除的记录数量
pub fn delete_invalid_backups(db: &Database) -> Result<usize, StorageError> {
    let backups_with_integrity = list_takeover_backups_with_integrity(db)?;

    let mut deleted_count = 0;

    for item in backups_with_integrity {
        let is_invalid = !item.backup_file_exists
            || item.hash_valid == Some(false);

        if is_invalid {
            db.delete_takeover_backup(&item.backup.id)?;
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

// ===== Story 11.23: 备份版本管理 =====

/// 清理旧备份，只保留最近 keep_count 个 (Story 11.23 - Task 1)
///
/// 每个 (工具 + Scope + 项目路径) 组合独立计算
/// 按照 DD-015 清理优先级：先删除 DB 记录，再删除文件，文件删除失败只记录警告
///
/// # Arguments
/// * `db` - 数据库
/// * `tool_type` - 工具类型
/// * `scope` - 作用域
/// * `project_path` - 项目路径 (project/local scope 需要)
/// * `keep_count` - 保留数量
///
/// # Returns
/// 清理结果
pub fn cleanup_old_backups(
    db: &Database,
    tool_type: &ToolType,
    scope: &TakeoverScope,
    project_path: Option<&str>,
    keep_count: usize,
) -> Result<crate::models::mcp::CleanupResult, StorageError> {
    use crate::models::mcp::CleanupResult;

    let mut result = CleanupResult::empty();

    // 1. 查询该组合的所有备份，按时间倒序排列
    let backups = db.get_backups_by_tool_scope(tool_type, scope, project_path)?;

    // 2. 跳过要保留的，删除其余的
    let to_delete = backups.into_iter().skip(keep_count);

    for backup in to_delete {
        // 3. 获取文件大小（在删除前）
        let file_size = if backup.backup_path.exists() {
            std::fs::metadata(&backup.backup_path)
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        // 4. 先删除 DB 记录 (DD-015)
        if let Err(e) = db.delete_takeover_backup(&backup.id) {
            result.warnings.push(format!(
                "Failed to delete DB record for backup {}: {}",
                backup.id, e
            ));
            continue; // 跳过此备份，继续处理下一个
        }

        // 5. 再删除文件（失败只警告）(DD-015)
        if backup.backup_path.exists() {
            match std::fs::remove_file(&backup.backup_path) {
                Ok(_) => {
                    result.deleted_size += file_size;
                }
                Err(e) => {
                    result.warnings.push(format!(
                        "Failed to delete backup file {:?}: {}",
                        backup.backup_path, e
                    ));
                }
            }
        }

        result.deleted_count += 1;
    }

    Ok(result)
}

/// 批量清理所有组合的旧备份 (Story 11.23 - AC 5)
///
/// 对每个 (工具 + Scope + 项目路径) 组合执行清理
///
/// # Arguments
/// * `db` - 数据库
/// * `keep_per_group` - 每组保留数量
///
/// # Returns
/// 批量清理结果
pub fn cleanup_all_old_backups(
    db: &Database,
    keep_per_group: usize,
) -> Result<crate::models::mcp::BatchCleanupResult, StorageError> {
    use crate::models::mcp::BatchCleanupResult;

    let mut result = BatchCleanupResult::empty();

    // 1. 获取所有唯一的 (tool, scope, project_path) 组合
    let groups = db.get_backup_groups()?;
    result.groups_processed = groups.len();

    // 2. 对每个组合执行清理
    for (tool, scope, project_path) in groups {
        match cleanup_old_backups(db, &tool, &scope, project_path.as_deref(), keep_per_group) {
            Ok(group_result) => {
                result.total_deleted += group_result.deleted_count;
                result.total_size += group_result.deleted_size;
                result.warnings.extend(group_result.warnings);
            }
            Err(e) => {
                result.warnings.push(format!(
                    "Failed to cleanup backups for {:?}/{:?}/{:?}: {}",
                    tool, scope, project_path, e
                ));
            }
        }
    }

    Ok(result)
}

/// 获取带版本序号的备份列表 (Story 11.23 - AC 3)
///
/// 为每个备份添加版本序号信息
///
/// # Arguments
/// * `db` - 数据库
///
/// # Returns
/// 带版本序号的备份列表
pub fn list_backups_with_version(
    db: &Database,
) -> Result<Vec<crate::models::mcp::TakeoverBackupWithVersion>, StorageError> {
    use crate::models::mcp::{TakeoverBackup, TakeoverBackupWithVersion, TakeoverStatus};
    use std::collections::HashMap;

    // 1. 只获取活跃的备份（排除已恢复的）
    let all_backups = db.get_takeover_backups(Some(TakeoverStatus::Active))?;

    // 2. 按 (tool, scope, project_path) 分组
    let mut groups: HashMap<String, Vec<TakeoverBackup>> = HashMap::new();
    for backup in &all_backups {
        let key = format!(
            "{}|{}|{}",
            backup.tool_type.as_str(),
            backup.scope.as_str(),
            backup.project_path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
        );
        groups.entry(key).or_default().push(backup.clone());
    }

    // 3. 对每组按时间倒序排序，计算版本序号并检查完整性
    let mut result = Vec::new();
    for (_, group) in &mut groups {
        // 按时间倒序排序
        group.sort_by(|a, b| b.taken_over_at.cmp(&a.taken_over_at));
        let total_versions = group.len();

        for (index, backup) in group.iter().enumerate() {
            // 检查完整性
            let backup_file_exists = backup.backup_path.exists();
            let original_file_exists = backup.original_path.exists();
            let hash_valid = if backup_file_exists {
                backup.backup_hash.as_ref().map(|expected_hash| {
                    atomic_fs::calculate_file_hash(&backup.backup_path)
                        .map(|actual_hash| &actual_hash == expected_hash)
                        .unwrap_or(false)
                })
            } else {
                backup.backup_hash.as_ref().map(|_| false)
            };

            result.push(TakeoverBackupWithVersion {
                backup: backup.clone(),
                version_index: index + 1, // 1-based
                total_versions,
                backup_file_exists,
                original_file_exists,
                hash_valid,
            });
        }
    }

    // 4. 按时间倒序排列整个列表
    result.sort_by(|a, b| b.backup.taken_over_at.cmp(&a.backup.taken_over_at));

    Ok(result)
}
