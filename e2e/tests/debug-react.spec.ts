import { test } from "@playwright/test";

test("Debug: Check React state and handlers", async ({ page }) => {
  // Intercept console logs
  const logs: string[] = [];
  page.on("console", msg => {
    if (msg.type() === "log") {
      logs.push(msg.text());
    }
  });

  await page.goto("/session/mock-session-file-edit?playwright");
  await page.waitForTimeout(2000);

  // Add debugging to window
  await page.evaluate(() => {
    // Get React fiber from DOM
    const _getReactFiber = (element: Element): any => {
      const key = Object.keys(element).find(k =>
        k.startsWith("__reactFiber$") ||
        k.startsWith("__reactInternalInstance$") ||
        k.startsWith("__reactProps$")
      );
      return key ? (element as any)[key] : null;
    };

    const toolCard = document.querySelector('[data-testid="tool-call-card"]');
    if (toolCard) {
      console.log("[Debug] Tool card element found");

      // Check props
      const propsKey = Object.keys(toolCard).find(k => k.startsWith("__reactProps$"));
      if (propsKey) {
        const props = (toolCard as any)[propsKey];
        console.log("[Debug] onClick is function:", typeof props?.onClick === 'function');
        console.log("[Debug] onClick type:", typeof props?.onClick);
      }
    } else {
      console.log("[Debug] Tool card NOT found");
    }
  });

  await page.waitForTimeout(500);

  // Find the tool card
  const toolCard = page.locator('[data-testid="tool-call-card"]').filter({ hasText: 'Edit' }).first();

  // Log the onclick status before clicking
  console.log("Before click - checking onClick handler...");

  // Click and check what happens
  await toolCard.click({ force: true });
  await page.waitForTimeout(1000);

  // Print collected logs
  console.log("Console logs:", logs);

  // Check if editor tab opened
  const tabs = await page.locator('[role="tab"]').allTextContents();
  console.log("Tabs:", tabs);

  // Take screenshot
  await page.screenshot({ path: "test-results/debug-react-state.png", fullPage: true });
});
