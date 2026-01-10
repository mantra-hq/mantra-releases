import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Mantra client E2E tests.
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',

  // Run tests in parallel
  fullyParallel: true,

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry failed tests (helps with flaky tests due to timing)
  retries: process.env.CI ? 2 : 1,

  // Limit workers to avoid concurrent access issues with dev server
  // Single worker ensures stable test execution
  workers: 1,

  // Reporter to use
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
  ],

  // Shared settings for all the projects below
  use: {
    // Base URL to use in actions like `await page.goto('/')`
    // Port 1420 is used by Tauri/Vite dev server (see vite.config.ts)
    // Note: ?playwright 参数在每次导航时动态添加，避免 baseURL 查询参数连接问题
    baseURL: 'http://localhost:1420',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Capture screenshot on failure
    screenshot: 'only-on-failure',

    // Record video on failure
    video: 'on-first-retry',
  },

  // Visual comparison (screenshot) settings
  // @see https://playwright.dev/docs/test-snapshots
  expect: {
    toHaveScreenshot: {
      // Maximum allowed pixel difference for full page screenshots
      maxDiffPixels: 200,
      // Pixel comparison threshold (0-1, lower is stricter)
      threshold: 0.2,
      // Disable animations for stable visual comparison
      animations: 'disabled',
    },
    toMatchSnapshot: {
      // Maximum allowed pixel difference for element screenshots
      maxDiffPixels: 100,
      // Slightly stricter threshold for component screenshots
      threshold: 0.15,
    },
  },

  // Snapshot directory for visual regression baselines
  // Organized by test file: screenshots/visual.spec.ts-snapshots/
  snapshotDir: './screenshots',

  // Snapshot path template for better organization
  // Format: {snapshotDir}/{testFileName}-snapshots/{snapshotName}-{projectName}{ext}
  snapshotPathTemplate: '{snapshotDir}/{testFileName}-snapshots/{arg}-{projectName}{ext}',

  // Configure projects for major browsers
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    // Uncomment to enable Firefox testing
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
  ],

  // Run your local dev server before starting the tests
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  // Output folder for test artifacts
  outputDir: 'test-results',
});
