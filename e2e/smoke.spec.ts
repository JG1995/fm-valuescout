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
    await expect(main.getByRole("tab", { name: "Club Setup" })).toHaveAttribute(
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
      main.getByRole("region", { name: "Tactic controls" }),
    ).toBeVisible();
    await expect(
      main.getByRole("button", { name: "IP: AML · Winger" }),
    ).toBeVisible();
    const rightMc = main.getByRole("button", {
      name: "IP: MCR · Central Midfielder",
    });
    const leftMc = main.getByRole("button", {
      name: "IP: MCL · Central Midfielder",
    });
    const leftWinger = main.getByRole("button", { name: "IP: AML · Winger" });
    const rightWinger = main.getByRole("button", {
      name: "IP: AMR · Winger",
    });
    const pitches = main.getByRole("group", { name: /pitch$/ });
    const leftWingerGroup = pitches
      .first()
      .locator('[data-position-group="AML"]');
    const rightWingerGroup = pitches
      .first()
      .locator('[data-position-group="AMR"]');
    await expect(rightMc).toBeVisible();
    await expect(leftMc).toBeVisible();
    await expect(pitches).toHaveCount(2);
    await expect(pitches.first()).toHaveAttribute("data-pitch-slot-count", "4");
    await expect(pitches.last()).toHaveAttribute("data-pitch-slot-count", "4");
    const [
      rightMcBox,
      leftMcBox,
      leftWingerBox,
      rightWingerBox,
      leftWingerGroupBox,
      rightWingerGroupBox,
      bothPitchBox,
    ] = await Promise.all([
      rightMc.boundingBox(),
      leftMc.boundingBox(),
      leftWinger.boundingBox(),
      rightWinger.boundingBox(),
      leftWingerGroup.boundingBox(),
      rightWingerGroup.boundingBox(),
      pitches.first().boundingBox(),
    ]);
    if (
      !rightMcBox ||
      !leftMcBox ||
      !leftWingerBox ||
      !rightWingerBox ||
      !leftWingerGroupBox ||
      !rightWingerGroupBox ||
      !bothPitchBox
    ) {
      throw new Error("Expected visible tactic cards and pitch geometry");
    }
    expect(rightMcBox.y).toBe(leftMcBox.y);
    expect(rightMcBox.width).toBeCloseTo(leftMcBox.width, 1);
    expect(rightMcBox.width).toBeCloseTo(leftWingerBox.width, 1);
    expect(rightMcBox.width).toBeCloseTo(rightWingerBox.width, 1);
    expect(rightMcBox.width).toBeGreaterThan(bothPitchBox.width * 0.2);
    expect(leftMcBox.x + leftMcBox.width).toBeLessThan(rightMcBox.x);
    expect(leftWingerBox.x + leftWingerBox.width).toBeLessThan(
      rightWingerBox.x,
    );
    expect(
      Math.abs(
        leftWingerBox.x +
          leftWingerBox.width / 2 -
          (leftWingerGroupBox.x + leftWingerGroupBox.width / 2),
      ),
    ).toBeLessThanOrEqual(1);
    expect(
      Math.abs(
        rightWingerBox.x +
          rightWingerBox.width / 2 -
          (rightWingerGroupBox.x + rightWingerGroupBox.width / 2),
      ),
    ).toBeLessThanOrEqual(1);
    const pairCentre = (leftMcBox.x + rightMcBox.x + rightMcBox.width) / 2;
    expect(pairCentre).toBeCloseTo(bothPitchBox.x + bothPitchBox.width / 2, 1);
    await main.getByRole("button", { name: "IP", exact: true }).click();
    await expect(pitches).toHaveCount(1);
    const singlePitchBox = await pitches.first().boundingBox();
    if (!singlePitchBox) {
      throw new Error("Expected visible single-phase pitch geometry");
    }
    expect(singlePitchBox.width).toBeCloseTo(bothPitchBox.width, 1);
    await main.getByRole("button", { name: "Both", exact: true }).click();
    await expect(pitches).toHaveCount(2);
    for (const index of [0, 1]) {
      const pitch = pitches.nth(index);
      await expect(pitch.getByRole("button").first()).toHaveAccessibleName(
        /: STC · /,
      );
      await expect(pitch.getByRole("button").last()).toHaveAccessibleName(
        /: GK · /,
      );
    }
    await expect(main.getByText("Left winger")).toHaveCount(0);
    await main
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
  });

  test("planner tactic workspace fits its supported desktop viewports", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner?view=tactic");

    const main = page.getByRole("main");
    const pitches = main.getByRole("group", { name: /pitch$/ });
    const settings = main.getByRole("region", {
      name: "Selected position settings",
    });
    const plannerHeading = main.getByRole("heading", {
      level: 1,
      name: "Squad Planner",
    });
    const workspaceTabs = main.getByRole("tablist", {
      name: "Planner workspaces",
    });
    const navToggle = page.getByRole("button", {
      name: "Toggle navigation",
    });

    const expectWorkspaceFit = async (
      width: number,
      height: number,
      requireVerticalFit: boolean,
    ) => {
      await page.setViewportSize({ width, height });
      for (const [view, pitchCount, visibleRole] of [
        ["Both", 2, "OOP GK role"],
        ["IP", 1, "IP GK role"],
        ["OOP", 1, "OOP GK role"],
      ] as const) {
        await main.getByRole("button", { name: view, exact: true }).click();
        await expect(pitches).toHaveCount(pitchCount);
        await expect(settings).toBeVisible();
        await expect(
          settings.getByRole("combobox", { name: visibleRole }),
        ).toBeVisible();

        const [headingBox, workspaceTabsBox] = await Promise.all([
          plannerHeading.boundingBox(),
          workspaceTabs.boundingBox(),
        ]);
        expect(headingBox).not.toBeNull();
        expect(workspaceTabsBox).not.toBeNull();
        expect(workspaceTabsBox?.y).toBeGreaterThanOrEqual(
          (headingBox?.y ?? 0) + (headingBox?.height ?? 0),
        );

        if (width >= 1600 && view === "Both") {
          const selectBoxes = await settings
            .getByRole("combobox")
            .evaluateAll((elements) =>
              elements.map(
                (element) =>
                  (
                    element as unknown as {
                      getBoundingClientRect: () => { top: number };
                    }
                  ).getBoundingClientRect().top,
              ),
            );
          expect(
            Math.max(...selectBoxes) - Math.min(...selectBoxes),
          ).toBeLessThanOrEqual(1);
        }

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
      }
    };

    for (const [width, height, requireVerticalFit] of [
      [1280, 800, false],
      [1600, 900, true],
      [1920, 1080, true],
    ] as const) {
      await expectWorkspaceFit(width, height, requireVerticalFit);
      await navToggle.click();
      await expect(navToggle).toHaveAttribute("aria-expanded", "true");
      await expectWorkspaceFit(width, height, requireVerticalFit);
      await navToggle.click();
      await expect(navToggle).toHaveAttribute("aria-expanded", "false");
    }
  });

  test("planner depth adds strings for Senior, Reserves, and Youth", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 900, height: 800 });
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

  test("planner depth optimizes current and potential squads within desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, { plannerSnapshot: true });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await page.goto("/planner");

      const main = page.getByRole("main");
      await main.getByRole("tab", { name: "Squad" }).click();
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
    await page.setViewportSize({ width: 900, height: 800 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Squad" }).click();
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

    await main.getByRole("tab", { name: "Reserves" }).click();
    await expect(
      main.getByRole("button", {
        name: /Reserves, 1st string, IP: GK .* Empty/,
      }),
    ).toBeVisible();
  });

  test("planner depth groups all teams when the matrix fits", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1920, height: 900 });
    await stubTauriIpc(page, { plannerSnapshot: true });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Squad" }).click();
    const matrix = main.getByRole("region", {
      name: "All squads depth matrix",
    });
    await expect(matrix).toBeVisible();
    await expect(
      matrix.getByRole("columnheader", { name: "Senior squad" }),
    ).toBeVisible();
    await expect(
      matrix.getByRole("columnheader", { name: "Reserves squad" }),
    ).toBeVisible();
    await expect(
      matrix.getByRole("columnheader", { name: "Youth squad" }),
    ).toBeVisible();
    await expect(main.getByRole("tab", { name: "Senior" })).toHaveCount(0);
  });

  test("planner depth keeps assigned current and potential scores readable at desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, {
      plannerSnapshot: true,
      plannerPotentialScores: true,
    });
    await page.goto("/planner");

    const main = page.getByRole("main");
    await main.getByRole("tab", { name: "Squad" }).click();
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
    const current = summary.getByRole("img", {
      name: "Best role (Current): 82, Starter",
    });
    const potential = summary.getByRole("img", {
      name: "Best potential role (Potential): 94, Elite",
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(summary).toBeVisible();
      await expect(attributes).toBeVisible();
      await expect(roleFit).toBeVisible();
      await expect(current).toBeVisible();
      await expect(potential).toBeVisible();

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

      const [currentBox, potentialBox] = await Promise.all([
        current.boundingBox(),
        potential.boundingBox(),
      ]);
      expect(currentBox).not.toBeNull();
      expect(potentialBox).not.toBeNull();
      if (!currentBox || !potentialBox) {
        throw new Error("Expected visible best-role summary badges.");
      }
      expect(currentBox.x + currentBox.width).toBeLessThanOrEqual(
        potentialBox.x,
      );
    }

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
    await expect(
      main
        .getByRole("tooltip")
        .filter({ hasText: "Ambition 10 → random 11–20" }),
    ).toBeVisible();
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

  test("player profile Attributes keeps visible potential pairs within desktop widths", async ({
    page,
  }) => {
    await stubTauriIpc(page, { playerProfile: true });
    await page.goto("/players/42?tab=technical");

    const technical = page.getByRole("region", { name: "Technical" });
    const passing = technical.locator("dd", {
      hasText: "Current 14, Potential 16",
    });

    for (const [width, height] of [
      [1280, 800],
      [1600, 900],
    ] as const) {
      await page.setViewportSize({ width, height });
      await expect(passing).toContainText("14→16");
      await expect(passing.locator('[data-tier="4"]')).toHaveCount(2);

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
    }
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
