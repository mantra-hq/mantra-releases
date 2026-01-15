/**
 * 安全检查与分享功能 E2E 测试
 * Story 3.4: 脱敏预览功能
 *
 * 测试内容:
 * - 进入安全检查预览模式
 * - 分享按钮下拉菜单交互
 * - 复制到剪贴板功能
 * - 导出为文件功能
 * - 退出预览模式
 */

import { test, expect } from "@playwright/test";
import { PlayerPage } from "../pages";

test.describe("安全检查与分享功能", () => {
  /**
   * 测试进入安全检查预览模式
   */
  test.describe("进入预览模式", () => {
    test("点击安全检查按钮应进入预览模式", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      // 导航到会话页面
      await playerPage.gotoSession("mock-session-alpha-1");

      // 等待消息列表加载完成
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 点击安全检查按钮
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await expect(sanitizeButton).toBeVisible();
      await sanitizeButton.click();

      // 验证进入预览模式 - 状态横幅应该可见
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });
    });

    test("预览模式下应显示分享按钮", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅出现
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // 验证分享按钮可见
      const shareButton = page.getByTestId("share-button");
      await expect(shareButton).toBeVisible();
      await expect(shareButton).toBeEnabled();
    });
  });

  /**
   * 测试分享按钮下拉菜单
   */
  test.describe("分享按钮下拉菜单", () => {
    test("点击分享按钮应显示下拉菜单选项", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // 点击分享按钮打开下拉菜单
      const shareButton = page.getByTestId("share-button");
      await shareButton.click();

      // 验证下拉菜单选项可见
      const copyOption = page.getByTestId("copy-to-clipboard");
      const exportOption = page.getByTestId("export-to-file");

      await expect(copyOption).toBeVisible();
      await expect(exportOption).toBeVisible();
    });

    test("复制到剪贴板选项应可点击", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // 点击分享按钮
      const shareButton = page.getByTestId("share-button");
      await shareButton.click();

      // 点击复制到剪贴板选项
      const copyOption = page.getByTestId("copy-to-clipboard");
      await expect(copyOption).toBeVisible();
      await copyOption.click();

      // 点击后应该退出预览模式（状态横幅消失）
      await expect(statusBanner).not.toBeVisible({ timeout: 5000 });
    });

    test("导出为文件选项应可点击", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // 点击分享按钮
      const shareButton = page.getByTestId("share-button");
      await shareButton.click();

      // 验证导出选项可见且可点击
      const exportOption = page.getByTestId("export-to-file");
      await expect(exportOption).toBeVisible();
      // 注意：在 E2E Mock 环境中，Tauri 的 save 对话框不会实际弹出
      // 所以这里只验证选项可点击，不验证文件保存结果
      await expect(exportOption).toBeEnabled();
    });
  });

  /**
   * 测试取消预览模式
   */
  test.describe("取消预览模式", () => {
    test("点击取消按钮应退出预览模式", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // 点击取消按钮
      const cancelButton = page.getByTestId("cancel-button");
      await expect(cancelButton).toBeVisible();
      await cancelButton.click();

      // 状态横幅应该消失
      await expect(statusBanner).not.toBeVisible({ timeout: 5000 });
    });
  });

  /**
   * 测试安全状态显示
   */
  test.describe("安全状态显示", () => {
    test("无敏感信息时应显示安全状态", async ({ page }) => {
      const playerPage = new PlayerPage(page);

      await playerPage.gotoSession("mock-session-alpha-1");
      await expect(playerPage.messageList).toBeVisible({ timeout: 10000 });

      // 进入预览模式
      const sanitizeButton = page.getByTestId("topbar-sanitize-button");
      await sanitizeButton.click();

      // 等待状态横幅
      const statusBanner = page.getByTestId("sanitize-status-banner");
      await expect(statusBanner).toBeVisible({ timeout: 5000 });

      // Mock 数据没有敏感信息，应该显示绿色安全状态
      // 验证使用了绿色背景
      await expect(statusBanner).toHaveClass(/bg-green-500/);
    });
  });
});
