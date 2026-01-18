/**
 * Local API Server IPC - 本地 API Server 通信模块
 * Story 3.11: Task 4.5 - AC #7
 *
 * 提供本地 API Server 管理功能
 */

import { invoke } from "./ipc-adapter";

/**
 * Server 状态
 */
export interface LocalServerStatus {
  /** 是否正在运行 */
  running: boolean;
  /** 当前端口 */
  port: number;
}

/**
 * Server 配置
 */
export interface LocalServerConfig {
  /** 配置的端口 */
  local_api_port: number;
  /** 默认端口 */
  default_port: number;
}

/**
 * 获取本地 API Server 状态
 *
 * @returns Server 状态
 */
export async function getLocalServerStatus(): Promise<LocalServerStatus> {
  return invoke<LocalServerStatus>("get_local_server_status");
}

/**
 * 获取本地 API Server 配置
 *
 * @returns Server 配置
 */
export async function getLocalServerConfig(): Promise<LocalServerConfig> {
  return invoke<LocalServerConfig>("get_local_server_config");
}

/**
 * 更新本地 API Server 端口
 *
 * @param port - 新端口号 (1024-65535)
 * @returns 更新后的状态
 */
export async function updateLocalServerPort(
  port: number
): Promise<LocalServerStatus> {
  return invoke<LocalServerStatus>("update_local_server_port", { port });
}

/**
 * 启动本地 API Server
 *
 * @returns Server 状态
 */
export async function startLocalServer(): Promise<LocalServerStatus> {
  return invoke<LocalServerStatus>("start_local_server");
}

/**
 * 停止本地 API Server
 *
 * @returns Server 状态
 */
export async function stopLocalServer(): Promise<LocalServerStatus> {
  return invoke<LocalServerStatus>("stop_local_server");
}

/**
 * 默认端口
 */
export const DEFAULT_PORT = 19836;

/**
 * 验证端口是否有效
 *
 * @param port - 端口号
 * @returns 是否有效
 */
export function isValidPort(port: number): boolean {
  return Number.isInteger(port) && port >= 1024 && port <= 65535;
}
