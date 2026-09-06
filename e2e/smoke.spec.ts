// Playwright smoke: Vite shell + stub IPC in Chromium — not real WebView, Rust, or SQLite.
// Scope: .wiki/ARCHITECTURE.md §6.4 Playwright smoke scope
import { expect, test } from "@playwright/test";
import { stubTauriIpc } from "./tauri-ipc-stub";

test.describe("application smoke", () => {
  test.beforeEach(async ({ page }) => {
    await stubTauriIpc(page);
  });

  test("Dashboard stays minimal and Settings hosts app management", async ({
    page,
  }) => {
    await page.goto("/");

    const main = page.getByRole("main");
    const header = page.getByTestId("app-header");

    await expect(
      main.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeVisible();
    await expect(main.getByText("Placeholder.")).toBeVisible();
    await expect(main.getByRole("heading", { name: "Saves" })).toHaveCount(0);
    await expect(
      header.getByRole("combobox", { name: "Active save" }),
    ).toBeVisible();
    await expect(
      header.getByRole("button", { name: "Load Data" }),
    ).toBeVisible();
    await page.getByRole("link", { name: "Settings" }).click();
    await expect(page).toHaveURL(/\/settings$/);
    await expect(
      main.getByRole("heading", { level: 1, name: "Settings" }),
    ).toBeVisible();
    await expect(main.getByRole("region", { name: "Save data" })).toBeVisible();
    await expect(
      main.getByRole("region", { name: "Managed club" }),
    ).toHaveCount(0);
    await expect(main.getByRole("region", { name: "Bridge" })).toBeVisible();
    await expect(main.getByText(/^Bridge:/i)).toContainText("ready");
    await expect(main.getByText("Status:")).toHaveCount(0);
    await expect(main.getByText("Stored value:")).toHaveCount(0);
  });

  test("My Club creates Club DNA and exposes its Squad column", async ({
    page,
  }) => {
    await stubTauriIpc(page, { squadOverview: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await main.getByRole("button", { name: "Define DNA" }).click();
    const dialog = page.getByRole("dialog", { name: "Define Club DNA" });
    await expect(dialog).toContainText("scales each selected 1–20 value by 5");
    await dialog.getByRole("checkbox", { name: "Acceleration" }).check();
    await expect(dialog.getByText("Selected attributes (1)")).toBeVisible();
    await dialog.getByRole("button", { name: "Save Club DNA" }).click();

    await expect(
      main.getByRole("columnheader", { name: "Club DNA" }),
    ).toBeVisible();
  });

  test("Settings renames and removes a historical snapshot", async ({
    page,
  }) => {
    await stubTauriIpc(page, { snapshotHistory: true });
    await page.goto("/settings");

    const main = page.getByRole("main");
    const history = main.getByRole("table", { name: "Snapshot history" });
    await expect(history.getByRole("row").nth(1)).toContainText("2026-08-01");
    await expect(history.getByRole("row").nth(2)).toContainText("2026-06-01");

    await history
      .getByRole("button", { name: "Rename snapshot 2026-08-01" })
      .click();
    const rename = page.getByRole("dialog", { name: /Rename snapshot/ });
    await rename.getByLabel("Snapshot name").fill("Transfer window");
    await rename.getByRole("button", { name: "Save name" }).click();
    await expect(history.getByRole("row").nth(1)).toContainText(
      "Transfer window",
    );
    await expect(history.getByRole("row").nth(1)).toContainText("2026-08-01");

    await history
      .getByRole("button", { name: /^Delete snapshot 2026-06-01/ })
      .click();
    const deletion = page.getByRole("dialog", { name: /Delete snapshot/ });
    await expect(deletion).toContainText("Moneyball import data");
    await deletion
      .getByRole("button", { name: "Delete snapshot", exact: true })
      .click();
    await expect(history.getByRole("row")).toHaveCount(2);
    await expect(
      history.getByRole("button", { name: /^Delete snapshot 2026-06-01/ }),
    ).toHaveCount(0);
  });

  test("Settings promotes the next dated snapshot after deleting current", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      snapshotHistory: true,
    });
    await page.goto("/settings");

    const main = page.getByRole("main");
    const history = main.getByRole("table", { name: "Snapshot history" });
    const snapshotSummary = main.getByText(/In database:/).locator("..");
    await expect(snapshotSummary).toContainText("24 players");

    await history
      .getByRole("button", { name: /^Delete snapshot 2026-08-01/ })
      .click();
    await page
      .getByRole("dialog", { name: /Delete snapshot/ })
      .getByRole("button", { name: "Delete snapshot", exact: true })
      .click();

    await expect(history.getByRole("row")).toHaveCount(2);
    await expect(history.getByRole("row").nth(1)).toContainText("2026-06-01");
    await expect(history.getByRole("row").nth(1)).toContainText("Current");
    await expect(snapshotSummary).toContainText("21 players");
  });

  test("Settings promotes an older snapshot after editing its date", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      snapshotHistory: true,
    });
    await page.goto("/settings");

    const main = page.getByRole("main");
    const history = main.getByRole("table", { name: "Snapshot history" });
    const snapshotSummary = main.getByText(/In database:/).locator("..");
    await expect(history.getByRole("row").nth(1)).toContainText("2026-08-01");
    await expect(snapshotSummary).toContainText("24 players");

    await history
      .getByRole("button", { name: /^Edit date for snapshot 2026-06-01/ })
      .click();
    const editor = page.getByRole("dialog", { name: /Edit date/ });
    await editor.getByLabel("In-game date").fill("2026-09-01");
    await editor.getByRole("button", { name: "Save date" }).click();

    await expect(history.getByRole("row").nth(1)).toContainText("2026-09-01");
    await expect(history.getByRole("row").nth(1)).toContainText("Current");
    await expect(snapshotSummary).toContainText("21 players");
  });

  test("Settings deletes inactive and active saves with the right fallback", async ({
    page,
  }) => {
    await page.goto("/settings");

    const main = page.getByRole("main");
    const activeSave = page.getByRole("combobox", { name: "Active save" });
    await main.getByLabel("New save").fill("Archive");
    await main.getByRole("button", { name: "Create save" }).click();
    await main.getByRole("button", { name: /^Delete save Archive/ }).click();
    const inactiveDeletion = page.getByRole("dialog", { name: /Delete save/ });
    await expect(inactiveDeletion).toContainText(
      "The active save stays unchanged",
    );
    await inactiveDeletion
      .getByRole("button", { name: "Delete save", exact: true })
      .click();
    await expect(activeSave).toHaveValue("1");

    await main.getByLabel("New save").fill("Archive");
    await main.getByRole("button", { name: "Create save" }).click();
    await main
      .getByRole("button", { name: /^Delete save Default save/ })
      .click();
    await page
      .getByRole("dialog", { name: /Delete save/ })
      .getByRole("button", { name: "Delete save", exact: true })
      .click();
    await expect(activeSave.locator("option:checked")).toHaveText("Archive");
  });

  test("Settings replaces the final deleted save with Default save", async ({
    page,
  }) => {
    await page.goto("/settings");

    const main = page.getByRole("main");
    await main
      .getByRole("button", { name: /^Delete save Default save/ })
      .click();
    const deletion = page.getByRole("dialog", { name: /Delete save/ });
    await expect(deletion).toContainText(
      "A blank Default save will replace it",
    );
    await deletion
      .getByRole("button", { name: "Delete save", exact: true })
      .click();
    await expect(
      page
        .getByRole("combobox", { name: "Active save" })
        .locator("option:checked"),
    ).toHaveText("Default save");
  });

  test("Planner IPC follows the stub save lifecycle", async ({ page }) => {
    await page.goto("/");

    const result = await page.evaluate(async () => {
      const invoke = (
        globalThis as unknown as {
          __TAURI_INTERNALS__: {
            invoke: (
              command: string,
              args?: Record<string, unknown>,
            ) => Promise<unknown>;
          };
        }
      ).__TAURI_INTERNALS__.invoke;
      const plannerError = async (context: {
        saveId: number;
        contextToken: string;
      }) => {
        try {
          await invoke("get_planner_tactic", context);
          return null;
        } catch (error) {
          return error instanceof Error ? error.message : String(error);
        }
      };

      const beforeCreate = await plannerError({
        saveId: 2,
        contextToken: "save-token-2",
      });
      const created = (await invoke("create_save", {
        name: "Planner save",
      })) as { id: number; contextToken: string };
      const createdContext = {
        saveId: created.id,
        contextToken: created.contextToken,
      };
      await invoke("get_planner_tactic_options", createdContext);
      await invoke("delete_save", createdContext);
      const afterDelete = await plannerError(createdContext);

      const replacementResult = (await invoke("delete_save", {
        saveId: 1,
        contextToken: "save-token-1",
      })) as { activeSave: { id: number; contextToken: string } };
      const replacementContext = {
        saveId: replacementResult.activeSave.id,
        contextToken: replacementResult.activeSave.contextToken,
      };
      const oldToken = await plannerError({
        saveId: 1,
        contextToken: "save-token-1",
      });
      await invoke("save_planner_tactic", {
        ...replacementContext,
        tactic: { lanes: [] },
      });
      const replacementTactic = (await invoke(
        "get_planner_tactic",
        replacementContext,
      )) as { lanes: unknown[] };

      return {
        beforeCreate,
        createdId: created.id,
        afterDelete,
        replacementContext,
        oldToken,
        replacementLaneCount: replacementTactic.lanes.length,
      };
    });

    expect(result).toEqual({
      beforeCreate: "Save 2 not found",
      createdId: 2,
      afterDelete: "Save 2 not found",
      replacementContext: {
        saveId: 1,
        contextToken: "save-token-1-replacement",
      },
      oldToken: "Save changed or no longer exists",
      replacementLaneCount: 0,
    });
  });

  test("top navigation fits every destination at 1280x800", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");

    const header = page.getByTestId("app-header");
    const nav = page.getByRole("navigation", { name: "Primary" });
    const main = page.getByRole("main");
    await expect(header).toBeVisible();
    await expect(nav).toBeVisible();
    const [headerBox, navBox, mainBox] = await Promise.all([
      header.boundingBox(),
      nav.boundingBox(),
      main.boundingBox(),
    ]);
    expect(headerBox).not.toBeNull();
    expect(navBox).not.toBeNull();
    expect(mainBox).not.toBeNull();
    if (!headerBox || !navBox || !mainBox) {
      throw new Error("Expected the shell bars to have a visible layout.");
    }
    expect(navBox.y).toBeGreaterThanOrEqual(headerBox.y + headerBox.height - 1);
    expect(mainBox.y).toBeGreaterThanOrEqual(navBox.y + navBox.height - 1);
    expect(mainBox.x + mainBox.width).toBeLessThanOrEqual(1280 + 1);
    await expect(page.getByTestId("app-nav-rail")).toHaveCount(0);

    for (const name of [
      "Dashboard",
      "Search",
      "Moneyball",
      "Staff Search",
      "My Staff",
      "Squad",
      "Planner",
      "Tactic",
      "Youth",
      "Settings",
    ] as const) {
      await expect(nav.getByRole("link", { name, exact: true })).toBeVisible();
    }
    await expect(
      nav.locator("[data-nav-caption]").allTextContents(),
    ).resolves.toEqual(["Players", "Staff", "Club"]);

    const navOverflow = await nav.evaluate((element) => {
      const navElement = element as unknown as {
        clientWidth: number;
        scrollWidth: number;
      };
      return navElement.scrollWidth - navElement.clientWidth;
    });
    expect(navOverflow).toBeLessThanOrEqual(1);
  });

  test("search route shows no-snapshot empty state from stubbed IPC", async ({
    page,
  }) => {
    await page.goto("/search");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeVisible();
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Search", exact: true }),
    ).toHaveAttribute("aria-current", "page");
  });

  test("Staff Search stays standalone while My Staff owns the managed club", async ({
    page,
  }) => {
    await stubTauriIpc(page, { staffWorkspace: true });
    await page.goto("/staff");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "Staff Search" }),
    ).toBeVisible();
    await expect(main.getByRole("tablist")).toHaveCount(0);
    await expect(
      page.getByRole("link", { name: "Staff Search" }),
    ).toHaveAttribute("aria-current", "page");
    const table = main.getByRole("table", { name: "Staff search results" });
    await expect(table).toBeVisible();
    await expect(table.getByRole("columnheader")).toHaveCount(26);
    await expect(
      table.getByRole("columnheader", { name: "Coach — Goalkeeping" }),
    ).toBeVisible();
    await expect(table.getByText("Alex Coach")).toBeVisible();
    await page.goto("/staff?view=my-staff");
    await expect(
      main.getByRole("heading", { level: 1, name: "My Staff" }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: "My Staff" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    const staffTable = main.getByRole("table", { name: "Staff overview" });
    await expect(staffTable).toBeVisible();
    await expect(staffTable.getByText("Alex Coach")).toBeVisible();
    await page.goto("/staff?shortlistOnly=true");
    await expect(main.getByText("No Staff Shortlist uploaded")).toBeVisible();
    await expect(
      main.getByRole("button", { name: "Upload CSV" }),
    ).toBeVisible();
    await expect(
      main.getByRole("combobox", { name: "Preferred Job" }),
    ).toHaveValue("");
  });

  test("Staff rows open profiles with staff-only surfaces", async ({
    page,
  }) => {
    await stubTauriIpc(page, { staffWorkspace: true });
    await page.goto("/staff");

    const main = page.getByRole("main");
    const table = main.getByRole("table", { name: "Staff search results" });
    await table.locator('tr[data-index="0"]').click();
    await expect(
      main.getByRole("heading", { name: "Alex Coach" }),
    ).toBeVisible();
    await expect(main.getByRole("heading", { name: "Role fit" })).toBeVisible();
    await expect(main.getByRole("region", { name: "Coaching" })).toBeVisible();
    await expect(main.getByRole("region", { name: "Mental" })).toBeVisible();
    await expect(main.getByRole("region", { name: "Knowledge" })).toBeVisible();
    await expect(
      main.getByRole("tablist", { name: "Staff attribute groups" }),
    ).toHaveCount(0);
    await expect(main.getByText("Authority")).toBeVisible();
    await expect(main.getByText("Wonderkid Mentality")).toHaveCount(0);
    await expect(main.getByText("Pitch")).toHaveCount(0);
    await main.getByRole("button", { name: "Hide hidden info" }).click();
    await expect(
      main.getByRole("button", { name: "Reveal hidden info" }),
    ).toBeVisible();
  });

  test("Staff Shortlist filters staff and adapts score columns", async ({
    page,
  }) => {
    await stubTauriIpc(page, { staffWorkspace: true, staffShortlist: true });
    await page.goto("/staff?shortlistOnly=true");

    const main = page.getByRole("main");
    const table = main.getByRole("table", { name: "Staff Shortlist" });
    const preferredJob = main.getByRole("combobox", { name: "Preferred Job" });

    await expect(table.getByText("Alex Coach")).toBeVisible();
    await preferredJob.selectOption("Technical Director");
    await expect(
      table.getByRole("columnheader", { name: "Technical Director" }),
    ).toBeVisible();
    await expect(
      table.getByRole("columnheader", { name: "Preferred Job" }),
    ).toHaveCount(0);

    await preferredJob.selectOption("Coach");
    await expect(
      table.getByRole("columnheader", { name: "Coach — Attacking Technical" }),
    ).toBeVisible();
    await expect(
      table.getByRole("columnheader", { name: "Coach — Fitness" }),
    ).toHaveCount(0);

    await preferredJob.selectOption("Manager");
    await expect(
      table.getByRole("columnheader", { name: "Manager" }),
    ).toBeVisible();
    const managerRows = table.locator("tr[data-index]");
    await expect(managerRows.nth(0)).toContainText("Manager Morgan");
    await expect(managerRows.nth(0).getByRole("cell").last()).toHaveText("90");
    await expect(managerRows.nth(1)).toContainText("Manager Taylor");
    await expect(managerRows.nth(1).getByRole("cell").last()).toHaveText("80");
    await expect(table.getByRole("columnheader")).toHaveCount(8);

    await preferredJob.selectOption("");
    await expect(
      table.getByRole("columnheader", { name: "Preferred Job" }),
    ).toBeVisible();
    await expect(
      table.getByRole("columnheader", { name: "Technical Director" }),
    ).toBeVisible();
    await main.getByRole("checkbox", { name: "Only unemployed" }).check();
    await expect(table.getByText("Coach Casey")).toBeVisible();
    await expect(table.getByText("Manager Morgan")).toBeVisible();
    await expect(table.getByText("Alex Coach")).toHaveCount(0);
    await main.getByRole("checkbox", { name: "Only unemployed" }).uncheck();
    await table
      .locator("tr[data-index]")
      .filter({ hasText: "Alex Coach" })
      .click();
    await expect(
      main.getByRole("heading", { name: "Alex Coach" }),
    ).toBeVisible();
  });

  test("Staff Shortlist configures assignment recommendations and clears them after snapshot replacement", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      staffAssignment: true,
      staffShortlist: true,
      staffWorkspace: true,
    });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/staff?shortlistOnly=true");

    const main = page.getByRole("main");
    await main.getByRole("button", { name: "Configure Club Staff" }).click();
    const dialog = page.getByRole("dialog", {
      name: "Configure assignment slots",
    });
    const firstTeam = dialog.getByRole("group", { name: "First Team" });
    const coaching = firstTeam.getByRole("group", { name: "Coaching" });
    const recruitment = firstTeam.getByRole("group", {
      name: "Recruitment",
    });
    const medical = firstTeam.getByRole("group", { name: "Medical" });
    const assistantManager = coaching.getByRole("spinbutton", {
      name: "Assistant Manager slots",
      exact: true,
    });
    const coaches = coaching.getByRole("spinbutton", {
      name: "Coaches slots",
      exact: true,
    });
    const headOfYouthDevelopment = coaching.getByRole("spinbutton", {
      name: "Head of Youth Development slots",
      exact: true,
    });
    const manager = dialog
      .getByRole("group", { name: "Reserves" })
      .getByRole("spinbutton", { name: "Manager slots", exact: true });
    const scout = recruitment.getByRole("spinbutton", {
      name: "Scout slots",
      exact: true,
    });

    await expect(firstTeam).toBeVisible();
    await expect(coaching).toBeVisible();
    await expect(recruitment).toBeVisible();
    await expect(medical).toBeVisible();
    await expect(dialog.getByRole("group", { name: "Club" })).toHaveCount(0);
    await expect(
      firstTeam.getByRole("spinbutton", {
        name: "Manager slots",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(
      recruitment.getByRole("spinbutton", {
        name: "Recruitment Analyst slots",
        exact: true,
      }),
    ).toBeVisible();
    await expect(
      dialog.getByRole("group", { name: "Reserves" }).getByRole("spinbutton", {
        name: "Head of Youth Development slots",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(
      dialog.getByRole("group", { name: "Youth" }).getByRole("spinbutton", {
        name: "Head of Youth Development slots",
        exact: true,
      }),
    ).toHaveCount(0);
    await expect(headOfYouthDevelopment).toHaveAttribute("max", "1");
    await expect(assistantManager).toHaveValue("0");
    await expect(assistantManager).toHaveAttribute("max", "50");
    await expect(coaches).toHaveValue("0");
    await expect(manager).toHaveValue("0");
    await expect(scout).toHaveValue("0");
    await expect(scout).toHaveAttribute("max", "50");
    const dialogBox = await dialog.boundingBox();
    expect(dialogBox).not.toBeNull();
    expect(dialogBox?.x).toBeGreaterThanOrEqual(0);
    expect(dialogBox?.y).toBeGreaterThanOrEqual(0);
    expect((dialogBox?.x ?? 0) + (dialogBox?.width ?? 0)).toBeLessThanOrEqual(
      1280,
    );
    expect((dialogBox?.y ?? 0) + (dialogBox?.height ?? 0)).toBeLessThanOrEqual(
      800,
    );
    await assistantManager.fill("50");
    await expect(assistantManager).toHaveValue("50");
    await assistantManager.fill("1");
    await coaches.fill("1");
    await manager.fill("1");
    await scout.fill("1");
    await dialog.getByRole("button", { name: "Save slots" }).click();
    await expect(main.getByRole("status")).toHaveText("Slot counts saved.");
    await expect(dialog).toBeHidden();

    await main.getByRole("button", { name: "Configure Club Staff" }).click();
    await expect(dialog).toBeVisible();
    await expect(assistantManager).toHaveValue("1");
    await expect(coaches).toHaveValue("1");
    await expect(manager).toHaveValue("1");
    await expect(scout).toHaveValue("1");
    await dialog.getByRole("button", { name: "Cancel" }).click();

    await main.getByRole("button", { name: "Optimize assignments" }).click();
    const assignments = main.getByRole("table", {
      name: "Staff assignment recommendations and vacancies",
    });
    await expect(assignments).toBeVisible();
    await expect(
      main.getByText("5 joined shortlisted candidates; 4 configured slots."),
    ).toBeVisible();
    await expect(assignments.getByRole("row")).toHaveCount(5);
    await expect(assignments).toContainText("First Team");
    await expect(assignments).toContainText("Reserves");
    await expect(assignments).toContainText("Club");
    const assignmentRows = assignments.locator("tbody tr");
    await expect(assignmentRows.nth(2)).toContainText("Club");
    await expect(assignmentRows.nth(3)).toContainText("Reserves");
    await expect(assignments).toContainText("Alex Assistant");
    await expect(assignments).toContainText(
      "Preferred Job: Assistant Manager. Eligible for this target.",
    );
    await expect(assignments).toContainText("Coach Casey");
    await expect(assignments).toContainText(
      "Preferred Job: Coach. Eligible for this target. Coach requirement: Attacking Technical.",
    );
    await expect(assignments).toContainText("Current staff");
    await expect(assignments).toContainText("Riley Scout");
    await expect(assignments).toContainText(
      "Preferred Job: Scout. Eligible for this target.",
    );
    await expect(assignments).toContainText("Recruitment");
    await expect(
      assignments.getByRole("img", {
        name: "Assistant Manager: 82, Excellent",
      }),
    ).toBeVisible();
    await expect(assignments).toContainText("Vacancy");
    await expect(assignments).toContainText(
      "Coach requirement: Goalkeeping. 0 eligible scores; 1 unavailable score; 1 joined shortlisted candidate.",
    );

    const collapse = main.getByRole("button", {
      name: "Collapse assignment recommendations",
    });
    const bodyId = await collapse.getAttribute("aria-controls");
    await expect(collapse).toHaveAttribute("aria-expanded", "true");
    await collapse.click();
    const expand = main.getByRole("button", {
      name: "Expand assignment recommendations",
    });
    await expect(expand).toHaveAttribute("aria-expanded", "false");
    await expect(expand).toHaveAttribute("aria-controls", bodyId ?? "");
    await expect(assignments).toBeHidden();
    await expect(main.getByText("Alex Assistant")).toBeHidden();
    await expand.click();
    await expect(assignments).toBeVisible();
    await expect(assignments).toContainText("Alex Assistant");

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(assignments).toBeVisible();
      const [assignmentsBox, mainBox] = await Promise.all([
        assignments.boundingBox(),
        main.boundingBox(),
      ]);
      expect(assignmentsBox).not.toBeNull();
      expect(mainBox).not.toBeNull();
      expect(assignmentsBox?.x).toBeGreaterThanOrEqual(mainBox?.x ?? 0);
      expect(
        (assignmentsBox?.x ?? 0) + (assignmentsBox?.width ?? 0),
      ).toBeLessThanOrEqual((mainBox?.x ?? 0) + (mainBox?.width ?? 0) + 1);
    }

    await page
      .getByTestId("app-header")
      .getByRole("button", { name: "Load Data" })
      .click();
    await expect(
      page.getByText("Loaded 4 players into the database."),
    ).toBeVisible();
    await expect(assignments).toHaveCount(0);
    await expect(
      main.getByRole("button", { name: "Optimize assignments" }),
    ).toBeEnabled();
  });

  test("Staff Shortlist renders standalone Club sections without Senior", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      staffAssignment: true,
      staffAssignmentSenior: false,
      staffShortlist: true,
      staffWorkspace: true,
    });
    await page.goto("/staff?shortlistOnly=true");

    const main = page.getByRole("main");
    await main.getByRole("button", { name: "Configure Club Staff" }).click();
    const dialog = page.getByRole("dialog", {
      name: "Configure assignment slots",
    });
    const club = dialog.getByRole("group", { name: "Club" });

    await expect(dialog.getByRole("group", { name: "First Team" })).toHaveCount(
      0,
    );
    await expect(club.getByRole("group", { name: "Coaching" })).toBeVisible();
    await expect(
      club.getByRole("group", { name: "Recruitment" }),
    ).toBeVisible();
    await expect(club.getByRole("group", { name: "Medical" })).toBeVisible();
    await expect(
      club.getByRole("spinbutton", {
        name: "Head of Youth Development slots",
        exact: true,
      }),
    ).toHaveAttribute("max", "1");
    await expect(
      club.getByRole("spinbutton", {
        name: "Recruitment Analyst slots",
        exact: true,
      }),
    ).toHaveAttribute("max", "50");
  });

  test("Staff Profile keeps role fit inside a virtual scrollport", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await stubTauriIpc(page, { staffWorkspace: true });
    await page.goto("/staff/101");

    const main = page.getByRole("main");
    const scrollport = main.getByTestId("staff-role-fit-scroller");
    await expect(scrollport).toBeVisible();
    const before = await main.evaluate((element) => {
      const scrollable = element as unknown as {
        clientHeight: number;
        scrollHeight: number;
        scrollTop: number;
      };
      return {
        clientHeight: scrollable.clientHeight,
        scrollHeight: scrollable.scrollHeight,
        scrollTop: scrollable.scrollTop,
      };
    });
    const roleFit = await scrollport.evaluate((element) => {
      const scrollable = element as unknown as {
        clientHeight: number;
        scrollHeight: number;
      };
      return {
        clientHeight: scrollable.clientHeight,
        scrollHeight: scrollable.scrollHeight,
      };
    });
    expect(before.scrollHeight).toBeLessThanOrEqual(before.clientHeight + 1);
    expect(roleFit.scrollHeight).toBeGreaterThan(roleFit.clientHeight);
    expect(
      await scrollport.locator("tbody tr[data-index]").count(),
    ).toBeLessThan(20);

    await scrollport.evaluate((element) => {
      const scrollable = element as unknown as {
        dispatchEvent: (event: Event) => boolean;
        scrollHeight: number;
        scrollTop: number;
      };
      scrollable.scrollTop = scrollable.scrollHeight;
      scrollable.dispatchEvent(new Event("scroll"));
    });
    await expect(scrollport.getByText("Sports Scientist")).toBeVisible();
    expect(
      await main.evaluate(
        (element) => (element as unknown as { scrollTop: number }).scrollTop,
      ),
    ).toBe(before.scrollTop);
  });

  test("Staff keeps a long Search result set inside the main table scroller", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      staffWorkspace: true,
      playerTableRowCount: 101,
    });
    await page.goto("/staff");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("table", { name: "Staff search results" }),
    ).toBeVisible();
    const dimensions = await main.evaluate((element) => {
      const mainElement = element as unknown as {
        clientHeight: number;
        scrollHeight: number;
        querySelector: (
          selector: string,
        ) => { clientHeight: number; scrollHeight: number } | null;
      };
      const scroller = mainElement.querySelector(
        '[data-testid="staff-search-results-scroller"]',
      );
      return {
        mainClientHeight: mainElement.clientHeight,
        mainScrollHeight: mainElement.scrollHeight,
        scrollerClientHeight: scroller?.clientHeight ?? 0,
        scrollerScrollHeight: scroller?.scrollHeight ?? 0,
      };
    });
    expect(dimensions.mainScrollHeight).toBeLessThanOrEqual(
      dimensions.mainClientHeight + 1,
    );
    expect(dimensions.scrollerScrollHeight).toBeGreaterThan(
      dimensions.scrollerClientHeight,
    );
  });

  test("My Staff fetches a later page from the configured family", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      playerTableRowCount: 101,
      staffWorkspace: true,
    });
    await page.goto("/staff?view=my-staff");

    const main = page.getByRole("main");
    const table = main.getByRole("table", { name: "Staff overview" });
    const scroller = main.getByTestId("my-staff-results-scroller");
    await expect(table).toBeVisible();
    await scroller.evaluate((element) => {
      const scrollable = element as unknown as {
        scrollHeight: number;
        scrollTop: number;
      };
      scrollable.scrollTop = scrollable.scrollHeight;
    });
    await expect(table.getByText("Staff member 101")).toBeVisible();
  });

  test("My Staff confirms a managed-club CA boost", async ({ page }) => {
    await stubTauriIpc(page, { staffWorkspace: true });
    await page.goto("/staff?view=my-staff");

    const main = page.getByRole("main");
    const table = main.getByRole("table", { name: "Staff overview" });
    await expect(table.getByRole("button", { name: "Boost CA" })).toHaveCount(
      0,
    );
    await main.getByRole("button", { name: "Boost all CA" }).click();
    const dialog = page.getByRole("dialog", { name: "Boost all CA?" });
    await expect(dialog).toContainText("at your managed club");
    await dialog.getByRole("button", { name: "Boost all CA" }).click();
    await expect(main.getByTestId("staff-boost-outcome")).toContainText(
      "1 processed — 1 updated, 0 skipped, 0 failed.",
    );
  });

  test("My Staff points an unconfigured save to My Club managed club", async ({
    page,
  }) => {
    await stubTauriIpc(page, { staffFamily: "none", staffWorkspace: true });
    await page.goto("/staff?view=my-staff");

    const main = page.getByRole("main");
    await expect(
      main.getByText("Choose your managed club", { exact: true }),
    ).toBeVisible();
    await expect(
      main.getByRole("link", { name: "Open Managed Club" }),
    ).toHaveAttribute("href", "/my-club#managed-club");
  });

  test("My Club shows no-snapshot Load Data guidance", async ({ page }) => {
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "My Club" }),
    ).toBeVisible();
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
    await expect(
      main.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeVisible();
  });

  test("legacy Planner links replace into the canonical My Club URL", async ({
    page,
  }) => {
    await page.goto("/planner?view=tactic&sort=name&dir=asc");

    await expect(page).toHaveURL(
      /\/my-club\?view=tactic&squadSort=name&squadDir=asc$/,
    );
    await expect(
      page.getByRole("main").getByRole("heading", {
        level: 1,
        name: "My Club",
      }),
    ).toBeVisible();
  });

  test("Squad points an unconfigured save to My Club managed club", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("heading", { level: 1, name: "My Club" }),
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Squad", exact: true }),
    ).toHaveAttribute("aria-current", "page");
    await expect(
      page.getByRole("tablist", { name: "My Club workspaces" }),
    ).toHaveCount(0);
    await expect(
      main.getByText("Choose your managed club", { exact: true }),
    ).toBeVisible();
    await main.getByRole("link", { name: "Open Managed Club" }).click();
    await expect(page).toHaveURL(/\/my-club#managed-club$/);
    await expect(
      page.getByRole("main").getByRole("combobox", { name: "Managed club" }),
    ).toBeVisible();
  });

  test("legacy Settings managed-club links save from My Club", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/settings#managed-club");
    await expect(page).toHaveURL(/\/my-club#managed-club$/);

    const managedClub = page.getByRole("main").getByRole("combobox", {
      name: "Managed club",
    });
    await managedClub.fill("Bar");
    await page.getByRole("option", { name: "Barcelona", exact: true }).click();
    await page
      .getByRole("main")
      .getByRole("button", { name: "Save managed club" })
      .click();
    await expect(managedClub).toHaveValue("Barcelona");
  });

  test("configured Squad shows its sortable player overview", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    const table = main.getByRole("table", { name: "Squad overview" });
    await expect(table).toBeVisible();
    await expect(main.getByTestId("squad-overview-scroller")).toBeVisible();
    await expect(
      table.getByRole("link", { name: "Alex Scout" }),
    ).toHaveAttribute("href", "/players/42");
    await expect(
      table.getByRole("columnheader", { name: "Suggested Training" }),
    ).toBeVisible();
    await table.getByRole("button", { name: "Name", exact: true }).click();
    await expect(
      table.getByRole("columnheader", { name: "Name" }),
    ).toHaveAttribute("aria-sort", "ascending");
    await expect(main.getByRole("button", { name: "Next page" })).toHaveCount(
      0,
    );
    await table
      .locator("tbody tr[data-index]")
      .first()
      .getByText("Barcelona")
      .click();
    await expect(page).toHaveURL(/\/players\/42$/);
  });

  test("configured Squad keeps its table inside desktop viewports", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      playerTableRowCount: 101,
      squadOverview: true,
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await page.goto("/my-club");

      const main = page.getByRole("main");
      const scroller = main.getByTestId("squad-overview-scroller");
      await expect(scroller).toBeVisible();
      await expect(
        main.getByRole("combobox", { name: "Managed club" }),
      ).toBeVisible();

      const [mainBox, scrollerBox, mainDimensions, dimensions] =
        await Promise.all([
          main.boundingBox(),
          scroller.boundingBox(),
          main.evaluate((element) => {
            const mainElement = element as unknown as {
              clientHeight: number;
              scrollHeight: number;
            };
            return {
              clientHeight: mainElement.clientHeight,
              scrollHeight: mainElement.scrollHeight,
            };
          }),
          scroller.evaluate((element) => {
            const scrollerElement = element as unknown as {
              clientHeight: number;
              clientWidth: number;
              scrollHeight: number;
              scrollWidth: number;
            };
            return {
              clientHeight: scrollerElement.clientHeight,
              clientWidth: scrollerElement.clientWidth,
              scrollHeight: scrollerElement.scrollHeight,
              scrollWidth: scrollerElement.scrollWidth,
            };
          }),
        ]);
      expect(mainBox).not.toBeNull();
      expect(scrollerBox).not.toBeNull();
      if (!mainBox || !scrollerBox) {
        throw new Error("Expected the Squad table to have a visible layout.");
      }
      expect(scrollerBox.height).toBeGreaterThan(100);
      expect(scrollerBox.y + scrollerBox.height).toBeLessThanOrEqual(
        mainBox.y + mainBox.height + 1,
      );
      expect(mainDimensions.scrollHeight).toBeLessThanOrEqual(
        mainDimensions.clientHeight + 1,
      );
      expect(dimensions.scrollWidth).toBeLessThanOrEqual(
        dimensions.clientWidth + 1,
      );
      expect(dimensions.scrollHeight).toBeGreaterThanOrEqual(
        dimensions.clientHeight + 1,
      );
      expect(
        await scroller.locator("tbody tr[data-index]").count(),
      ).toBeLessThan(101);
    }
  });

  test("configured Squad keeps a later-page retry visible over its scrollport", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
      squadPageFailure: true,
    });
    await page.goto("/my-club");

    const scroller = page.getByTestId("squad-overview-scroller");
    await expect(scroller).toBeVisible();
    await scroller.evaluate((element) => {
      const scrollElement = element as unknown as {
        dispatchEvent: (event: Event) => boolean;
        scrollHeight: number;
        scrollTop: number;
      };
      scrollElement.scrollTop = scrollElement.scrollHeight;
      scrollElement.dispatchEvent(new Event("scroll"));
    });

    const alert = page.getByRole("alert");
    await expect(alert).toBeVisible();
    const [scrollportBox, alertBox] = await Promise.all([
      scroller.boundingBox(),
      alert.boundingBox(),
    ]);
    expect(scrollportBox).not.toBeNull();
    expect(alertBox).not.toBeNull();
    if (!scrollportBox || !alertBox) {
      throw new Error("Expected the retry control to have a visible layout.");
    }
    expect(alertBox.y).toBeGreaterThanOrEqual(scrollportBox.y);
    expect(alertBox.y + alertBox.height).toBeLessThanOrEqual(
      scrollportBox.y + scrollportBox.height,
    );
  });

  test("Search keeps its table inside desktop viewports", async ({ page }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      playerTableRowCount: 101,
      squadOverview: true,
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await page.goto("/search");

      const main = page.getByRole("main");
      const scroller = main.getByTestId("search-results-scroller");
      const table = scroller.getByRole("table", {
        name: "Player search results",
      });
      await expect(scroller).toBeVisible();

      const [mainBox, scrollerBox, mainDimensions, dimensions] =
        await Promise.all([
          main.boundingBox(),
          scroller.boundingBox(),
          main.evaluate((element) => {
            const mainElement = element as unknown as {
              clientHeight: number;
              scrollHeight: number;
            };
            return {
              clientHeight: mainElement.clientHeight,
              scrollHeight: mainElement.scrollHeight,
            };
          }),
          scroller.evaluate((element) => {
            const scrollerElement = element as unknown as {
              clientHeight: number;
              clientWidth: number;
              scrollHeight: number;
              scrollWidth: number;
            };
            return {
              clientHeight: scrollerElement.clientHeight,
              clientWidth: scrollerElement.clientWidth,
              scrollHeight: scrollerElement.scrollHeight,
              scrollWidth: scrollerElement.scrollWidth,
            };
          }),
        ]);
      expect(mainBox).not.toBeNull();
      expect(scrollerBox).not.toBeNull();
      if (!mainBox || !scrollerBox) {
        throw new Error("Expected the Search table to have a visible layout.");
      }
      expect(scrollerBox.height).toBeGreaterThan(100);
      expect(scrollerBox.y + scrollerBox.height).toBeLessThanOrEqual(
        mainBox.y + mainBox.height + 1,
      );
      expect(mainDimensions.scrollHeight).toBeLessThanOrEqual(
        mainDimensions.clientHeight + 1,
      );
      expect(dimensions.scrollWidth).toBeLessThanOrEqual(
        dimensions.clientWidth + 1,
      );
      expect(dimensions.scrollHeight).toBeGreaterThanOrEqual(
        dimensions.clientHeight + 1,
      );
      const tableBox = await table.boundingBox();
      expect(tableBox).not.toBeNull();
      expect(tableBox?.width).toBeGreaterThanOrEqual(
        dimensions.clientWidth - 1,
      );
      expect(tableBox?.width).toBeLessThanOrEqual(dimensions.clientWidth + 1);
      expect(
        await scroller.locator("tbody tr[data-index]").count(),
      ).toBeLessThan(101);
    }
  });

  test("Search filter options remain fully interactive outside the modal scrollport", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/search");

    await page.getByRole("button", { name: "Edit filters" }).click();
    const dialog = page.getByRole("dialog", { name: "Edit filters" });
    await dialog.getByRole("button", { name: "Add filter" }).click();
    await dialog.getByRole("button", { name: "Field: CA" }).click();

    const listbox = dialog.getByRole("listbox", { name: "Field options" });
    const searchFields = dialog.getByRole("combobox", {
      name: "Search fields",
    });
    await expect(listbox).toBeVisible();
    expect(
      await listbox.evaluate((element) => {
        const listboxElement = element as unknown as {
          contains: (node: unknown) => boolean;
          getBoundingClientRect: () => {
            bottom: number;
            left: number;
            width: number;
          };
        };
        const browser = globalThis as unknown as {
          document: {
            elementFromPoint: (x: number, y: number) => unknown;
          };
          innerHeight: number;
        };
        const bounds = listboxElement.getBoundingClientRect();
        const hit = browser.document.elementFromPoint(
          bounds.left + Math.min(8, bounds.width / 2),
          Math.min(bounds.bottom - 4, browser.innerHeight - 4),
        );
        return (
          hit === element || (hit !== null && listboxElement.contains(hit))
        );
      }),
    ).toBe(true);
    await expect(searchFields).toBeFocused();
    await page.keyboard.type("club");
    await page.keyboard.press("Enter");
    await expect(
      dialog.getByRole("button", { name: "Field: Club" }),
    ).toBeVisible();
  });

  test("Search scrolls horizontally after columns reach their readable minimums", async ({
    page,
  }) => {
    const defaultColumns = [
      "name",
      "age",
      "nationality",
      "club",
      "division",
      "ca",
      "pa",
      "value",
    ];
    const searchColumns = [
      ...defaultColumns,
      "birth_year",
      "preferred_foot",
      "parent_club",
      "height",
      "wage",
      "contract_year",
      "transfer_listed",
      "loan_listed",
    ];
    await page.addInitScript(
      ({ defaultColumnIds, searchColumnIds }) => {
        const browser = globalThis as unknown as {
          localStorage: {
            setItem: (key: string, value: string) => void;
          };
        };
        browser.localStorage.setItem(
          "fm-valuescout-player-table-layouts",
          JSON.stringify({
            state: {
              layouts: {
                search: { columnIds: searchColumnIds, widths: {} },
                squad: { columnIds: defaultColumnIds, widths: {} },
              },
            },
            version: 1,
          }),
        );
      },
      { defaultColumnIds: defaultColumns, searchColumnIds: searchColumns },
    );
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/search");

    const scroller = page.getByTestId("search-results-scroller");
    await expect(
      scroller.getByRole("columnheader", { name: "Loan listed" }),
    ).toBeVisible();
    expect(
      await scroller.evaluate((element) => {
        const scrollerElement = element as unknown as {
          clientWidth: number;
          scrollWidth: number;
        };
        return scrollerElement.scrollWidth > scrollerElement.clientWidth;
      }),
    ).toBe(true);
  });

  test("Search persists a reordered resized column without changing Squad", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.goto("/search");

    const search = page.getByRole("main");
    const searchTable = search.getByRole("table", {
      name: "Player search results",
    });
    await searchTable
      .getByRole("columnheader", { name: "CA" })
      .click({ button: "right" });
    await page.getByRole("menuitem", { name: "Add column" }).click();
    await page.getByRole("button", { name: "Column: Choose a metric" }).click();
    await page
      .getByRole("combobox", { name: "Search columns" })
      .fill("acceleration");
    await page.getByRole("option", { name: "Acceleration" }).click();

    const acceleration = search.getByRole("columnheader", {
      name: "Acceleration",
    });
    await expect(acceleration).toBeVisible();
    const resizeAcceleration = search.getByRole("separator", {
      name: "Resize Acceleration column",
    });
    await resizeAcceleration.press("ArrowRight");
    await expect(resizeAcceleration).toHaveAttribute("aria-valuenow", "104");

    await acceleration.click({ button: "right" });
    await page.getByRole("menuitem", { name: "Add column" }).click();
    await page.getByRole("button", { name: "Column: Choose a metric" }).click();
    await page
      .getByRole("combobox", { name: "Search columns" })
      .fill("agility");
    await page.getByRole("option", { name: "Agility" }).click();

    const agility = search.getByRole("columnheader", { name: "Agility" });
    await expect(agility).toBeVisible();
    const scroller = search.getByTestId("search-results-scroller");
    await scroller.evaluate((element) => {
      const scrollport = element as unknown as {
        scrollLeft: number;
        scrollWidth: number;
      };
      scrollport.scrollLeft = scrollport.scrollWidth;
    });
    await expect(acceleration).toBeInViewport();
    await expect(agility).toBeInViewport();
    await acceleration.click({ button: "right" });
    await page.getByRole("menuitem", { name: "Move right" }).click();
    await expect
      .poll(async () =>
        searchTable.locator("thead th").evaluateAll((headers) =>
          headers.map((header) =>
            (
              header as unknown as {
                getAttribute: (name: string) => string | null;
              }
            ).getAttribute("aria-label"),
          ),
        ),
      )
      .toEqual([
        "Name",
        "Age / DOB",
        "Nationality",
        "CA",
        "PA",
        "Value",
        "Agility",
        "Acceleration",
      ]);

    await page.reload();
    await expect(
      search.getByRole("columnheader", { name: "Acceleration" }),
    ).toBeVisible();
    await expect
      .poll(async () =>
        searchTable.locator("thead th").evaluateAll((headers) =>
          headers.map((header) =>
            (
              header as unknown as {
                getAttribute: (name: string) => string | null;
              }
            ).getAttribute("aria-label"),
          ),
        ),
      )
      .toEqual([
        "Name",
        "Age / DOB",
        "Nationality",
        "CA",
        "PA",
        "Value",
        "Agility",
        "Acceleration",
      ]);
    await expect(
      search.getByRole("separator", { name: "Resize Acceleration column" }),
    ).toHaveAttribute("aria-valuenow", "104");

    await page.goto("/my-club");
    const squad = page.getByRole("main");
    await expect(
      squad
        .getByRole("table", { name: "Squad overview" })
        .getByRole("columnheader", { name: "Acceleration" }),
    ).toHaveCount(0);
  });

  test("Search dismisses a column menu with either pointer button outside", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.goto("/search");

    const caHeader = page
      .getByRole("table", { name: "Player search results" })
      .getByRole("columnheader", { name: "CA" });
    const menu = page.getByRole("menu", { name: "CA column actions" });
    const heading = page.getByRole("heading", {
      level: 1,
      name: "Player Search",
    });

    await caHeader.click({ button: "right" });
    await expect(menu).toBeVisible();
    await heading.click();
    await expect(menu).toHaveCount(0);

    await caHeader.click({ button: "right" });
    await expect(menu).toBeVisible();
    await heading.click({ button: "right" });
    await expect(menu).toHaveCount(0);
  });

  test("Moneyball Search and Squad expose Moneyball uploads while Squad keeps Youth Academy", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
      csvImportFormat: "moneyball",
    });
    await page.goto("/search?view=moneyball");

    const main = page.getByRole("main");
    await expect(
      main.getByRole("button", { name: "Upload Moneyball CSV" }),
    ).toBeVisible();

    await main.getByRole("button", { name: "Upload Moneyball CSV" }).click();
    const moneyballDialog = page.getByRole("dialog", {
      name: /(?:Upload|Replace) Moneyball CSV/,
    });
    await expect(moneyballDialog).toContainText(
      "Drop one CSV file here, or browse your files.",
    );
    await moneyballDialog.getByRole("button", { name: "Browse files" }).click();
    await expect(
      moneyballDialog.getByText(/Moneyball imported/i),
    ).toBeVisible();
    expect(await moneyballDialog.textContent()).not.toContain(
      "/tmp/smoke-import.csv",
    );
    await moneyballDialog.getByRole("button", { name: "Close" }).click();

    await page.goto("/my-club");
    await main.getByRole("button", { name: "Upload Squad CSV" }).click();
    const squadDialog = page.getByRole("dialog", {
      name: "Upload Moneyball CSV",
    });
    await expect(squadDialog).toContainText(
      "Only a Moneyball export can be imported",
    );
    await squadDialog.getByRole("button", { name: "Browse files" }).click();
    await expect(squadDialog.getByText(/Moneyball imported/i)).toBeVisible();
    expect(await squadDialog.textContent()).not.toContain(
      "/tmp/smoke-import.csv",
    );
    await squadDialog.getByRole("button", { name: "Close" }).click();

    await main
      .getByRole("button", { name: "Upload Youth Academy CSV" })
      .click();
    await expect(
      page.getByRole("dialog", { name: "Upload Youth Academy CSV" }),
    ).toContainText("Only a Youth Academy export can be imported");
  });

  test("Moneyball Search exposes role scores and restores its analysis state", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await stubTauriIpc(page, {
      moneyballSearch: true,
      plannerSnapshot: true,
      playerProfile: true,
    });
    await page.goto("/search?view=moneyball");

    const main = page.getByRole("main");
    const scroller = main.getByTestId("search-results-scroller");
    const table = scroller.getByRole("table", {
      name: "Player search results",
    });
    await expect(table.getByText("Moneyball player 001")).toBeVisible();
    await expect(
      table.getByRole("img", { name: /average rating: 50/i }),
    ).toBeVisible();
    expect(await scroller.locator("tbody tr[data-index]").count()).toBeLessThan(
      101,
    );

    const averageRatingHeader = table.getByRole("columnheader", {
      name: "Average Rating",
    });
    await averageRatingHeader.click({ button: "right" });
    const columnMenu = page.getByRole("menu", {
      name: "Average Rating column actions",
    });
    await columnMenu.getByRole("menuitem", { name: "Add column" }).click();
    const columnDialog = page.getByRole("dialog", { name: "Add a column" });
    await columnDialog
      .getByRole("button", { name: "Column: Choose a metric" })
      .click();
    const columnSearch = columnDialog.getByRole("combobox", {
      name: "Search columns",
    });
    await columnSearch.fill("wing-back");
    await expect(
      columnDialog.getByRole("option", {
        name: "Wing-Back (IP · WBR/WBL)",
        exact: true,
      }),
    ).toBeVisible();
    await columnSearch.press("ArrowDown");
    await columnSearch.press("ArrowDown");
    await columnSearch.press("ArrowDown");
    await columnSearch.press("Enter");
    const roleHeader = table.getByRole("columnheader", {
      name: "Wing-Back (IP · WBR/WBL)",
    });
    await expect(roleHeader).toBeVisible();
    await expect(
      table.getByRole("img", {
        name: /Moneyball role · Wing-Back \(IP · WBR\/WBL\): 0, Weak/,
      }),
    ).toBeVisible();

    await scroller.evaluate((element) => {
      (
        element as unknown as { scrollTo: (options: { top: number }) => void }
      ).scrollTo({ top: 1_950 });
    });
    await expect(table.getByText("Moneyball player 051")).toBeVisible();

    await main.getByRole("button", { name: "Full CSV" }).click();
    await expect(
      table.getByRole("img", { name: /average rating: 71/i }).first(),
    ).toBeVisible();

    await main.getByRole("button", { name: "Edit filters" }).click();
    const dialog = page.getByRole("dialog", { name: "Edit filters" });
    await dialog.getByRole("button", { name: "Add filter" }).click();
    await dialog.getByRole("button", { name: "Field: Average Rating" }).click();
    const fieldSearch = dialog.getByRole("combobox", {
      name: "Search fields",
    });
    await fieldSearch.fill("wing-back");
    await expect(
      dialog.getByRole("option", {
        name: "Wing-Back (IP · WBR/WBL)",
        exact: true,
      }),
    ).toBeVisible();
    await fieldSearch.press("ArrowDown");
    await fieldSearch.press("ArrowDown");
    await fieldSearch.press("ArrowDown");
    await fieldSearch.press("Enter");
    await expect(
      dialog.getByText(
        /role filters apply after the comparison cohort is calculated/i,
      ),
    ).toBeVisible();
    await dialog.getByLabel("Value").fill("70");
    await dialog.getByRole("button", { name: "Done" }).click();
    await expect(
      main.getByRole("button", { name: /Remove filter/i }),
    ).toBeVisible();
    await expect(table.getByText("Moneyball player 001")).not.toBeVisible();
    await expect(table.getByText("Moneyball player 027")).toBeVisible();

    await roleHeader
      .getByRole("button", {
        name: "Wing-Back (IP · WBR/WBL)",
      })
      .click();
    await expect(roleHeader).toHaveAttribute("aria-sort", "descending");
    await expect(table.getByText("Moneyball player 055")).toBeVisible();

    await table.getByText("Moneyball player 055").click();
    await expect(page).toHaveURL(/\/players\/55\?view=moneyball$/);
    await expect(
      page.getByRole("heading", { name: "Potential Scout" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Moneyball", exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("tab", { name: "Shooting", selected: true }),
    ).toBeVisible();
    const summary = page.getByRole("region", {
      name: "Potential Scout summary",
    });
    await expect(
      summary.getByRole("img", { name: "Moneyball IP: 86, Excellent" }),
    ).toBeVisible();
    await expect(
      summary.getByRole("img", { name: "Moneyball OOP: 64, Good" }),
    ).toBeVisible();
    const moneyballRoleFit = page.getByRole("region", {
      name: "Moneyball role fit for MC",
    });
    await expect(moneyballRoleFit).toBeVisible();
    await moneyballRoleFit
      .getByText("Central Midfielder", { exact: true })
      .click();
    await expect(
      moneyballRoleFit
        .getByText("Catalog v1 · natural-position comparison cohort.")
        .first(),
    ).toBeVisible();

    await page.goBack();
    await expect(page).toHaveURL(/\/search\?.*view=moneyball/);
    await expect(
      main.getByRole("button", { name: "Full CSV" }),
    ).toHaveAttribute("aria-pressed", "true");
    await expect(
      main.getByRole("columnheader", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    ).toBeVisible();
    await expect(
      main.getByRole("button", { name: /Remove filter/i }),
    ).toBeVisible();
  });

  test("Moneyball default applies to silent routes while explicit General stays in history", async ({
    page,
  }) => {
    await page.addInitScript({
      content: `
        window.localStorage.setItem(
          "fm-valuescout-moneyball-preferences",
          JSON.stringify({
            state: { defaultAnalysisView: "moneyball" },
            version: 1,
          }),
        );
      `,
    });
    await stubTauriIpc(page, {
      moneyballSearch: true,
      playerProfile: true,
    });

    await page.goto("/search");
    const nav = page.getByTestId("app-nav-bar");
    const searchLink = nav.getByRole("link", { name: "Search", exact: true });
    const moneyballLink = nav.getByRole("link", { name: "Moneyball" });
    await expect(moneyballLink).toHaveAttribute("aria-current", "page");

    await page.goto("/players/1");
    await expect(
      page.getByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeVisible();

    await page.goto("/search");
    await searchLink.click();
    await expect(page).toHaveURL(/\/search\?.*view=general/);
    await expect(searchLink).toHaveAttribute("aria-current", "page");

    await page.reload();
    await expect(searchLink).toHaveAttribute("aria-current", "page");

    await page.goBack();
    await expect(page).not.toHaveURL(/view=/);
    await expect(moneyballLink).toHaveAttribute("aria-current", "page");
  });

  test("configured Squad confirms and reports a closed CA boost", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    const caButton = main.getByRole("button", { name: "Boost all CA" });
    const wonderkidButton = main.getByRole("button", {
      name: "Make all Wonderkids",
    });
    const caBefore = await caButton.boundingBox();
    const wonderkidBefore = await wonderkidButton.boundingBox();
    if (!caBefore || !wonderkidBefore) {
      throw new Error("Expected Squad boost action bounds before submission.");
    }
    await caButton.click();
    const dialog = page.getByRole("dialog", { name: "Boost all CA?" });
    await expect(dialog).toContainText(
      "Players aged 20 or younger receive +5 CA.",
    );
    await expect(dialog).toContainText(
      "Players aged 21 through 28 receive +10 CA.",
    );
    await expect(dialog).toContainText("Players aged 29 or older are skipped.");
    await dialog.getByRole("button", { name: "Boost all CA" }).click();

    await expect(dialog).toContainText("0 of 2 players processed.");
    await expect(
      main.getByTestId("squad-boost-feedback").getByRole("status"),
    ).toContainText("2 processed — 2 updated, 0 skipped, 0 failed.");
    await expect(
      main.getByTestId("squad-boost-feedback").getByRole("status"),
    ).toHaveCount(1);
    const caAfter = await caButton.boundingBox();
    const wonderkidAfter = await wonderkidButton.boundingBox();
    expect(caAfter).toEqual(caBefore);
    expect(wonderkidAfter).toEqual(wonderkidBefore);
  });

  test("configured Squad confirms and reports a closed Wonderkid action", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await main.getByRole("button", { name: "Make all Wonderkids" }).click();
    const dialog = page.getByRole("dialog", { name: "Make all Wonderkids?" });
    await expect(dialog).toContainText(
      "Known Ambition, Professionalism, and Determination values at 10 or below can change.",
    );
    await expect(dialog).toContainText(
      "Unknown and higher values are unchanged.",
    );
    await dialog.getByRole("button", { name: "Make all Wonderkids" }).click();

    await expect(
      main.getByTestId("squad-boost-feedback").getByRole("status"),
    ).toContainText("2 processed — 2 updated, 0 skipped, 0 failed.");
  });

  test("planner tactic editor saves a linked phase adjustment", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Tactic", exact: true }).click();
    await expect(
      main.getByRole("region", { name: "Tactic controls" }),
    ).toBeVisible();
    const pitches = main.getByRole("group", { name: /pitch$/ });
    const ipPitch = pitches.first();
    await expect(
      ipPitch.getByRole("button", { name: "IP: AML · Winger" }),
    ).toBeVisible();
    const rightMc = ipPitch.getByRole("button", {
      name: "IP: MCR · Central Midfielder",
    });
    const leftMc = ipPitch.getByRole("button", {
      name: "IP: MCL · Central Midfielder",
    });
    const leftWinger = ipPitch.getByRole("button", {
      name: "IP: AML · Winger",
    });
    const rightWinger = ipPitch.getByRole("button", {
      name: "IP: AMR · Winger",
    });
    await expect(pitches).toHaveCount(1);
    // Both renders two phase-distinguished markers per lane on one canvas.
    await expect(ipPitch.locator("[data-pitch-marker]")).toHaveCount(22);
    await expect(
      ipPitch.locator('[data-pitch-marker="left_central_midfielder"]'),
    ).toHaveCount(2);
    const rightMcMarker = ipPitch.locator(
      '[data-pitch-marker="left_central_midfielder"][data-phase="ip"]',
    );
    await expect(rightMcMarker).toHaveAttribute("data-placement", "MCR");
    await expect(rightMc).toBeVisible();
    await expect(leftMc).toBeVisible();
    await rightMc.click();
    await main
      .getByRole("combobox", { name: "IP MCR position" })
      .selectOption("MC");
    await expect(
      main.getByRole("combobox", { name: "IP MC role" }),
    ).toHaveValue("central_midfielder_ip");
    await expect(rightMcMarker).toHaveAttribute("data-placement", "MC");
    await main
      .getByRole("combobox", { name: "IP MC position" })
      .selectOption("MCR");
    await expect(rightMcMarker).toHaveAttribute("data-placement", "MCR");
    const striker = ipPitch.getByRole("button", {
      name: "IP: STC · Centre Forward",
    });
    const goalkeeper = ipPitch.getByRole("button", {
      name: "IP: GK · Goalkeeper",
    });
    const [
      rightMcBox,
      leftMcBox,
      leftWingerBox,
      rightWingerBox,
      strikerBox,
      goalkeeperBox,
      bothPitchBox,
    ] = await Promise.all([
      rightMc.boundingBox(),
      leftMc.boundingBox(),
      leftWinger.boundingBox(),
      rightWinger.boundingBox(),
      striker.boundingBox(),
      goalkeeper.boundingBox(),
      pitches.first().boundingBox(),
    ]);
    if (
      !rightMcBox ||
      !leftMcBox ||
      !leftWingerBox ||
      !rightWingerBox ||
      !strikerBox ||
      !goalkeeperBox ||
      !bothPitchBox
    ) {
      throw new Error("Expected visible tactic cards and pitch geometry");
    }
    expect(rightMcBox.y).toBe(leftMcBox.y);
    expect(rightMcBox.width).toBeCloseTo(leftMcBox.width, 1);
    expect(rightMcBox.width).toBeCloseTo(leftWingerBox.width, 1);
    expect(rightMcBox.width).toBeCloseTo(rightWingerBox.width, 1);
    expect(rightMcBox.width).toBeLessThan(bothPitchBox.width * 0.15);
    expect(rightMcBox.width).toBeGreaterThanOrEqual(44);
    expect(rightMcBox.height).toBeGreaterThanOrEqual(44);
    expect(leftMcBox.x + leftMcBox.width).toBeLessThan(rightMcBox.x);
    expect(leftWingerBox.x + leftWingerBox.width).toBeLessThan(
      rightWingerBox.x,
    );
    for (const box of [
      rightMcBox,
      leftMcBox,
      leftWingerBox,
      rightWingerBox,
      strikerBox,
      goalkeeperBox,
    ]) {
      expect(box.x).toBeGreaterThanOrEqual(bothPitchBox.x);
      expect(box.x + box.width).toBeLessThanOrEqual(
        bothPitchBox.x + bothPitchBox.width + 1,
      );
    }
    const strikerCentre = strikerBox.y + strikerBox.height / 2;
    const midfieldCentre = rightMcBox.y + rightMcBox.height / 2;
    const goalkeeperCentre = goalkeeperBox.y + goalkeeperBox.height / 2;
    expect(strikerCentre).toBeLessThan(midfieldCentre);
    expect(midfieldCentre).toBeLessThan(goalkeeperCentre);
    // Both-mode draws a connector only where the canonical placement
    // changes: the two winger lanes connect, unchanged lanes do not.
    await expect(ipPitch.locator("[data-tactic-connector]")).toHaveCount(2);
    const wingerConnector = ipPitch.locator(
      '[data-tactic-connector="left_winger"]',
    );
    await expect(wingerConnector).toBeVisible();
    await expect(wingerConnector).toHaveAttribute("x1", "13");
    await expect(wingerConnector).toHaveAttribute("y1", "28");
    await expect(wingerConnector).toHaveAttribute("x2", "12");
    await expect(wingerConnector).toHaveAttribute("y2", "46");
    await expect(
      ipPitch.locator('[data-tactic-connector="goalkeeper"]'),
    ).toHaveCount(0);
    // The selected-slot transition is readable without hover: it follows
    // the existing marker selection for pointer and keyboard activation.
    await leftWinger.click();
    const selectedTransition = ipPitch.locator(
      "[data-selected-slot-transition]",
    );
    await expect(selectedTransition).toBeVisible();
    await expect(selectedTransition).toHaveText(
      "IP: AML · Winger OOP: ML · Tracking Wide Midfielder",
    );
    await expect(selectedTransition).not.toHaveClass(/sr-only/);
    await expect(selectedTransition).toHaveCSS("white-space", "pre-line");
    expect(await selectedTransition.textContent()).toContain("Winger\nOOP:");
    const transitionBox = await selectedTransition.boundingBox();
    if (
      !transitionBox ||
      transitionBox.width <= 0 ||
      transitionBox.height <= 0
    ) {
      throw new Error("Expected a visible selected-slot transition");
    }
    const transitionFontSize = await selectedTransition.evaluate((element) => {
      const scope = element as unknown as {
        ownerDocument: {
          defaultView: {
            getComputedStyle: (target: unknown) => { fontSize: string };
          } | null;
        };
      };
      return (
        scope.ownerDocument.defaultView?.getComputedStyle(element).fontSize ??
        ""
      );
    });
    expect(Number.parseFloat(transitionFontSize)).toBeGreaterThan(0);
    await rightWinger.focus();
    await page.keyboard.press("Enter");
    await expect(selectedTransition).toHaveText(
      "IP: AMR · Winger OOP: MR · Tracking Wide Midfielder",
    );
    await expect(selectedTransition).toBeVisible();
    // A cross-lane DCR/DCL swap shares coordinates across lanes: each
    // connector end must land inside its own displayed marker, not the
    // 4px gap between the split pair.
    await ipPitch
      .getByRole("button", { name: "IP: DCR · Centre-Back" })
      .click();
    await main
      .getByRole("combobox", { name: "IP DCR position" })
      .selectOption("DCL");
    await expect(ipPitch.locator("[data-tactic-connector]")).toHaveCount(4);
    const splitAttachment = await ipPitch.evaluate((pitch) => {
      const scope = pitch as unknown as {
        querySelector: (selector: string) => unknown;
      };
      const endpointFor = (laneId: string, end: "start" | "end") => {
        const line = scope.querySelector(
          `[data-tactic-connector="${laneId}"]`,
        ) as unknown as {
          getAttribute: (name: string) => string | null;
          ownerSVGElement: {
            getBoundingClientRect: () => {
              x: number;
              y: number;
              width: number;
              height: number;
            };
          } | null;
        } | null;
        // The overlay uses preserveAspectRatio="none" on a 0-100
        // viewBox, so endpoint percents map linearly onto the SVG box.
        const svg = line?.ownerSVGElement?.getBoundingClientRect();
        if (!line || !svg) {
          return null;
        }
        const at = end === "start" ? "1" : "2";
        return {
          x: svg.x + (Number(line.getAttribute(`x${at}`)) / 100) * svg.width,
          y: svg.y + (Number(line.getAttribute(`y${at}`)) / 100) * svg.height,
        };
      };
      const rectFor = (laneId: string, phase: string) => {
        const marker = scope.querySelector(
          `[data-pitch-marker="${laneId}"][data-phase="${phase}"]`,
        ) as unknown as {
          getBoundingClientRect: () => {
            x: number;
            y: number;
            width: number;
            height: number;
          };
        } | null;
        return marker?.getBoundingClientRect() ?? null;
      };
      const inside = (
        point: { x: number; y: number } | null,
        rect: {
          x: number;
          y: number;
          width: number;
          height: number;
        } | null,
      ) => {
        if (!point || !rect) {
          return false;
        }
        const tolerance = 0.5;
        return (
          point.x >= rect.x - tolerance &&
          point.x <= rect.x + rect.width + tolerance &&
          point.y >= rect.y - tolerance &&
          point.y <= rect.y + rect.height + tolerance
        );
      };
      return {
        leftStartInOwn: inside(
          endpointFor("left_centre_back", "start"),
          rectFor("left_centre_back", "ip"),
        ),
        leftEndInOwn: inside(
          endpointFor("left_centre_back", "end"),
          rectFor("left_centre_back", "oop"),
        ),
        rightStartInOwn: inside(
          endpointFor("right_centre_back", "start"),
          rectFor("right_centre_back", "ip"),
        ),
        rightEndInOwn: inside(
          endpointFor("right_centre_back", "end"),
          rectFor("right_centre_back", "oop"),
        ),
        leftEndInOther: inside(
          endpointFor("left_centre_back", "end"),
          rectFor("right_centre_back", "ip"),
        ),
        rightStartInOther: inside(
          endpointFor("right_centre_back", "start"),
          rectFor("left_centre_back", "oop"),
        ),
      };
    });
    expect(splitAttachment.leftStartInOwn).toBe(true);
    expect(splitAttachment.leftEndInOwn).toBe(true);
    expect(splitAttachment.rightStartInOwn).toBe(true);
    expect(splitAttachment.rightEndInOwn).toBe(true);
    expect(splitAttachment.leftEndInOther).toBe(false);
    expect(splitAttachment.rightStartInOther).toBe(false);
    await main
      .getByRole("combobox", { name: "IP DCL position" })
      .selectOption("DCR");
    await expect(ipPitch.locator("[data-tactic-connector]")).toHaveCount(2);
    // Form the supported unique MC triple (MCL/MC/MCR) through the edit flow.
    await ipPitch
      .getByRole("button", { name: "IP: DM · Defensive Midfielder" })
      .click();
    await main
      .getByRole("combobox", { name: "IP DM position" })
      .selectOption("MC");
    await main
      .getByRole("combobox", { name: "IP MC role" })
      .selectOption("central_midfielder_ip");
    const dmMarker = ipPitch.locator(
      '[data-pitch-marker="defensive_midfielder"][data-phase="ip"]',
    );
    await expect(dmMarker).toHaveAttribute("data-placement", "MC");
    // Same-band and vertical-neighbour markers share no pixels at the
    // supported desktop widths, covering the MC triple and the other
    // central families present in the layout.
    const pitchRectsDisjoint = (
      left: { x: number; y: number; width: number; height: number },
      right: { x: number; y: number; width: number; height: number },
    ) =>
      left.x + left.width <= right.x ||
      right.x + right.width <= left.x ||
      left.y + left.height <= right.y ||
      right.y + right.height <= left.y;
    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      const pitchBox = await pitches.first().boundingBox();
      const markerBoxes = await ipPitch
        .locator("[data-pitch-marker]")
        .evaluateAll((elements) =>
          elements.map((element) => {
            const rect = (
              element as unknown as {
                getBoundingClientRect: () => {
                  x: number;
                  y: number;
                  width: number;
                  height: number;
                };
              }
            ).getBoundingClientRect();
            return {
              x: rect.x,
              y: rect.y,
              width: rect.width,
              height: rect.height,
            };
          }),
        );
      if (!pitchBox) {
        throw new Error("Expected visible pitch geometry");
      }
      expect(markerBoxes).toHaveLength(22);
      for (const box of markerBoxes) {
        expect(box.width).toBeLessThan(pitchBox.width * 0.15);
        expect(box.width).toBeGreaterThanOrEqual(44);
        expect(box.height).toBeGreaterThanOrEqual(44);
      }
      for (let left = 0; left < markerBoxes.length; left += 1) {
        for (let right = left + 1; right < markerBoxes.length; right += 1) {
          expect(
            pitchRectsDisjoint(markerBoxes[left], markerBoxes[right]),
          ).toBe(true);
        }
      }
    }
    await page.setViewportSize({ width: 1280, height: 720 });
    await main.getByRole("button", { name: "IP", exact: true }).click();
    await expect(pitches).toHaveCount(1);
    await expect(
      pitches.first().locator("[data-tactic-connector]"),
    ).toHaveCount(0);
    const singlePitchBox = await pitches.first().boundingBox();
    if (!singlePitchBox) {
      throw new Error("Expected visible single-phase pitch geometry");
    }
    expect(singlePitchBox.width).toBeCloseTo(bothPitchBox.width, 1);
    await main.getByRole("button", { name: "Both", exact: true }).click();
    await expect(pitches).toHaveCount(1);
    await expect(
      pitches.first().locator('[data-tactic-connector="left_winger"]'),
    ).toBeVisible();
    await expect(pitches.first().locator("[data-pitch-marker]")).toHaveCount(
      22,
    );
    await expect(
      pitches.first().getByRole("button").first(),
    ).toHaveAccessibleName(/: STC · /);
    await expect(
      pitches.first().getByRole("button").last(),
    ).toHaveAccessibleName(/: GK · /);
    await expect(main.getByText("Left winger")).toHaveCount(0);
    await ipPitch
      .getByRole("button", { name: "IP: GK · Goalkeeper" })
      .press("Enter");

    const weight = main.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    await weight.press("ArrowRight");
    await expect(main.getByText("IP 51% / OOP 49%")).toBeVisible();
    await main
      .getByRole("combobox", { name: "Importance rank" })
      .selectOption("1");
    await main
      .getByRole("combobox", { name: "Preferred foot" })
      .selectOption("left");
    await main
      .getByRole("combobox", { name: "Foot preference" })
      .selectOption("strict");
    await main.getByRole("button", { name: "Save tactic" }).click();

    await expect(main.getByRole("status")).toHaveText("Tactic saved.");

    // The saved MC triple (MCL/MC/MCR) must also render without overlap in
    // the narrow Best role fit modal: marker buttons keep the 44px floor,
    // stay pairwise disjoint and contained, with no horizontal overflow.
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    await main.getByRole("button", { name: "Best role fit" }).click();
    const referenceDialog = page.getByRole("dialog", {
      name: "Best role fit reference",
    });
    await expect(referenceDialog).toBeVisible();
    const referencePitch = referenceDialog.getByRole("group", {
      name: /pitch$/,
    });
    await expect(referencePitch.locator("[data-pitch-marker]")).toHaveCount(11);
    await expect(referencePitch.locator('[data-placement="MC"]')).toHaveCount(
      1,
    );
    const referenceCanvasBox = await referencePitch
      .locator("div.relative")
      .first()
      .boundingBox();
    if (!referenceCanvasBox) {
      throw new Error("Expected visible modal pitch canvas");
    }
    // Adjacent MC-triple gaps report the exact overlap measurement.
    const tripleBoxes = (
      await Promise.all(
        [
          "IP: MCL · Central Midfielder",
          "IP: MC · Central Midfielder",
          "IP: MCR · Central Midfielder",
        ].map((name) =>
          referencePitch.getByRole("button", { name }).boundingBox(),
        ),
      )
    ).sort((left, right) => (left?.x ?? 0) - (right?.x ?? 0));
    if (tripleBoxes.some((box) => !box)) {
      throw new Error("Expected visible saved MC triple buttons");
    }
    const [tripleLeft, tripleMiddle, tripleRight] = tripleBoxes as [
      { x: number; y: number; width: number; height: number },
      { x: number; y: number; width: number; height: number },
      { x: number; y: number; width: number; height: number },
    ];
    expect(
      tripleMiddle.x - (tripleLeft.x + tripleLeft.width),
    ).toBeGreaterThanOrEqual(0);
    expect(
      tripleRight.x - (tripleMiddle.x + tripleMiddle.width),
    ).toBeGreaterThanOrEqual(0);
    // Central triple lanes sit 15% of the canvas apart, so the 44px marker
    // floor needs a canvas of at least 44 / 0.15 px to stay disjoint.
    expect(referenceCanvasBox.width).toBeGreaterThanOrEqual(44 / 0.15);
    const referenceButtonBoxes = await referencePitch
      .getByRole("button")
      .evaluateAll((elements) =>
        elements.map((element) => {
          const rect = (
            element as unknown as {
              getBoundingClientRect: () => {
                x: number;
                y: number;
                width: number;
                height: number;
              };
            }
          ).getBoundingClientRect();
          return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
          };
        }),
      );
    expect(referenceButtonBoxes).toHaveLength(11);
    const referencePitchBox = await referencePitch.boundingBox();
    if (!referencePitchBox) {
      throw new Error("Expected visible modal pitch geometry");
    }
    for (const box of referenceButtonBoxes) {
      expect(box.width).toBeGreaterThanOrEqual(44);
      expect(box.height).toBeGreaterThanOrEqual(44);
      expect(box.x).toBeGreaterThanOrEqual(referencePitchBox.x);
      expect(box.x + box.width).toBeLessThanOrEqual(
        referencePitchBox.x + referencePitchBox.width + 1,
      );
    }
    const modalRectsDisjoint = (
      left: { x: number; y: number; width: number; height: number },
      right: { x: number; y: number; width: number; height: number },
    ) =>
      left.x + left.width <= right.x ||
      right.x + right.width <= left.x ||
      left.y + left.height <= right.y ||
      right.y + right.height <= left.y;
    for (let left = 0; left < referenceButtonBoxes.length; left += 1) {
      for (
        let right = left + 1;
        right < referenceButtonBoxes.length;
        right += 1
      ) {
        expect(
          modalRectsDisjoint(
            referenceButtonBoxes[left],
            referenceButtonBoxes[right],
          ),
        ).toBe(true);
      }
    }
    const referenceOverflow = await referenceDialog.evaluate((element) => {
      const dialogElement = element as unknown as {
        clientWidth: number;
        scrollWidth: number;
      };
      return {
        clientWidth: dialogElement.clientWidth,
        scrollWidth: dialogElement.scrollWidth,
      };
    });
    expect(referenceOverflow.scrollWidth).toBeLessThanOrEqual(
      referenceOverflow.clientWidth + 1,
    );
    const plannerOverflow = await main.evaluate((element) => {
      const mainElement = element as unknown as {
        clientWidth: number;
        scrollWidth: number;
      };
      return {
        clientWidth: mainElement.clientWidth,
        scrollWidth: mainElement.scrollWidth,
      };
    });
    expect(plannerOverflow.scrollWidth).toBeLessThanOrEqual(
      plannerOverflow.clientWidth + 1,
    );
  });

  test("planner tactic workspace fits its supported desktop viewports", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club?view=tactic");

    const main = page.getByRole("main");
    const pitches = main.getByRole("group", { name: /pitch$/ });
    const settings = main.getByRole("region", {
      name: "Selected Slot",
    });
    const laneList = main.getByRole("region", {
      name: "Tactical XI",
    });
    const plannerHeading = main.getByRole("heading", {
      level: 1,
      name: "My Club",
    });
    await expect(
      page.getByRole("link", { name: "Tactic", exact: true }),
    ).toHaveAttribute("aria-current", "page");
    await expect(
      page.getByRole("tablist", { name: "My Club workspaces" }),
    ).toHaveCount(0);

    const expectWorkspaceFit = async (
      width: number,
      height: number,
      requireVerticalFit: boolean,
    ) => {
      await page.setViewportSize({ width, height });
      for (const [view, pitchCount, visibleRole] of [
        ["Both", 1, "OOP GK role"],
        ["IP", 1, "IP GK role"],
        ["OOP", 1, "OOP GK role"],
      ] as const) {
        await main.getByRole("button", { name: view, exact: true }).click();
        await expect(pitches).toHaveCount(pitchCount);
        await expect(settings).toBeVisible();
        await expect(
          settings.getByRole("combobox", { name: visibleRole }),
        ).toBeVisible();

        const [headingBox] = await Promise.all([plannerHeading.boundingBox()]);
        expect(headingBox).not.toBeNull();

        // The beside-pitch inspector wraps its controls instead of sharing
        // one bottom-shelf row: every combobox stays visible, inside the
        // inspector bounds, and pairwise disjoint, with phase/weight
        // association readable from the existing fieldset and slider labels.
        const inspectorBox = await settings.boundingBox();
        expect(inspectorBox).not.toBeNull();
        if (!inspectorBox) {
          throw new Error("Expected a visible inspector layout.");
        }
        const inspectorControls = await settings
          .getByRole("combobox")
          .evaluateAll((elements) =>
            elements.map((element) => {
              const rect = (
                element as unknown as {
                  getBoundingClientRect: () => {
                    x: number;
                    y: number;
                    width: number;
                    height: number;
                  };
                }
              ).getBoundingClientRect();
              return {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
              };
            }),
          );
        expect(inspectorControls.length).toBeGreaterThan(0);
        for (let index = 0; index < inspectorControls.length; index += 1) {
          await expect(settings.getByRole("combobox").nth(index)).toBeVisible();
          const controlBox = inspectorControls[index];
          expect(controlBox.x).toBeGreaterThanOrEqual(inspectorBox.x - 1);
          expect(controlBox.y).toBeGreaterThanOrEqual(inspectorBox.y - 1);
          expect(controlBox.x + controlBox.width).toBeLessThanOrEqual(
            inspectorBox.x + inspectorBox.width + 1,
          );
          expect(controlBox.y + controlBox.height).toBeLessThanOrEqual(
            inspectorBox.y + inspectorBox.height + 1,
          );
        }
        for (let left = 0; left < inspectorControls.length; left += 1) {
          for (
            let right = left + 1;
            right < inspectorControls.length;
            right += 1
          ) {
            const leftBox = inspectorControls[left];
            const rightBox = inspectorControls[right];
            const xOverlap =
              Math.min(leftBox.x + leftBox.width, rightBox.x + rightBox.width) -
              Math.max(leftBox.x, rightBox.x);
            const yOverlap =
              Math.min(
                leftBox.y + leftBox.height,
                rightBox.y + rightBox.height,
              ) - Math.max(leftBox.y, rightBox.y);
            expect(xOverlap <= 0 || yOverlap <= 0).toBe(true);
          }
        }
        await expect(
          settings.getByRole("group", { name: "In-Possession settings" }),
        ).toBeVisible();
        await expect(
          settings.getByRole("group", { name: "Out-of-Possession settings" }),
        ).toBeVisible();
        // The inspector fills the available sibling height with hairline-
        // separated sections instead of a compact auto-fit grid.
        for (const sectionName of [
          "Phase Influence",
          "General Settings",
          "In Possession (IP)",
          "Out of Possession (OOP)",
        ] as const) {
          await expect(
            settings.getByRole("heading", { name: sectionName }),
          ).toBeVisible();
        }
        const weightSlider = settings.getByRole("slider", {
          name: "IP/OOP score weight",
        });
        await expect(weightSlider).toBeVisible();
        // The weight slider spans the inspector width.
        const [sliderBox, sliderInspectorBox] = await Promise.all([
          weightSlider.boundingBox(),
          settings.boundingBox(),
        ]);
        expect(sliderBox).not.toBeNull();
        expect(sliderInspectorBox).not.toBeNull();
        if (!sliderBox || !sliderInspectorBox) {
          throw new Error("Expected a visible weight slider layout.");
        }
        expect(sliderBox.x).toBeGreaterThanOrEqual(sliderInspectorBox.x - 1);
        expect(sliderBox.x + sliderBox.width).toBeLessThanOrEqual(
          sliderInspectorBox.x + sliderInspectorBox.width + 1,
        );
        expect(sliderBox.width).toBeGreaterThanOrEqual(
          sliderInspectorBox.width - 32,
        );
        await expect(weightSlider).toHaveAttribute(
          "aria-valuetext",
          /IP \d+%, OOP \d+/,
        );

        const dimensions = await main.evaluate((element) => {
          const mainElement = element as unknown as {
            clientHeight: number;
            clientWidth: number;
            scrollHeight: number;
            scrollWidth: number;
          };
          return {
            clientHeight: mainElement.clientHeight,
            clientWidth: mainElement.clientWidth,
            scrollHeight: mainElement.scrollHeight,
            scrollWidth: mainElement.scrollWidth,
          };
        });
        expect(dimensions.scrollWidth).toBeLessThanOrEqual(
          dimensions.clientWidth + 1,
        );
        if (requireVerticalFit) {
          expect(dimensions.scrollHeight).toBeLessThanOrEqual(
            dimensions.clientHeight + 1,
          );
        }
        if (width >= 1600) {
          // The Selected Slot inspector sits beside the pitch (horizontally
          // adjacent with vertical overlap), not on a bottom shelf below it.
          // Wide visual order is Tactical XI left, pitch middle, inspector
          // right, and the inspector stretches to the row height.
          const [pitchBox, settingsBox, xiBox] = await Promise.all([
            pitches.first().boundingBox(),
            settings.boundingBox(),
            laneList.boundingBox(),
          ]);
          expect(pitchBox).not.toBeNull();
          expect(settingsBox).not.toBeNull();
          expect(xiBox).not.toBeNull();
          if (!pitchBox || !settingsBox || !xiBox) {
            throw new Error(
              "Expected the XI, pitch, and inspector to have a visible layout.",
            );
          }
          await expect(laneList).toBeVisible();
          expect(xiBox.x + xiBox.width).toBeLessThanOrEqual(pitchBox.x + 1);
          expect(settingsBox.x).toBeGreaterThanOrEqual(
            pitchBox.x + pitchBox.width - 1,
          );
          expect(settingsBox.y).toBeLessThanOrEqual(
            pitchBox.y + pitchBox.height - 1,
          );
          expect(pitchBox.y).toBeLessThanOrEqual(
            settingsBox.y + settingsBox.height - 1,
          );
          expect(xiBox.y).toBeLessThanOrEqual(pitchBox.y + pitchBox.height - 1);
          expect(pitchBox.y).toBeLessThanOrEqual(xiBox.y + xiBox.height - 1);
          // The inspector fills its stretched grid wrapper, and both grid
          // children share the row height: the Selected Slot uses the full
          // vertical region instead of collapsing to its content.
          const inspectorGeometry = await settings.evaluate((element) => {
            const node = element as unknown as {
              getBoundingClientRect: () => { height: number };
              parentElement: {
                getBoundingClientRect: () => { height: number };
                previousElementSibling: {
                  getBoundingClientRect: () => { height: number };
                } | null;
              } | null;
            };
            return {
              selfHeight: node.getBoundingClientRect().height,
              parentHeight:
                node.parentElement?.getBoundingClientRect().height ?? -1,
              siblingHeight:
                node.parentElement?.previousElementSibling?.getBoundingClientRect()
                  .height ?? -1,
            };
          });
          expect(
            Math.abs(
              inspectorGeometry.selfHeight - inspectorGeometry.parentHeight,
            ),
          ).toBeLessThanOrEqual(1);
          expect(
            Math.abs(
              inspectorGeometry.parentHeight - inspectorGeometry.siblingHeight,
            ),
          ).toBeLessThanOrEqual(1);
        }
      }
      if (width >= 3000) {
        // Ultrawide containment: the tactic workspace stays bounded and
        // centered instead of stretching pitch and side panels across the
        // full viewport width.
        expect(
          await page.evaluate(
            () => (globalThis as unknown as { innerWidth: number }).innerWidth,
          ),
        ).toBe(width);
        const [
          ultrawidePitchBox,
          ultrawideSettingsBox,
          ultrawideMainBox,
          ultrawideXiBox,
        ] = await Promise.all([
          pitches.first().boundingBox(),
          settings.boundingBox(),
          main.boundingBox(),
          laneList.boundingBox(),
        ]);
        expect(ultrawidePitchBox).not.toBeNull();
        expect(ultrawideSettingsBox).not.toBeNull();
        expect(ultrawideMainBox).not.toBeNull();
        expect(ultrawideXiBox).not.toBeNull();
        if (
          !ultrawidePitchBox ||
          !ultrawideSettingsBox ||
          !ultrawideMainBox ||
          !ultrawideXiBox
        ) {
          throw new Error(
            "Expected the ultrawide workspace to have a visible layout.",
          );
        }
        const spreadLeft = Math.min(
          ultrawidePitchBox.x,
          ultrawideSettingsBox.x,
          ultrawideXiBox.x,
        );
        const spreadRight = Math.max(
          ultrawidePitchBox.x + ultrawidePitchBox.width,
          ultrawideSettingsBox.x + ultrawideSettingsBox.width,
          ultrawideXiBox.x + ultrawideXiBox.width,
        );
        expect(spreadRight - spreadLeft).toBeLessThanOrEqual(2000);
        const leftMargin = spreadLeft - ultrawideMainBox.x;
        const rightMargin =
          ultrawideMainBox.x + ultrawideMainBox.width - spreadRight;
        expect(leftMargin).toBeGreaterThan(100);
        expect(rightMargin).toBeGreaterThan(100);
        expect(Math.abs(leftMargin - rightMargin)).toBeLessThanOrEqual(16);
      }
    };

    for (const [width, height, requireVerticalFit] of [
      [1280, 800, false],
      [1600, 900, true],
      [1920, 1080, true],
      [3440, 1440, true],
    ] as const) {
      await expectWorkspaceFit(width, height, requireVerticalFit);
    }
  });

  test("planner tactic pitch orients landscape at wide viewports", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });

    // Initial load at 2100 renders the landscape orientation immediately.
    await page.setViewportSize({ width: 2100, height: 1080 });
    await page.goto("/my-club?view=tactic");

    const main = page.getByRole("main");
    const pitches = main.getByRole("group", { name: /pitch$/ });
    const pitch = pitches.first();
    const striker = pitch.getByRole("button", {
      name: "IP: STC · Centre Forward",
    });
    const goalkeeper = pitch.getByRole("button", {
      name: "IP: GK · Goalkeeper",
    });
    await expect(striker).toBeVisible();
    await expect(pitch).toContainText("Attack toward the right");

    const centre = (box: {
      x: number;
      y: number;
      width: number;
      height: number;
    }) => ({
      x: box.x + box.width / 2,
      y: box.y + box.height / 2,
    });
    const landscapeBoxes = await Promise.all([
      striker.boundingBox(),
      goalkeeper.boundingBox(),
    ]);
    if (landscapeBoxes.some((box) => !box)) {
      throw new Error("Expected visible landscape striker and goalkeeper");
    }
    const [landscapeStriker, landscapeGoalkeeper] = landscapeBoxes as [
      { x: number; y: number; width: number; height: number },
      { x: number; y: number; width: number; height: number },
    ];
    // Portrait attack-up (striker above the goalkeeper) inverts to
    // landscape attack-right (striker right of the goalkeeper).
    expect(landscapeStriker.x).toBeGreaterThan(
      landscapeGoalkeeper.x + landscapeGoalkeeper.width,
    );
    const landscapeHorizontalGap =
      landscapeStriker.x - (landscapeGoalkeeper.x + landscapeGoalkeeper.width);
    const landscapeVerticalDrift = Math.abs(
      centre(landscapeStriker).y - centre(landscapeGoalkeeper).y,
    );
    expect(landscapeVerticalDrift).toBeLessThan(landscapeHorizontalGap);

    // Both-mode connectors project with the markers: the portrait AML/ML
    // geometry (x1 13, y1 28, x2 12, y2 46) becomes landscape geometry.
    const wingerConnector = pitch.locator(
      '[data-tactic-connector="left_winger"]',
    );
    await expect(wingerConnector).toBeVisible();
    await expect(wingerConnector).toHaveAttribute("x1", "72");
    await expect(wingerConnector).toHaveAttribute("y1", "13");
    await expect(wingerConnector).toHaveAttribute("x2", "54");
    await expect(wingerConnector).toHaveAttribute("y2", "12");

    // Marker text stays upright: no marker button — and no ancestor up
    // to and including the pitch — carries rotation. Both the CSS
    // individual `rotate` property and the rotation component of the
    // computed `transform` matrix count; pure translation (used for marker
    // placement) does not.
    type RotationProbeNode = {
      getAttribute: (name: string) => string | null;
      parentElement: RotationProbeNode | null;
    };
    const rotatedMarkers = await pitch.evaluate((element) => {
      const scope = element as unknown as {
        ownerDocument: {
          defaultView: {
            getComputedStyle: (target: unknown) => { transform: string };
          } | null;
        };
        querySelectorAll: (selector: string) => RotationProbeNode[];
      };
      const view = scope.ownerDocument.defaultView;
      const hasRotation = (target: unknown) => {
        if (!view) {
          return false;
        }
        const style = view.getComputedStyle(target);
        const rotate =
          (style as unknown as { rotate?: string }).rotate ?? "none";
        if (rotate !== "none") {
          return true;
        }
        const matrix = style.transform.match(/^matrix\((.+)\)$/);
        if (!matrix) {
          return false;
        }
        const [, b, c] = matrix[1].split(",").map(Number);
        return Math.abs(b ?? 0) > 1e-6 || Math.abs(c ?? 0) > 1e-6;
      };
      const offenders: string[] = [];
      for (const button of scope.querySelectorAll(
        "[data-pitch-marker] button",
      )) {
        let node: RotationProbeNode | null = button;
        while (node) {
          if (hasRotation(node)) {
            offenders.push(button.getAttribute("aria-label") ?? "marker");
            break;
          }
          if ((node as unknown) === (element as unknown)) {
            break;
          }
          node = node.parentElement;
        }
      }
      return offenders;
    });
    expect(rotatedMarkers).toEqual([]);

    // Keyboard order follows the current visual pitch order: focusing the
    // first marker button and pressing Tab through every marker button
    // visits the accessible names in the same top-to-bottom,
    // left-to-right order as their boxes. Reused at 2100 and 2099.
    const tabOrderMatchesVisual = async () => {
      const visualLabels = await pitch
        .getByRole("button")
        .evaluateAll((elements) =>
          elements
            .map((element) => {
              const node = element as unknown as {
                getAttribute: (name: string) => string | null;
                getBoundingClientRect: () => { x: number; y: number };
              };
              const rect = node.getBoundingClientRect();
              return {
                label: node.getAttribute("aria-label") ?? "",
                x: rect.x,
                y: rect.y,
              };
            })
            .sort((left, right) => left.y - right.y || left.x - right.x)
            .map((entry) => entry.label),
        );
      expect(visualLabels).toHaveLength(22);
      await pitch.getByRole("button").first().focus();
      const tabbedLabels: string[] = [];
      for (let index = 0; index < visualLabels.length; index += 1) {
        const focused = await pitch.evaluate((activeElement) => {
          const scope = activeElement as unknown as {
            ownerDocument: {
              activeElement: {
                getAttribute: (name: string) => string | null;
              } | null;
            };
          };
          return (
            scope.ownerDocument.activeElement?.getAttribute("aria-label") ?? ""
          );
        });
        tabbedLabels.push(focused);
        await page.keyboard.press("Tab");
      }
      expect(tabbedLabels).toEqual(visualLabels);
    };

    // Realistic edited layout at 2100: a cross-lane DCR/DCL swap (a shared
    // coordinate collides across lanes) plus the supported MCL/MC/MCR
    // triple, using the existing combobox flows from the portrait seam.
    await pitch.getByRole("button", { name: "IP: DCR · Centre-Back" }).click();
    await main
      .getByRole("combobox", { name: "IP DCR position" })
      .selectOption("DCL");
    await expect(pitch.locator("[data-tactic-connector]")).toHaveCount(4);
    await pitch
      .getByRole("button", { name: "IP: DM · Defensive Midfielder" })
      .click();
    await main
      .getByRole("combobox", { name: "IP DM position" })
      .selectOption("MC");
    await main
      .getByRole("combobox", { name: "IP MC role" })
      .selectOption("central_midfielder_ip");
    await expect(
      pitch.locator(
        '[data-pitch-marker="defensive_midfielder"][data-phase="ip"]',
      ),
    ).toHaveAttribute("data-placement", "MC");

    // Edited landscape markers keep the 44px target floor and stay disjoint,
    // covering the swap collision and the MC triple.
    const landscapeMarkerBoxes = await pitch
      .locator("[data-pitch-marker]")
      .evaluateAll((elements) =>
        elements.map((element) => {
          const rect = (
            element as unknown as {
              getBoundingClientRect: () => {
                x: number;
                y: number;
                width: number;
                height: number;
              };
            }
          ).getBoundingClientRect();
          return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
          };
        }),
      );
    expect(landscapeMarkerBoxes).toHaveLength(22);
    for (const box of landscapeMarkerBoxes) {
      expect(box.width).toBeGreaterThanOrEqual(44);
      expect(box.height).toBeGreaterThanOrEqual(44);
    }
    for (let left = 0; left < landscapeMarkerBoxes.length; left += 1) {
      for (
        let right = left + 1;
        right < landscapeMarkerBoxes.length;
        right += 1
      ) {
        const leftBox = landscapeMarkerBoxes[left];
        const rightBox = landscapeMarkerBoxes[right];
        expect(
          leftBox.x + leftBox.width <= rightBox.x ||
            rightBox.x + rightBox.width <= leftBox.x ||
            leftBox.y + leftBox.height <= rightBox.y ||
            rightBox.y + rightBox.height <= leftBox.y,
        ).toBe(true);
      }
    }
    // Every Both-mode connector endpoint lands inside its owning displayed
    // marker: the line start in the lane's IP marker, the line end in the
    // lane's OOP marker. The overlay maps its 0-100 viewBox linearly onto
    // the SVG box (preserveAspectRatio="none"), as in the portrait seam.
    const landscapeAttachment = await pitch.evaluate((element) => {
      const scope = element as unknown as {
        querySelector: (selector: string) => {
          getBoundingClientRect: () => {
            x: number;
            y: number;
            width: number;
            height: number;
          };
        } | null;
        querySelectorAll: (selector: string) => {
          getAttribute: (name: string) => string | null;
          ownerSVGElement: {
            getBoundingClientRect: () => {
              x: number;
              y: number;
              width: number;
              height: number;
            };
          } | null;
        }[];
      };
      const boxOf = (selector: string) => {
        const target = scope.querySelector(selector);
        if (!target) {
          return null;
        }
        const rect = target.getBoundingClientRect();
        return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
      };
      const inside = (
        point: { x: number; y: number } | null,
        rect: { x: number; y: number; width: number; height: number } | null,
      ) => {
        if (!point || !rect) {
          return false;
        }
        const tolerance = 0.5;
        return (
          point.x >= rect.x - tolerance &&
          point.x <= rect.x + rect.width + tolerance &&
          point.y >= rect.y - tolerance &&
          point.y <= rect.y + rect.height + tolerance
        );
      };
      return Array.from(scope.querySelectorAll("[data-tactic-connector]")).map(
        (line) => {
          const laneId = line.getAttribute("data-tactic-connector") ?? "";
          const svg = line.ownerSVGElement?.getBoundingClientRect() ?? null;
          const point = (at: "1" | "2") => {
            if (!svg) {
              return null;
            }
            return {
              x:
                svg.x + (Number(line.getAttribute(`x${at}`)) / 100) * svg.width,
              y:
                svg.y +
                (Number(line.getAttribute(`y${at}`)) / 100) * svg.height,
            };
          };
          return {
            lane: laneId,
            startInOwn: inside(
              point("1"),
              boxOf(`[data-pitch-marker="${laneId}"][data-phase="ip"]`),
            ),
            endInOwn: inside(
              point("2"),
              boxOf(`[data-pitch-marker="${laneId}"][data-phase="oop"]`),
            ),
          };
        },
      );
    });
    // The edited layout draws five connectors: the winger pair, the swap
    // pair, and the triple lane whose IP moved DM → MC.
    expect(landscapeAttachment).toHaveLength(5);
    for (const entry of landscapeAttachment) {
      expect(entry.startInOwn).toBe(true);
      expect(entry.endInOwn).toBe(true);
    }

    // Tab order matches the edited visual order at 2100.
    await tabOrderMatchesVisual();

    // Phase identity rides categorical accents, not text colour: IP and OOP
    // markers carry distinct border colours while their labels share one
    // on-surface colour, and OOP keeps its dashed edge.
    const phaseIdentity = await pitch.evaluate((element) => {
      const scope = element as unknown as {
        ownerDocument: {
          defaultView: {
            getComputedStyle: (target: unknown) => {
              borderColor: string;
              borderStyle: string;
              color: string;
            };
          } | null;
        };
        querySelector: (selector: string) => unknown | null;
      };
      const styleOf = (selector: string) => {
        const target = scope.querySelector(selector);
        if (!target || !scope.ownerDocument.defaultView) {
          return null;
        }
        const style = scope.ownerDocument.defaultView.getComputedStyle(target);
        return {
          borderColor: style.borderColor,
          borderStyle: style.borderStyle,
          color: style.color,
        };
      };
      return {
        ipButton: styleOf(
          '[data-pitch-marker="goalkeeper"][data-phase="ip"] button',
        ),
        oopButton: styleOf(
          '[data-pitch-marker="goalkeeper"][data-phase="oop"] button',
        ),
        ipBadge: styleOf(
          '[data-pitch-marker="goalkeeper"][data-phase="ip"] span[aria-hidden="true"]',
        ),
        oopBadge: styleOf(
          '[data-pitch-marker="goalkeeper"][data-phase="oop"] span[aria-hidden="true"]',
        ),
      };
    });
    expect(phaseIdentity.ipButton).not.toBeNull();
    expect(phaseIdentity.oopButton).not.toBeNull();
    expect(phaseIdentity.ipButton?.borderColor).not.toBe(
      phaseIdentity.oopButton?.borderColor,
    );
    expect(phaseIdentity.ipButton?.color).toBe(phaseIdentity.oopButton?.color);
    expect(phaseIdentity.oopButton?.borderStyle).toBe("dashed");
    expect(phaseIdentity.ipBadge?.borderColor).not.toBe(
      phaseIdentity.oopBadge?.borderColor,
    );

    // Selecting a marker takes the gold treatment but keeps its phase
    // badge edge, so phase identity stays visible when selected.
    await pitch.getByRole("button", { name: "IP: GK · Goalkeeper" }).click();
    const selectedIdentity = await pitch.evaluate((element) => {
      const scope = element as unknown as {
        ownerDocument: {
          defaultView: {
            getComputedStyle: (target: unknown) => {
              borderColor: string;
            };
          } | null;
        };
        querySelector: (selector: string) => unknown | null;
      };
      const borderOf = (selector: string) => {
        const target = scope.querySelector(selector);
        if (!target || !scope.ownerDocument.defaultView) {
          return null;
        }
        return scope.ownerDocument.defaultView.getComputedStyle(target)
          .borderColor;
      };
      return {
        selectedBadge: borderOf(
          '[data-pitch-marker="goalkeeper"][data-phase="ip"] span[aria-hidden="true"]',
        ),
        oopBadge: borderOf(
          '[data-pitch-marker="goalkeeper"][data-phase="oop"] span[aria-hidden="true"]',
        ),
      };
    });
    expect(selectedIdentity.selectedBadge).toBe(
      phaseIdentity.ipBadge?.borderColor,
    );
    expect(selectedIdentity.selectedBadge).not.toBe(selectedIdentity.oopBadge);

    // Categorical borders must clear 3:1 against both adjacent surfaces.
    // Every input below is a live computed style; translucent borders are
    // composited over the real adjacent background before the WCAG ratio.
    const phaseContrast = await pitch.evaluate((element) => {
      const scope = element as unknown as {
        ownerDocument: {
          createElement: (tag: string) => {
            width: number;
            height: number;
            getContext: (kind: string) => {
              fillStyle: string;
              clearRect: (x: number, y: number, w: number, h: number) => void;
              fillRect: (x: number, y: number, w: number, h: number) => void;
              getImageData: (
                x: number,
                y: number,
                w: number,
                h: number,
              ) => { data: ArrayLike<number> };
            } | null;
          };
          defaultView: {
            getComputedStyle: (target: unknown) => {
              [key: string]: string;
            };
          } | null;
        };
        querySelector: (selector: string) => unknown | null;
      };
      // Read back one filled pixel: the browser resolves oklch, color-mix,
      // and rgba syntaxes to the same sRGB bytes the screen composites.
      const probe = scope.ownerDocument.createElement("canvas");
      probe.width = 1;
      probe.height = 1;
      const toRgba = (css: string) => {
        const context = probe.getContext("2d");
        if (!context) {
          throw new Error("Expected a 2d canvas context");
        }
        context.clearRect(0, 0, 1, 1);
        context.fillStyle = css;
        context.fillRect(0, 0, 1, 1);
        const [r, g, b, a] = Array.from(context.getImageData(0, 0, 1, 1).data);
        return { r, g, b, a: (a ?? 255) / 255 };
      };
      const luminance = (color: { r: number; g: number; b: number }) => {
        const channel = (value: number) => {
          const s = value / 255;
          return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
        };
        return (
          0.2126 * channel(color.r) +
          0.7152 * channel(color.g) +
          0.0722 * channel(color.b)
        );
      };
      const composite = (
        foreground: { r: number; g: number; b: number; a: number },
        background: { r: number; g: number; b: number },
      ) => ({
        r: foreground.r * foreground.a + background.r * (1 - foreground.a),
        g: foreground.g * foreground.a + background.g * (1 - foreground.a),
        b: foreground.b * foreground.a + background.b * (1 - foreground.a),
      });
      const ratio = (
        first: { r: number; g: number; b: number },
        second: { r: number; g: number; b: number },
      ) => {
        const high = Math.max(luminance(first), luminance(second));
        const low = Math.min(luminance(first), luminance(second));
        return (high + 0.05) / (low + 0.05);
      };
      const styleOf = (selector: string, property: string) => {
        const target = scope.querySelector(selector);
        if (!target || !scope.ownerDocument.defaultView) {
          throw new Error(`Missing ${selector}`);
        }
        return scope.ownerDocument.defaultView.getComputedStyle(target)[
          property
        ];
      };
      const canvasBg = toRgba(styleOf("div.relative", "backgroundColor"));
      // The goalkeeper IP marker is selected (gold container); the
      // left-winger pair is untouched, covering both states.
      const report = (lane: string, phase: string) => {
        const base = `[data-pitch-marker="${lane}"][data-phase="${phase}"]`;
        const border = toRgba(styleOf(`${base} button`, "borderTopColor"));
        const buttonBg = toRgba(styleOf(`${base} button`, "backgroundColor"));
        const badgeBorder = toRgba(
          styleOf(`${base} span[aria-hidden="true"]`, "borderTopColor"),
        );
        const badgeBg = toRgba(
          styleOf(`${base} span[aria-hidden="true"]`, "backgroundColor"),
        );
        return {
          buttonOuter: ratio(composite(border, canvasBg), canvasBg),
          buttonInner: ratio(composite(border, buttonBg), buttonBg),
          badgeOuter: ratio(composite(badgeBorder, buttonBg), buttonBg),
          badgeInner: ratio(composite(badgeBorder, badgeBg), badgeBg),
        };
      };
      return {
        selectedIp: report("goalkeeper", "ip"),
        plainIp: report("left_winger", "ip"),
        plainOop: report("left_winger", "oop"),
      };
    });
    for (const [state, ratios] of Object.entries(phaseContrast)) {
      for (const [pair, value] of Object.entries(ratios)) {
        expect(`${state} ${pair}`).not.toBe("");
        expect(value).toBeGreaterThanOrEqual(3);
      }
    }

    // Live crossing: the orientation follows actual viewport bounds.
    await page.setViewportSize({ width: 2099, height: 1080 });
    await expect(pitch).toContainText("Attack toward the top");
    await expect(wingerConnector).toHaveAttribute("x1", "13");
    await page.setViewportSize({ width: 2100, height: 1080 });
    await expect(pitch).toContainText("Attack toward the right");
    await expect(wingerConnector).toHaveAttribute("x1", "72");
    await page.setViewportSize({ width: 2099, height: 1080 });
    await expect(pitch).toContainText("Attack toward the top");
    const portraitBoxes = await Promise.all([
      striker.boundingBox(),
      goalkeeper.boundingBox(),
    ]);
    if (portraitBoxes.some((box) => !box)) {
      throw new Error("Expected visible portrait striker and goalkeeper");
    }
    const [portraitStriker, portraitGoalkeeper] = portraitBoxes as [
      { x: number; y: number; width: number; height: number },
      { x: number; y: number; width: number; height: number },
    ];
    expect(centre(portraitStriker).y).toBeLessThan(
      centre(portraitGoalkeeper).y,
    );
    const portraitVerticalGap =
      centre(portraitGoalkeeper).y - centre(portraitStriker).y;
    const portraitHorizontalDrift = Math.abs(
      centre(portraitStriker).x - centre(portraitGoalkeeper).x,
    );
    expect(portraitVerticalGap).toBeGreaterThan(portraitHorizontalDrift);

    // Tab order matches the edited visual order in portrait as well.
    await tabOrderMatchesVisual();

    // Explicit 1920x1080 portrait: below the 2100 breakpoint the workspace
    // keeps attack-up geometry on a visibly vertical canvas, not unrotated
    // positions on a wide horizontal rectangle.
    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(pitch).toContainText("Attack toward the top");
    const workspaceCanvasBox = await pitch
      .locator("div.relative")
      .first()
      .boundingBox();
    expect(workspaceCanvasBox).not.toBeNull();
    if (!workspaceCanvasBox) {
      throw new Error("Expected visible workspace pitch canvas");
    }
    expect(workspaceCanvasBox.height).toBeGreaterThan(workspaceCanvasBox.width);
    const explicitBoxes = await Promise.all([
      striker.boundingBox(),
      goalkeeper.boundingBox(),
    ]);
    if (explicitBoxes.some((box) => !box)) {
      throw new Error("Expected visible portrait striker and goalkeeper");
    }
    const [explicitStriker, explicitGoalkeeper] = explicitBoxes as [
      { x: number; y: number; width: number; height: number },
      { x: number; y: number; width: number; height: number },
    ];
    expect(centre(explicitStriker).y).toBeLessThan(
      centre(explicitGoalkeeper).y,
    );
    const explicitVerticalGap =
      centre(explicitGoalkeeper).y - centre(explicitStriker).y;
    const explicitHorizontalDrift = Math.abs(
      centre(explicitStriker).x - centre(explicitGoalkeeper).x,
    );
    expect(explicitVerticalGap).toBeGreaterThan(explicitHorizontalDrift);

    // The role-reference modal stays portrait at wide viewports.
    await page.setViewportSize({ width: 2100, height: 1080 });
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    await main.getByRole("button", { name: "Best role fit" }).click();
    const dialog = page.getByRole("dialog", {
      name: "Best role fit reference",
    });
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Attack toward the top");
    await expect(dialog).not.toContainText("Attack toward the right");
    await dialog.getByRole("button", { name: "Close" }).click();
  });

  test("planner depth adds strings for Senior, Reserves, and Youth", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 844, height: 800 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    await main.getByRole("button", { name: "Manage teams" }).click();
    const management = page.getByRole("dialog", {
      name: "Manage squad teams",
    });
    await management
      .getByRole("button", { name: "Add string to Senior" })
      .click();
    await management
      .getByRole("button", { name: "Add string to Reserves" })
      .click();
    await management
      .getByRole("button", { name: "Add string to Youth" })
      .click();
    await management.getByRole("button", { name: "Save teams" }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    const board = main.getByRole("region", { name: "Squad depth board" });
    await expect(board).toBeVisible();
    // Every enabled squad renders at once: no tabs remain.
    await expect(main.getByRole("tab", { name: "Senior" })).toHaveCount(0);
    for (const squad of ["Senior", "Reserves", "Youth"]) {
      await expect(
        board.getByRole("columnheader", { name: squad }),
      ).toBeVisible();
    }
    await expect(
      board.getByRole("columnheader", { name: "2nd string" }),
    ).toHaveCount(3);
  });

  test("planner opens the best role fit reference modal", async ({ page }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club?view=planner");

    const main = page.getByRole("main");
    const trigger = main.getByRole("button", { name: "Best role fit" });
    await trigger.click();

    const dialog = page.getByRole("dialog", {
      name: "Best role fit reference",
    });
    await expect(dialog).toBeVisible();
    const dialogPitch = dialog.getByRole("group", { name: /pitch$/ });
    await expect(dialogPitch.locator("[data-pitch-marker]")).toHaveCount(11);
    await expect(dialogPitch.getByText(/attack/i)).toBeVisible();
    const dialogPitchBox = await dialogPitch.boundingBox();
    const dialogMarkerBoxes = await dialogPitch
      .locator("[data-pitch-marker]")
      .evaluateAll((elements) =>
        elements.map((element) => {
          const rect = (
            element as unknown as {
              getBoundingClientRect: () => {
                x: number;
                y: number;
                width: number;
                height: number;
              };
            }
          ).getBoundingClientRect();
          return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
          };
        }),
      );
    if (!dialogPitchBox) {
      throw new Error("Expected visible modal pitch geometry");
    }
    for (const box of dialogMarkerBoxes) {
      // The narrow modal pitch keeps the 44px target floor instead of the
      // proportional width, so markers stay usable without overlapping.
      expect(box.width).toBeGreaterThanOrEqual(44);
      expect(box.height).toBeGreaterThanOrEqual(44);
      expect(box.x).toBeGreaterThanOrEqual(dialogPitchBox.x);
      expect(box.x + box.width).toBeLessThanOrEqual(
        dialogPitchBox.x + dialogPitchBox.width + 1,
      );
    }
    for (let left = 0; left < dialogMarkerBoxes.length; left += 1) {
      for (let right = left + 1; right < dialogMarkerBoxes.length; right += 1) {
        const leftBox = dialogMarkerBoxes[left];
        const rightBox = dialogMarkerBoxes[right];
        expect(
          leftBox.x + leftBox.width <= rightBox.x ||
            rightBox.x + rightBox.width <= leftBox.x ||
            leftBox.y + leftBox.height <= rightBox.y ||
            rightBox.y + rightBox.height <= leftBox.y,
        ).toBe(true);
      }
    }
    await expect(
      dialog.getByRole("radio", { name: "In Possession" }),
    ).toBeChecked();
    await expect(dialog.getByRole("radio", { name: "Current" })).toBeChecked();
    await expect(
      dialog.getByRole("table", {
        name: "Players best suited to GK Goalkeeper",
      }),
    ).toContainText("Potential Keeper");

    await dialog.getByRole("button", { name: "IP: DL · Full-Back" }).click();
    const leftBackTable = dialog.getByRole("table", {
      name: "Players best suited to DL Full-Back",
    });
    await expect(leftBackTable).toContainText("Potential Full-Back");
    const currentHeader = leftBackTable.getByRole("columnheader", {
      name: "Current",
    });
    await currentHeader.getByRole("button").click();
    await expect(currentHeader).toHaveAttribute("aria-sort", "ascending");

    await dialog.getByRole("radio", { name: "Out of Possession" }).click();
    await dialog.getByRole("radio", { name: "Potential" }).click();
    await expect(
      dialog.getByRole("radio", { name: "Potential" }),
    ).toBeChecked();
    await dialog.getByRole("button", { name: "Close" }).click();
    await expect(dialog).toBeHidden();
    await expect(trigger).toBeFocused();
  });

  test("planner team management renames, removes, and restores a populated team", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 600, height: 800 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    await main.getByRole("button", { name: "Optimize squads" }).click();
    await expect(main.getByRole("status")).toHaveText(
      "Squads optimized by current scores.",
    );

    await main.getByRole("button", { name: "Manage teams" }).click();
    const management = page.getByRole("dialog", {
      name: "Manage squad teams",
    });
    await management.getByLabel("Senior display name").fill("First Team");
    await management.getByRole("checkbox", { name: "Reserves" }).uncheck();
    await management.getByRole("button", { name: "Save teams" }).click();

    const removal = page.getByRole("dialog", {
      name: "Remove planner teams?",
    });
    await expect(removal).toContainText("Reserves: 1 assignment");
    await removal.getByRole("button", { name: "Remove teams" }).click();
    const removedBoard = main.getByRole("region", {
      name: "Squad depth board",
    });
    await expect(
      removedBoard.getByRole("columnheader", { name: "First Team" }),
    ).toBeVisible();
    await expect(
      removedBoard.getByRole("columnheader", { name: "Youth" }),
    ).toBeVisible();
    await expect(
      removedBoard.getByRole("columnheader", { name: "Reserves" }),
    ).toHaveCount(0);

    await main.getByRole("button", { name: "Manage teams" }).click();
    const restoration = page.getByRole("dialog", {
      name: "Manage squad teams",
    });
    await restoration.getByRole("checkbox", { name: "Reserves" }).check();
    await restoration.getByLabel("Reserves display name").fill("B Team");
    await restoration.getByRole("button", { name: "Save teams" }).click();
    await expect(
      main
        .getByRole("region", { name: "Squad depth board" })
        .getByRole("columnheader", { name: "B Team" }),
    ).toBeVisible();

    await main.getByRole("button", { name: "Manage teams" }).click();
    const strings = page.getByRole("dialog", {
      name: "Manage squad teams",
    });
    await strings.getByLabel("Senior string 1 name").fill("First Choice");
    await strings.getByRole("button", { name: "Add string to Senior" }).click();
    await strings.getByLabel("Senior string 2 name").fill("Second Line");
    await strings
      .getByRole("button", { name: "Move Senior string 2 up" })
      .click();
    await expect(strings.getByLabel("Senior string 1 name")).toHaveValue(
      "Second Line",
    );
    await strings.getByRole("button", { name: "Save teams" }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await page.setViewportSize({ width: 1920, height: 900 });
    const board = main.getByRole("region", { name: "Squad depth board" });
    await expect(
      board.getByRole("columnheader", { name: "First Team" }),
    ).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "B Team" }),
    ).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "Youth" }),
    ).toBeVisible();
    await expect(main.getByRole("tab", { name: "First Team" })).toHaveCount(0);
  });

  test("planner depth optimizes current and potential squads within desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await page.goto("/my-club");

      const main = page.getByRole("main");
      await page.getByRole("link", { name: "Planner", exact: true }).click();
      const controls = main.getByRole("group", { name: "Squad controls" });
      const current = main.getByRole("button", { name: "Optimize squads" });
      const potential = main.getByRole("button", {
        name: "Optimize by potential",
      });
      await expect(controls).toBeVisible();
      await current.click();
      await expect(main.getByRole("status")).toHaveText(
        "Squads optimized by current scores.",
      );
      await potential.click();
      await expect(main.getByRole("status")).toHaveText(
        "Squads optimized by potential.",
      );
      await expect(
        main.getByRole("button", {
          name: /Reserves, 1st string, IP: GK .* Optimized Keeper, Resolved/,
        }),
      ).toBeVisible();

      const [controlsBox, potentialBox] = await Promise.all([
        controls.boundingBox(),
        potential.boundingBox(),
      ]);
      expect(controlsBox).not.toBeNull();
      expect(potentialBox).not.toBeNull();
      if (!controlsBox || !potentialBox) {
        throw new Error("Expected visible Planner optimization controls.");
      }
      expect(potentialBox.x + potentialBox.width).toBeLessThanOrEqual(
        controlsBox.x + controlsBox.width,
      );
      expect(controlsBox.x + controlsBox.width).toBeLessThanOrEqual(width);
      expect(
        await page
          .locator("html")
          .evaluate(
            (element) =>
              (element as unknown as { scrollWidth: number }).scrollWidth,
          ),
      ).toBeLessThanOrEqual(width);
    }
  });

  test("planner depth clears every squad from one confirmed action", async ({
    page,
  }) => {
    // 844 keeps the 900px content geometry from before the 56px rail retired,
    // so the depth matrix stays in tabbed mode.
    await page.setViewportSize({ width: 844, height: 800 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    await main.getByRole("button", { name: "Optimize squads" }).click();
    await expect(main.getByRole("status")).toHaveText(
      "Squads optimized by current scores.",
    );

    const clearAll = main.getByRole("button", { name: "Clear all" });
    await clearAll.click();
    const confirmation = page.getByRole("dialog", {
      name: "Clear all squads?",
    });
    await expect(confirmation).toContainText("Senior, Reserves, and Youth");
    await confirmation.getByRole("button", { name: "Clear all" }).click();
    await expect(main.getByRole("status")).toHaveText("All squads cleared.");

    await expect(
      main.getByRole("button", {
        name: /Reserves, 1st string, IP: GK .* Empty/,
      }),
    ).toBeVisible();
  });

  test("planner board renders every squad simultaneously", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 900 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    const board = main.getByRole("region", {
      name: "Squad depth board",
    });
    await expect(board).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "Senior" }),
    ).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "Reserves" }),
    ).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "Youth" }),
    ).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "1st string" }),
    ).toHaveCount(3);
    await expect(main.getByRole("tab", { name: "Senior" })).toHaveCount(0);
  });

  test("planner board shows all tactical slots without vertical scrolling at 1920x1080", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await stubTauriIpc(page, {
      plannerPotentialScores: true,
      plannerSnapshot: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    const board = main.getByRole("region", { name: "Squad depth board" });
    await expect(board).toBeVisible();
    await expect(board.getByRole("rowheader")).toHaveCount(11);

    const verticalBounds = await board.evaluate((element) => {
      const boardElement = element as unknown as {
        clientHeight: number;
        scrollHeight: number;
      };
      return {
        clientHeight: boardElement.clientHeight,
        scrollHeight: boardElement.scrollHeight,
      };
    });
    expect(verticalBounds.scrollHeight).toBeLessThanOrEqual(
      verticalBounds.clientHeight + 1,
    );
  });

  test("planner board rounds its edge after sparse fixed-width strings", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    const board = main.getByRole("region", { name: "Squad depth board" });
    const table = board.getByRole("table", { name: "Squad depth board" });
    await expect(board).toBeVisible();

    const [boardBox, tableBox] = await Promise.all([
      board.boundingBox(),
      table.boundingBox(),
    ]);
    expect(boardBox).not.toBeNull();
    expect(tableBox).not.toBeNull();
    if (!boardBox || !tableBox) {
      throw new Error("Expected visible sparse board geometry.");
    }
    expect(
      Math.abs(boardBox.x + boardBox.width - (tableBox.x + tableBox.width)),
    ).toBeLessThanOrEqual(2);
  });

  test("planner board bounds overflow with a sticky slot band from 1280 to 3440", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await stubTauriIpc(page, {
      plannerPotentialScores: true,
      plannerSnapshot: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    // Three strings per squad overflows the 1280 board: 9 fixed string
    // columns plus the slot band exceed the available content width.
    await main.getByRole("button", { name: "Manage teams" }).click();
    const management = page.getByRole("dialog", {
      name: "Manage squad teams",
    });
    for (const squad of ["Senior", "Reserves", "Youth"]) {
      await management
        .getByRole("button", { name: `Add string to ${squad}` })
        .click();
      await management
        .getByRole("button", { name: `Add string to ${squad}` })
        .click();
    }
    await management.getByRole("button", { name: "Save teams" }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);

    const board = main.getByRole("region", { name: "Squad depth board" });
    await expect(board).toBeVisible();
    await expect(
      board.getByRole("columnheader", { name: "3rd string" }),
    ).toHaveCount(3);

    const boardOverflow = await board.evaluate((element) => {
      const boardElement = element as unknown as {
        clientWidth: number;
        scrollWidth: number;
      };
      return {
        clientWidth: boardElement.clientWidth,
        scrollWidth: boardElement.scrollWidth,
      };
    });
    // The board owns horizontal overflow at 1280.
    expect(boardOverflow.scrollWidth).toBeGreaterThan(
      boardOverflow.clientWidth,
    );
    // No page-level horizontal overflow at 1280x800.
    expect(
      await page
        .locator("html")
        .evaluate(
          (element) =>
            (element as unknown as { scrollWidth: number }).scrollWidth,
        ),
    ).toBeLessThanOrEqual(1280);

    const cardWidthAt1280 = await board
      .getByRole("columnheader", { name: "1st string" })
      .first()
      .evaluate(
        (element) =>
          (
            element as unknown as {
              getBoundingClientRect: () => { width: number };
            }
          ).getBoundingClientRect().width,
      );

    // Measure actual card buttons, not just the fixed string headers: one
    // assigned card and one empty Assign card share the fixed column width.
    const assignedCard = board
      .getByRole("button", { name: /Potential Keeper/ })
      .first();
    const emptyCard = board.getByRole("button", { name: /, Empty$/ }).first();
    await expect(assignedCard).toBeVisible();
    await expect(emptyCard).toBeVisible();
    const assignedCardWidthAt1280 = await assignedCard.evaluate(
      (element) =>
        (
          element as unknown as {
            getBoundingClientRect: () => { width: number };
          }
        ).getBoundingClientRect().width,
    );
    const emptyCardWidthAt1280 = await emptyCard.evaluate(
      (element) =>
        (
          element as unknown as {
            getBoundingClientRect: () => { width: number };
          }
        ).getBoundingClientRect().width,
    );
    // Both cards fill the fixed column despite different content lengths, so
    // a fit-content regression would split them apart.
    expect(
      Math.abs(assignedCardWidthAt1280 - emptyCardWidthAt1280),
    ).toBeLessThanOrEqual(2);
    expect(assignedCardWidthAt1280).toBeGreaterThanOrEqual(
      cardWidthAt1280 - 48,
    );
    expect(assignedCardWidthAt1280).toBeLessThanOrEqual(cardWidthAt1280 + 1);
    expect(emptyCardWidthAt1280).toBeGreaterThanOrEqual(cardWidthAt1280 - 48);
    expect(emptyCardWidthAt1280).toBeLessThanOrEqual(cardWidthAt1280 + 1);

    // The sticky slot band stays visible after board-level horizontal scroll.
    const slotHeader = board.getByRole("rowheader").first();
    await expect(slotHeader).toBeVisible();
    await board.evaluate((element) => {
      (element as unknown as { scrollLeft: number }).scrollLeft = 400;
    });
    const [boardBox, slotBox] = await Promise.all([
      board.boundingBox(),
      slotHeader.boundingBox(),
    ]);
    expect(boardBox).not.toBeNull();
    expect(slotBox).not.toBeNull();
    if (!boardBox || !slotBox) {
      throw new Error("Expected visible board geometry after scrolling.");
    }
    expect(slotBox.x).toBeGreaterThanOrEqual(boardBox.x - 1);
    expect(slotBox.x).toBeLessThanOrEqual(boardBox.x + 1);

    const visibleAt1280 = await board
      .getByRole("columnheader", { name: /string/ })
      .evaluateAll(
        (elements, boardBox) => {
          const box = boardBox as unknown as {
            x: number;
            width: number;
          };
          return elements.filter((element) => {
            const rect = (
              element as unknown as {
                getBoundingClientRect: () => {
                  left: number;
                  right: number;
                };
              }
            ).getBoundingClientRect();
            return (
              rect.left >= box.x - 1 && rect.right <= box.x + box.width + 1
            );
          }).length;
        },
        await board.boundingBox(),
      );

    await page.setViewportSize({ width: 3440, height: 1440 });
    const cardWidthAt3440 = await board
      .getByRole("columnheader", { name: "1st string" })
      .first()
      .evaluate(
        (element) =>
          (
            element as unknown as {
              getBoundingClientRect: () => { width: number };
            }
          ).getBoundingClientRect().width,
      );
    // Fixed card widths do not grow with the viewport.
    expect(cardWidthAt3440).toBeGreaterThanOrEqual(cardWidthAt1280 - 1);
    expect(cardWidthAt3440).toBeLessThanOrEqual(cardWidthAt1280 + 1);

    // Actual card buttons keep their bounded widths at ultrawide.
    const assignedCardWidthAt3440 = await assignedCard.evaluate(
      (element) =>
        (
          element as unknown as {
            getBoundingClientRect: () => { width: number };
          }
        ).getBoundingClientRect().width,
    );
    const emptyCardWidthAt3440 = await emptyCard.evaluate(
      (element) =>
        (
          element as unknown as {
            getBoundingClientRect: () => { width: number };
          }
        ).getBoundingClientRect().width,
    );
    expect(assignedCardWidthAt3440).toBeGreaterThanOrEqual(
      assignedCardWidthAt1280 - 1,
    );
    expect(assignedCardWidthAt3440).toBeLessThanOrEqual(
      assignedCardWidthAt1280 + 1,
    );
    expect(emptyCardWidthAt3440).toBeGreaterThanOrEqual(
      emptyCardWidthAt1280 - 1,
    );
    expect(emptyCardWidthAt3440).toBeLessThanOrEqual(emptyCardWidthAt1280 + 1);
    expect(assignedCardWidthAt3440).toBeLessThanOrEqual(cardWidthAt3440 + 1);
    expect(emptyCardWidthAt3440).toBeLessThanOrEqual(cardWidthAt3440 + 1);

    const visibleAt3440 = await board
      .getByRole("columnheader", { name: /string/ })
      .evaluateAll(
        (elements, boardBox) => {
          const box = boardBox as unknown as {
            x: number;
            width: number;
          };
          return elements.filter((element) => {
            const rect = (
              element as unknown as {
                getBoundingClientRect: () => {
                  left: number;
                  right: number;
                };
              }
            ).getBoundingClientRect();
            return (
              rect.left >= box.x - 1 && rect.right <= box.x + box.width + 1
            );
          }).length;
        },
        await board.boundingBox(),
      );
    // Ultrawide reveals more fixed-width columns.
    expect(visibleAt3440).toBeGreaterThan(visibleAt1280);
    expect(
      await page
        .locator("html")
        .evaluate(
          (element) =>
            (element as unknown as { scrollWidth: number }).scrollWidth,
        ),
    ).toBeLessThanOrEqual(3440);
  });

  test("planner depth keeps assigned current and potential scores readable at desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      plannerPotentialScores: true,
    });
    await page.goto("/my-club");

    const main = page.getByRole("main");
    await page.getByRole("link", { name: "Planner", exact: true }).click();
    const scoreCell = main.getByRole("button", {
      name: /Senior, 1st string, IP: GK .* Potential Keeper, Resolved, current score 82, potential score 91/,
    });
    const currentScore = scoreCell.getByRole("img", {
      name: /Current combined role score: 82/,
    });
    const potentialScore = scoreCell.getByRole("img", {
      name: /Potential combined role score: 91/,
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(scoreCell).toContainText("Potential Keeper");
      await expect(currentScore).toBeVisible();
      await expect(potentialScore).toBeVisible();

      const [cellBox, currentBox, potentialBox] = await Promise.all([
        scoreCell.boundingBox(),
        currentScore.boundingBox(),
        potentialScore.boundingBox(),
      ]);
      expect(cellBox).not.toBeNull();
      expect(currentBox).not.toBeNull();
      expect(potentialBox).not.toBeNull();
      if (!cellBox || !currentBox || !potentialBox) {
        throw new Error(
          "Expected assigned score content to have layout bounds.",
        );
      }
      expect(currentBox.x).toBeGreaterThanOrEqual(cellBox.x);
      expect(potentialBox.x + potentialBox.width).toBeLessThanOrEqual(
        cellBox.x + cellBox.width,
      );
    }
  });

  test("player profile route shows no-snapshot empty state from stubbed IPC", async ({
    page,
  }) => {
    await page.goto("/players/42");

    const main = page.getByRole("main");
    await expect(main.getByText("No data loaded for this save")).toBeVisible();
  });

  test("player profile keeps its scouting workspace inside desktop viewports", async ({
    page,
  }) => {
    await stubTauriIpc(page, { playerProfile: true });
    await page.goto("/players/42?tab=technical");

    const main = page.getByRole("main");
    const summary = main.getByRole("region", {
      name: "Potential Scout summary",
    });
    const attributes = main
      .getByRole("heading", { name: "Attributes" })
      .locator("..")
      .locator("..");
    const roleFitPanel = main
      .getByRole("heading", { name: "Role fit" })
      .locator("..")
      .locator("..");
    const roleFit = main.getByRole("region", { name: "Role fit for MC" });
    const currentIp = summary.getByRole("img", {
      name: "Current IP: 82, Excellent",
    });
    const currentOop = summary.getByRole("img", {
      name: "Current OOP: 60, Average",
    });
    const potentialIp = summary.getByRole("img", {
      name: "Potential IP: 94, Excellent",
    });
    const potentialOop = summary.getByRole("img", {
      name: "Potential OOP: 77, Good",
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(summary).toBeVisible();
      await expect(attributes).toBeVisible();
      await expect(roleFit).toBeVisible();
      await expect(
        main.getByRole("button", { name: "DL, familiarity 1" }),
      ).toBeVisible();
      await expect(
        main.getByRole("button", { name: "SW, familiarity 18" }),
      ).toHaveCount(0);
      await expect(currentIp).toBeVisible();
      await expect(currentOop).toBeVisible();
      await expect(potentialIp).toBeVisible();
      await expect(potentialOop).toBeVisible();

      const [mainBox, attributesBox, roleFitBox] = await Promise.all([
        main.boundingBox(),
        attributes.boundingBox(),
        roleFitPanel.boundingBox(),
      ]);
      expect(mainBox).not.toBeNull();
      expect(attributesBox).not.toBeNull();
      expect(roleFitBox).not.toBeNull();
      if (!mainBox || !attributesBox || !roleFitBox) {
        throw new Error(
          "Expected the complete player workspace to be visible.",
        );
      }
      expect(attributesBox.y).toBe(roleFitBox.y);
      expect(roleFitBox.y + roleFitBox.height).toBeLessThanOrEqual(
        mainBox.y + mainBox.height,
      );

      const summaryBadgeBoxes = await Promise.all(
        [currentIp, currentOop, potentialIp, potentialOop].map((badge) =>
          badge.boundingBox(),
        ),
      );
      expect(summaryBadgeBoxes.every((box) => box !== null)).toBe(true);
      const [currentIpBox, currentOopBox, potentialIpBox, potentialOopBox] =
        summaryBadgeBoxes;
      if (
        !currentIpBox ||
        !currentOopBox ||
        !potentialIpBox ||
        !potentialOopBox
      ) {
        throw new Error("Expected visible best-role summary badges.");
      }
      expect(currentIpBox.x + currentIpBox.width).toBeLessThanOrEqual(
        currentOopBox.x,
      );
      expect(potentialIpBox.x + potentialIpBox.width).toBeLessThanOrEqual(
        potentialOopBox.x,
      );

      const detailLabels = await Promise.all(
        ["Age / DOB", "Nationality", "Height", "Foot"].map((label) =>
          summary.getByText(label, { exact: true }).boundingBox(),
        ),
      );
      expect(detailLabels.every((box) => box !== null)).toBe(true);
      const detailRowY = detailLabels[0]?.y;
      if (detailRowY === undefined) {
        throw new Error("Expected a visible player-summary detail row.");
      }
      for (const labelBox of detailLabels) {
        expect(
          Math.abs((labelBox?.y ?? detailRowY) - detailRowY),
        ).toBeLessThanOrEqual(1);
      }
      for (const label of [
        "Current IP",
        "Current OOP",
        "Potential IP",
        "Potential OOP",
      ]) {
        await expect(summary.getByText(label, { exact: true })).toBeVisible();
      }

      const [
        boostBox,
        wonderkidBox,
        hiddenInformationBox,
        caLabelBox,
        abilityRowBox,
      ] = await Promise.all([
        summary.getByRole("button", { name: "Boost CA" }).boundingBox(),
        summary
          .getByRole("button", { name: "Wonderkid Mentality" })
          .boundingBox(),
        summary
          .getByRole("button", { name: "Reveal hidden information" })
          .boundingBox(),
        summary.getByText("CA", { exact: true }).boundingBox(),
        summary
          .getByText("Value", { exact: true })
          .locator("..")
          .locator("..")
          .boundingBox(),
      ]);
      expect(boostBox).not.toBeNull();
      expect(wonderkidBox).not.toBeNull();
      expect(hiddenInformationBox).not.toBeNull();
      expect(caLabelBox).not.toBeNull();
      expect(abilityRowBox).not.toBeNull();
      if (
        !boostBox ||
        !wonderkidBox ||
        !hiddenInformationBox ||
        !caLabelBox ||
        !abilityRowBox
      ) {
        throw new Error("Expected visible player-development actions.");
      }
      expect(Math.abs(wonderkidBox.y - boostBox.y)).toBeLessThanOrEqual(1);
      expect(Math.abs(hiddenInformationBox.y - boostBox.y)).toBeLessThanOrEqual(
        1,
      );
      expect(boostBox.y + boostBox.height).toBeLessThanOrEqual(caLabelBox.y);
      expect(caLabelBox.y - (boostBox.y + boostBox.height)).toBeLessThanOrEqual(
        24,
      );
      expect(
        Math.abs(
          hiddenInformationBox.x +
            hiddenInformationBox.width -
            (abilityRowBox.x + abilityRowBox.width),
        ),
      ).toBeLessThanOrEqual(1);

      if (width === 1280) {
        await page
          .getByTestId("app-header")
          .getByRole("button", { name: "Load Data" })
          .click();
        await expect(
          page.getByText("Loaded 0 players into the database."),
        ).toBeVisible();
        const mainDimensions = await main.evaluate((element) => {
          const htmlElement = element as unknown as {
            clientHeight: number;
            scrollHeight: number;
          };
          return {
            clientHeight: htmlElement.clientHeight,
            scrollHeight: htmlElement.scrollHeight,
          };
        });
        expect(mainDimensions.scrollHeight).toBeLessThanOrEqual(
          mainDimensions.clientHeight + 1,
        );
        const [bannerMainBox, bannerRoleFitBox] = await Promise.all([
          main.boundingBox(),
          roleFitPanel.boundingBox(),
        ]);
        expect(bannerMainBox).not.toBeNull();
        expect(bannerRoleFitBox).not.toBeNull();
        if (!bannerMainBox || !bannerRoleFitBox) {
          throw new Error("Expected the profile workspace below the banner.");
        }
        expect(
          bannerRoleFitBox.y + bannerRoleFitBox.height,
        ).toBeLessThanOrEqual(bannerMainBox.y + bannerMainBox.height);
      }
    }

    const currentHeader = roleFit.getByRole("columnheader", {
      name: "Current",
    });
    const potentialHeader = roleFit.getByRole("columnheader", {
      name: "Potential",
    });
    await potentialHeader.getByRole("button").click();
    await expect(potentialHeader).toHaveAttribute("aria-sort", "descending");
    await expect(
      roleFit.getByRole("row").nth(1).getByText("Advanced Playmaker"),
    ).toBeVisible();
    await currentHeader.getByRole("button").click();
    await expect(currentHeader).toHaveAttribute("aria-sort", "descending");

    await main.getByRole("button", { name: "ST, familiarity 15" }).click();
    await expect(
      main
        .getByRole("region", { name: "Role fit for ST" })
        .getByText("Potential Specialist"),
    ).toBeVisible();
  });

  test("player profile confirms Wonderkid Mentality at desktop size", async ({
    page,
  }) => {
    await stubTauriIpc(page, { playerProfile: true });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/players/42");

    const main = page.getByRole("main");
    const action = main.getByRole("button", { name: "Wonderkid Mentality" });
    await expect(action).toBeVisible();
    await action.focus();
    const tooltip = main
      .getByRole("tooltip")
      .filter({ hasText: "Ambition 10 → random 11–20" });
    await expect(tooltip).toBeVisible();
    const [actionBox, tooltipBox] = await Promise.all([
      action.boundingBox(),
      tooltip.boundingBox(),
    ]);
    expect(actionBox).not.toBeNull();
    expect(tooltipBox).not.toBeNull();
    if (!actionBox || !tooltipBox) {
      throw new Error("Expected the development tooltip below its action.");
    }
    expect(tooltipBox.y).toBeGreaterThanOrEqual(actionBox.y + actionBox.height);
    expect(tooltipBox.y + tooltipBox.height).toBeLessThanOrEqual(800);
    await action.click();

    const dialog = page.getByRole("dialog");
    await expect(
      dialog.getByRole("heading", { name: "Apply Wonderkid Mentality?" }),
    ).toBeVisible();
    await expect(
      dialog.getByText(
        "FM assigns each eligible value a random number from 11 to 20.",
      ),
    ).toBeVisible();
    await dialog
      .getByRole("button", { name: "Apply Wonderkid Mentality" })
      .click();

    await expect(main.getByRole("status")).toContainText(
      "Wonderkid Mentality updated Ambition from 10 to 20, Determination from 8 to 18.",
    );
    await expect(action).toBeDisabled();
  });

  test("player profile hides sensitive information across profiles and restores it", async ({
    page,
  }) => {
    await stubTauriIpc(page, { playerProfile: true });
    await page.goto("/players/42");

    const main = page.getByRole("main");
    const summary = main.getByRole("region", {
      name: "Potential Scout summary",
    });
    const toggle = summary.getByRole("button", {
      name: "Reveal hidden information",
    });
    const revealedToggleBox = await toggle.boundingBox();
    expect(revealedToggleBox).not.toBeNull();
    await toggle.focus();
    await page.keyboard.press("Enter");

    const concealedToggle = summary.getByRole("button", {
      name: "Reveal hidden information",
    });
    await expect(concealedToggle).toHaveAttribute("aria-pressed", "false");
    const concealedToggleBox = await concealedToggle.boundingBox();
    expect(concealedToggleBox).not.toBeNull();
    if (!revealedToggleBox || !concealedToggleBox) {
      throw new Error("Expected the hidden-information toggle in both states.");
    }
    expect(concealedToggleBox.y).toBe(revealedToggleBox.y);
    await expect(summary.getByText("PA", { exact: true })).toHaveCount(0);
    await expect(summary.getByText("160", { exact: true })).toHaveCount(0);
    await expect(
      summary.getByRole("img", { name: "Potential IP: concealed" }),
    ).toBeVisible();
    await expect(
      summary.getByRole("img", { name: "Potential OOP: concealed" }),
    ).toBeVisible();
    await expect(main.getByRole("button", { name: "Boost CA" })).toHaveCount(0);

    await page.goto("/players/99");
    const otherSummary = main.getByRole("region", {
      name: "Other Scout summary",
    });
    const otherToggle = otherSummary.getByRole("button", {
      name: "Reveal hidden information",
    });
    await expect(otherToggle).toHaveAttribute("aria-pressed", "false");

    await otherToggle.click();
    await expect(otherToggle).toHaveAttribute("aria-pressed", "true");
    await expect(otherSummary.getByText("PA", { exact: true })).toBeVisible();
  });

  test("player profile Attributes keeps visible potential pairs within desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, { playerProfile: true });
    await page.goto("/players/42?tab=technical");

    const technical = page.getByRole("region", { name: "Technical" });
    const mental = page.getByRole("region", { name: "Mental" });
    const physical = page.getByRole("region", { name: "Physical" });
    const roleFit = page.getByRole("region", { name: "Role fit for MC" });
    const passing = technical.locator("dd", {
      hasText: "Current 14, Potential 16",
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(passing).toContainText("14→16");
      await expect(passing.locator('[data-tier="3"]')).toHaveCount(1);
      await expect(passing.locator('[data-tier="4"]')).toHaveCount(1);

      const [technicalBox, passingBox] = await Promise.all([
        technical.boundingBox(),
        passing.boundingBox(),
      ]);
      expect(technicalBox).not.toBeNull();
      expect(passingBox).not.toBeNull();
      if (!technicalBox || !passingBox) {
        throw new Error("Expected visible projected Passing attribute pair.");
      }
      expect(passingBox.x + passingBox.width).toBeLessThanOrEqual(
        technicalBox.x + technicalBox.width,
      );

      for (const [region, label] of [
        [technical, "Passing"],
        [mental, "Off The Ball"],
        [physical, "Natural Fitness"],
      ] as const) {
        const regionBox = await region.boundingBox();
        const labelBox = await region
          .getByText(label, { exact: true })
          .boundingBox();
        expect(regionBox).not.toBeNull();
        expect(labelBox).not.toBeNull();
        if (!regionBox || !labelBox) {
          throw new Error(`Expected readable ${label} attribute label.`);
        }
        expect(labelBox.width).toBeGreaterThan(0);
        expect(labelBox.x + labelBox.width).toBeLessThanOrEqual(
          regionBox.x + regionBox.width,
        );
        const dimensions = await region.evaluate((element) => {
          const htmlElement = element as unknown as {
            clientWidth: number;
            scrollWidth: number;
          };
          return {
            clientWidth: htmlElement.clientWidth,
            scrollWidth: htmlElement.scrollWidth,
          };
        });
        expect(dimensions.scrollWidth).toBeLessThanOrEqual(
          dimensions.clientWidth,
        );
        expect(
          await region.getByText(label, { exact: true }).evaluate((element) => {
            const htmlElement = element as unknown as {
              clientWidth: number;
              scrollWidth: number;
            };
            return htmlElement.scrollWidth <= htmlElement.clientWidth;
          }),
        ).toBe(true);
      }
    }

    await page.setViewportSize({ width: 1280, height: 800 });
    const roleFitDimensions = await roleFit.evaluate((element) => {
      const htmlElement = element as unknown as {
        clientWidth: number;
        scrollWidth: number;
      };
      return {
        clientWidth: htmlElement.clientWidth,
        scrollWidth: htmlElement.scrollWidth,
      };
    });
    expect(roleFitDimensions.scrollWidth).toBeLessThanOrEqual(
      roleFitDimensions.clientWidth,
    );
    for (const [region, label] of [
      [technical, "Passing"],
      [mental, "Off The Ball"],
      [physical, "Natural Fitness"],
    ] as const) {
      const labelBox = await region
        .getByText(label, { exact: true })
        .boundingBox();
      expect(labelBox).not.toBeNull();
      if (!labelBox) {
        throw new Error(`Expected readable ${label} attribute label.`);
      }
      expect(
        await region.getByText(label, { exact: true }).evaluate((element) => {
          const htmlElement = element as unknown as {
            clientWidth: number;
            scrollWidth: number;
          };
          return htmlElement.scrollWidth <= htmlElement.clientWidth;
        }),
      ).toBe(true);
    }
  });

  test("top bar exposes global player search", async ({ page }) => {
    await page.goto("/");

    const header = page.getByTestId("app-header");
    await expect(
      header.getByRole("combobox", { name: "Search players" }),
    ).toBeVisible();
  });

  test("Search shows General and Moneyball views in navigation with legacy shortlist normalization", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/search");

    const nav = page.getByTestId("app-nav-bar");
    const searchLink = nav.getByRole("link", { name: "Search", exact: true });
    const moneyballLink = nav.getByRole("link", { name: "Moneyball" });

    await expect(searchLink).toBeVisible();
    await expect(moneyballLink).toBeVisible();
    await expect(searchLink).toHaveAttribute("aria-current", "page");
    await expect(
      page.getByRole("tablist", { name: "Search view" }),
    ).toHaveCount(0);

    await moneyballLink.click();
    await expect(page).toHaveURL(/\/search\?.*view=moneyball/);
    await expect(moneyballLink).toHaveAttribute("aria-current", "page");
    await expect(
      page.getByRole("tablist", { name: "Search view" }),
    ).toHaveCount(0);

    await searchLink.click();
    await expect(page).toHaveURL(/\/search\?.*view=general/);
    await expect(searchLink).toHaveAttribute("aria-current", "page");

    await expect(
      page.getByRole("button", { name: "Upload Shortlist" }),
    ).toBeVisible();

    await page.goto("/search?view=shortlist");
    await expect(searchLink).toHaveAttribute("aria-current", "page");
    await expect(
      page.getByRole("switch", { name: "Shortlist: On" }),
    ).toBeVisible();
    await expect(page.getByText("No shortlist yet")).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Go to Moneyball" }),
    ).toHaveCount(0);
  });

  test("Shortlist toggle filters General rows with virtual paging and desktop layout", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      squadOverview: true,
      shortlistSearch: true,
    });
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/search?shortlistOnly=true");

    const main = page.getByRole("main");
    const scroller = main.getByTestId("search-results-scroller");
    const table = scroller.getByRole("table", {
      name: "Player search results",
    });
    await expect(table.getByText("Shortlist player 001")).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Upload Moneyball CSV" }),
    ).toHaveCount(0);
    await expect(table.getByRole("columnheader", { name: "CA" })).toBeVisible();
    await expect(table.getByText("No shortlist yet")).toHaveCount(0);

    await table.getByText("Shortlist player 001").click();
    await expect(page).toHaveURL(/\/players\/1001\?view=general/);
    await expect(
      page.getByRole("heading", { name: "Shortlist player 001" }),
    ).toHaveCount(0);

    await page.goto("/search?shortlistOnly=true");
    await expect(table.getByText("Shortlist player 001")).toBeVisible();
    const toggle = page.getByRole("switch", { name: "Shortlist: On" });
    await expect(toggle).toHaveAttribute("aria-checked", "true");
    await toggle.click();
    await expect(
      page.getByRole("switch", { name: "Shortlist: Off" }),
    ).toBeVisible();
    const [mainBox, scrollerBox, mainDimensions] = await Promise.all([
      main.boundingBox(),
      scroller.boundingBox(),
      main.evaluate((element) => {
        const mainElement = element as unknown as {
          clientHeight: number;
          scrollHeight: number;
        };
        return {
          clientHeight: mainElement.clientHeight,
          scrollHeight: mainElement.scrollHeight,
        };
      }),
    ]);
    expect(mainBox).not.toBeNull();
    expect(scrollerBox).not.toBeNull();
    if (!mainBox || !scrollerBox) {
      throw new Error("Expected the General table to have a visible layout.");
    }
    expect(scrollerBox.height).toBeGreaterThan(100);
    expect(scrollerBox.y + scrollerBox.height).toBeLessThanOrEqual(
      mainBox.y + mainBox.height + 1,
    );
    expect(mainDimensions.scrollHeight).toBeLessThanOrEqual(
      mainDimensions.clientHeight + 1,
    );
  });

  test("unknown routes render the not-found page", async ({ page }) => {
    await page.goto("/does-not-exist");

    await expect(
      page.getByRole("heading", { name: "Page not found" }),
    ).toBeVisible();
  });
});
