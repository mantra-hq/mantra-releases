/**
 * 压缩模式消息插入功能 E2E 测试
 * Story 10.5: AC #1, #2, #3, #4
 *
 * 测试内容:
 * - AC1: 插入热区显示与交互
 * - AC2: 插入对话框功能
 * - AC3: 插入确认流程
 * - AC4: 插入删除功能
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";

test.describe("压缩模式消息插入功能 (Story 10.5)", () => {
  /**
   * AC1: 插入热区测试
   */
  test.describe("AC1: 插入热区", () => {
    test("切换到压缩模式后应显示消息列表", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // 导航到会话
      await playerPage.gotoSession("mock-session-alpha-1");

      // 等待消息列表加载
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 点击压缩模式 Tab
      const compressModeTab = page.locator('[data-testid="mode-compress"]');
      await compressModeTab.click();

      // 等待压缩模式消息列表可见
      const originalMessageList = page.locator('[data-testid="original-message-list"]');
      await expect(originalMessageList).toBeVisible({ timeout: 10000 });
    });

    test("消息之间应显示插入触发器", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 验证插入触发器存在
      const insertTriggers = page.locator('[data-testid="insert-message-trigger"]');
      
      // 至少应该有一个触发器 (列表开头的 index=-1)
      await expect(insertTriggers.first()).toBeVisible({ timeout: 5000 });
    });

    test("悬停插入热区应显示虚线边框和 Plus 图标", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      // 找到第一个插入触发器 (默认状态透明占位)
      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await expect(insertTrigger).toBeVisible({ timeout: 5000 });

      // AC1: 默认状态透明占位 h-2
      await expect(insertTrigger).toHaveClass(/h-2/);

      // 悬停
      await insertTrigger.hover();

      // 等待过渡动画
      await page.waitForTimeout(300);

      // 悬停后触发器应该展开到 h-6
      await expect(insertTrigger).toHaveClass(/h-6/);
    });

    test("点击插入热区应打开插入对话框", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定
      await page.waitForTimeout(500);

      // 关闭可能打开的任何 tooltip 或 dialog overlay
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      // 点击第一个插入触发器 (使用 dispatchEvent 确保点击事件触发)
      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      // 验证对话框打开
      const dialog = page.locator('[data-testid="insert-message-dialog"]');
      await expect(dialog).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * AC2: 插入对话框测试
   */
  test.describe("AC2: 插入对话框", () => {
    test.beforeEach(async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式并打开对话框
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      await expect(page.locator('[data-testid="insert-message-dialog"]')).toBeVisible({ timeout: 5000 });
    });

    test("应显示角色选择按钮 (默认选中用户)", async ({ page }) => {
      const userButton = page.locator('[data-testid="role-user-button"]');
      const assistantButton = page.locator('[data-testid="role-assistant-button"]');

      await expect(userButton).toBeVisible();
      await expect(assistantButton).toBeVisible();

      // 用户按钮应该是激活状态
      await expect(userButton).toHaveAttribute("data-state", "on");
      await expect(assistantButton).toHaveAttribute("data-state", "off");
    });

    test("点击助手按钮应切换角色选择", async ({ page }) => {
      const userButton = page.locator('[data-testid="role-user-button"]');
      const assistantButton = page.locator('[data-testid="role-assistant-button"]');

      await assistantButton.click();

      await expect(userButton).toHaveAttribute("data-state", "off");
      await expect(assistantButton).toHaveAttribute("data-state", "on");
    });

    test("应显示内容输入区域", async ({ page }) => {
      const contentInput = page.locator('[data-testid="content-input"]');
      await expect(contentInput).toBeVisible();
      await expect(contentInput).toBeFocused(); // 应该自动聚焦
    });

    test("应实时显示 Token 统计", async ({ page }) => {
      const tokenDisplay = page.locator('[data-testid="token-count-display"]');
      await expect(tokenDisplay).toBeVisible();

      // 初始应显示 0
      await expect(tokenDisplay).toContainText("0");

      // 输入内容
      const contentInput = page.locator('[data-testid="content-input"]');
      await contentInput.fill("这是一段测试文本，用于验证 Token 计算功能");

      // 等待 debounce (150ms)
      await page.waitForTimeout(300);

      // Token 数量应该大于 0
      const tokenText = await tokenDisplay.textContent();
      const tokenMatch = tokenText?.match(/(\d+)/);
      expect(tokenMatch).toBeTruthy();
      expect(Number(tokenMatch?.[1])).toBeGreaterThan(0);
    });

    test("空内容时确认按钮应禁用", async ({ page }) => {
      const confirmButton = page.locator('[data-testid="confirm-button"]');
      await expect(confirmButton).toBeDisabled();
    });

    test("有内容时确认按钮应启用", async ({ page }) => {
      const contentInput = page.locator('[data-testid="content-input"]');
      await contentInput.fill("测试内容");

      const confirmButton = page.locator('[data-testid="confirm-button"]');
      await expect(confirmButton).toBeEnabled();
    });

    test("点击取消按钮应关闭对话框", async ({ page }) => {
      const dialog = page.locator('[data-testid="insert-message-dialog"]');
      const cancelButton = page.locator('[data-testid="cancel-button"]');

      await cancelButton.click();

      await expect(dialog).not.toBeVisible();
    });

    test("按 Escape 键应关闭对话框", async ({ page }) => {
      const dialog = page.locator('[data-testid="insert-message-dialog"]');

      await page.keyboard.press("Escape");

      await expect(dialog).not.toBeVisible();
    });
  });

  /**
   * AC3: 插入确认测试
   */
  test.describe("AC3: 插入确认", () => {
    test("确认插入后应在列表中显示新消息卡片", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      // 记录插入前的已插入卡片数量
      const initialInsertedCards = await page.locator('[data-testid="inserted-message-card"]').count();

      // 打开对话框并插入消息
      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      await expect(page.locator('[data-testid="insert-message-dialog"]')).toBeVisible({ timeout: 5000 });

      // 输入内容
      const contentInput = page.locator('[data-testid="content-input"]');
      await contentInput.fill("E2E 测试插入的消息内容");

      // 点击确认
      const confirmButton = page.locator('[data-testid="confirm-button"]');
      await confirmButton.click();

      // 对话框应关闭
      await expect(page.locator('[data-testid="insert-message-dialog"]')).not.toBeVisible();

      // 应显示新的已插入消息卡片
      const insertedCards = page.locator('[data-testid="inserted-message-card"]');
      await expect(insertedCards).toHaveCount(initialInsertedCards + 1, { timeout: 5000 });
    });

    test("已插入消息卡片应显示绿色边框和 Sparkles 图标", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式并插入消息
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      const contentInput = page.locator('[data-testid="content-input"]');
      await contentInput.fill("测试消息");

      await page.locator('[data-testid="confirm-button"]').click();

      // 验证已插入卡片样式
      const insertedCard = page.locator('[data-testid="inserted-message-card"]').first();
      await expect(insertedCard).toBeVisible({ timeout: 5000 });

      // 验证绿色边框类
      await expect(insertedCard).toHaveClass(/border-green-500/);

      // 验证显示 "已插入" 或 "Inserted" 标识
      await expect(insertedCard.getByText(/已插入|Inserted/)).toBeVisible();
    });

    test("Ctrl+Enter 快捷键应确认插入", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式并打开对话框
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      await expect(page.locator('[data-testid="insert-message-dialog"]')).toBeVisible({ timeout: 5000 });

      // 输入内容
      const contentInput = page.locator('[data-testid="content-input"]');
      await contentInput.fill("快捷键测试");

      // 使用 Ctrl+Enter 确认
      await page.keyboard.press("Control+Enter");

      // 对话框应关闭
      await expect(page.locator('[data-testid="insert-message-dialog"]')).not.toBeVisible();

      // 应显示已插入消息卡片
      await expect(page.locator('[data-testid="inserted-message-card"]')).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * AC4: 插入删除测试
   */
  test.describe("AC4: 插入删除", () => {
    test.beforeEach(async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式并插入一条消息
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      await page.locator('[data-testid="content-input"]').fill("待删除的消息");
      await page.locator('[data-testid="confirm-button"]').click();

      await expect(page.locator('[data-testid="inserted-message-card"]')).toBeVisible({ timeout: 5000 });
    });

    test("已插入消息卡片应显示删除按钮", async ({ page }) => {
      const insertedCard = page.locator('[data-testid="inserted-message-card"]').first();
      const removeButton = insertedCard.locator('[data-testid="remove-inserted-button"]');

      await expect(removeButton).toBeVisible();
    });

    test("点击删除按钮应移除已插入的消息", async ({ page }) => {
      // 确保没有 overlay
      await page.keyboard.press("Escape");
      await page.waitForTimeout(300);

      // 等待 inserted card 和删除按钮都可见
      const insertedCard = page.locator('[data-testid="inserted-message-card"]').first();
      await expect(insertedCard).toBeVisible();

      const removeButton = insertedCard.locator('[data-testid="remove-inserted-button"]');
      await expect(removeButton).toBeVisible();

      // 先滚动到元素确保可见
      await removeButton.scrollIntoViewIfNeeded();
      await page.waitForTimeout(200);

      // 记录删除前的数量
      const countBefore = await page.locator('[data-testid="inserted-message-card"]').count();

      // 点击删除按钮 - 使用 dispatchEvent 绕过覆盖问题
      await removeButton.dispatchEvent("click");

      // 等待卡片消失
      await expect(insertedCard).not.toBeVisible({ timeout: 5000 });

      // 验证数量减少
      const countAfter = await page.locator('[data-testid="inserted-message-card"]').count();
      expect(countAfter).toBe(countBefore - 1);
    });
  });

  /**
   * 预览列表同步测试 (AC3 扩展)
   */
  test.describe("预览列表同步", () => {
    test("插入消息后预览列表应显示新消息", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 切换到压缩模式
      await page.locator('[data-testid="mode-compress"]').click();
      await expect(page.locator('[data-testid="original-message-list"]')).toBeVisible({ timeout: 10000 });

      // 等待页面稳定并关闭可能的 overlay
      await page.waitForTimeout(500);
      await page.keyboard.press("Escape");
      await page.waitForTimeout(200);

      // 等待预览列表可见
      const previewList = page.locator('[data-testid="compress-preview-list"]');
      await expect(previewList).toBeVisible({ timeout: 10000 });

      // 插入消息
      const insertTrigger = page.locator('[data-testid="insert-message-trigger"]').first();
      await insertTrigger.dispatchEvent("click");

      await page.locator('[data-testid="content-input"]').fill("预览同步测试消息");
      await page.locator('[data-testid="confirm-button"]').click();

      // 预览列表应包含插入类型的消息卡片
      const insertPreviewCard = previewList.locator('[data-testid="preview-message-card"][data-operation="insert"]');
      await expect(insertPreviewCard).toBeVisible({ timeout: 5000 });
    });
  });
});
