/**
 * BasePage - E2E 测试基础页面类
 * Story 9.3: Task 1
 *
 * 提供所有页面类的通用功能:
 * - 导航方法: goto()
 * - 等待方法: waitForAppReady()
 * - 截图方法: screenshot()
 * - 选择器辅助: getByTestId()
 */

import { Page, Locator, expect } from "@playwright/test";

export class BasePage {
  constructor(protected readonly page: Page) {}

  /**
   * 导航到指定路径
   * @param path - 相对路径，默认为 "/"
   */
  async goto(path: string = "/"): Promise<void> {
    await this.page.goto(path);
  }

  /**
   * 等待应用加载完成
   * 检测 React root 元素可见
   */
  async waitForAppReady(): Promise<void> {
    const root = this.page.locator("#root");
    await expect(root).toBeVisible();
    // 等待应用内容渲染
    await this.page.waitForLoadState("domcontentloaded");
  }

  /**
   * 截取页面截图
   * @param name - 截图文件名（不含扩展名）
   */
  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({
      path: `e2e/screenshots/${name}.png`,
      fullPage: true,
    });
  }

  /**
   * 通过 data-testid 获取 Locator
   * @param testId - data-testid 属性值
   * @returns Playwright Locator
   */
  getByTestId(testId: string): Locator {
    return this.page.locator(`[data-testid="${testId}"]`);
  }

  /**
   * 获取页面标题
   */
  async getTitle(): Promise<string> {
    return await this.page.title();
  }

  /**
   * 等待指定的 data-testid 元素可见
   * @param testId - data-testid 属性值
   * @param options - 等待选项
   */
  async waitForTestId(
    testId: string,
    options?: { timeout?: number }
  ): Promise<void> {
    await expect(this.getByTestId(testId)).toBeVisible(options);
  }

  /**
   * 检查指定的 data-testid 元素是否可见
   * @param testId - data-testid 属性值
   */
  async isTestIdVisible(testId: string): Promise<boolean> {
    return await this.getByTestId(testId).isVisible();
  }
}
