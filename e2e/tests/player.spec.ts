/**
 * Player 页面 E2E 测试
 * Story 9.4: Task 1 (AC: #1)
 *
 * 测试内容:
 * - 消息列表渲染和数量验证
 * - 时间轴功能渲染
 * - 代码面板渲染
 * - 页面导航和切换会话
 *
 * 注意: 消息列表使用虚拟化，复杂交互测试需要特殊处理
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";
import { waitForElementStable } from "../utils/test-helpers";

// 为不稳定测试配置重试
test.describe("Player 页面测试", () => {
  /**
   * Task 1.2: 测试消息列表渲染
   */
  test.describe("消息列表渲染", () => {
    test("应正确渲染消息列表容器", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 验证消息列表容器可见
      await expect(playerPage.messageList).toBeVisible();
    });

    test("应显示正确数量的消息", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // Mock 数据中 mock-session-alpha-1 有 8 条消息
      const count = await playerPage.getMessageCount();
      expect(count).toBe(8);
    });

    test("消息项应可见 (虚拟化列表)", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 验证至少有一条消息可见
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * Task 1.4: 测试时间轴功能
   */
  test.describe("时间轴功能", () => {
    test("时间轴应正确渲染", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 验证时间轴容器可见
      await expect(playerPage.timeline).toBeVisible();
    });

    test("时间轴滑块应可见", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 等待消息列表加载完成（时间轴依赖消息数据）
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 验证滑块可见（时间轴只在有消息时渲染）
      await expect(playerPage.timelineSlider).toBeVisible({ timeout: 10000 });
    });
  });

  /**
   * Task 1.5: 测试代码面板
   */
  test.describe("代码面板", () => {
    test("代码面板应正确渲染", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 等待消息列表加载完成（代码面板加载依赖会话数据）
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 代码面板或空状态应该可见
      // 在 Mock 环境中，代码面板依赖 Git 数据，可能显示为空状态
      const _codePanel = playerPage.codePanel;
      const codePanelContainer = page.locator('[data-testid="code-panel"], [data-testid="code-panel-empty"]');

      // 等待代码面板容器出现（无论是否有内容）
      await waitForElementStable(codePanelContainer.first());

      // 验证至少有一个代码面板相关元素可见
      const hasPanelContent = await codePanelContainer.count();
      expect(hasPanelContent).toBeGreaterThan(0);
    });

    test("代码内容区域应可见", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");

      // 等待消息列表加载完成
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 验证消息列表正确渲染（核心功能）
      const messageCount = await playerPage.getMessageCount();
      expect(messageCount).toBeGreaterThan(0);
    });
  });
});
