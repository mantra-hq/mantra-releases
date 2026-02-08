/**
 * 更新组件 E2E 测试
 * Story 14.9: Task 8
 *
 * 测试内容:
 * - 8.3: UpdateNotificationBar 在 ready 状态下正确渲染
 * - 8.4: 通知条的版本号显示、更新日志展开/折叠、关闭按钮交互
 * - 8.5: 设置页「关于与更新」区域的版本号显示和按钮存在性
 */

import { test, expect } from "@playwright/test";
import { BasePage } from "../pages";

// =============================================================================
// 8.3: UpdateNotificationBar 在 ready 状态下正确渲染
// =============================================================================

test.describe("UpdateNotificationBar 渲染", () => {
  test("有更新时通知条应在 ready 状态后显示", async ({ page }) => {
    // 配置 mock 返回有更新
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: true,
        version: "0.8.0",
        body: "Bug fixes and improvements",
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    // 等待自动检查 + 下载完成后通知条出现（5s 延迟 + 处理时间）
    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });
  });

  test("无更新时通知条不应显示", async ({ page }) => {
    // 配置 mock 返回无更新
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: false,
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    // 等待足够时间确认通知条不会出现
    await page.waitForTimeout(7000);

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).not.toBeVisible();
  });

  test("通知条应包含正确的 role 和 aria 属性", async ({ page }) => {
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: true,
        version: "0.8.0",
        body: "Release notes content",
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // 验证无障碍属性
    await expect(notificationBar).toHaveAttribute("role", "status");
    await expect(notificationBar).toHaveAttribute("data-state", "open");
  });
});

// =============================================================================
// 8.4: 通知条版本号显示、更新日志展开/折叠、关闭按钮交互
// =============================================================================

test.describe("UpdateNotificationBar 交互", () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: true,
        version: "0.8.0",
        body: "Bug fixes and improvements\n- Fixed crash on startup\n- Improved performance",
      };
    });
  });

  test("通知条应显示正确的版本号", async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // 验证版本号 0.8.0 出现在通知条文本中
    await expect(notificationBar).toContainText("0.8.0");
  });

  test("点击更新日志按钮应展开/折叠内容", async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // 关闭可能遮挡的项目抽屉 Sheet
    await page.keyboard.press("Escape");
    await page.waitForTimeout(300);

    // 更新日志内容初始应不可见
    const releaseNotes = basePage.getByTestId("update-release-notes-content");
    await expect(releaseNotes).not.toBeVisible();

    // 点击更新日志按钮展开
    const releaseNotesBtn = basePage.getByTestId("update-release-notes-btn");
    await releaseNotesBtn.click();

    // 更新日志内容应可见并包含正确文本
    await expect(releaseNotes).toBeVisible();
    await expect(releaseNotes).toContainText("Bug fixes and improvements");
    await expect(releaseNotes).toContainText("Fixed crash on startup");

    // 再次点击折叠
    await releaseNotesBtn.click();
    await expect(releaseNotes).not.toBeVisible();
  });

  test("关闭按钮应隐藏通知条", async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // 关闭可能遮挡的项目抽屉 Sheet
    await page.keyboard.press("Escape");
    await page.waitForTimeout(300);

    // 点击关闭按钮
    const dismissBtn = basePage.getByTestId("update-dismiss-btn");
    await dismissBtn.click();

    // 通知条应消失（动画后）
    await expect(notificationBar).not.toBeVisible({ timeout: 5000 });
  });

  test("重启更新按钮应存在且可点击", async ({ page }) => {
    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // 验证重启更新按钮存在
    const restartBtn = basePage.getByTestId("update-restart-btn");
    await expect(restartBtn).toBeVisible();
    await expect(restartBtn).toBeEnabled();
  });
});

// 独立于 "交互" describe 的 beforeEach，避免 addInitScript 累积
test.describe("UpdateNotificationBar 无 body 场景", () => {
  test("无 body 时不应显示更新日志按钮", async ({ page }) => {
    // 配置 mock 返回无 body 的更新
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ = {
        hasUpdate: true,
        version: "0.8.0",
      };
    });

    const basePage = new BasePage(page);
    await basePage.goto("/");
    await basePage.waitForAppReady();

    const notificationBar = basePage.getByTestId("update-notification-bar");
    await expect(notificationBar).toBeVisible({ timeout: 15000 });

    // body 为空时不应显示更新日志按钮
    const releaseNotesBtn = basePage.getByTestId("update-release-notes-btn");
    await expect(releaseNotesBtn).not.toBeVisible();
  });
});

// =============================================================================
// 8.5: 设置页「关于与更新」区域
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

    // 等待自动检查完成
    await page.waitForTimeout(7000);

    // 点击手动检查
    const checkUpdateBtn = basePage.getByTestId("check-update-button");
    await expect(checkUpdateBtn).toBeVisible({ timeout: 10000 });
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
