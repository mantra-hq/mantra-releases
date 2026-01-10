/**
 * ImportPage - 导入向导页面类
 * Story 9.3: Task 3
 *
 * 封装导入向导的交互操作:
 * - 来源选择
 * - 文件选择
 * - 导入进度查看
 * - 完成页面操作
 */

import { Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export type ImportSource = "claude" | "gemini" | "cursor" | "codex";

export class ImportPage extends BasePage {
  // ===========================================================================
  // Selectors
  // ===========================================================================

  /** 导入向导容器 */
  get importWizard(): Locator {
    return this.getByTestId("import-wizard");
  }

  /** 导入按钮（触发导入流程） */
  get importButton(): Locator {
    return this.getByTestId("topbar-import-button");
  }

  /** 来源选择器容器 */
  get sourceSelector(): Locator {
    return this.getByTestId("source-selector");
  }

  /** 文件列表 */
  get fileList(): Locator {
    return this.getByTestId("file-list");
  }

  /** 所有文件项 */
  get fileItems(): Locator {
    return this.getByTestId("file-item");
  }

  /** 导入进度指示器 */
  get importProgress(): Locator {
    return this.getByTestId("import-progress");
  }

  /** 导入结果区域 */
  get importResults(): Locator {
    return this.getByTestId("import-results");
  }

  // ===========================================================================
  // 来源选择 (AC #3: 来源选择)
  // ===========================================================================

  /**
   * 选择导入来源
   * @param source - 来源类型: 'claude' | 'gemini' | 'cursor' | 'codex'
   */
  async selectSource(source: ImportSource): Promise<void> {
    const sourceOption = this.getByTestId(`source-option-${source}`);
    await sourceOption.click();
    // 等待来源选中状态更新
    await expect(sourceOption).toHaveAttribute("data-selected", "true");
  }

  /**
   * 获取当前选中的来源
   */
  async getSelectedSource(): Promise<string | null> {
    const selectedOption = this.sourceSelector.locator('[data-selected="true"]');
    const count = await selectedOption.count();
    if (count === 0) return null;
    return await selectedOption.getAttribute("data-source");
  }

  // ===========================================================================
  // 文件选择 (AC #3: 文件选择)
  // ===========================================================================

  /**
   * 获取发现的文件列表
   */
  async getDiscoveredFiles(): Promise<string[]> {
    const files: string[] = [];
    const count = await this.fileItems.count();

    for (let i = 0; i < count; i++) {
      const filePath = await this.fileItems.nth(i).getAttribute("data-file-path");
      if (filePath) files.push(filePath);
    }

    return files;
  }

  /**
   * 选择指定索引的文件
   * @param index - 文件索引（从 0 开始）
   */
  async selectFile(index: number): Promise<void> {
    const fileItem = this.fileItems.nth(index);
    const checkbox = fileItem.locator('input[type="checkbox"]');
    await checkbox.check();
  }

  /**
   * 取消选择指定索引的文件
   * @param index - 文件索引（从 0 开始）
   */
  async deselectFile(index: number): Promise<void> {
    const fileItem = this.fileItems.nth(index);
    const checkbox = fileItem.locator('input[type="checkbox"]');
    await checkbox.uncheck();
  }

  /**
   * 选择所有文件
   */
  async selectAllFiles(): Promise<void> {
    const selectAllCheckbox = this.getByTestId("select-all-files");
    await selectAllCheckbox.check();
  }

  /**
   * 取消选择所有文件
   */
  async deselectAllFiles(): Promise<void> {
    const selectAllCheckbox = this.getByTestId("select-all-files");
    await selectAllCheckbox.uncheck();
  }

  /**
   * 获取已选择的文件数量
   */
  async getSelectedFileCount(): Promise<number> {
    const checkedItems = this.fileItems.locator('input[type="checkbox"]:checked');
    return await checkedItems.count();
  }

  // ===========================================================================
  // 导入控制 (AC #3: 导入进度)
  // ===========================================================================

  /**
   * 开始导入
   */
  async startImport(): Promise<void> {
    const startButton = this.getByTestId("start-import-button");
    await startButton.click();
    // 等待进度指示器出现
    await expect(this.importProgress).toBeVisible({ timeout: 5000 });
  }

  /**
   * 获取导入进度（百分比）
   */
  async getProgress(): Promise<number> {
    const progressText = await this.importProgress.getAttribute("data-progress");
    return progressText ? parseInt(progressText, 10) : 0;
  }

  /**
   * 获取导入状态信息
   */
  async getProgressStatus(): Promise<string> {
    const statusText = this.getByTestId("import-status-text");
    return await statusText.textContent() ?? "";
  }

  /**
   * 取消导入
   */
  async cancelImport(): Promise<void> {
    const cancelButton = this.getByTestId("cancel-import-button");
    await cancelButton.click();
  }

  /**
   * 等待导入完成
   */
  async waitForImportComplete(timeout: number = 30000): Promise<void> {
    await expect(this.importResults).toBeVisible({ timeout });
  }

  // ===========================================================================
  // 完成页操作 (AC #3: 完成页面操作)
  // ===========================================================================

  /**
   * 获取导入结果摘要
   */
  async getImportResults(): Promise<{
    total: number;
    success: number;
    failed: number;
  }> {
    const resultsData = await this.importResults.getAttribute("data-results");
    if (!resultsData) {
      return { total: 0, success: 0, failed: 0 };
    }

    try {
      return JSON.parse(resultsData);
    } catch {
      return { total: 0, success: 0, failed: 0 };
    }
  }

  /**
   * 获取导入成功的项目列表
   */
  async getImportedProjects(): Promise<string[]> {
    const projects: string[] = [];
    const projectItems = this.getByTestId("imported-project-item");
    const count = await projectItems.count();

    for (let i = 0; i < count; i++) {
      const name = await projectItems.nth(i).getAttribute("data-project-name");
      if (name) projects.push(name);
    }

    return projects;
  }

  /**
   * 导航到指定索引的导入项目
   * @param index - 项目索引（从 0 开始）
   */
  async navigateToProject(index: number): Promise<void> {
    const projectItems = this.getByTestId("imported-project-item");
    await projectItems.nth(index).click();
  }

  /**
   * 关闭导入向导
   */
  async closeWizard(): Promise<void> {
    const closeButton = this.getByTestId("close-import-wizard");
    await closeButton.click();
    await expect(this.importWizard).not.toBeVisible();
  }

  // ===========================================================================
  // 页面生命周期
  // ===========================================================================

  /**
   * 打开导入向导
   */
  async openImportWizard(): Promise<void> {
    await this.importButton.click();
    await expect(this.importWizard).toBeVisible();
  }

  /**
   * 等待导入向导就绪
   */
  async waitForWizardReady(): Promise<void> {
    await expect(this.importWizard).toBeVisible();
    await expect(this.sourceSelector).toBeVisible();
  }
}
