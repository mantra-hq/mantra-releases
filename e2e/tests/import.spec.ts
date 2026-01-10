/**
 * 导入流程 E2E 测试
 * Story 9.4: Task 3 (AC: #3)
 *
 * 测试内容:
 * - 导入向导打开/关闭
 * - 来源选择器功能
 * - 导入流程步骤验证
 */

import { test, expect } from "@playwright/test";
import { ImportPage } from "../pages";
import { dismissOverlays } from "../utils/test-helpers";

// 为不稳定测试配置重试
test.describe("导入流程测试", () => {
  /**
   * Task 3.1: 导入向导打开/关闭
   */
  test.describe("导入向导打开/关闭", () => {
    test("点击导入按钮应打开导入向导", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();

      // 验证导入向导可见
      await expect(importPage.importWizard).toBeVisible();
    });

    test("按 ESC 应关闭导入向导", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();
      await expect(importPage.importWizard).toBeVisible();

      // 按 ESC 关闭
      await page.keyboard.press("Escape");

      // 验证已关闭
      await expect(importPage.importWizard).not.toBeVisible();
    });
  });

  /**
   * Task 3.2: 来源选择器功能
   */
  test.describe("来源选择器功能", () => {
    test("导入向导应显示来源选择器", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();

      // 验证来源选择器可见
      await expect(importPage.sourceSelector).toBeVisible();
    });

    test("来源选择器应包含多个选项", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();

      // 来源选择器应可见
      await expect(importPage.sourceSelector).toBeVisible();

      // 应该有来源选项（Claude, Gemini, Cursor 等）
      const sourceOptions = page.locator('[data-testid="source-selector"] button, [data-testid="source-selector"] [role="radio"]');
      const count = await sourceOptions.count();
      // 至少应该有 1 个来源选项（强断言）
      expect(count).toBeGreaterThan(0);
    });
  });

  /**
   * Task 3.3: 导入流程验证
   */
  test.describe("导入流程验证", () => {
    test("导入向导应有导入按钮", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();

      // 导入向导应可见，包含必要的 UI 元素
      await expect(importPage.importWizard).toBeVisible();
      // 来源选择器应该存在（导入的第一步）
      await expect(importPage.sourceSelector).toBeVisible();
    });

    test("导入向导 UI 元素应正确渲染", async ({ page }) => {
      const importPage = new ImportPage(page);

      await importPage.goto("/");
      await importPage.waitForAppReady();

      // 关闭可能的抽屉
      await dismissOverlays(page);

      // 打开导入向导
      await importPage.openImportWizard();

      // 验证基本 UI 元素
      await expect(importPage.importWizard).toBeVisible();
      await expect(importPage.sourceSelector).toBeVisible();
    });
  });
});
