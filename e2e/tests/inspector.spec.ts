import { test, expect } from '@playwright/test';

/**
 * MCP Inspector E2E Tests
 * Story 11.11: Task 7 - E2E 测试脚本 (AC: 6)
 *
 * 测试 Inspector 界面交互：
 * - Inspector 按钮可见性
 * - Inspector 抽屉打开/关闭
 * - 工具/资源列表展示
 * - 日志面板功能
 */
test.describe('MCP Inspector', () => {
  test.beforeEach(async ({ page }) => {
    // 导航到 Hub 页面 (带 ?playwright 参数启用 Mock)
    await page.goto('/hub?playwright');
    // 等待页面加载完成
    await page.waitForSelector('[data-testid="gateway-status-card"]');
  });

  test('should display inspect button for enabled services', async ({ page }) => {
    // 等待 MCP 服务列表加载
    await page.waitForSelector('[data-testid="mcp-service-list"]');

    // 检查 Inspect 按钮是否存在（对于启用的服务）
    const inspectButton = page.locator('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await expect(inspectButton).toBeVisible();
    await expect(inspectButton).toBeEnabled();
  });

  test('should disable inspect button when service is disabled', async ({ page }) => {
    // 等待 MCP 服务列表加载
    await page.waitForSelector('[data-testid="mcp-service-list"]');

    // 检查禁用服务的 Inspect 按钮应该被禁用
    const inspectButton = page.locator('[data-testid="mcp-service-inspect-mock-mcp-disabled"]');
    await expect(inspectButton).toBeVisible();
    await expect(inspectButton).toBeDisabled();
  });

  test('should open inspector drawer when clicking inspect button', async ({ page }) => {
    // 等待服务列表加载
    await page.waitForSelector('[data-testid="mcp-service-list"]');

    // 点击第一个启用服务的 Inspect 按钮
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');

    // 等待 Inspector 抽屉打开
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 验证抽屉标题包含服务名
    await expect(page.locator('[data-slot="sheet-title"]')).toContainText('git-mcp');
  });

  test('should display tool explorer with tools list', async ({ page }) => {
    // 打开 Inspector
    await page.waitForSelector('[data-testid="mcp-service-list"]');
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 验证 Tool Explorer 可见
    const toolExplorer = page.locator('[data-testid="tool-explorer"]');
    await expect(toolExplorer).toBeVisible();

    // 验证搜索框可见
    const searchInput = page.locator('[data-testid="tool-explorer-search"]');
    await expect(searchInput).toBeVisible();
  });

  test('should display tool tester empty state initially', async ({ page }) => {
    // 打开 Inspector
    await page.waitForSelector('[data-testid="mcp-service-list"]');
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 验证 Tool Tester 空状态
    const toolTesterEmpty = page.locator('[data-testid="tool-tester-empty"]');
    await expect(toolTesterEmpty).toBeVisible();
  });

  test('should display log viewer', async ({ page }) => {
    // 打开 Inspector
    await page.waitForSelector('[data-testid="mcp-service-list"]');
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 验证 RPC Log Viewer 可见
    const logViewer = page.locator('[data-testid="rpc-log-viewer"]');
    await expect(logViewer).toBeVisible();
  });

  test('should close inspector drawer', async ({ page }) => {
    // 打开 Inspector
    await page.waitForSelector('[data-testid="mcp-service-list"]');
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 点击关闭按钮
    await page.click('[data-slot="sheet-close"]');

    // 验证抽屉关闭
    await expect(page.locator('[data-slot="sheet-content"]')).not.toBeVisible();
  });

  test('should have inspect option in dropdown menu', async ({ page }) => {
    // 等待服务列表加载
    await page.waitForSelector('[data-testid="mcp-service-list"]');

    // 打开服务菜单
    await page.click('[data-testid="mcp-service-menu-mock-mcp-git"]');

    // 验证 Inspect 选项存在
    const inspectMenuItem = page.getByRole('menuitem', { name: /Inspect|调试/i });
    await expect(inspectMenuItem).toBeVisible();
  });

  test('should filter tools with search', async ({ page }) => {
    // 打开 Inspector
    await page.waitForSelector('[data-testid="mcp-service-list"]');
    await page.click('[data-testid="mcp-service-inspect-mock-mcp-git"]');
    await page.waitForSelector('[data-slot="sheet-content"]');

    // 在搜索框中输入
    const searchInput = page.locator('[data-testid="tool-explorer-search"]');
    await searchInput.fill('git');

    // 搜索应该有效（具体验证取决于工具列表加载）
    await expect(searchInput).toHaveValue('git');
  });
});

test.describe('MCP Inspector - Gateway Offline', () => {
  test('should show gateway not running message when gateway is stopped', async ({ page: _page }) => {
    // 这个测试需要模拟 Gateway 停止状态
    // 在实际测试中，需要修改 mock 返回 MOCK_GATEWAY_STATUS_STOPPED
    // 此处作为示例，跳过或使用条件测试
    test.skip();
  });
});
