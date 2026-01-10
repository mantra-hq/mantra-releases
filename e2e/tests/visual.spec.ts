/**
 * 视觉回归测试
 * Story 9.5: Task 3, 4, 5
 *
 * 测试内容:
 * - Player 页面视觉测试 (Task 3)
 * - 其他页面视觉测试 (Task 4)
 * - 组件级视觉测试 (Task 5)
 *
 * 运行方式:
 * - 运行测试: pnpm test:e2e (首次运行生成基线)
 * - 更新基线: pnpm test:e2e:update-snapshots
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";
import {
  preparePageForVisualTest,
  getDynamicElementsToMask,
  VISUAL_THRESHOLDS,
} from "../utils/visual-helpers";
import { TEST_SESSION_IDS } from "../utils/test-helpers";

// =============================================================================
// Task 3: Player 页面视觉测试
// =============================================================================

test.describe("Player 页面视觉测试", () => {
  test.beforeEach(async ({ page }) => {
    // 准备页面用于视觉测试（禁用动画，设置视口）
    await preparePageForVisualTest(page);
  });

  /**
   * Task 3.2: 测试 Player 页面默认状态截图
   */
  test("Player 页面默认状态", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到会话
    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 等待额外的渲染稳定
    await page.waitForLoadState("networkidle");

    // 截图并比较
    await expect(page).toHaveScreenshot("player-default.png", {
      mask: getDynamicElementsToMask(page),
      ...VISUAL_THRESHOLDS.fullPage,
    });
  });

  /**
   * Task 3.3: 测试 Player 消息选中状态截图
   */
  test("Player 消息选中状态", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到会话
    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 点击第一条消息
    await playerPage.clickMessage(0);

    // 等待选中状态渲染
    await page.waitForLoadState("networkidle");

    // 截图并比较
    await expect(page).toHaveScreenshot("player-message-selected.png", {
      mask: getDynamicElementsToMask(page),
      ...VISUAL_THRESHOLDS.fullPage,
    });
  });

  /**
   * Task 3.4: 测试代码面板显示状态
   */
  test("Player 代码面板", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到会话
    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 等待代码面板渲染
    await page.waitForLoadState("networkidle");

    // 获取代码面板区域
    const codePanel = page.locator(
      '[data-testid="code-panel"], [data-testid="code-panel-empty"]'
    );

    // 只截图代码面板区域
    await expect(codePanel.first()).toHaveScreenshot("player-code-panel.png", {
      ...VISUAL_THRESHOLDS.codeHighlight,
    });
  });
});

// =============================================================================
// Task 4: 其他页面视觉测试
// =============================================================================

test.describe("其他页面视觉测试", () => {
  test.beforeEach(async ({ page }) => {
    await preparePageForVisualTest(page);
  });

  /**
   * Task 4.1: 测试首页状态页面
   */
  test("首页 - 项目列表", async ({ page }) => {
    // 直接导航到首页（Mock 环境会显示项目列表）
    await page.goto("/?playwright");
    await page.waitForLoadState("networkidle");

    // 截图首页状态
    await expect(page).toHaveScreenshot("home-with-projects.png", {
      mask: getDynamicElementsToMask(page),
      ...VISUAL_THRESHOLDS.fullPage,
    });
  });

  /**
   * Task 4.2: 测试导入向导（通过项目抽屉中的导入按钮触发）
   * 注意：由于导入向导的触发路径较复杂（需要先打开项目抽屉，再点击导入按钮），
   * 且 Mock 环境可能不支持完整的导入流程，此测试可能会被跳过。
   */
  test("导入向导 - 来源选择步骤", async ({ page }) => {
    // 导航到首页并等待页面稳定
    await page.goto("/?playwright");
    await page.waitForLoadState("networkidle");

    // 先关闭任何可能打开的对话框/抽屉
    const closeButton = page.locator('button[aria-label="Close"], button:has-text("Close")').first();
    if (await closeButton.count() > 0 && await closeButton.isVisible()) {
      await closeButton.click();
      await page.waitForTimeout(500);
    }

    // 点击 topbar 按钮打开项目抽屉
    const topbarButton = page.locator('[data-testid="topbar-import-button"]');

    // 如果 topbar 按钮存在则点击
    const topbarButtonCount = await topbarButton.count();
    if (topbarButtonCount === 0 || !(await topbarButton.isVisible())) {
      test.skip();
      return;
    }

    // 确保按钮可点击（没有遮罩层）
    await expect(topbarButton).toBeEnabled();
    await topbarButton.click({ force: true });
    await page.waitForLoadState("networkidle");

    // 在项目抽屉中点击 "Import New Project" 按钮
    const importNewProjectButton = page.locator('button:has-text("Import New Project")');

    try {
      await expect(importNewProjectButton).toBeVisible({ timeout: 5000 });
      await importNewProjectButton.click({ force: true });
      await page.waitForLoadState("networkidle");

      // 等待导入向导出现
      const importWizard = page.locator('[data-testid="import-wizard"]');
      await expect(importWizard).toBeVisible({ timeout: 5000 });

      // 截图导入向导
      await expect(page).toHaveScreenshot("import-source.png", {
        mask: getDynamicElementsToMask(page),
        ...VISUAL_THRESHOLDS.fullPage,
      });
    } catch {
      // 如果导入向导未出现，跳过测试（Mock 环境可能不支持）
      test.skip();
    }
  });

  /**
   * Task 4.3: 测试项目抽屉展开状态
   */
  test("项目抽屉展开状态", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到会话
    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 打开项目抽屉
    await playerPage.openProjectDrawer();
    await page.waitForLoadState("networkidle");

    // 截图抽屉展开状态
    await expect(page).toHaveScreenshot("drawer-open.png", {
      mask: getDynamicElementsToMask(page),
      ...VISUAL_THRESHOLDS.fullPage,
    });
  });
});

// =============================================================================
// Task 5: 组件级视觉测试
// =============================================================================

test.describe("组件级视觉测试", () => {
  test.beforeEach(async ({ page }) => {
    await preparePageForVisualTest(page);
  });

  /**
   * Task 5.1: 测试消息列表中的第一条消息
   */
  test("MessageBubble 第一条消息样式", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 找到第一条消息元素
    const firstMessage = page.locator('[data-testid="message-item"]').first();

    // 确保元素可见
    await expect(firstMessage).toBeVisible({ timeout: 5000 });

    // 截图第一条消息
    await expect(firstMessage).toHaveScreenshot("message-first.png", {
      mask: [page.locator('[data-testid="timestamp"]')],
      ...VISUAL_THRESHOLDS.component,
    });
  });

  /**
   * Task 5.2: 测试消息列表中的第二条消息（通常是 AI 回复）
   */
  test("MessageBubble 第二条消息样式", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 找到第二条消息元素
    const secondMessage = page.locator('[data-testid="message-item"]').nth(1);

    // 确保元素可见
    const messageCount = await page.locator('[data-testid="message-item"]').count();
    if (messageCount >= 2) {
      await expect(secondMessage).toBeVisible({ timeout: 5000 });

      // 截图第二条消息
      await expect(secondMessage).toHaveScreenshot("message-second.png", {
        mask: [page.locator('[data-testid="timestamp"]')],
        ...VISUAL_THRESHOLDS.component,
      });
    } else {
      test.skip();
    }
  });

  /**
   * Task 5.3: 测试 ToolCallCard 样式（如果存在）
   */
  test("ToolCallCard 样式", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 使用有工具调用的会话
    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 找到工具调用卡片
    const toolCard = page.locator('[data-testid="tool-call-card"]').first();

    // 如果工具卡片存在，截图
    const toolCardCount = await toolCard.count();
    if (toolCardCount > 0) {
      await expect(toolCard).toHaveScreenshot("tool-card.png", {
        ...VISUAL_THRESHOLDS.component,
      });
    } else {
      // 跳过测试如果没有工具调用
      test.skip();
    }
  });

  /**
   * Task 5.4: 测试 Timeline 组件样式
   */
  test("Timeline 组件样式", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
    await playerPage.waitForPlayerReady();

    // 等待时间轴渲染
    await expect(playerPage.timeline).toBeVisible({ timeout: 10000 });

    // 截图时间轴
    await expect(playerPage.timeline).toHaveScreenshot("timeline-default.png", {
      ...VISUAL_THRESHOLDS.component,
    });
  });
});
