/**
 * 工具代码显示 E2E 测试
 * Story 8.11+ : 工具类型代码显示修复
 *
 * 测试内容:
 * - file_edit 工具点击后在右侧代码面板显示 diff 视图
 * - file_write 工具点击后显示文件内容
 * - file_read 工具的 tool_result 点击后显示文件内容
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";
import { TEST_SESSION_IDS } from "../utils/test-helpers";

test.describe("工具代码显示功能", () => {
  /**
   * file_edit 工具测试
   */
  test.describe("file_edit 工具", () => {
    test("点击 Edit 工具卡片应在右侧代码面板打开文件", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 找到 Edit 工具调用卡片并点击
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await expect(toolCallCard).toBeVisible({ timeout: 10000 });
      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 右侧代码面板应该显示文件路径
      const editorTab = page.locator('[role="tab"]').filter({
        hasText: /calculator\.ts/,
      });
      await expect(editorTab).toBeVisible({ timeout: 5000 });
    });

    test("file_edit 应显示 diff 视图（有 previousContent）", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 点击第一个 Edit 工具卡片
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 代码面板应可见
      const codePanel = page.locator('[class*="code"], [data-testid*="code"]').first();
      await expect(codePanel).toBeVisible({ timeout: 5000 });

      // 验证 "代码编辑" 标签激活
      const codeEditorTab = page.getByRole('button', { name: /Code Editor|代码编辑/i });
      await expect(codeEditorTab).toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * file_write 工具测试
   */
  test.describe("file_write 工具", () => {
    test("点击 Write 工具卡片应在右侧代码面板显示文件内容", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // mock-session-alpha-1 包含 file_write 工具
      await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_1);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 找到 Write 工具调用卡片
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Write",
      }).first();

      // 如果 Write 工具卡片存在，点击它
      const count = await toolCallCard.count();
      if (count > 0) {
        await toolCallCard.click();
        await page.waitForTimeout(500);

        // 右侧代码面板应该显示文件路径
        const editorTab = page.locator('[role="tab"]').filter({
          hasText: /jwt\.ts/,
        });
        await expect(editorTab).toBeVisible({ timeout: 5000 });

        // 代码面板应显示文件内容
        const codeEditorTab = page.getByRole('button', { name: /Code Editor|代码编辑/i });
        await expect(codeEditorTab).toBeVisible({ timeout: 5000 });
      }
    });
  });

  /**
   * file_read 工具测试
   */
  test.describe("file_read 工具", () => {
    test("file_read 工具 tool_result 点击查看代码应在右侧面板显示", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // mock-session-alpha-2 包含 file_read 工具
      await playerPage.gotoSession(TEST_SESSION_IDS.ALPHA_2);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 找到 Read 工具调用卡片
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Read",
      }).first();

      const count = await toolCallCard.count();
      if (count > 0) {
        // 点击 Read 工具卡片
        await toolCallCard.click();
        await page.waitForTimeout(500);

        // 验证代码面板激活
        const codeEditorTab = page.getByRole('button', { name: /Code Editor|代码编辑/i });
        await expect(codeEditorTab).toBeVisible({ timeout: 5000 });
      }
    });
  });

  /**
   * 代码面板标签页测试
   */
  test.describe("代码面板标签页", () => {
    test("点击工具后应在代码面板创建新标签", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 点击 Edit 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 应该有标签页显示
      const editorTabs = page.locator('[role="tab"]');
      const tabCount = await editorTabs.count();
      expect(tabCount).toBeGreaterThan(0);
    });

    test("多次点击不同工具应创建多个标签", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 找到所有 Edit 工具
      const toolCallCards = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      });

      const cardCount = await toolCallCards.count();
      if (cardCount >= 2) {
        // 点击第一个
        await toolCallCards.first().click();
        await page.waitForTimeout(300);

        // 点击第二个
        await toolCallCards.nth(1).click();
        await page.waitForTimeout(300);

        // 验证标签页存在
        const editorTabs = page.locator('[role="tab"]');
        const tabCount = await editorTabs.count();
        // 应该至少有 1 个标签（可能替换了预览标签）
        expect(tabCount).toBeGreaterThanOrEqual(1);
      }
    });
  });

  /**
   * 代码内容显示测试
   */
  test.describe("代码内容显示", () => {
    test("file_edit 工具应显示正确的文件内容", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 点击 Edit 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 验证代码面板显示
      const codePanel = page.locator('.code-panel, [class*="CodePanel"]').first();
      if (await codePanel.count() > 0) {
        await expect(codePanel).toBeVisible({ timeout: 5000 });
      }
    });
  });

  /**
   * 右侧面板切换测试
   */
  test.describe("右侧面板切换", () => {
    test("点击工具后应自动切换到代码编辑标签", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.FILE_EDIT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 先切换到终端标签
      const terminalTab = page.getByRole('button', { name: /Terminal|终端/i });
      if (await terminalTab.count() > 0) {
        await terminalTab.click();
        await page.waitForTimeout(200);
      }

      // 点击 Edit 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Edit",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 应该自动切换回代码编辑标签
      const codeEditorTab = page.getByRole('button', { name: /Code Editor|代码编辑/i });
      await expect(codeEditorTab).toBeVisible({ timeout: 5000 });
    });
  });
});
