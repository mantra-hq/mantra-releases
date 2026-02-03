/**
 * MCP 相关类型定义
 * Story 11.10 → Story 11.18: 简化 Tool Policy 模型
 */

/**
 * Tool Policy 配置
 *
 * Story 11.18: 简化的权限模型
 *
 * ## 语义
 * - `allowedTools = null` → 继承全局默认（仅项目级有效）
 * - `allowedTools = []` → 全选（允许所有工具）
 * - `allowedTools = [...]` → 部分选（仅允许指定工具）
 * - 不关联服务 = 禁用
 */
export interface ToolPolicy {
  allowedTools: string[] | null;
}

/**
 * 默认 Tool Policy (全选)
 */
export const DEFAULT_TOOL_POLICY: ToolPolicy = {
  allowedTools: [],
};

/**
 * 继承策略
 */
export const INHERIT_TOOL_POLICY: ToolPolicy = {
  allowedTools: null,
};

/**
 * 判断是否为继承模式
 */
export function isInheritPolicy(policy: ToolPolicy): boolean {
  return policy.allowedTools === null;
}

/**
 * 判断是否为全选模式
 */
export function isAllowAllPolicy(policy: ToolPolicy): boolean {
  return Array.isArray(policy.allowedTools) && policy.allowedTools.length === 0;
}

/**
 * 判断是否为部分选模式
 */
export function isCustomPolicy(policy: ToolPolicy): boolean {
  return Array.isArray(policy.allowedTools) && policy.allowedTools.length > 0;
}

/**
 * MCP 工具定义
 */
export interface McpTool {
  name: string;
  description?: string;
  inputSchema?: object;
}

/**
 * 工具发现结果
 */
export interface ToolDiscoveryResult {
  serviceId: string;
  tools: McpTool[];
  fromCache: boolean;
  cachedAt?: string;
}

/**
 * 检查工具是否被允许
 *
 * Story 11.18: 简化的检查逻辑
 * - `null` (继承): 返回 true（实际继承由上层处理）
 * - `[]` (全选): 返回 true
 * - `[...]` (部分选): 工具在列表中才返回 true
 */
export function isToolAllowed(policy: ToolPolicy, toolName: string): boolean {
  // 继承或全选: 允许所有
  if (policy.allowedTools === null || policy.allowedTools.length === 0) {
    return true;
  }

  // 部分选: 检查列表
  return policy.allowedTools.includes(toolName);
}

// ===== 向后兼容：旧类型别名（标记为废弃） =====

/**
 * @deprecated Story 11.18: 使用 isAllowAllPolicy/isCustomPolicy/isInheritPolicy 替代
 */
export type ToolPolicyMode = 'allow_all' | 'deny_all' | 'custom';
