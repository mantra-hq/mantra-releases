/**
 * smart-takeover-ipc - 智能接管 Tauri IPC 封装
 * Story 11.19: MCP 智能接管合并引擎 - Task 5
 *
 * 提供智能接管功能的 Tauri IPC 调用封装：
 * - previewSmartTakeover - 生成智能接管预览
 * - executeSmartTakeover - 执行智能接管
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
