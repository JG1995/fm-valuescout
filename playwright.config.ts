import { defineConfig, devices } from "@playwright/test";

const host = "127.0.0.1";
const port = 5173;
const baseURL = `http://${host}:${port}`;

export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "list",
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
    command: `pnpm exec vite --host ${host} --port ${port}`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
  },
});
