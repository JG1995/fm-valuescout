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

  test("planner route shows no-snapshot Load Data guidance", async ({
    page,
  }) => {
    await page.goto("/planner");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "Squad Planner" }),
    ).toBeVisible();
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
    await expect(
      main.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeVisible();
  });

  test("planner route shows first-use club setup for a loaded snapshot", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "Squad Planner" }),
    ).toBeVisible();
    await expect(main.getByRole("tab", { name: "Club setup" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(
      main.getByRole("combobox", { name: "Primary club" }),
    ).toBeVisible();
    await expect(main.getByText("Set up your club family")).toBeVisible();
  });

  test("planner tactic editor saves a linked phase adjustment", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Tactic" }).click();
    await expect(
      main.getByRole("heading", { name: "Tactic editor" }),
    ).toBeVisible();
    await main
      .getByRole("button", { name: "IP lane 1: GK, Goalkeeper" })
      .press("Enter");

    const weight = main.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    await weight.press("ArrowRight");
    await expect(main.getByText("IP 51% / OOP 49%")).toBeVisible();
    await main
      .getByRole("combobox", { name: "Lane 1 importance rank" })
      .selectOption("1");
    await main
      .getByRole("combobox", { name: "Lane 1 preferred foot" })
      .selectOption("left");
    await main
      .getByRole("combobox", { name: "Lane 1 foot preference" })
      .selectOption("strict");
    await main.getByRole("button", { name: "Save tactic" }).click();

    await expect(main.getByRole("status")).toHaveText("Tactic saved.");
  });

  test("planner depth adds strings for Senior, Reserves, and Youth", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Squad" }).click();
    for (const team of ["Senior", "Reserves", "Youth"]) {
      await main.getByRole("tab", { name: team }).click();
      await main.getByRole("button", { name: "Manage 1st string" }).click();
      await main.getByRole("menuitem", { name: "Add string" }).click();
      await expect(
        main.getByRole("columnheader", { name: "2nd string" }),
      ).toBeVisible();
    }
  });

  test("planner depth optimizes squads and shows the reconciled matrix", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Squad" }).click();
    await expect(
      main.getByRole("group", { name: "Squad controls" }),
    ).toBeVisible();
    await main.getByRole("button", { name: "Optimize squads" }).click();
    await expect(main.getByRole("status")).toHaveText("Squads optimized.");
    await main.getByRole("tab", { name: "Reserves" }).click();
    await expect(
      main.getByRole("button", {
        name: /Reserves, 1st string, Goalkeeper, Optimized Keeper, Resolved/,
      }),
    ).toBeVisible();
  });

  test("player profile route shows no-snapshot empty state from stubbed IPC", async ({
    page,
  }) => {
    await page.goto("/players/42");

    const main = page.getByRole("main");
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
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
