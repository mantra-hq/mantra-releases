import { test } from "@playwright/test";

test("Debug: Screenshot FileEdit Diff", async ({ page }) => {
  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(2000);
  
  // Screenshot initial
  await page.screenshot({ path: "test-results/file-edit-1-initial.png", fullPage: true });
  console.log("Screenshot 1 saved: test-results/file-edit-1-initial.png");
  
  // Find and click expand button
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();
  const expandBtn = toolCard.locator('button svg.lucide-chevron-down').first();
  
  if (await expandBtn.count() > 0) {
    await expandBtn.click();
    console.log("Clicked expand button");
    await page.waitForTimeout(500);
  } else {
    console.log("No expand button found!");
  }
  
  // Screenshot after expand
  await page.screenshot({ path: "test-results/file-edit-2-expanded.png", fullPage: true });
  console.log("Screenshot 2 saved: test-results/file-edit-2-expanded.png");
  
  // Log element counts
  const greenText = await page.locator('.text-green-600').count();
  const greenTextDark = await page.locator('[class*="text-green"]').count();
  const diffBg = await page.locator('[class*="bg-green-500"]').count();
  console.log(`Green text: ${greenText}, All green classes: ${greenTextDark}, Diff bg: ${diffBg}`);
});
