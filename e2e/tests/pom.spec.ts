/**
 * POM (Page Object Model) 验证测试
 * Story 9.3: Task 6
 *
 * 验证 Page Object 类的正确性和可用性：
 * - BasePage 基础方法
 * - PlayerPage 播放器页面选择器
 * - ImportPage 导入向导选择器
 * - SearchPage 搜索页面选择器
 *
 * 注意：这些测试依赖 IPC Mock Layer (Story 9.2)
 * 部分测试需要完整的 Mock 环境才能通过
 */

import { test, expect } from "@playwright/test";
import { BasePage, PlayerPage, ImportPage, SearchPage } from "../pages";

/**
 * BasePage 测试 - 基础页面功能
 */
test.describe("BasePage - 基础页面类", () => {
  test("goto() 应正确导航到指定路径", async ({ page }) => {
    const basePage = new BasePage(page);

    await basePage.goto("/");
    await expect(page).toHaveURL(/\//);
  });

  test("waitForAppReady() 应等待 React root 元素可见", async ({ page }) => {
    const basePage = new BasePage(page);

    await basePage.goto("/");
    await basePage.waitForAppReady();

    const root = page.locator("#root");
    await expect(root).toBeVisible();
  });

  test("getByTestId() 应返回正确的 Locator", async ({ page }) => {
    const basePage = new BasePage(page);

    await basePage.goto("/");
    await basePage.waitForAppReady();

    // TopBar 组件应该有 data-testid="top-bar"
    const topBar = basePage.getByTestId("top-bar");
    await expect(topBar).toBeVisible();
  });

  test("getTitle() 应返回页面标题", async ({ page }) => {
    const basePage = new BasePage(page);

    await basePage.goto("/");
    await basePage.waitForAppReady();

    const title = await basePage.getTitle();
    expect(typeof title).toBe("string");
  });
});

/**
 * PlayerPage 测试 - 播放器页面选择器
 * 注意：需要有效的 Mock 会话数据才能完整测试
 */
test.describe("PlayerPage - 播放器页面类", () => {
  test("选择器定义应正确", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    // 验证选择器 getter 存在且返回 Locator
    expect(playerPage.messageList).toBeDefined();
    expect(playerPage.codePanel).toBeDefined();
    expect(playerPage.editorTabs).toBeDefined();
    expect(playerPage.timeline).toBeDefined();
  });

  test("navigateToSession() 应构造正确的 URL", async ({ page }) => {
    const playerPage = new PlayerPage(page);

    await playerPage.navigateToSession("test-session-123");
    await expect(page).toHaveURL(/\/session\/test-session-123/);
  });
});

/**
 * ImportPage 测试 - 导入向导页面选择器
 */
test.describe("ImportPage - 导入向导页面类", () => {
  test("选择器定义应正确", async ({ page }) => {
    const importPage = new ImportPage(page);

    // 验证选择器 getter 存在且返回 Locator
    expect(importPage.importWizard).toBeDefined();
    expect(importPage.importButton).toBeDefined();
    expect(importPage.sourceSelector).toBeDefined();
    expect(importPage.fileList).toBeDefined();
    expect(importPage.importProgress).toBeDefined();
    expect(importPage.importResults).toBeDefined();
  });

  test("openImportWizard() 应打开导入向导", async ({ page }) => {
    const importPage = new ImportPage(page);

    await importPage.goto("/");
    await importPage.waitForAppReady();

    // 关闭可能打开的 drawer（按 ESC 并等待其消失）
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    // 点击导入按钮打开向导
    await importPage.openImportWizard();

    // 导入向导应该可见
    await expect(importPage.importWizard).toBeVisible();
  });

  test("closeWizard() 应关闭导入向导", async ({ page }) => {
    const importPage = new ImportPage(page);

    await importPage.goto("/");
    await importPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await importPage.openImportWizard();

    // ESC 关闭向导
    await page.keyboard.press("Escape");
    await expect(importPage.importWizard).not.toBeVisible();
  });
});

/**
 * SearchPage 测试 - 搜索页面选择器
 */
test.describe("SearchPage - 搜索页面类", () => {
  test("选择器定义应正确", async ({ page }) => {
    const searchPage = new SearchPage(page);

    // 验证选择器 getter 存在且返回 Locator
    expect(searchPage.globalSearch).toBeDefined();
    expect(searchPage.searchTrigger).toBeDefined();
    expect(searchPage.searchInput).toBeDefined();
    expect(searchPage.searchResults).toBeDefined();
    expect(searchPage.resultItems).toBeDefined();
  });

  test("openSearch() 应打开搜索面板", async ({ page }) => {
    const searchPage = new SearchPage(page);

    await searchPage.goto("/");
    await searchPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    // 打开搜索
    await searchPage.openSearch();

    // 搜索面板应该可见
    await expect(searchPage.globalSearch).toBeVisible();
  });

  test("closeSearch() 应关闭搜索面板", async ({ page }) => {
    const searchPage = new SearchPage(page);

    await searchPage.goto("/");
    await searchPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await searchPage.openSearch();
    await expect(searchPage.globalSearch).toBeVisible();

    await searchPage.closeSearch();
    await expect(searchPage.globalSearch).not.toBeVisible();
  });

  test("openSearchWithKeyboard() 应通过快捷键打开搜索", async ({ page }) => {
    const searchPage = new SearchPage(page);

    await searchPage.goto("/");
    await searchPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    // 使用快捷键打开搜索
    await searchPage.openSearchWithKeyboard();

    // 搜索面板应该可见
    await expect(searchPage.globalSearch).toBeVisible();
  });
});

/**
 * data-testid 属性验证
 */
test.describe("data-testid 属性验证", () => {
  test("TopBar 应有 data-testid='top-bar'", async ({ page }) => {
    const basePage = new BasePage(page);

    await basePage.goto("/");
    await basePage.waitForAppReady();

    const topBar = page.locator('[data-testid="top-bar"]');
    await expect(topBar).toBeVisible();
  });

  test("导入向导应有 data-testid='import-wizard'", async ({ page }) => {
    const importPage = new ImportPage(page);

    await importPage.goto("/");
    await importPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await importPage.openImportWizard();

    const importWizard = page.locator('[data-testid="import-wizard"]');
    await expect(importWizard).toBeVisible();
  });

  test("全局搜索应有 data-testid='global-search'", async ({ page }) => {
    const searchPage = new SearchPage(page);

    await searchPage.goto("/");
    await searchPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await searchPage.openSearch();

    const globalSearch = page.locator('[data-testid="global-search"]');
    await expect(globalSearch).toBeVisible();
  });

  test("搜索输入框应有 data-testid='search-input'", async ({ page }) => {
    const searchPage = new SearchPage(page);

    await searchPage.goto("/");
    await searchPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await searchPage.openSearch();

    const searchInput = page.locator('[data-testid="search-input"]');
    await expect(searchInput).toBeVisible();
  });

  test("来源选择器应有 data-testid='source-selector'", async ({ page }) => {
    const importPage = new ImportPage(page);

    await importPage.goto("/");
    await importPage.waitForAppReady();

    // 关闭可能打开的 drawer
    await page.keyboard.press("Escape");
    await expect(page.locator('[data-testid="project-drawer"]')).toBeHidden({ timeout: 1000 }).catch(() => {});

    await importPage.openImportWizard();

    const sourceSelector = page.locator('[data-testid="source-selector"]');
    await expect(sourceSelector).toBeVisible();
  });
});
