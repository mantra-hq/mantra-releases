/**
 * smart-takeover-ipc - 智能接管 Tauri IPC 封装
 * Story 11.19: MCP 智能接管合并引擎 - Task 5
 * Story 11.20: 全工具自动接管生成 - Task 6
 *
 * 提供智能接管功能的 Tauri IPC 调用封装：
 * - previewSmartTakeover - 生成智能接管预览
 * - executeSmartTakeover - 执行智能接管
 * - previewFullToolTakeover - 生成全工具接管预览 (Story 11.20)
 * - executeFullToolTakeover - 执行全工具接管 (Story 11.20)
 * - detectInstalledTools - 检测已安装工具 (Story 11.20)
 */

import { invoke } from "@/lib/ipc-adapter";

// ===== 类型定义 =====

/** 接管 Scope */
export type TakeoverScope = "project" | "user";

/** 配置摘要 */
export interface ServiceConfigSummary {
  command: string;
  args: string[] | null;
  transport_type: string;
  url: string | null;
}

/** MCP 服务摘要 */
export interface McpServiceSummary {
  id: string;
  name: string;
  source_adapter_id: string | null;
  source_scope: string | null;
  config_summary: ServiceConfigSummary;
}

/** 自动创建项 */
export interface AutoCreateItem {
  service_name: string;
  adapter_id: string;
  config_path: string;
  scope: TakeoverScope;
  config_summary: ServiceConfigSummary;
}

/** 自动跳过项 */
export interface AutoSkipItem {
  service_name: string;
  detected_adapter_id: string;
  detected_config_path: string;
  detected_scope: TakeoverScope;
  existing_service: McpServiceSummary;
}

/** 冲突类型 */
export type ConflictType = "config_diff" | "scope_conflict" | "multi_source";

/** 冲突候选项 */
export interface ConflictCandidate {
  adapter_id: string;
  config_path: string;
  scope: TakeoverScope;
  config_summary: ServiceConfigSummary;
}

/** 配置差异详情 */
export interface ConfigDiffDetail {
  field: string;
  existing_value: string | null;
  candidate_value: string | null;
}

/** 冲突详情 */
export interface ConflictDetail {
  service_name: string;
  conflict_type: ConflictType;
  existing_service: McpServiceSummary | null;
  candidates: ConflictCandidate[];
  diff_details?: ConfigDiffDetail[];
}

/** 用户决策选项 */
export type TakeoverDecisionOption =
  | "keep_existing"
  | "use_new"
  | "keep_both"
  | "use_project_scope"
  | "use_user_scope";

/** 用户决策 */
export interface TakeoverDecision {
  service_name: string;
  decision: TakeoverDecisionOption;
  selected_candidate_index?: number;
}

/** 智能接管预览结果 */
export interface TakeoverPreview {
  project_path: string;
  auto_create: AutoCreateItem[];
  auto_skip: AutoSkipItem[];
  needs_decision: ConflictDetail[];
  env_vars_needed: string[];
  total_services: number;
}

/** 智能接管执行结果 */
export interface SmartTakeoverResult {
  created_count: number;
  skipped_count: number;
  updated_count: number;
  created_service_ids: string[];
  errors: string[];
  gateway_running: boolean;
}

// ===== Story 11.20: 全工具接管类型定义 =====

/** 工具类型 */
export type ToolType = "claude_code" | "cursor" | "codex" | "gemini_cli";

/** 单个工具的检测结果 */
export interface ToolDetectionResult {
  tool_type: ToolType;
  installed: boolean;
  user_config_path: string;
  user_config_exists: boolean;
  display_name: string;
  adapter_id: string;
}

/** 所有工具的检测结果 */
export interface AllToolsDetectionResult {
  tools: ToolDetectionResult[];
  installed_count: number;
  total_count: number;
}

/** 单个 Scope 的扫描结果 */
export interface ScopeScanResult {
  config_path: string;
  exists: boolean;
  service_count: number;
  service_names: string[];
  parse_errors: string[];
}

/** 单个工具的扫描结果 */
export interface ToolScanResult {
  tool_type: ToolType;
  display_name: string;
  adapter_id: string;
  installed: boolean;
  user_scope: ScopeScanResult | null;
  local_scopes: Array<{ project_path: string; service_count: number; service_names: string[] }>;
  project_scope: ScopeScanResult | null;
  total_service_count: number;
}

/** 所有工具的扫描结果 */
export interface AllToolsScanResult {
  tools: ToolScanResult[];
  project_path: string;
  installed_count: number;
  tools_with_config_count: number;
  total_service_count: number;
}

/** 单个 Scope 的接管预览 */
export interface ScopeTakeoverPreview {
  scope: TakeoverScope;
  config_path: string;
  exists: boolean;
  auto_create: AutoCreateItem[];
  auto_skip: AutoSkipItem[];
  needs_decision: ConflictDetail[];
  service_count: number;
}

/** 单个工具的接管预览 */
export interface ToolTakeoverPreview {
  tool_type: ToolType;
  display_name: string;
  adapter_id: string;
  installed: boolean;
  selected: boolean;
  user_scope_preview: ScopeTakeoverPreview | null;
  project_scope_preview: ScopeTakeoverPreview | null;
  total_service_count: number;
  conflict_count: number;
}

/** 全工具接管预览 */
export interface FullToolTakeoverPreview {
  project_path: string;
  tools: ToolTakeoverPreview[];
  installed_count: number;
  env_vars_needed: string[];
  total_service_count: number;
  total_conflict_count: number;
  can_auto_execute: boolean;
}

/** 接管统计信息 */
export interface TakeoverStats {
  created_count: number;
  skipped_count: number;
  updated_count: number;
  renamed_count: number;
  takeover_count: number;
  tool_count: number;
}

/** 全工具接管结果 */
export interface FullTakeoverResult {
  success: boolean;
  rolled_back: boolean;
  stats: TakeoverStats;
  errors: string[];
  warnings: string[];
  created_service_ids: string[];
  takeover_backup_ids: string[];
  takeover_config_paths: string[];
  gateway_running: boolean;
}

// ===== IPC 函数 =====

/**
 * 生成智能接管预览
 *
 * @param projectId - 项目 ID
 * @param projectPath - 项目路径
 * @returns 智能接管预览结果
 */
export async function previewSmartTakeover(
  projectId: string,
  projectPath: string
): Promise<TakeoverPreview> {
  return invoke<TakeoverPreview>("preview_smart_takeover", {
    projectId,
    projectPath,
  });
}

/**
 * 执行智能接管
 *
 * @param projectId - 项目 ID
 * @param preview - 智能接管预览结果
 * @param decisions - 用户决策列表
 * @returns 执行结果
 */
export async function executeSmartTakeover(
  projectId: string,
  preview: TakeoverPreview,
  decisions: TakeoverDecision[]
): Promise<SmartTakeoverResult> {
  return invoke<SmartTakeoverResult>("execute_smart_takeover_cmd", {
    projectId,
    preview,
    decisions,
  });
}

// ===== Story 11.20: 全工具接管 IPC 函数 =====

/**
 * 检测已安装的 AI 编程工具
 *
 * @returns 所有工具的检测结果
 */
export async function detectInstalledTools(): Promise<AllToolsDetectionResult> {
  return invoke<AllToolsDetectionResult>("detect_installed_tools", {});
}

/**
 * 扫描所有工具的配置（按工具分组）
 *
 * @param projectPath - 项目路径
 * @returns 所有工具的扫描结果
 */
export async function scanAllToolConfigs(
  projectPath: string
): Promise<AllToolsScanResult> {
  return invoke<AllToolsScanResult>("scan_all_tool_configs", {
    projectPath,
  });
}

/**
 * 生成全工具接管预览
 *
 * @param projectPath - 项目路径
 * @returns 全工具接管预览
 */
export async function previewFullToolTakeover(
  projectPath: string
): Promise<FullToolTakeoverPreview> {
  return invoke<FullToolTakeoverPreview>("preview_full_tool_takeover", {
    projectPath,
  });
}

/**
 * 执行全工具接管（带事务支持）
 *
 * @param projectId - 项目 ID
 * @param preview - 智能接管预览结果（兼容 TakeoverPreview）
 * @param decisions - 用户决策列表
 * @returns 全工具接管结果
 */
export async function executeFullToolTakeover(
  projectId: string,
  preview: TakeoverPreview,
  decisions: TakeoverDecision[]
): Promise<FullTakeoverResult> {
  return invoke<FullTakeoverResult>("execute_full_tool_takeover_cmd", {
    projectId,
    preview,
    decisions,
  });
}

// ===== 辅助函数 =====

/**
 * 获取冲突类型的显示文本键
 */
export function getConflictTypeKey(type: ConflictType): string {
  switch (type) {
    case "config_diff":
      return "hub.smartTakeover.conflictConfigDiff";
    case "scope_conflict":
      return "hub.smartTakeover.conflictScopeConflict";
    case "multi_source":
      return "hub.smartTakeover.conflictMultiSource";
    default:
      return "hub.smartTakeover.conflictUnknown";
  }
}

/**
 * 获取决策选项的显示文本键
 */
export function getDecisionOptionKey(option: TakeoverDecisionOption): string {
  switch (option) {
    case "keep_existing":
      return "hub.smartTakeover.decisionKeepExisting";
    case "use_new":
      return "hub.smartTakeover.decisionUseNew";
    case "keep_both":
      return "hub.smartTakeover.decisionKeepBoth";
    case "use_project_scope":
      return "hub.smartTakeover.decisionUseProjectScope";
    case "use_user_scope":
      return "hub.smartTakeover.decisionUseUserScope";
    default:
      return "hub.smartTakeover.decisionUnknown";
  }
}

/**
 * 检查预览是否需要用户决策
 */
export function previewNeedsDecision(preview: TakeoverPreview): boolean {
  return preview.needs_decision.length > 0;
}

/**
 * 检查预览是否为空（没有任何可导入的服务）
 */
export function previewIsEmpty(preview: TakeoverPreview): boolean {
  return (
    preview.auto_create.length === 0 &&
    preview.auto_skip.length === 0 &&
    preview.needs_decision.length === 0
  );
}

/**
 * 获取预览统计信息
 */
export function getPreviewStats(preview: TakeoverPreview): {
  autoCreateCount: number;
  autoSkipCount: number;
  needsDecisionCount: number;
  totalCount: number;
} {
  return {
    autoCreateCount: preview.auto_create.length,
    autoSkipCount: preview.auto_skip.length,
    needsDecisionCount: preview.needs_decision.length,
    totalCount: preview.total_services,
  };
}

// ===== Story 11.20: 全工具预览辅助函数 =====

/**
 * 检查全工具预览是否需要用户决策
 */
export function fullPreviewNeedsDecision(preview: FullToolTakeoverPreview): boolean {
  return preview.total_conflict_count > 0;
}

/**
 * 检查全工具预览是否为空
 */
export function fullPreviewIsEmpty(preview: FullToolTakeoverPreview): boolean {
  return preview.total_service_count === 0;
}

/**
 * 获取全工具预览统计信息
 */
export function getFullPreviewStats(preview: FullToolTakeoverPreview): {
  installedCount: number;
  selectedCount: number;
  totalServiceCount: number;
  conflictCount: number;
  canAutoExecute: boolean;
} {
  const selectedTools = preview.tools.filter((t) => t.selected);
  return {
    installedCount: preview.installed_count,
    selectedCount: selectedTools.length,
    totalServiceCount: preview.total_service_count,
    conflictCount: preview.total_conflict_count,
    canAutoExecute: preview.can_auto_execute,
  };
}

/**
 * 将全工具预览转换为标准 TakeoverPreview
 * 用于执行接管时传递给后端
 */
export function convertToTakeoverPreview(
  fullPreview: FullToolTakeoverPreview,
  selectedAdapterIds: string[]
): TakeoverPreview {
  const selectedTools = fullPreview.tools.filter(
    (t) => selectedAdapterIds.includes(t.adapter_id)
  );

  const auto_create: AutoCreateItem[] = [];
  const auto_skip: AutoSkipItem[] = [];
  const needs_decision: ConflictDetail[] = [];
  const env_vars_needed: string[] = [...fullPreview.env_vars_needed];

  for (const tool of selectedTools) {
    // User scope
    if (tool.user_scope_preview) {
      auto_create.push(...tool.user_scope_preview.auto_create);
      auto_skip.push(...tool.user_scope_preview.auto_skip);
      needs_decision.push(...tool.user_scope_preview.needs_decision);
    }
    // Project scope
    if (tool.project_scope_preview) {
      auto_create.push(...tool.project_scope_preview.auto_create);
      auto_skip.push(...tool.project_scope_preview.auto_skip);
      needs_decision.push(...tool.project_scope_preview.needs_decision);
    }
  }

  return {
    project_path: fullPreview.project_path,
    auto_create,
    auto_skip,
    needs_decision,
    env_vars_needed,
    total_services: auto_create.length + auto_skip.length + needs_decision.length,
  };
}

/**
 * 获取工具的显示名称
 */
export function getToolDisplayName(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "Claude Code";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    case "gemini_cli":
      return "Gemini CLI";
    default:
      return toolType;
  }
}

/**
 * 获取工具的适配器 ID
 */
export function getToolAdapterId(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "claude";
    case "cursor":
      return "cursor";
    case "codex":
      return "codex";
    case "gemini_cli":
      return "gemini";
    default:
      return toolType;
  }
}
