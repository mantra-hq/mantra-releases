/**
 * IPC Adapter - Tauri IPC 适配层
 * Story 9.2: Task 1
 *
 * 提供统一的 IPC 调用接口:
 * - 生产环境使用真实 Tauri invoke
 * - 测试环境使用 Mock 实现
 */

import { invoke as tauriInvoke, type InvokeArgs } from "@tauri-apps/api/core";

/**
 * 扩展 Window 类型以支持测试标志
 */
declare global {
  interface Window {
    __PLAYWRIGHT_TEST__?: boolean;
  }
}

/**
 * Mock invoke 处理器类型
 * 接收命令和参数，返回模拟结果
 */
type MockInvokeHandler = <T>(cmd: string, args?: InvokeArgs) => Promise<T>;

/**
 * Mock invoke 处理器
 * 在测试环境下注入
 */
let mockInvokeHandler: MockInvokeHandler | null = null;

/**
 * 检测是否为测试环境
 *
 * 测试环境判断条件:
 * 1. window.__PLAYWRIGHT_TEST__ === true (URL 参数 ?playwright 触发)
 *
 * @returns true 如果在测试环境中运行
 */
export function isTestEnv(): boolean {
  if (typeof window === "undefined") return false;
  return window.__PLAYWRIGHT_TEST__ === true;
}

/**
 * 设置 Mock invoke 处理器
 *
 * 在测试环境初始化时调用，注入 mock 实现
 *
 * @param handler - Mock invoke 处理函数
 */
export function setMockInvoke(handler: MockInvokeHandler): void {
  mockInvokeHandler = handler;
}

/**
 * 清除 Mock invoke 处理器
 *
 * 用于测试清理
 */
export function clearMockInvoke(): void {
  mockInvokeHandler = null;
}

/**
 * 统一的 IPC 调用函数
 *
 * 在测试环境中使用 mock 处理器，
 * 生产环境中使用真实 Tauri invoke
 *
 * @param cmd - Tauri 命令名
 * @param args - 命令参数
 * @returns 命令执行结果
 */
export async function invoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  if (isTestEnv() && mockInvokeHandler) {
    return mockInvokeHandler<T>(cmd, args);
  }
  return tauriInvoke<T>(cmd, args);
}

/**
 * 检查 Mock 处理器是否已设置
 *
 * 用于调试和测试验证
 *
 * @returns true 如果 mock 处理器已设置
 */
export function hasMockHandler(): boolean {
  return mockInvokeHandler !== null;
}
