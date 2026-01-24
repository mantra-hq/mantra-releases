import { test } from "@playwright/test";

test("Debug: Click file_edit card", async ({ page }) => {
  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(2000);
  
  // Screenshot initial
  await page.screenshot({ path: "test-results/click-1-initial.png", fullPage: true });
  
  // Find the tool card
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();
  console.log("Tool card found:", await toolCard.count());
  
  // Click the tool card
  await toolCard.click();
  console.log("Clicked tool card");
  await page.waitForTimeout(1000);
  
  // Screenshot after click
  await page.screenshot({ path: "test-results/click-2-after.png", fullPage: true });
  
  // Log all tabs
  const tabs = await page.locator('[role="tab"]').allTextContents();
  console.log("Tabs:", tabs);
  
  // Check if calculator.ts appears anywhere
  const calcText = await page.getByText(/calculator/).count();
  console.log("calculator text count:", calcText);
});
