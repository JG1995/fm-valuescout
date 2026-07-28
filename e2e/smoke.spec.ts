// Playwright smoke: Vite shell + stub IPC in Chromium — not real WebView, Rust, or SQLite.
// Scope: .wiki/ARCHITECTURE.md §6.4 Playwright smoke scope
import { expect, test } from "@playwright/test";
import { stubTauriIpc } from "./tauri-ipc-stub";

test.describe("walking skeleton smoke", () => {
  test.beforeEach(async ({ page }) => {
    await stubTauriIpc(page);
  });

  test("home route shows health status from stubbed IPC", async ({ page }) => {
    await page.goto("/");

    await expect(
      page.getByRole("heading", { name: "FM ValueScout" }),
    ).toBeVisible();
    await expect(page.getByText("Status:")).toContainText("ok");
    await expect(page.getByText("Stored value:")).toBeVisible();
  });

  test("home route saves demo value through stubbed IPC", async ({ page }) => {
    await page.goto("/");

    await page.getByLabel("Demo value (SQLite):").fill("smoke-value");
    await page.getByRole("button", { name: "Save demo value" }).click();

    await expect(page.getByText("Stored value:")).toContainText("smoke-value");
  });

  test("layout sidebar toggles from the header control", async ({ page }) => {
    await page.goto("/");

    const toggle = page.getByRole("button", { name: "Toggle sidebar" });
    const sidebar = page.getByTestId("app-sidebar");

    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await expect(sidebar).toHaveAttribute("data-open", "false");

    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-expanded", "true");
    await expect(sidebar).toHaveAttribute("data-open", "true");
  });

  test("unknown routes render the not-found page", async ({ page }) => {
    await page.goto("/does-not-exist");

    await expect(
      page.getByRole("heading", { name: "Page not found" }),
    ).toBeVisible();
  });
});
