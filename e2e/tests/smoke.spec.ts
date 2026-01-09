import { test, expect } from '@playwright/test';

/**
 * Smoke test to verify Playwright infrastructure works correctly.
 * This test validates that the dev server starts and serves the application.
 */
test.describe('Smoke Tests', () => {
  test('app loads successfully', async ({ page }) => {
    await page.goto('/');

    // Verify the React root element exists and is visible
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });

  test('page has valid title', async ({ page }) => {
    await page.goto('/');

    // Verify the page title is not empty
    await expect(page).toHaveTitle(/.+/);
  });
});
