import { defineConfig, devices } from "@playwright/test";

// Set PGLS_E2E_PORT to run the e2e tests next to something else on port 3000, e.g. in another worktree.
const port = Number(process.env.PGLS_E2E_PORT ?? 3000);
const baseURL = `http://localhost:${port}`;

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",
  use: {
    baseURL,
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "bun run e2e/server.ts",
    url: baseURL,
    env: { PGLS_E2E_PORT: String(port) },
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
