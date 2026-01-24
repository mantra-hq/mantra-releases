import { test } from "@playwright/test";

test("Debug: Capture errors after click", async ({ page }) => {
  // Capture page errors
  const errors: string[] = [];
  page.on("pageerror", err => {
    errors.push(`[PageError] ${err.message}\n${err.stack}`);
    console.log(`[PageError] ${err.message}`);
  });

  // Capture console errors
  page.on("console", msg => {
    if (msg.type() === "error") {
      console.log(`[ConsoleError] ${msg.text()}`);
      errors.push(`[ConsoleError] ${msg.text()}`);
    }
  });

  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(2000);

  console.log("=== Before Click ===");
  console.log("Errors so far:", errors.length);

  // Screenshot before
  await page.screenshot({ path: "test-results/debug-error-1-before.png", fullPage: true });

  // Click tool card
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();
  console.log("Clicking tool card...");
  await toolCard.click();

  // Wait and capture errors
  await page.waitForTimeout(2000);

  console.log("=== After Click ===");
  console.log("Errors:", errors);

  // Screenshot after
  await page.screenshot({ path: "test-results/debug-error-2-after.png", fullPage: true });

  // Check if page crashed
  const bodyText = await page.evaluate(() => document.body?.innerText || "");
  console.log("Body text length:", bodyText.length);
  console.log("Body text preview:", bodyText.slice(0, 500));
});
