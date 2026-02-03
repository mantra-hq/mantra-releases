//! 配置文件扫描器
//!
//! 扫描 MCP 配置文件，检测已安装的 AI 编程工具

use std::fs;
use std::path::{Path, PathBuf};

use super::parsers::parse_config_file_legacy;
use super::types::*;
use crate::models::mcp::{
    AutoCreateItem, McpServiceSummary, McpTransportType, ScopeTakeoverPreview,
    ServiceConfigSummary, TakeoverScope,
};
use crate::services::mcp_adapters::{ConfigScope, ToolAdapterRegistry, GATEWAY_SERVICE_NAME};
use crate::storage::{Database, StorageError};

/// 获取平台相关的 Claude Desktop 配置路径
fn get_claude_desktop_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|p| p.join("claude-desktop").join("claude_desktop_config.json"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|p| p.join("Claude").join("claude_desktop_config.json"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|p| p.join("Claude").join("claude_desktop_config.json"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// 扫描 MCP 配置文件 (使用新的适配器架构)
///
/// Story 11.8: 使用 `ToolAdapterRegistry` 统一扫描所有工具的配置文件
///
/// # Arguments
/// * `project_path` - 项目路径（可选，用于扫描项目级配置）
///
/// # Returns
/// 扫描结果，包含所有检测到的配置文件和服务
pub fn scan_mcp_configs(project_path: Option<&Path>) -> ScanResult {
    let registry = ToolAdapterRegistry::new();
    let mut configs = Vec::new();
    let mut scanned_paths = Vec::new();

    let home_dir = dirs::home_dir();

    // 遍历所有适配器
    for adapter in registry.all() {
        for (scope, pattern) in adapter.scan_patterns() {
            // 解析路径模式
            let path = match scope {
                ConfigScope::Project => {
                    if let Some(project) = project_path {
                        project.join(&pattern)
                    } else {
                        continue;
                    }
                }
                ConfigScope::User => {
                    if let Some(ref home) = home_dir {
                        if pattern.starts_with("~/") {
                            home.join(&pattern[2..])
                        } else {
                            home.join(&pattern)
                        }
                    } else {
                        continue;
                    }
                }
            };

            scanned_paths.push(path.clone());

            if path.exists() {
                // 读取并解析配置文件
                match fs::read_to_string(&path) {
                    Ok(content) => match adapter.parse(&path, &content, scope) {
                        Ok(services) => {
                            configs.push(DetectedConfig {
                                adapter_id: adapter.id().to_string(),
                                path: path.clone(),
                                scope: Some(scope),
                                services: services.into_iter().map(Into::into).collect(),
                                parse_errors: Vec::new(),
                            });
                        }
                        Err(e) => {
                            configs.push(DetectedConfig {
                                adapter_id: adapter.id().to_string(),
                                path: path.clone(),
                                scope: Some(scope),
                                services: Vec::new(),
                                parse_errors: vec![e.to_string()],
                            });
                        }
                    },
                    Err(e) => {
                        configs.push(DetectedConfig {
                            adapter_id: adapter.id().to_string(),
                            path: path.clone(),
                            scope: Some(scope),
                            services: Vec::new(),
                            parse_errors: vec![format!("Failed to read file: {}", e)],
                        });
                    }
                }
            }
        }
    }

    // 向后兼容：扫描 Claude Desktop 配置
    if let Some(claude_desktop_path) = get_claude_desktop_config_path() {
        scanned_paths.push(claude_desktop_path.clone());
        if claude_desktop_path.exists() {
            #[allow(deprecated)]
            let config = parse_config_file_legacy(&claude_desktop_path, ConfigSource::ClaudeDesktop);
            configs.push(config);
        }
    }

    ScanResult {
        configs,
        scanned_paths,
    }
}

/// 检测已安装的 AI 编程工具
///
/// Story 11.20: 全工具自动接管生成 - AC 1
///
/// 扫描所有支持的 AI 编程工具，检测其用户级配置文件是否存在
/// 配置文件存在即视为工具已安装
pub fn detect_installed_tools() -> crate::models::mcp::AllToolsDetectionResult {
    use crate::models::mcp::{AllToolsDetectionResult, ToolDetectionResult, ToolType};

    let tools: Vec<ToolDetectionResult> = ToolType::all()
        .into_iter()
        .map(|tool_type| {
            let user_config_path = tool_type.get_user_config_path();
            let user_config_exists = user_config_path.exists();

            ToolDetectionResult {
                display_name: tool_type.display_name().to_string(),
                adapter_id: tool_type.to_adapter_id().to_string(),
                installed: user_config_exists,
                user_config_path,
                user_config_exists,
                tool_type,
            }
        })
        .collect();

    let installed_count = tools.iter().filter(|t| t.installed).count();
    let total_count = tools.len();

    AllToolsDetectionResult {
        tools,
        installed_count,
        total_count,
    }
}

/// 扫描所有工具的配置（按工具分组）
///
/// Story 11.20: 全工具自动接管生成 - AC 2
///
/// 扫描所有支持的 AI 编程工具的 MCP 配置，按工具分组返回结果。
pub fn scan_all_tool_configs(project_path: &Path) -> crate::models::mcp::AllToolsScanResult {
    use crate::models::mcp::{AllToolsScanResult, ScopeScanResult, ToolScanResult, ToolType};

    let registry = ToolAdapterRegistry::new();
    let home_dir = dirs::home_dir();

    let mut tool_results: Vec<ToolScanResult> = Vec::new();

    // 遍历所有工具类型
    for tool_type in ToolType::all() {
        let adapter_id = tool_type.to_adapter_id();
        let mut result = ToolScanResult::new(tool_type.clone());

        // 获取对应的适配器
        if let Some(adapter) = registry.get(adapter_id) {
            // 扫描各 Scope
            for (scope, pattern) in adapter.scan_patterns() {
                let path = match scope {
                    ConfigScope::Project => project_path.join(&pattern),
                    ConfigScope::User => {
                        if let Some(ref home) = home_dir {
                            if pattern.starts_with("~/") {
                                home.join(&pattern[2..])
                            } else {
                                home.join(&pattern)
                            }
                        } else {
                            continue;
                        }
                    }
                };

                let scope_result = if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => match adapter.parse(&path, &content, scope) {
                            Ok(services) => {
                                // 过滤掉 Gateway 服务
                                let service_names: Vec<String> = services
                                    .into_iter()
                                    .filter(|s| s.name != GATEWAY_SERVICE_NAME)
                                    .map(|s| s.name)
                                    .collect();
                                ScopeScanResult::success(path.clone(), service_names)
                            }
                            Err(e) => ScopeScanResult::with_error(path.clone(), e.to_string()),
                        },
                        Err(e) => ScopeScanResult::with_error(path.clone(), e.to_string()),
                    }
                } else {
                    ScopeScanResult::not_found(path.clone())
                };

                // 根据 scope 分配到相应字段
                match scope {
                    ConfigScope::User => {
                        result.installed = scope_result.exists;
                        result.user_scope = Some(scope_result);
                    }
                    ConfigScope::Project => {
                        result.project_scope = Some(scope_result);
                    }
                }
            }
        }

        // Note: Local Scope (Claude Code projects.*) 将在 Story 11-21 中实现
        // 目前 local_scopes 保持为空

        // 计算总服务数量
        result.update_total_service_count();
        tool_results.push(result);
    }

    // 计算汇总统计
    let installed_count = tool_results.iter().filter(|t| t.installed).count();
    let tools_with_config_count = tool_results.iter().filter(|t| t.has_any_config()).count();
    let total_service_count: usize = tool_results.iter().map(|t| t.total_service_count).sum();

    AllToolsScanResult {
        tools: tool_results,
        project_path: project_path.to_string_lossy().to_string(),
        installed_count,
        tools_with_config_count,
        total_service_count,
    }
}

/// 生成全工具接管预览
///
/// Story 11.20: 全工具自动接管生成 - AC 3
pub fn generate_full_tool_takeover_preview(
    project_path: &Path,
    db: &Database,
) -> Result<crate::models::mcp::FullToolTakeoverPreview, StorageError> {
    use crate::models::mcp::{
        FullToolTakeoverPreview, ScopeTakeoverPreview, ToolTakeoverPreview,
    };

    let scan_result = scan_all_tool_configs(project_path);
    let mut tool_previews: Vec<ToolTakeoverPreview> = Vec::new();
    let mut all_env_vars: Vec<String> = Vec::new();

    for tool_scan in &scan_result.tools {
        let mut tool_preview = ToolTakeoverPreview {
            tool_type: tool_scan.tool_type.clone(),
            display_name: tool_scan.display_name.clone(),
            adapter_id: tool_scan.adapter_id.clone(),
            installed: tool_scan.installed,
            selected: tool_scan.installed, // 默认选中已安装的工具
            user_scope_preview: None,
            project_scope_preview: None,
            total_service_count: 0,
            conflict_count: 0,
        };

        // 处理 User Scope
        if let Some(ref user_scope) = tool_scan.user_scope {
            if user_scope.exists && user_scope.service_count > 0 {
                let scope_preview = generate_scope_takeover_preview(
                    &user_scope.service_names,
                    &user_scope.config_path,
                    TakeoverScope::User,
                    &tool_scan.adapter_id,
                    db,
                    &mut all_env_vars,
                )?;
                tool_preview.conflict_count += scope_preview.needs_decision.len();
                tool_preview.user_scope_preview = Some(scope_preview);
            } else {
                tool_preview.user_scope_preview = Some(ScopeTakeoverPreview::empty(
                    TakeoverScope::User,
                    user_scope.config_path.to_string_lossy().to_string(),
                ));
            }
        }

        // 处理 Project Scope
        if let Some(ref project_scope) = tool_scan.project_scope {
            if project_scope.exists && project_scope.service_count > 0 {
                let scope_preview = generate_scope_takeover_preview(
                    &project_scope.service_names,
                    &project_scope.config_path,
                    TakeoverScope::Project,
                    &tool_scan.adapter_id,
                    db,
                    &mut all_env_vars,
                )?;
                tool_preview.conflict_count += scope_preview.needs_decision.len();
                tool_preview.project_scope_preview = Some(scope_preview);
            } else {
                tool_preview.project_scope_preview = Some(ScopeTakeoverPreview::empty(
                    TakeoverScope::Project,
                    project_scope.config_path.to_string_lossy().to_string(),
                ));
            }
        }

        // 计算工具总服务数
        tool_preview.total_service_count = tool_preview
            .user_scope_preview
            .as_ref()
            .map_or(0, |s| s.service_count)
            + tool_preview
                .project_scope_preview
                .as_ref()
                .map_or(0, |s| s.service_count);

        tool_previews.push(tool_preview);
    }

    // 构建最终预览
    let mut preview = FullToolTakeoverPreview {
        project_path: project_path.to_string_lossy().to_string(),
        tools: tool_previews,
        installed_count: 0,
        env_vars_needed: all_env_vars,
        total_service_count: 0,
        total_conflict_count: 0,
        can_auto_execute: true,
    };
    preview.update_stats();

    Ok(preview)
}

/// 为单个 Scope 生成接管预览
///
/// Story 11.20: 内部辅助函数
fn generate_scope_takeover_preview(
    service_names: &[String],
    config_path: &Path,
    scope: TakeoverScope,
    adapter_id: &str,
    db: &Database,
    _env_vars: &mut Vec<String>,
) -> Result<ScopeTakeoverPreview, StorageError> {
    use crate::models::mcp::AutoSkipItem;

    let config_path_str = config_path.to_string_lossy().to_string();
    let mut auto_create = Vec::new();
    let mut auto_skip = Vec::new();
    let needs_decision = Vec::new();

    for service_name in service_names {
        // 查询全局池中是否已有该服务
        let existing = db.get_mcp_service_by_name(service_name)?;

        // 简化分类逻辑：单一来源，无需复杂的多候选处理
        if existing.is_none() {
            // 无现有服务 -> 自动创建
            auto_create.push(AutoCreateItem {
                service_name: service_name.clone(),
                adapter_id: adapter_id.to_string(),
                config_path: config_path_str.clone(),
                scope: scope.clone(),
                config_summary: ServiceConfigSummary {
                    transport_type: McpTransportType::Stdio, // 默认
                    command: None,
                    args_count: 0,
                    env_count: 0,
                    url: None,
                },
            });
        } else {
            let existing = existing.unwrap();
            // 有现有服务 -> 自动跳过（简化处理，配置差异由完整扫描处理）
            auto_skip.push(AutoSkipItem {
                service_name: service_name.clone(),
                detected_adapter_id: adapter_id.to_string(),
                detected_config_path: config_path_str.clone(),
                detected_scope: scope.clone(),
                existing_service: McpServiceSummary::from_service(&existing),
            });
        }
    }

    Ok(ScopeTakeoverPreview {
        scope,
        config_path: config_path_str,
        exists: true,
        auto_create,
        auto_skip,
        needs_decision,
        service_count: service_names.len(),
    })
}
