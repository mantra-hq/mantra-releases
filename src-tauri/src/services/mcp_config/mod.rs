//! MCP 配置解析与导入服务
//!
//! Story 11.3: 配置导入与接管
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 提供 MCP 配置文件的解析、扫描、备份、导入功能。
//!
//! ## 模块结构
//!
//! - `types`: 数据类型定义
//! - `parsers`: 配置解析器（旧版，向后兼容）
//! - `scanner`: 配置文件扫描器
//! - `preview`: 导入预览和智能接管预览
//! - `backup`: 备份管理器
//! - `executor`: 导入执行器
//! - `takeover`: 接管恢复和执行引擎
//!
//! ## 架构变更 (Story 11.8)
//!
//! - 使用 `adapter_id: String` 替代旧的 `ConfigSource` 枚举
//! - 通过 `ToolAdapterRegistry` 统一管理适配器
//! - 支持 Claude, Cursor, Codex, Gemini CLI 四大工具

mod backup;
mod executor;
mod parsers;
mod preview;
mod scanner;
mod takeover;
mod types;

// Re-export 数据类型
pub use types::{
    ConfigSource, ConflictResolution, DetectedConfig, DetectedService, FullTakeoverResult,
    ImportPreview, ImportRequest, ImportResult, McpConfigFile, McpServerConfig, ParseError,
    ScanResult, ServiceConflict, SmartTakeoverResult, SyncTakeoverResult, TakeoverStats,
};

// Re-export 备份管理
pub use backup::{rollback_from_backups, BackupManager};

// Re-export 配置解析器（向后兼容）
#[allow(deprecated)]
pub use parsers::{
    generate_shadow_config, generate_shadow_config_v2, strip_json_comments,
    ClaudeCodeConfigParser, ClaudeDesktopConfigParser, CursorConfigParser, McpConfigParser,
};

// Re-export 扫描功能
pub use scanner::{
    detect_installed_tools, generate_full_tool_takeover_preview, scan_all_tool_configs,
    scan_mcp_configs,
};

// Re-export 预览功能
pub use preview::{
    extract_env_var_references, generate_import_preview, generate_smart_takeover_preview,
};

// Re-export 执行器
pub use executor::ImportExecutor;

// Re-export 接管功能
pub use takeover::{
    delete_invalid_backups, execute_full_tool_takeover, execute_smart_takeover,
    get_takeover_status, list_takeover_backups_with_integrity,
    restore_all_local_scope_takeovers, restore_local_scope_takeover, restore_mcp_takeover,
    restore_mcp_takeover_by_tool, sync_active_takeovers,
};

#[cfg(test)]
mod tests;

// Re-export 测试专用的内部函数
#[cfg(test)]
pub(crate) use preview::{
    classify_for_merge, compute_config_diff, config_equals, create_config_summary,
    determine_conflict_type, has_scope_conflict,
};
#[cfg(test)]
pub(crate) use takeover::is_service_linked;
