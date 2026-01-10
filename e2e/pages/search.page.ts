/**
 * SearchPage - 搜索页面类
 * Story 9.3: Task 4
 *
 * 封装全局搜索的交互操作:
 * - 打开/关闭搜索
 * - 执行搜索
 * - 搜索结果操作
 * - 搜索过滤
 */

import { Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class SearchPage extends BasePage {
  // ===========================================================================
  // Selectors
  // ===========================================================================

  /** 全局搜索容器 */
  get globalSearch(): Locator {
    return this.getByTestId("global-search");
  }

  /** 搜索触发按钮 */
  get searchTrigger(): Locator {
    return this.getByTestId("topbar-search-button");
  }

  /** 搜索输入框 */
  get searchInput(): Locator {
    return this.getByTestId("search-input");
  }

  /** 搜索结果容器 */
  get searchResults(): Locator {
    return this.getByTestId("search-results");
  }

  /** 所有搜索结果项 */
  get resultItems(): Locator {
    return this.getByTestId("search-result-item");
  }

  /** 搜索过滤器容器 */
  get searchFilters(): Locator {
    return this.getByTestId("search-filters");
  }

  /** 无结果提示 */
  get noResultsMessage(): Locator {
    return this.getByTestId("no-results");
  }

  /** 搜索加载指示器 */
  get searchLoading(): Locator {
    return this.getByTestId("search-loading");
  }

  // ===========================================================================
  // 全局搜索操作 (Task 4.2: 全局搜索)
  // ===========================================================================

  /**
   * 打开搜索面板
   */
  async openSearch(): Promise<void> {
    // 检查搜索面板是否已打开
    const isVisible = await this.globalSearch.isVisible();
    if (isVisible) return;

    await this.searchTrigger.click();
    await expect(this.globalSearch).toBeVisible();
    await expect(this.searchInput).toBeFocused();
  }

  /**
   * 使用键盘快捷键打开搜索 (Cmd/Ctrl + K)
   */
  async openSearchWithKeyboard(): Promise<void> {
    const modifier = process.platform === "darwin" ? "Meta" : "Control";
    await this.page.keyboard.press(`${modifier}+k`);
    await expect(this.globalSearch).toBeVisible();
  }

  /**
   * 关闭搜索面板
   */
  async closeSearch(): Promise<void> {
    await this.page.keyboard.press("Escape");
    await expect(this.globalSearch).not.toBeVisible();
  }

  /**
   * 执行搜索
   * @param query - 搜索关键词
   */
  async search(query: string): Promise<void> {
    await this.openSearch();
    await this.searchInput.fill(query);
    // 等待搜索完成（加载指示器消失或结果出现）
    await this.waitForSearchComplete();
  }

  /**
   * 清空搜索框
   */
  async clearSearch(): Promise<void> {
    await this.searchInput.clear();
    // 结果应该被清空或显示默认状态
  }

  /**
   * 获取搜索结果
   */
  async getResults(): Promise<
    Array<{
      title: string;
      projectName: string;
      sessionName: string;
    }>
  > {
    const results: Array<{
      title: string;
      projectName: string;
      sessionName: string;
    }> = [];

    const count = await this.resultItems.count();
    for (let i = 0; i < count; i++) {
      const item = this.resultItems.nth(i);
      const title = (await item.getAttribute("data-title")) ?? "";
      const projectName = (await item.getAttribute("data-project-name")) ?? "";
      const sessionName = (await item.getAttribute("data-session-name")) ?? "";
      results.push({ title, projectName, sessionName });
    }

    return results;
  }

  /**
   * 获取搜索结果数量
   */
  async getResultCount(): Promise<number> {
    return await this.resultItems.count();
  }

  /**
   * 点击指定索引的搜索结果
   * @param index - 结果索引（从 0 开始）
   */
  async clickResult(index: number): Promise<void> {
    await this.resultItems.nth(index).click();
    // 点击结果后搜索面板应该关闭
    await expect(this.globalSearch).not.toBeVisible();
  }

  /**
   * 使用键盘选择搜索结果
   * @param steps - 向下移动的步数
   */
  async selectResultWithKeyboard(steps: number): Promise<void> {
    for (let i = 0; i < steps; i++) {
      await this.page.keyboard.press("ArrowDown");
    }
    await this.page.keyboard.press("Enter");
  }

  // ===========================================================================
  // 搜索过滤 (Task 4.3: 搜索过滤)
  // ===========================================================================

  /**
   * 按项目过滤搜索结果
   * @param projectId - 项目 ID
   */
  async filterByProject(projectId: string): Promise<void> {
    const projectFilter = this.getByTestId(`filter-project-${projectId}`);
    await projectFilter.click();
    await this.waitForSearchComplete();
  }

  /**
   * 按来源过滤搜索结果
   * @param source - 来源类型
   */
  async filterBySource(source: string): Promise<void> {
    const sourceFilter = this.getByTestId(`filter-source-${source}`);
    await sourceFilter.click();
    await this.waitForSearchComplete();
  }

  /**
   * 清除所有过滤器
   */
  async clearFilters(): Promise<void> {
    const clearButton = this.getByTestId("clear-filters");
    await clearButton.click();
    await this.waitForSearchComplete();
  }

  /**
   * 获取当前激活的过滤器
   */
  async getActiveFilters(): Promise<string[]> {
    const activeFilters = this.searchFilters.locator('[data-active="true"]');
    const count = await activeFilters.count();
    const filters: string[] = [];

    for (let i = 0; i < count; i++) {
      const filterType = await activeFilters.nth(i).getAttribute("data-filter");
      if (filterType) filters.push(filterType);
    }

    return filters;
  }

  // ===========================================================================
  // 辅助方法
  // ===========================================================================

  /**
   * 等待搜索完成
   */
  private async waitForSearchComplete(): Promise<void> {
    // 等待加载指示器消失
    await expect(this.searchLoading).not.toBeVisible({ timeout: 10000 });
  }

  /**
   * 检查是否显示无结果提示
   */
  async hasNoResults(): Promise<boolean> {
    return await this.noResultsMessage.isVisible();
  }

  /**
   * 获取高亮的搜索文本
   * @param resultIndex - 结果索引
   */
  async getHighlightedText(resultIndex: number): Promise<string[]> {
    const highlights = this.resultItems
      .nth(resultIndex)
      .locator(".search-highlight, mark, [data-highlight]");
    const count = await highlights.count();
    const texts: string[] = [];

    for (let i = 0; i < count; i++) {
      const text = await highlights.nth(i).textContent();
      if (text) texts.push(text);
    }

    return texts;
  }
}
