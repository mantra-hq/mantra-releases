import { test, expect } from '@playwright/test';

/**
 * Smoke test to verify Playwright infrastructure works correctly.
 * This test validates that the dev server starts and serves the application.
 *
 * Story 9.4: 更新 - 添加 ?playwright 参数启用 Mock 环境
 */
test.describe('Smoke Tests', () => {
  test('app loads successfully', async ({ page }) => {
    // 添加 ?playwright 参数以启用 IPC Mock 环境
    await page.goto('/?playwright');

    // Verify the React root element exists and is visible
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });

  test('page has valid title', async ({ page }) => {
    // 添加 ?playwright 参数以启用 IPC Mock 环境
    await page.goto('/?playwright');

    // Verify the page title is not empty
    await expect(page).toHaveTitle(/.+/);
  });
});
