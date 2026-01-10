/**
 * PlayerPage - 播放器页面类
 * Story 9.3: Task 2
 *
 * 封装会话播放器页面的交互操作:
 * - 消息列表操作
 * - 代码面板操作
 * - 时间轴操作
 * - 导航操作
 */

import { Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class PlayerPage extends BasePage {
  // ===========================================================================
  // Selectors (使用 getter 实现惰性加载)
  // ===========================================================================

  /** 消息列表容器 */
  get messageList(): Locator {
    return this.getByTestId("message-list");
  }

  /** 所有消息项 */
  get messageItems(): Locator {
    return this.getByTestId("message-item");
  }

  /** 代码面板 */
  get codePanel(): Locator {
    return this.getByTestId("code-panel");
  }

  /** 编辑器标签页容器 */
  get editorTabs(): Locator {
    return this.getByTestId("editor-tabs");
  }

  /** 所有标签项 */
  get tabItems(): Locator {
    return this.getByTestId("tab-item");
  }

  /** 代码内容区域 */
  get codeContent(): Locator {
    return this.getByTestId("code-content");
  }

  /** 时间轴 */
  get timeline(): Locator {
    return this.getByTestId("timeline");
  }

  /** 时间轴滑块 */
  get timelineSlider(): Locator {
    return this.getByTestId("timeline-slider");
  }

  /** 抽屉切换按钮 */
  get drawerToggle(): Locator {
    return this.getByTestId("drawer-toggle");
  }

  /** 项目列表 */
  get projectList(): Locator {
    return this.getByTestId("project-list");
  }

  /** 面包屑导航 */
  get breadcrumb(): Locator {
    return this.getByTestId("breadcrumb");
  }

  /** 会话下拉选择器 */
  get sessionDropdown(): Locator {
    return this.getByTestId("session-dropdown");
  }

  // ===========================================================================
  // 消息列表操作 (AC #2: 消息列表选择器)
  // ===========================================================================

  /**
   * 获取消息列表 Locator
   */
  getMessageList(): Locator {
    return this.messageList;
  }

  /**
   * 获取消息数量
   */
  async getMessageCount(): Promise<number> {
    return await this.messageItems.count();
  }

  /**
   * 点击指定索引的消息
   * @param index - 消息索引（从 0 开始）
   */
  async clickMessage(index: number): Promise<void> {
    const message = this.messageItems.nth(index);
    await message.click();
  }

  /**
   * 获取当前选中的消息
   */
  getSelectedMessage(): Locator {
    return this.getByTestId("message-selected");
  }

  /**
   * 断言指定索引的消息被选中
   * @param index - 消息索引
   */
  async expectMessageSelected(index: number): Promise<void> {
    const message = this.messageItems.nth(index);
    await expect(message).toHaveAttribute("data-selected", "true");
  }

  // ===========================================================================
  // 代码面板操作 (AC #2: 代码面板操作)
  // ===========================================================================

  /**
   * 获取代码面板 Locator
   */
  getCodePanel(): Locator {
    return this.codePanel;
  }

  /**
   * 获取当前激活的标签页
   */
  getActiveTab(): Locator {
    return this.getByTestId("tab-item").filter({
      has: this.page.locator('[data-active="true"]'),
    });
  }

  /**
   * 切换到指定名称的标签页
   * @param name - 标签页名称
   */
  async switchTab(name: string): Promise<void> {
    const tab = this.tabItems.filter({ hasText: name });
    await tab.click();
  }

  /**
   * 获取代码内容文本
   */
  async getCodeContent(): Promise<string> {
    return await this.codeContent.textContent() ?? "";
  }

  // ===========================================================================
  // 时间轴操作 (AC #2: 时间轴交互)
  // ===========================================================================

  /**
   * 获取时间轴 Locator
   */
  getTimeline(): Locator {
    return this.timeline;
  }

  /**
   * 拖拽时间轴滑块到指定位置
   * @param position - 位置百分比 (0-100)
   * @throws Error 如果无法获取滑块边界框
   */
  async dragTimelineSlider(position: number): Promise<void> {
    const slider = this.timelineSlider;
    const box = await slider.boundingBox();

    if (!box) {
      throw new Error("Timeline slider bounding box not available - element may not be visible");
    }

    const targetX = box.x + (box.width * position) / 100;
    const targetY = box.y + box.height / 2;

    await this.page.mouse.click(targetX, targetY);
  }

  // ===========================================================================
  // 导航操作 (AC #2: 导航)
  // ===========================================================================

  /**
   * 打开项目抽屉
   */
  async openProjectDrawer(): Promise<void> {
    await this.drawerToggle.click();
    await expect(this.projectList).toBeVisible();
  }

  /**
   * 导航到指定会话
   * @param sessionId - 会话 ID
   */
  async navigateToSession(sessionId: string): Promise<void> {
    await this.goto(`/session/${sessionId}`);
    await this.waitForAppReady();
  }

  /**
   * 导航到指定项目
   * @param projectId - 项目 ID
   */
  async navigateToProject(projectId: string): Promise<void> {
    await this.goto(`/project/${projectId}`);
    await this.waitForAppReady();
  }

  /**
   * 从会话下拉选择器切换会话
   * @param sessionId - 目标会话 ID
   */
  async switchSession(sessionId: string): Promise<void> {
    await this.sessionDropdown.click();
    const sessionItem = this.getByTestId("session-item").filter({
      has: this.page.locator(`[data-session-id="${sessionId}"]`),
    });
    await sessionItem.click();
  }

  // ===========================================================================
  // 页面生命周期
  // ===========================================================================

  /**
   * 导航到会话并等待加载完成
   * @param sessionId - 会话 ID
   */
  async gotoSession(sessionId: string): Promise<void> {
    await this.navigateToSession(sessionId);
    // 等待消息列表加载
    await expect(this.messageList).toBeVisible({ timeout: 10000 });
  }

  /**
   * 等待播放器页面完全加载
   */
  async waitForPlayerReady(): Promise<void> {
    await this.waitForAppReady();
    // 播放器核心组件应该可见
    await expect(this.messageList).toBeVisible({ timeout: 10000 });
  }
}
