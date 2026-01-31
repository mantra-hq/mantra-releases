/**
 * MCP 相关类型定义
 * Story 11.10: Project-Level Tool Management - Task 4
 */

/**
 * Tool Policy 模式
 */
export type ToolPolicyMode = 'allow_all' | 'deny_all' | 'custom';

/**
 * Tool Policy 配置
 *
 * 用于控制项目级别的 MCP 工具访问权限。
 *
 * ## 优先级规则
 * `deniedTools` > `allowedTools` > `mode`
 */
export interface ToolPolicy {
  mode: ToolPolicyMode;
  allowedTools: string[];
  deniedTools: string[];
}

/**
 * 默认 Tool Policy (AllowAll)
 */
export const DEFAULT_TOOL_POLICY: ToolPolicy = {
  mode: 'allow_all',
  allowedTools: [],
  deniedTools: [],
};

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
 */
export function isToolAllowed(policy: ToolPolicy, toolName: string): boolean {
  // 1. deniedTools 优先级最高
  if (policy.deniedTools.includes(toolName)) {
    return false;
  }

  // 2. 根据 mode 判断
  switch (policy.mode) {
    case 'allow_all':
      return true;
    case 'deny_all':
      return false;
    case 'custom':
      return policy.allowedTools.includes(toolName);
    default:
      return true;
  }
}
