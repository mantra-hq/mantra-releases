/**
 * 更新组件 E2E 测试
 * Story 14.9: Task 8
 * Story 14.10: 移除 UpdateNotificationBar 测试，保留设置页测试
 *
 * 测试内容:
 * - 设置页「关于与更新」区域的版本号显示和按钮存在性
 */

import { test, expect } from "@playwright/test";
import { BasePage } from "../pages";

// =============================================================================
// 设置页「关于与更新」区域
// =============================================================================

test.describe("设置页关于与更新区域", () => {
  test("应显示版本号和检查更新按钮", async ({ page }) => {
    // 配置无更新，以便观察 idle 状态
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: false,
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/settings/general");
    await basePage.waitForAppReady();

    // 「关于 Mantra」区域应可见
    const aboutSection = basePage.getByTestId("about-mantra-section");
    await expect(aboutSection).toBeVisible({ timeout: 10000 });

    // 版本号应显示（从 plugin:app|version mock 返回 "0.7.0"）
    const appVersion = basePage.getByTestId("app-version");
    await expect(appVersion).toBeVisible();
    await expect(appVersion).toContainText("0.7.0");

    // 检查更新按钮应存在且可点击
    const checkUpdateBtn = basePage.getByTestId("check-update-button");
    await expect(checkUpdateBtn).toBeVisible();
    await expect(checkUpdateBtn).toBeEnabled();
  });

  test("手动检查无更新时应显示已是最新", async ({ page }) => {
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: false,
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/settings/general");
    await basePage.waitForAppReady();

    // 等待自动检查完成（按钮变为可点击状态）
    const checkUpdateBtn = basePage.getByTestId("check-update-button");
    await expect(checkUpdateBtn).toBeEnabled({ timeout: 15000 });
    await checkUpdateBtn.click();

    // 应显示「已是最新版本」状态
    const upToDateStatus = basePage.getByTestId("up-to-date-status");
    await expect(upToDateStatus).toBeVisible({ timeout: 10000 });
  });

  test("有更新时设置页应显示 ready 状态和重启按钮", async ({ page }) => {
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: true,
        version: "0.8.0",
        body: "New features",
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/settings/general");
    await basePage.waitForAppReady();

    // 等待自动检查 + 下载完成
    const readyStatus = basePage.getByTestId("ready-status");
    await expect(readyStatus).toBeVisible({ timeout: 15000 });

    // 应包含版本号
    await expect(readyStatus).toContainText("0.8.0");

    // 重启更新按钮应可见
    const restartBtn = basePage.getByTestId("restart-to-update-button");
    await expect(restartBtn).toBeVisible();
    await expect(restartBtn).toBeEnabled();
  });
});
