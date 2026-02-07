/**
 * 工具配置路径管理 IPC
 *
 * Story 13.1: 工具配置路径可配置化 - Task 6
 */

import { invoke } from "./ipc-adapter";

export interface ToolConfigPathInfo {
  toolType: string;
  displayName: string;
  /** 默认配置目录（如 ~/.claude、~/.cursor） */
  defaultDir: string;
  /** 用户覆盖的目录（null = 使用默认） */
  overrideDir: string | null;
}

export async function getToolConfigPaths(): Promise<ToolConfigPathInfo[]> {
  return invoke<ToolConfigPathInfo[]>("get_tool_config_paths");
}

export async function setToolConfigPath(
  toolType: string,
  dir: string
): Promise<void> {
  return invoke<void>("set_tool_config_path", {
    toolType,
    dir,
  });
}

export async function resetToolConfigPath(toolType: string): Promise<void> {
  return invoke<void>("reset_tool_config_path", { toolType });
}
