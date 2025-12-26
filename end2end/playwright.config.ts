import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for djv.sh e2e tests.
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",

  // Test timeout
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },

  // Run tests in parallel
  fullyParallel: true,

  // Fail if test.only is left in source
  forbidOnly: !!process.env.CI,

  // Retry failed tests in CI
  retries: process.env.CI ? 2 : 0,

  // Single worker in CI for stability
  workers: process.env.CI ? 1 : undefined,

  // HTML reporter
  reporter: "html",

  // Shared settings
  use: {
    baseURL: "http://localhost:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  // Browser projects
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    // Firefox and WebKit for local testing only
    ...(!process.env.CI
      ? [
          {
            name: "firefox",
            use: { ...devices["Desktop Firefox"] },
          },
          {
            name: "webkit",
            use: { ...devices["Desktop Safari"] },
          },
        ]
      : []),
  ],

  // Start dev server before tests
  webServer: {
    command: "cargo leptos watch",
    url: "http://localhost:3000",
    reuseExistingServer: true,
    timeout: 120_000, // 2 minutes for cargo build
    cwd: "..",
    env: {
      DATABASE_URL: process.env.DATABASE_URL || "postgres:///djv_dev?host=/run/postgresql",
    },
  },
});
