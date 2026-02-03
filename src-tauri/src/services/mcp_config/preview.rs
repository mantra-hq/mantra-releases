//! 导入预览和智能接管预览引擎
//!
//! Story 11.3: 导入预览生成
//! Story 11.19: 智能接管预览引擎

use std::collections::HashMap;

use super::types::*;
use crate::models::mcp::{
    AutoCreateItem, AutoSkipItem, ConflictCandidate, ConflictDetail, ConflictType,
    ConfigDiffDetail, McpService, McpServiceSummary, MergeClassification, ServiceConfigSummary,
    TakeoverPreview, TakeoverScope,
};
use crate::services::mcp_adapters::{ConfigScope, GATEWAY_SERVICE_NAME};
use crate::storage::{Database, StorageError};

// ===== 导入预览生成器 =====

/// 提取环境变量引用
///
/// 从环境变量映射中识别 `$VAR_NAME` 或 `${VAR_NAME}` 格式的引用
pub fn extract_env_var_references(env: &Option<HashMap<String, String>>) -> Vec<String> {
    let mut vars = Vec::new();
    if let Some(env_map) = env {
        for value in env_map.values() {
            // 匹配 $VAR_NAME 或 ${VAR_NAME}
            if value.starts_with('$') {
                let var_name = value
                    .trim_start_matches('$')
                    .trim_start_matches('{')
                    .trim_end_matches('}');
                if !var_name.is_empty() && !vars.contains(&var_name.to_string()) {
                    vars.push(var_name.to_string());
                }
            }
        }
    }
    vars
}

/// 生成导入预览
///
/// # Arguments
/// * `configs` - 检测到的配置文件列表
/// * `db` - 数据库连接
///
/// # Returns
/// 导入预览，包含冲突检测和环境变量需求
pub fn generate_import_preview(
    configs: &[DetectedConfig],
    db: &Database,
) -> Result<ImportPreview, StorageError> {
    let mut service_map: HashMap<String, Vec<DetectedService>> = HashMap::new();
    let mut env_vars_needed: Vec<String> = Vec::new();

    // 收集所有服务并按名称分组
    for config in configs {
        for service in &config.services {
            // 跳过 Mantra Gateway 自身注入的服务，不应出现在导入列表中
            if service.name == GATEWAY_SERVICE_NAME {
                continue;
            }

            service_map
                .entry(service.name.clone())
                .or_default()
                .push(service.clone());

            // 提取环境变量引用
            for var in extract_env_var_references(&service.env) {
                if !env_vars_needed.contains(&var) {
                    env_vars_needed.push(var);
                }
            }
        }
    }

    let mut conflicts = Vec::new();
    let mut new_services = Vec::new();
    let total_services = service_map.len();

    // 检查冲突
    for (name, candidates) in service_map {
        let existing = db.get_mcp_service_by_name(&name)?;

        if candidates.len() > 1 || existing.is_some() {
            // 存在冲突：多个候选或已存在同名服务
            conflicts.push(ServiceConflict {
                name,
                existing,
                candidates,
            });
        } else {
            // 无冲突，可直接导入
            new_services.extend(candidates);
        }
    }

    // 检查环境变量是否已存在
    let mut missing_env_vars = Vec::new();
    for var in &env_vars_needed {
        if !db.env_variable_exists(var)? {
            missing_env_vars.push(var.clone());
        }
    }

    Ok(ImportPreview {
        configs: configs.to_vec(),
        conflicts,
        new_services,
        env_vars_needed: missing_env_vars,
        total_services,
    })
}

// ===== Story 11.19: 智能接管预览引擎 =====

/// 比较检测到的服务与现有服务的配置是否相等 (Story 11.19)
///
/// 比较的字段：transport_type, command, args, url
/// 不比较的字段：env, headers（可能包含敏感信息）
pub(crate) fn config_equals(existing: &McpService, detected: &DetectedService) -> bool {
    // 传输类型必须一致
    if existing.transport_type != detected.transport_type {
        return false;
    }

    // 命令必须一致
    if existing.command != detected.command {
        return false;
    }

    // 参数必须一致
    let existing_args = existing.args.clone().unwrap_or_default();
    let detected_args = detected.args.clone().unwrap_or_default();
    if existing_args != detected_args {
        return false;
    }

    // URL 必须一致 (HTTP 模式)
    if existing.url != detected.url {
        return false;
    }

    true
}

/// 计算配置差异详情 (Story 11.19)
pub(crate) fn compute_config_diff(
    existing: &McpService,
    detected: &DetectedService,
) -> Vec<ConfigDiffDetail> {
    let mut diffs = Vec::new();

    // 检查传输类型
    if existing.transport_type != detected.transport_type {
        diffs.push(ConfigDiffDetail {
            field: "transport_type".to_string(),
            existing_value: Some(existing.transport_type.as_str().to_string()),
            new_value: Some(detected.transport_type.as_str().to_string()),
        });
    }

    // 检查命令
    if existing.command != detected.command {
        diffs.push(ConfigDiffDetail {
            field: "command".to_string(),
            existing_value: if existing.command.is_empty() {
                None
            } else {
                Some(existing.command.clone())
            },
            new_value: if detected.command.is_empty() {
                None
            } else {
                Some(detected.command.clone())
            },
        });
    }

    // 检查参数
    let existing_args = existing.args.clone().unwrap_or_default();
    let detected_args = detected.args.clone().unwrap_or_default();
    if existing_args != detected_args {
        diffs.push(ConfigDiffDetail {
            field: "args".to_string(),
            existing_value: if existing_args.is_empty() {
                None
            } else {
                Some(existing_args.join(", "))
            },
            new_value: if detected_args.is_empty() {
                None
            } else {
                Some(detected_args.join(", "))
            },
        });
    }

    // 检查 URL
    if existing.url != detected.url {
        diffs.push(ConfigDiffDetail {
            field: "url".to_string(),
            existing_value: existing.url.clone(),
            new_value: detected.url.clone(),
        });
    }

    diffs
}

/// 从检测到的服务创建配置摘要 (Story 11.19)
pub(crate) fn create_config_summary(detected: &DetectedService) -> ServiceConfigSummary {
    ServiceConfigSummary {
        transport_type: detected.transport_type.clone(),
        command: if detected.command.is_empty() {
            None
        } else {
            Some(detected.command.clone())
        },
        args_count: detected.args.as_ref().map_or(0, |a| a.len()),
        env_count: detected.env.as_ref().map_or(0, |e| e.len()),
        url: detected.url.clone(),
    }
}

/// 将 ConfigScope 转换为 TakeoverScope (Story 11.19, 11.21)
fn config_scope_to_takeover_scope(scope: &ConfigScope) -> TakeoverScope {
    match scope {
        ConfigScope::Project => TakeoverScope::Project,
        ConfigScope::User => TakeoverScope::User,
        ConfigScope::Local => TakeoverScope::Local,
    }
}

/// 对检测到的服务进行合并分类 (Story 11.19)
///
/// 返回三档分类：AutoCreate, AutoSkip, NeedsDecision
pub(crate) fn classify_for_merge(
    _service_name: &str,
    candidates: &[DetectedService],
    existing: Option<&McpService>,
) -> MergeClassification {
    // 无现有服务
    if existing.is_none() {
        if candidates.len() == 1 {
            // 单一候选，无冲突 -> 自动创建
            return MergeClassification::AutoCreate;
        } else {
            // 多个候选 -> 需要决策（多来源或多 Scope 冲突）
            return MergeClassification::NeedsDecision;
        }
    }

    let existing = existing.unwrap();

    // 有现有服务
    if candidates.len() == 1 {
        // 单一候选，检查配置是否一致
        if config_equals(existing, &candidates[0]) {
            // 配置完全一致 -> 自动跳过
            return MergeClassification::AutoSkip;
        } else {
            // 配置不同 -> 需要决策
            return MergeClassification::NeedsDecision;
        }
    }

    // 多个候选 + 现有服务 -> 复杂冲突，需要决策
    MergeClassification::NeedsDecision
}

/// 检测 Scope 冲突 (Story 11.19)
///
/// 检查同一服务名在 project + user 级是否都存在
pub(crate) fn has_scope_conflict(candidates: &[DetectedService]) -> bool {
    if candidates.len() < 2 {
        return false;
    }

    let mut has_project = false;
    let mut has_user = false;

    for candidate in candidates {
        let path_str = candidate.source_file.to_string_lossy();
        if path_str.starts_with("~") || path_str.contains("/.") {
            has_user = true;
        } else {
            has_project = true;
        }
    }

    has_project && has_user
}

/// 确定冲突类型 (Story 11.19)
pub(crate) fn determine_conflict_type(
    candidates: &[DetectedService],
    _existing: Option<&McpService>,
) -> ConflictType {
    // 检查 Scope 冲突
    if has_scope_conflict(candidates) {
        return ConflictType::ScopeConflict;
    }

    // 检查多来源冲突（不同适配器）
    if candidates.len() > 1 {
        let adapter_ids: std::collections::HashSet<&str> =
            candidates.iter().map(|c| c.adapter_id.as_str()).collect();
        if adapter_ids.len() > 1 {
            return ConflictType::MultiSource;
        }
    }

    // 默认为配置差异冲突
    ConflictType::ConfigDiff
}

/// 生成智能接管预览 (Story 11.19)
///
/// 将检测到的服务分为三档：
/// - auto_create: 全局池无此服务，将自动创建
/// - auto_skip: 全局池有同名服务且配置完全一致，自动跳过
/// - needs_decision: 需用户决策（配置冲突 / 多 scope 冲突 / 多来源冲突）
pub fn generate_smart_takeover_preview(
    configs: &[DetectedConfig],
    db: &Database,
    project_path: &str,
) -> Result<TakeoverPreview, StorageError> {
    let mut service_map: HashMap<String, Vec<DetectedService>> = HashMap::new();
    let mut env_vars_needed: Vec<String> = Vec::new();

    // 收集所有服务并按名称分组
    for config in configs {
        for service in &config.services {
            // 跳过 Mantra Gateway 自身注入的服务，不应出现在接管列表中
            if service.name == GATEWAY_SERVICE_NAME {
                continue;
            }

            service_map
                .entry(service.name.clone())
                .or_default()
                .push(service.clone());

            // 提取环境变量引用
            for var in extract_env_var_references(&service.env) {
                if !env_vars_needed.contains(&var) {
                    env_vars_needed.push(var);
                }
            }
        }
    }

    let total_services = service_map.len();

    let mut auto_create = Vec::new();
    let mut auto_skip = Vec::new();
    let mut needs_decision = Vec::new();

    // 对每个服务进行分类
    for (name, candidates) in service_map {
        let existing = db.get_mcp_service_by_name(&name)?;
        let classification = classify_for_merge(&name, &candidates, existing.as_ref());

        match classification {
            MergeClassification::AutoCreate => {
                // 取第一个候选（单一候选情况）
                let candidate = &candidates[0];
                let scope = configs
                    .iter()
                    .find(|c| c.services.iter().any(|s| s.name == name))
                    .and_then(|c| c.scope.as_ref())
                    .map(config_scope_to_takeover_scope)
                    .unwrap_or(TakeoverScope::Project);

                auto_create.push(AutoCreateItem {
                    service_name: name,
                    adapter_id: candidate.adapter_id.clone(),
                    config_path: candidate.source_file.to_string_lossy().to_string(),
                    scope,
                    config_summary: create_config_summary(candidate),
                });
            }
            MergeClassification::AutoSkip => {
                // 配置完全一致，跳过导入
                let candidate = &candidates[0];
                let existing = existing.unwrap();
                let scope = configs
                    .iter()
                    .find(|c| c.services.iter().any(|s| s.name == name))
                    .and_then(|c| c.scope.as_ref())
                    .map(config_scope_to_takeover_scope)
                    .unwrap_or(TakeoverScope::Project);

                auto_skip.push(AutoSkipItem {
                    service_name: name,
                    detected_adapter_id: candidate.adapter_id.clone(),
                    detected_config_path: candidate.source_file.to_string_lossy().to_string(),
                    detected_scope: scope,
                    existing_service: McpServiceSummary::from_service(&existing),
                });
            }
            MergeClassification::NeedsDecision => {
                // 需要用户决策
                let conflict_type = determine_conflict_type(&candidates, existing.as_ref());

                // 构建冲突候选项
                let conflict_candidates: Vec<ConflictCandidate> = candidates
                    .iter()
                    .map(|c| {
                        let scope = configs
                            .iter()
                            .find(|cfg| cfg.services.iter().any(|s| s.name == name))
                            .and_then(|cfg| cfg.scope.as_ref())
                            .map(config_scope_to_takeover_scope)
                            .unwrap_or(TakeoverScope::Project);

                        ConflictCandidate {
                            adapter_id: c.adapter_id.clone(),
                            config_path: c.source_file.to_string_lossy().to_string(),
                            scope,
                            config_summary: create_config_summary(c),
                        }
                    })
                    .collect();

                // 计算配置差异（如果有现有服务且是 ConfigDiff 类型）
                let diff_details = if conflict_type == ConflictType::ConfigDiff {
                    if let Some(ref existing) = existing {
                        candidates
                            .first()
                            .map(|c| compute_config_diff(existing, c))
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                needs_decision.push(ConflictDetail {
                    service_name: name,
                    conflict_type,
                    existing_service: existing.as_ref().map(McpServiceSummary::from_service),
                    candidates: conflict_candidates,
                    diff_details,
                });
            }
        }
    }

    // 检查环境变量是否已存在
    let mut missing_env_vars = Vec::new();
    for var in &env_vars_needed {
        if !db.env_variable_exists(var)? {
            missing_env_vars.push(var.clone());
        }
    }

    Ok(TakeoverPreview {
        project_path: project_path.to_string(),
        auto_create,
        auto_skip,
        needs_decision,
        env_vars_needed: missing_env_vars,
        total_services,
    })
}
