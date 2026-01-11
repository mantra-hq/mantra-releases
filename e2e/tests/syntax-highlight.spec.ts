/**
 * 语法高亮 E2E 测试
 *
 * 测试内容:
 * - Dart 文件应正确应用语法高亮
 * - Monaco Editor 应使用正确的语言模式
 * - 语法高亮应渲染多个 token 类别
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";
import { TEST_SESSION_IDS } from "../utils/test-helpers";

test.describe("语法高亮测试", () => {
  /**
   * Dart 语法高亮测试
   */
  test.describe("Dart 语法高亮", () => {
    test("应正确渲染包含 Dart 文件的会话", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.DART_HIGHLIGHT);

      // 验证消息列表加载
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 会话应有 3 条消息
      const count = await playerPage.getMessageCount();
      expect(count).toBe(3);
    });

    test("Dart 文件工具调用卡片应可见", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.DART_HIGHLIGHT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // ToolCallCard 显示工具名 "Write"
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Write",
      });

      // 应该有 Write 工具调用卡片
      await expect(toolCallCard.first()).toBeVisible({ timeout: 10000 });
    });

    test("点击 Dart 文件应在右侧面板打开", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.DART_HIGHLIGHT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 找到 Write 工具调用卡片并点击
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Write",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 右侧代码面板应该显示文件路径 (user.dart)
      const editorTab = page.locator('[role="tab"]').filter({
        hasText: /user\.dart/,
      });
      await expect(editorTab).toBeVisible({ timeout: 5000 });
    });

    test("Dart 文件应应用语法高亮 (多个 token 类)", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.DART_HIGHLIGHT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 点击 Write 工具卡片打开 Dart 文件
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Write",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(1000); // 等待 Monaco 完全加载

      // Monaco Editor 使用 .mtk* 类进行语法高亮
      // 如果语法高亮工作正常，应该有多个不同的 token 类
      // 例如: .mtk1 (默认), .mtk6 (关键字), .mtk8 (字符串) 等
      const monacoView = page.locator('.view-lines');
      await expect(monacoView).toBeVisible({ timeout: 10000 });

      // 检查是否有语法高亮 token
      // Dart 关键字如 class, final, void 应该有特殊的 token 类
      const tokens = page.locator('.view-lines span[class*="mtk"]');
      const tokenCount = await tokens.count();

      // 至少应该有多个 token（说明不是纯文本）
      expect(tokenCount).toBeGreaterThan(5);

      // 检查是否有多个不同的 token 类型
      const uniqueClasses = new Set<string>();
      for (let i = 0; i < Math.min(tokenCount, 20); i++) {
        const className = await tokens.nth(i).getAttribute('class');
        if (className) {
          const mtkMatch = className.match(/mtk\d+/);
          if (mtkMatch) {
            uniqueClasses.add(mtkMatch[0]);
          }
        }
      }

      // 语法高亮应该产生至少 2 种不同的 token 类型
      // (plaintext 通常只有 1 种或没有)
      expect(uniqueClasses.size).toBeGreaterThanOrEqual(2);
    });

    test("编辑器标签页应显示 .dart 文件名", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.DART_HIGHLIGHT);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await expect(playerPage.messageItems.first()).toBeVisible({ timeout: 5000 });

      // 点击 Write 工具
      const toolCallCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: "Write",
      }).first();

      await toolCallCard.click();
      await page.waitForTimeout(500);

      // 标签页应显示 .dart 文件名
      const tabText = page.getByText(/user\.dart/);
      await expect(tabText.first()).toBeVisible({ timeout: 5000 });
    });
  });
});
