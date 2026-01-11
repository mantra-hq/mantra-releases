import { test, expect } from "@playwright/test";

test("Debug: Check UI structure after click", async ({ page }) => {
  // Intercept console logs
  const logs: string[] = [];
  page.on("console", msg => {
    logs.push(`[${msg.type()}] ${msg.text()}`);
  });

  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(3000); // 等待更长时间确保渲染完成

  // Screenshot initial state
  await page.screenshot({ path: "test-results/debug-ui-1-initial.png", fullPage: true });

  // 检查初始 UI 结构
  console.log("=== Initial UI Check ===");

  // 检查右侧面板 tab 按钮
  const codeEditorBtn = page.getByRole('button', { name: /Code Edit/i });
  console.log("Code Editor tab button:", await codeEditorBtn.count());

  // 检查 editor-tabs 容器
  const editorTabs = page.locator('[data-testid="editor-tabs"]');
  console.log("EditorTabs container:", await editorTabs.count());
  console.log("EditorTabs visible:", await editorTabs.isVisible());

  // 检查初始标签页
  const tabItems = page.locator('[data-testid="tab-item"]');
  console.log("Initial tab items:", await tabItems.count());

  // 点击工具卡片
  console.log("=== Clicking Tool Card ===");
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();
  await toolCard.click();
  await page.waitForTimeout(1500); // 等待更新

  // Screenshot after click
  await page.screenshot({ path: "test-results/debug-ui-2-after-click.png", fullPage: true });

  console.log("=== After Click UI Check ===");

  // 检查 editor-tabs 容器
  console.log("EditorTabs visible after click:", await editorTabs.isVisible());

  // 检查标签页
  const tabItemsAfter = page.locator('[data-testid="tab-item"]');
  console.log("Tab items after click:", await tabItemsAfter.count());

  // 检查 role="tab" 的元素
  const roleTabs = page.locator('[role="tab"]');
  console.log("role=tab elements:", await roleTabs.count());
  const roleTabTexts = await roleTabs.allTextContents();
  console.log("role=tab texts:", roleTabTexts);

  // 检查所有 tab 相关元素
  const allTabs = page.locator('[data-tab], [data-testid*="tab"]');
  console.log("All tab-related elements:", await allTabs.count());

  // 打印所有相关日志
  console.log("=== Console Logs ===");
  const editorLogs = logs.filter(l => l.includes("EditorStore") || l.includes("openTab") || l.includes("FileEdit"));
  for (const log of editorLogs) {
    console.log(log);
  }
});
