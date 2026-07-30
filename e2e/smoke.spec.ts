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

    const main = page.getByRole("main");
    const header = page.getByTestId("app-header");

    await expect(
      main.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeVisible();
    await expect(main.getByRole("heading", { name: "Saves" })).toBeVisible();
    await expect(
      header.getByRole("combobox", { name: "Active save" }),
    ).toBeVisible();
    await expect(main.getByRole("heading", { name: "Snapshot" })).toBeVisible();
    await expect(
      header.getByRole("button", { name: "Load Data" }),
    ).toBeVisible();
    await expect(main.getByText(/^Bridge:/i)).toContainText("ready");
    await expect(main.getByText("Status:")).toContainText("ok");
    await expect(main.getByText("Stored value:")).toBeVisible();
  });

  test("home route saves demo value through stubbed IPC", async ({ page }) => {
    await page.goto("/");

    await page.getByLabel("Demo value (SQLite):").fill("smoke-value");
    await page.getByRole("button", { name: "Save demo value" }).click();

    await expect(
      page.getByRole("main").getByText("Stored value:"),
    ).toContainText("smoke-value");
  });

  test("nav rail expands from its own toggle", async ({ page }) => {
    await page.goto("/");

    const toggle = page.getByRole("button", { name: "Toggle navigation" });
    const rail = page.getByTestId("app-nav-rail");

    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await expect(rail).toHaveAttribute("data-expanded", "false");
    await expect(rail.getByText("Dashboard")).toBeHidden();

    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-expanded", "true");
    await expect(rail).toHaveAttribute("data-expanded", "true");
    await expect(rail.getByText("Dashboard")).toBeVisible();
  });

  test("search route shows no-snapshot empty state from stubbed IPC", async ({
    page,
  }) => {
    await page.goto("/search");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "Search" }),
    ).toBeVisible();
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
    await expect(page.getByRole("link", { name: "Search" })).toBeVisible();
  });

  test("top bar exposes global player search", async ({ page }) => {
    await page.goto("/");

    const header = page.getByTestId("app-header");
    await expect(
      header.getByRole("combobox", { name: "Search players" }),
    ).toBeVisible();
  });

  test("unknown routes render the not-found page", async ({ page }) => {
    await page.goto("/does-not-exist");

    await expect(
      page.getByRole("heading", { name: "Page not found" }),
    ).toBeVisible();
  });
});
