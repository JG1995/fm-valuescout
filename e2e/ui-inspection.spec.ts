import { mkdir } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import { type SmokeStubOptions, stubTauriIpc } from "./tauri-ipc-stub";

const populatedOptions: SmokeStubOptions = {
  academyWorkspace: true,
  moneyballSearch: true,
  plannerPotentialScores: true,
  plannerSnapshot: true,
  playerProfile: true,
  playerProfileLayout: true,
  playerTableRowCount: 24,
  shortlistSearch: true,
  snapshotHistory: true,
  squadOverview: true,
  staffAssignment: true,
  staffShortlist: true,
  staffWorkspace: true,
};

type InspectionPage = {
  name: string;
  route: string;
  readyText?: string;
};

const canonicalPages: readonly InspectionPage[] = [
  { name: "dashboard", route: "/", readyText: "Placeholder." },
  { name: "search", route: "/search", readyText: "Player 001" },
  {
    name: "moneyball-search",
    route: "/search?view=moneyball",
    readyText: "Moneyball player 001",
  },
  {
    name: "player-shortlist",
    route: "/search?shortlistOnly=true",
    readyText: "Shortlist player 001",
  },
  { name: "staff-search", route: "/staff", readyText: "Staff member 001" },
  {
    name: "my-staff",
    route: "/staff?view=my-staff",
    readyText: "Staff member 001",
  },
  {
    name: "staff-shortlist",
    route: "/staff?shortlistOnly=true",
    readyText: "Staff member 001",
  },
  { name: "squad", route: "/my-club", readyText: "Player 001" },
  {
    name: "planner",
    route: "/my-club?view=planner",
    readyText: "Potential Keeper",
  },
  { name: "tactic", route: "/my-club?view=tactic", readyText: "Tactical XI" },
  {
    name: "academy-overview",
    route: "/academy",
    readyText: "Class of 2026",
  },
  {
    name: "academy-graduates",
    route: "/academy?view=graduates",
    readyText: "Alex Scout",
  },
  {
    name: "academy-class",
    route: "/academy?view=class&classId=7",
    readyText: "Jamie Prospect",
  },
  { name: "settings", route: "/settings" },
  {
    name: "player-profile-overview",
    route: "/players/42?section=overview",
    readyText: "Potential Scout",
  },
  {
    name: "player-profile-attributes",
    route: "/players/42?section=attributes",
    readyText: "Acceleration",
  },
  {
    name: "player-profile-role-fit",
    route: "/players/42?section=role-fit",
    readyText: "Wide Covering Defensive Midfielder",
  },
  {
    name: "player-profile-moneyball",
    route: "/players/42?section=moneyball",
    readyText: "Minutes",
  },
  { name: "staff-profile", route: "/staff/101", readyText: "Alex Coach" },
];

const requestedRoute = process.env.UI_INSPECTION_ROUTE ?? "all";
const viewport = {
  width: Number(process.env.UI_INSPECTION_WIDTH ?? 1600),
  height: Number(process.env.UI_INSPECTION_HEIGHT ?? 900),
};
const outputDirectory = path.resolve(".work/ui-inspection");

function routeName(route: string) {
  const name = route
    .replace(/^\//, "")
    .replace(/[^a-z0-9]+/gi, "-")
    .replace(/^-|-$/g, "")
    .toLowerCase();
  return name || "dashboard";
}

const pages =
  requestedRoute === "all"
    ? canonicalPages
    : [{ name: routeName(requestedRoute), route: requestedRoute }];

for (const inspectedPage of pages) {
  test(`capture populated ${inspectedPage.name}`, async ({ page }) => {
    const pageErrors: string[] = [];
    page.on("pageerror", (error) => pageErrors.push(error.message));

    await page.setViewportSize(viewport);
    await stubTauriIpc(
      page,
      inspectedPage.route.includes("#inspection-assignment-results")
        ? {
            ...populatedOptions,
            plannerSnapshot: false,
            playerProfile: false,
            squadOverview: false,
            staffWorkspace: false,
            snapshotHistory: false,
          }
        : populatedOptions,
    );
    await page.goto(inspectedPage.route);

    await expect(
      page
        .getByRole("combobox", { name: "Active save" })
        .locator("option:checked"),
    ).toHaveText("Default save - 1st August 2026");
    await expect(page.getByRole("main")).toBeVisible();
    await expect
      .poll(() => page.locator('[aria-busy="true"]:visible').count())
      .toBe(0);
    if (inspectedPage.readyText) {
      await expect(
        page.getByText(inspectedPage.readyText).first(),
      ).toBeVisible();
    } else if (inspectedPage.name === "settings") {
      await expect(
        page.getByRole("table", { name: "Snapshot history" }),
      ).toBeVisible();
    }
    expect(pageErrors).toEqual([]);

    if (inspectedPage.route.includes("#inspection-assignment-modal")) {
      await page
        .getByRole("button", { name: "Configure staffing needs" })
        .click();
      const dialog = page.getByRole("dialog", {
        name: "Configure staffing needs",
      });
      await expect(dialog).toBeVisible();
      await expect(
        dialog.getByText("Zero excludes a role from recommendations."),
      ).toBeVisible();
      const coachesInput = dialog
        .getByRole("spinbutton", { name: "Coaches slots" })
        .first();
      const coachesBefore = Number(await coachesInput.inputValue());
      const assistantManagerInput = dialog
        .getByRole("spinbutton", { name: "Assistant Manager slots" })
        .first();
      const assistantManagerBefore = await assistantManagerInput.inputValue();
      await expect(
        dialog.getByRole("button", { name: "Increase Coaches slots" }).first(),
      ).toBeVisible();
      await dialog
        .getByRole("button", { name: "Increase Coaches slots" })
        .first()
        .click();
      await expect(coachesInput).toHaveValue(String(coachesBefore + 1));
      await expect(assistantManagerInput).toHaveValue(assistantManagerBefore);
    }

    if (inspectedPage.route.includes("#inspection-assignment-results")) {
      await page
        .getByRole("button", { name: "Configure staffing needs" })
        .click();
      const dialog = page.getByRole("dialog", {
        name: "Configure staffing needs",
      });
      await expect(dialog).toBeVisible();
      for (const label of [
        "Assistant Manager slots",
        "Coaches slots",
        "Manager slots",
        "Scout slots",
      ]) {
        await dialog.getByRole("spinbutton", { name: label }).first().fill("1");
      }
      await dialog.getByRole("button", { name: "Save slots" }).click();
      await expect(dialog).toBeHidden();
      await expect(page.getByRole("status")).toHaveText("Slot counts saved.");
      await page
        .getByRole("button", { name: "Configure staffing needs" })
        .click();
      const reopenedDialog = page.getByRole("dialog", {
        name: "Configure staffing needs",
      });
      await expect(
        reopenedDialog
          .getByRole("spinbutton", { name: "Assistant Manager slots" })
          .first(),
      ).toHaveValue("1");
      await reopenedDialog.getByRole("button", { name: "Cancel" }).click();
      const optimize = page.getByRole("button", {
        name: "Optimize assignments",
      });
      await expect(optimize).toBeEnabled();
      await optimize.click();
      const staleContext = page.getByRole("alert", {
        name: "Staff assignment context changed",
      });
      if (await staleContext.isVisible()) {
        await optimize.click();
      }
      await expect(
        page.getByRole("table", {
          name: "Staff assignment recommendations and vacancies",
        }),
      ).toBeVisible();
      await expect(
        page.getByRole("rowheader", { name: /First Team/ }),
      ).toBeVisible();
    }

    await mkdir(outputDirectory, { recursive: true });
    const screenshotPath = path.join(
      outputDirectory,
      `${inspectedPage.name}-${viewport.width}x${viewport.height}.png`,
    );
    await page.screenshot({ path: screenshotPath });
    console.log(`UI screenshot: ${screenshotPath}`);
  });
}
