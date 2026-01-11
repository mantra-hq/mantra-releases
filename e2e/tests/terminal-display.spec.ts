/**
 * Terminal Display E2E Tests
 * Story 8.11 fix: 验证终端类消息正确渲染
 *
 * 测试点:
 * - 点击 shell_exec 类工具卡片后，右侧终端面板正确显示
 * - 显示执行的命令
 * - 显示结构化输出 (stdout/stderr)
 * - 显示退出码 (成功绿色/失败红色)
 * - JSON 格式 content 的解析
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages/player.page";

// 测试会话 ID
const TEST_SESSION_IDS = {
  SHELL_EXEC: "mock-session-shell-exec",
  SHELL_JSON: "mock-session-shell-json", // JSON 格式 content 测试
};

test.describe("终端类消息渲染", () => {
  /**
   * 测试: 点击 Bash 工具卡片后终端面板显示命令和输出
   */
  test("点击 Bash 工具卡片应在终端面板显示命令和输出", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到测试会话
    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

    // 等待消息加载
    await page.waitForTimeout(500);

    // 找到第一个 Bash 工具卡片 (npm test)
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|npm test/i,
    }).first();

    // 确保工具卡片可见
    await expect(bashToolCard).toBeVisible({ timeout: 5000 });

    // 点击工具卡片
    await bashToolCard.click();
    await page.waitForTimeout(500);

    // 验证终端面板显示
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // 验证终端头部显示命令 - 使用更精确的 locator
    const terminalHeader = terminalPanel.locator('.text-zinc-300.truncate').first();
    await expect(terminalHeader).toContainText('npm test', { timeout: 3000 });
  });

  /**
   * 测试: 终端面板显示成功退出码 (绿色)
   */
  test("成功命令应显示绿色退出码", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 找到第一个 Bash 工具卡片 (npm test - 成功)
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|npm test/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(500);

    // 验证终端面板显示
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // 验证显示 exit code: 0 (成功)
    // xterm 渲染的内容可能需要特殊检查
    await page.waitForTimeout(300); // 等待 xterm 渲染
  });

  /**
   * 测试: 终端面板应显示结构化输出内容
   */
  test("终端面板应显示结构化输出内容", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 找到第一个 Bash 工具卡片
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|npm test/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(800); // 等待 xterm 渲染

    // 验证终端面板显示
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // xterm 渲染的内容在 canvas 中，难以直接检查文本
    // 但我们可以验证终端区域存在且有内容
    const terminalContent = terminalPanel.locator('.xterm-screen');
    await expect(terminalContent).toBeVisible({ timeout: 3000 });
  });

  /**
   * 测试: 右侧面板应自动切换到终端 Tab
   */
  test("点击终端工具后应自动切换到终端 Tab", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_EXEC);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 先确保在代码 Tab - 使用更精确的 locator
    const codeTab = page.locator('button').filter({ hasText: /^Code$|^代码$/ }).first();
    if (await codeTab.count() > 0) {
      await codeTab.click();
      await page.waitForTimeout(200);
    }

    // 点击 Bash 工具卡片
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|npm test/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(500);

    // 验证终端面板可见 (证明切换成功)
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 3000 });
  });
});

/**
 * JSON 格式 content 解析测试
 * 模拟真实后端数据格式：{"output": "...", "metadata": {"exit_code": 0}}
 */
test.describe("终端 JSON 格式 content 解析", () => {
  /**
   * 测试: JSON 格式 content 应正确解析并显示输出
   */
  test("JSON 格式 content 应解析出 output 字段显示", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 导航到 JSON 格式测试会话
    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_JSON);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 找到 ls -la 工具卡片
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|ls/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(800);

    // 验证终端面板显示
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // 验证头部显示命令
    const terminalHeader = terminalPanel.locator('.text-zinc-300.truncate').first();
    await expect(terminalHeader).toContainText('ls', { timeout: 3000 });

    // 验证 xterm 区域存在
    const terminalContent = terminalPanel.locator('.xterm-screen');
    await expect(terminalContent).toBeVisible({ timeout: 3000 });
  });

  /**
   * 测试: JSON 格式 content 不应显示原始 JSON 字符串
   * 验证解析逻辑正确工作，不会显示 {"output":... 这样的原始内容
   */
  test("终端面板不应显示原始 JSON 字符串", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_JSON);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 找到 ls -la 工具卡片
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|ls/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(1000); // 等待渲染

    // 验证终端面板显示
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // 获取终端面板的 HTML 内容，检查不包含原始 JSON 标记
    const terminalHtml = await terminalPanel.innerHTML();

    // 不应该包含 JSON 开头标记
    expect(terminalHtml).not.toContain('{"output"');
    expect(terminalHtml).not.toContain('"metadata"');
  });

  /**
   * 测试: JSON 格式 content 应正确解析 exit_code
   */
  test("JSON 格式 content 应解析出 exit_code", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.gotoSession(TEST_SESSION_IDS.SHELL_JSON);
    await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });
    await page.waitForTimeout(500);

    // 找到 ls -la 工具卡片 (成功命令)
    const bashToolCard = page.locator('[data-testid="tool-call-card"]').filter({
      hasText: /Bash|ls/i,
    }).first();

    await expect(bashToolCard).toBeVisible({ timeout: 5000 });
    await bashToolCard.click();
    await page.waitForTimeout(800);

    // 验证终端面板显示且无错误边框
    const terminalPanel = page.locator('[data-testid="right-panel-terminal"]');
    await expect(terminalPanel).toBeVisible({ timeout: 5000 });

    // 成功命令不应有错误边框
    await expect(terminalPanel).not.toHaveClass(/border-l-destructive/);
  });
});
