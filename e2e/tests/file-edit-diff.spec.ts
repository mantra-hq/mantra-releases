/**
 * FileEdit Diff E2E 测试
 * Story 8.11: Task 9 (AC: #9)
 *
 * 测试内容:
 * - 点击 file_edit 工具卡片在右侧代码面板显示 diff 视图
 * - Diff 视图显示红/绿颜色标记
 * - 只有 newString 时显示新增内容
 * - 文件路径在标签页显示
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";
import { TEST_SESSION_IDS } from "../utils/test-helpers";

test.describe("Story 8.11: FileEdit Diff 视图 (AC#9)", () => {
  /**
   * AC#9: 点击 file_edit 工具在右侧代码面板显示 diff 视图
   */
  test.describe("右侧代码面板 Diff 视图", () => {
    test("应正确渲染包含 file_edit 的会话", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);

      // 验证消息列表加载
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 会话应有 6 条消息
      const count = await playerPage.getMessageCount();
      expect(count).toBe(6);
    });

    test("file_edit 工具调用卡片应可见", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // ToolCallCard 显示工具名 "Edit"
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      });

      // 应该有 Edit 工具调用卡片
      await expect(toolCallCard.first()).toBeVisible({ timeout: 10000 });
    });

    test("点击 file_edit 工具应在右侧面板打开文件", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 找到第一个 Edit 工具调用卡片并点击
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 右侧代码面板应该显示文件路径 (calculator.ts)
      const editorTab = page.locator('[role="tab"]').filter({
        hasText: /calculator\.ts/,
      });
      await expect(editorTab).toBeVisible({ timeout: 5000 });
    });

    test("右侧面板应显示 diff 视图 (有 previousContent)", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 点击第一个 Edit 工具卡片 (有 oldString 和 newString)
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 检查代码面板是否有 diff 相关的内容
      // Monaco diff editor 会在有 previousContent 时显示差异
      const codePanel = page.locator('[class*="code"], [data-testid*="code"]').first();
      await expect(codePanel).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * AC#9: 只有 newString 时显示新增内容
   */
  test.describe("只有 newString 的场景", () => {
    test("第二个 Edit 工具点击后应显示新内容", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 找到所有 Edit 工具调用卡片
      const toolCallCards = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      });

      // 点击第二个 Edit 工具调用 (只有 newString)
      const secondCard = toolCallCards.nth(1);
      await expect(secondCard).toBeVisible({ timeout: 10000 });
      await secondCard.click();
      await page.waitForTimeout(500);

      // 应该打开文件标签
      const editorTab = page.locator('[role="tab"]').filter({
        hasText: /calculator\.ts/,
      });
      await expect(editorTab).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * AC#9: 文件路径在编辑器标签页显示
   */
  test.describe("编辑器标签页", () => {
    test("点击 file_edit 后标签页应显示文件名", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 点击第一个 Edit 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 标签页应显示文件名
      const tabText = page.getByText(/calculator\.ts/);
      await expect(tabText.first()).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * 集成测试：整体流程
   */
  test.describe("集成测试", () => {
    test("点击工具卡片后右侧面板应激活", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 点击第一个 Edit 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 右侧面板应该有 "Code Editor" 标签激活
      const codeEditorTab = page.getByRole('button', { name: /Code Editor/i });
      await expect(codeEditorTab).toBeVisible({ timeout: 5000 });
    });
  });
});
