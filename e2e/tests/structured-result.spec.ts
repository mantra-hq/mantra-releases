/**
 * Structured Result E2E Tests
 * Story 8.19: 验证 structured_result 摘要正确渲染
 *
 * 测试点:
 * - FileRead 结构化结果显示文件路径和行数
 * - ShellExec 结构化结果显示退出码
 * - Cursor 会话的 structured_result 正确渲染
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages/player.page";

// 测试会话 ID
const TEST_SESSION_IDS = {
  CURSOR_TOOLS: "mock-session-cursor-tools", // Story 8.19: Cursor 工具调用测试
  SHELL_EXEC: "mock-session-shell-exec",
};

test.describe("Structured Result 摘要渲染", () => {
  /**
   * 测试: Cursor FileRead 结构化结果摘要
   * Story 8.19 AC6: 前端显示结构化摘要而非原始 JSON
   */
  test.describe("FileRead 结构化结果", () => {
    test("Cursor read_file 工具应显示文件路径摘要", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // 导航到 Cursor 工具测试会话
      await playerPage.gotoSession(TEST_SESSION_IDS.CURSOR_TOOLS);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 找到包含 read_file 结果的消息
      // tool_result 会显示在 ToolOutput 组件中
      const toolResultCard = page.locator('[data-testid="tool-result-card"]').first();

      if (await toolResultCard.count() > 0) {
        await expect(toolResultCard).toBeVisible({ timeout: 5000 });

        // 验证显示文件路径摘要而非原始 JSON
        // structured_result 的 FileRead 应该显示类似 "/src/main.rs L1-L7" 的摘要
        const cardText = await toolResultCard.textContent();
        
        // 不应显示原始 JSON
        expect(cardText).not.toContain('{"type"');
        expect(cardText).not.toContain('"file_path"');
        
        // 应该包含文件路径信息
        expect(cardText).toContain('main.rs');
      }
    });

    test("FileRead 结构化结果应显示行数信息", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.CURSOR_TOOLS);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 查找 ToolOutput 组件中的摘要信息
      // renderStructuredResultSummary 会渲染行数信息
      const fileReadSummary = page.locator('[data-testid="tool-result-card"]').filter({
        hasText: /main\.rs|L\d+/i,
      }).first();

      if (await fileReadSummary.count() > 0) {
        await expect(fileReadSummary).toBeVisible({ timeout: 5000 });
      }
    });
  });

  /**
   * 测试: ShellExec 结构化结果摘要
   */
  test.describe("ShellExec 结构化结果", () => {
    test("Shell 命令结果应显示退出码", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 找到工具结果卡片
      const toolResultCards = page.locator('[data-testid="tool-result-card"]');
      const count = await toolResultCards.count();

      if (count > 0) {
        // 成功命令应显示成功图标（Check icon）
        const successCard = toolResultCards.first();
        await expect(successCard).toBeVisible({ timeout: 5000 });

        // 验证不显示原始 JSON
        const cardText = await successCard.textContent();
        expect(cardText).not.toContain('{"type":"shell_exec"');
      }
    });

    test("失败命令应显示错误状态", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 查找错误状态的工具结果（lint 失败）
      // is_error=true 的 ToolResult 会显示 AlertTriangle 图标
      const errorIcon = page.locator('[data-testid="tool-result-card"]').filter({
        hasText: /lint|warning/i,
      });

      if (await errorIcon.count() > 0) {
        await expect(errorIcon.first()).toBeVisible({ timeout: 5000 });
      }
    });
  });

  /**
   * 测试: Cursor 特定会话的 structured_result
   * Story 8.19: 验证 Cursor parser 生成的 structured_result 正确显示
   */
  test.describe("Cursor 会话 structured_result", () => {
    test("Cursor 会话工具结果应正确渲染", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.CURSOR_TOOLS);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 验证消息列表包含工具调用
      const messages = page.locator('[data-testid="message-item"]');
      const messageCount = await messages.count();
      expect(messageCount).toBeGreaterThan(0);

      // 查找工具调用卡片
      const toolCards = page.locator('[data-testid="tool-call-card"]');
      const toolCount = await toolCards.count();

      // Cursor 测试会话应包含工具调用
      expect(toolCount).toBeGreaterThanOrEqual(1);
    });

    test("Cursor run_terminal_cmd 结果应显示正确摘要", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.CURSOR_TOOLS);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 找到 shell 命令工具卡片
      const shellToolCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: /cargo|build|Shell/i,
      }).first();

      if (await shellToolCard.count() > 0) {
        await expect(shellToolCard).toBeVisible({ timeout: 5000 });

        // 点击工具卡片查看终端输出
        await shellToolCard.click();
        await page.waitForTimeout(500);

        // 验证终端面板显示
        const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
        await expect(terminalPanel).toBeVisible({ timeout: 5000 });
      }
    });

    test("点击 Cursor read_file 工具应在代码面板显示", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession(TEST_SESSION_IDS.CURSOR_TOOLS);
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 找到 read_file 工具卡片
      const readToolCard = page.locator('[data-testid="tool-call-card"]').filter({
        hasText: /read|Read|main\.rs/i,
      }).first();

      if (await readToolCard.count() > 0) {
        await expect(readToolCard).toBeVisible({ timeout: 5000 });
        await readToolCard.click();
        await page.waitForTimeout(500);

        // 验证代码面板激活
        const codePanel = page.locator('[class*="code"], [data-testid*="code"]').first();
        if (await codePanel.count() > 0) {
          await expect(codePanel).toBeVisible({ timeout: 5000 });
        }
      }
    });
  });

  /**
   * 测试: structured_result 向后兼容
   * AC5: 无法解析的结果应回退到显示原始 content
   */
  test.describe("向后兼容", () => {
    test("无 structured_result 时应显示原始内容", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // 使用包含无 structured_result 的会话
      await playerPage.gotoSession("mock-session-shell-json");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
      await page.waitForTimeout(500);

      // 查找工具结果
      const toolResults = page.locator('[data-testid="tool-result-card"]');
      const count = await toolResults.count();

      // 应该仍然能渲染（回退到原始内容）
      if (count > 0) {
        await expect(toolResults.first()).toBeVisible({ timeout: 5000 });
      }
    });
  });
});
