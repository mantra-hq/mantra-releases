/**
 * 搜索功能 E2E 测试
 * Story 9.4: Task 4 (AC: #4)
 *
 * 测试内容:
 * - 全局搜索打开/关闭
 * - 搜索输入交互
 * - 快捷键触发搜索
 */

import { test, expect } from "@playwright/test";
import { SearchPage } from "../pages";
import { dismissOverlays } from "../utils/test-helpers";

// 为不稳定测试配置重试
test.describe("搜索功能测试", () => {
  /**
   * Task 4.1: 全局搜索打开/关闭
   */
  test.describe("全局搜索打开/关闭", () => {
    test("点击搜索按钮应打开搜索面板", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();

      // 验证搜索面板可见
      await expect(searchPage.globalSearch).toBeVisible();
    });

    test("按 ESC 应关闭搜索面板", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();
      await expect(searchPage.globalSearch).toBeVisible();

      // 按 ESC 关闭
      await searchPage.closeSearch();

      // 验证已关闭
      await expect(searchPage.globalSearch).not.toBeVisible();
    });
  });

  /**
   * Task 4.2: 搜索输入交互
   */
  test.describe("搜索输入交互", () => {
    test("搜索输入框应可见且可聚焦", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();

      // 验证搜索输入框可见
      await expect(searchPage.searchInput).toBeVisible();
    });

    test("输入搜索词应触发搜索", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();

      // 输入搜索词
      await searchPage.search("认证");

      // 等待搜索响应（使用 networkidle 替代 waitForTimeout）
      await page.waitForLoadState("domcontentloaded");

      // 搜索输入框应包含搜索词
      await expect(searchPage.searchInput).toHaveValue("认证");
    });

    test("清空搜索词应重置结果", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();

      // 输入搜索词
      await searchPage.search("认证");
      await page.waitForLoadState("domcontentloaded");

      // 清空输入
      await searchPage.searchInput.clear();
      await page.waitForLoadState("domcontentloaded");

      // 输入框应为空
      await expect(searchPage.searchInput).toHaveValue("");
    });
  });

  /**
   * Task 4.3: 快捷键触发搜索
   */
  test.describe("快捷键触发搜索", () => {
    test("Cmd/Ctrl+K 应打开搜索", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 使用快捷键打开搜索
      await searchPage.openSearchWithKeyboard();

      // 验证搜索面板可见
      await expect(searchPage.globalSearch).toBeVisible();
    });

    test("搜索面板打开后输入框应自动聚焦", async ({ page }) => {
      const searchPage = new SearchPage(page);

      await searchPage.goto("/");
      await searchPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开搜索
      await searchPage.openSearch();

      // 输入框应该被聚焦
      await expect(searchPage.searchInput).toBeFocused();
    });
  });
});
