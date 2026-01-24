import { test } from "@playwright/test";

test("Debug: Check standardTool in rendered content", async ({ page }) => {
  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(2000);

  // Screenshot initial state
  await page.screenshot({ path: "test-results/debug-standardtool-1.png", fullPage: true });

  // Find the tool card
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();
  console.log("Tool card found:", await toolCard.count());

  // Get the tool use ID
  const toolUseId = await toolCard.getAttribute('data-tool-use-id');
  console.log("Tool use ID:", toolUseId);

  // Check if tool card is clickable (has cursor-pointer class)
  const cardClasses = await toolCard.getAttribute('class');
  console.log("Card classes:", cardClasses);
  const hasClickHandler = cardClasses?.includes('cursor-pointer') ?? false;
  console.log("Has click handler:", hasClickHandler);

  // First expand the card to see the raw JSON
  const expandBtn = toolCard.locator('button').filter({ has: page.locator('svg.lucide-chevron-down') }).first();
  console.log("Expand button count:", await expandBtn.count());

  if (await expandBtn.count() > 0) {
    await expandBtn.click();
    await page.waitForTimeout(500);

    // Get the expanded JSON content
    const preContent = await toolCard.locator('pre').textContent();
    console.log("Raw JSON:", preContent);

    // Screenshot expanded state
    await page.screenshot({ path: "test-results/debug-standardtool-2.png", fullPage: true });
  }

  // Now test clicking the card itself (not the expand button)
  // First collapse it
  const collapseBtn = toolCard.locator('button').filter({ has: page.locator('svg.lucide-chevron-up') }).first();
  if (await collapseBtn.count() > 0) {
    await collapseBtn.click();
    await page.waitForTimeout(300);
  }

  // Click the tool card main area
  console.log("About to click tool card...");
  await toolCard.click();
  console.log("Clicked tool card");
  await page.waitForTimeout(1000);

  // Screenshot after click
  await page.screenshot({ path: "test-results/debug-standardtool-3.png", fullPage: true });

  // Check tabs
  const tabs = await page.locator('[role="tab"]').allTextContents();
  console.log("Tabs after click:", tabs);

  // Check right panel
  const rightPanel = page.locator('[data-testid="right-panel"]').first();
  console.log("Right panel visible:", await rightPanel.isVisible());

  // Check code editor
  const codeEditor = page.locator('[class*="monaco"], [class*="editor"]');
  console.log("Monaco/editor count:", await codeEditor.count());

  // Check any visible text containing calculator.ts
  const calcText = await page.locator('text=calculator').count();
  console.log("calculator text count:", calcText);
});
