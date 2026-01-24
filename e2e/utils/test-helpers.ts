/**
 * E2E 测试辅助函数
 * Story 9.4: Task 5
 *
 * 提供:
 * - Mock 数据加载辅助
 * - 常用断言封装
 * - 等待策略辅助（替代 waitForTimeout）
 */

import { Page, Locator, expect } from "@playwright/test";
import {
  MOCK_PROJECTS,
  MOCK_SESSION_SUMMARIES,
  getSessionById,
  getSessionsByProjectId,
} from "../fixtures/mock-data";

// =============================================================================
// Mock 数据加载辅助 (Task 5.2)
// =============================================================================

/**
 * 获取 Mock 项目列表
 */
export function getMockProjects() {
  return MOCK_PROJECTS;
}

/**
 * 获取 Mock 会话摘要列表
 */
export function getMockSessionSummaries() {
  return MOCK_SESSION_SUMMARIES;
}

/**
 * 根据项目 ID 获取会话列表
 */
export function getMockSessionsByProject(projectId: string) {
  return getSessionsByProjectId(projectId);
}

/**
 * 根据会话 ID 获取完整会话数据
 */
export function getMockSession(sessionId: string) {
  return getSessionById(sessionId);
}

/**
 * 获取预期的消息数量（根据 Mock 数据）
 */
export function getExpectedMessageCount(sessionId: string): number {
  const session = getSessionById(sessionId);
  return session?.messages?.length ?? 0;
}

// =============================================================================
// 常用断言封装 (Task 5.3)
// =============================================================================

/**
 * 断言元素可见（带超时）
 */
export async function assertVisible(
  locator: Locator,
  options?: { timeout?: number; message?: string }
): Promise<void> {
  const timeout = options?.timeout ?? 10000;
  await expect(locator, options?.message).toBeVisible({ timeout });
}

/**
 * 断言元素不可见
 */
export async function assertNotVisible(
  locator: Locator,
  options?: { timeout?: number; message?: string }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await expect(locator, options?.message).not.toBeVisible({ timeout });
}

/**
 * 断言元素包含文本
 */
export async function assertContainsText(
  locator: Locator,
  text: string | RegExp,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await expect(locator).toContainText(text, { timeout });
}

/**
 * 断言元素数量
 */
export async function assertCount(
  locator: Locator,
  count: number,
  options?: { timeout?: number; message?: string }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await expect(locator, options?.message).toHaveCount(count, { timeout });
}

/**
 * 断言元素数量大于指定值
 */
export async function assertCountGreaterThan(
  locator: Locator,
  minCount: number,
  options?: { timeout?: number; message?: string }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  // 使用 poll 等待元素数量满足条件
  await expect
    .poll(async () => await locator.count(), { timeout })
    .toBeGreaterThan(minCount);
}

/**
 * 断言 URL 匹配
 */
export async function assertUrlMatches(
  page: Page,
  pattern: string | RegExp,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await expect(page).toHaveURL(pattern, { timeout });
}

/**
 * 断言元素被聚焦
 */
export async function assertFocused(
  locator: Locator,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await expect(locator).toBeFocused({ timeout });
}

// =============================================================================
// 等待策略辅助（替代 waitForTimeout）
// =============================================================================

/**
 * 等待网络请求完成
 * 替代 waitForTimeout，等待页面网络空闲
 */
export async function waitForNetworkIdle(
  page: Page,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 10000;
  await page.waitForLoadState("networkidle", { timeout });
}

/**
 * 等待元素稳定（不再移动或改变大小）
 * 用于动画完成后的操作
 */
export async function waitForElementStable(
  locator: Locator,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  // 等待元素可见
  await expect(locator).toBeVisible({ timeout });
  // 等待元素稳定（Playwright 会自动等待动画完成）
  await locator.waitFor({ state: "visible", timeout });
}

/**
 * 等待条件满足
 * 用于复杂的等待场景
 */
export async function waitForCondition(
  condition: () => Promise<boolean>,
  options?: { timeout?: number; interval?: number; message?: string }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  const interval = options?.interval ?? 100;
  const message = options?.message ?? "Condition not met within timeout";

  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    if (await condition()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
  throw new Error(message);
}

/**
 * 等待元素内容变化
 */
export async function waitForContentChange(
  locator: Locator,
  previousContent: string,
  options?: { timeout?: number }
): Promise<string> {
  const timeout = options?.timeout ?? 5000;
  await expect
    .poll(async () => await locator.textContent(), { timeout })
    .not.toBe(previousContent);
  return (await locator.textContent()) ?? "";
}

/**
 * 安全点击 - 等待元素可点击后再点击
 */
export async function safeClick(
  locator: Locator,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;
  await locator.waitFor({ state: "visible", timeout });
  await locator.click({ timeout });
}

/**
 * 关闭可能存在的弹出层（模态框、抽屉等）
 */
export async function dismissOverlays(page: Page): Promise<void> {
  // 按 Escape 关闭可能的弹出层
  await page.keyboard.press("Escape");
  // 等待动画完成
  await page.waitForLoadState("domcontentloaded");
}

// =============================================================================
// 测试数据常量
// =============================================================================

/**
 * 测试会话 ID 常量
 */
export const TEST_SESSION_IDS = {
  ALPHA_1: "mock-session-alpha-1",
  ALPHA_2: "mock-session-alpha-2",
  BETA_1: "mock-session-beta-1",
  BETA_2: "mock-session-beta-2",
  GAMMA_1: "mock-session-gamma-1",
  GAMMA_2: "mock-session-gamma-2",
  FILE_EDIT: "mock-session-file-edit", // Story 8.11: FileEdit Diff 测试
  DART_HIGHLIGHT: "mock-session-dart-highlight", // Dart 语法高亮测试
} as const;

/**
 * 测试项目 ID 常量
 */
export const TEST_PROJECT_IDS = {
  ALPHA: "mock-project-alpha",
  BETA: "mock-project-beta",
  GAMMA: "mock-project-gamma",
} as const;
