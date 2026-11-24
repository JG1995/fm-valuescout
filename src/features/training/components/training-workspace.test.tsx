import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type {
  FmTraining,
  FmTrainingPlayer,
  FmTrainingResult,
} from "../types/fm-training";
import { TrainingWorkspace } from "./training-workspace";

const unplaced: FmTrainingPlayer = {
  playerUid: 78,
  name: "Unplaced Prospect",
  age: 19,
  slot: null,
  position: null,
  inPossessionRole: null,
  outOfPossessionRole: null,
  learning: null,
  focus: "Quickness",
};

const plan: FmTraining = {
  players: [
    {
      playerUid: 79,
      name: "Veteran Keeper",
      age: 31,
      slot: {
        team: "senior",
        teamName: "Senior",
        stringName: "Starter",
        ipPosition: "GK",
        oopPosition: "GK",
        inPossessionRole: "Goalkeeper",
      },
      position: "GK",
      inPossessionRole: "Goalkeeper",
      outOfPossessionRole: "Line-Holding Keeper",
      learning: null,
      focus: null,
    },
    {
      playerUid: 77,
      name: "Slot Winger",
      age: 21,
      slot: {
        team: "senior",
        teamName: "Senior",
        stringName: "Starter",
        ipPosition: "AMR",
        oopPosition: "MR",
        inPossessionRole: "Inside Forward",
      },
      position: "MR",
      inPossessionRole: "Inside Winger",
      outOfPossessionRole: "Tracking Wide Midfielder",
      learning: { position: "MR", familiarity: 9, outOfPossession: true },
      focus: "Ball Control",
    },
    {
      playerUid: 80,
      name: "Youth Stopper",
      age: 17,
      slot: {
        team: "youth",
        teamName: "Under 18s",
        stringName: "1st string",
        ipPosition: "DCR",
        oopPosition: "DCR",
        inPossessionRole: "Centre-Back",
      },
      position: "DC",
      inPossessionRole: "Centre-Back",
      outOfPossessionRole: "Stopping Centre-Back",
      learning: { position: "DC", familiarity: 0, outOfPossession: false },
      focus: "Tackling",
    },
    unplaced,
  ],
  unavailableReason: null,
};

function renderTraining(
  training: FmTraining,
  result: FmTrainingResult | null = null,
) {
  const applied: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "get_squad_fm_training") return training;
    if (cmd === "apply_squad_fm_training") {
      applied.push(args);
      return result;
    }
    return null;
  });
  const rootRoute = createRootRoute();
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      createRoute({
        getParentRoute: () => rootRoute,
        path: "/",
        component: () => (
          <TrainingWorkspace activeSaveId={1} disabled={false} />
        ),
      }),
      createRoute({ getParentRoute: () => rootRoute, path: "/players/$uid" }),
    ]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      {/* The test router has its own route tree, not the app's. */}
      <RouterProvider router={router as never} />
    </QueryClientProvider>,
  );
  return applied;
}

async function row(name: string) {
  return (
    await screen.findByRole("rowheader", { name: new RegExp(name) })
  ).closest("tr") as HTMLElement;
}

describe("Training workspace", () => {
  it("lists each player's training under his squad and string", async () => {
    renderTraining(plan);

    expect(await row("Veteran Keeper")).toHaveTextContent(
      "Veteran Keeper · 31GKGoalkeeper GKLine-Holding KeeperCleared",
    );
    expect(await row("Unplaced Prospect")).toHaveTextContent(
      "Unplaced Prospect · 19No slotUnchangedUnchangedQuickness",
    );
    const groups = [
      "Senior · Starter",
      "Under 18s · 1st string",
      "Not in a squad",
    ];
    expect(
      screen
        .getAllByRole("rowheader")
        .map((header) => header.textContent ?? "")
        .filter((text) => groups.includes(text)),
    ).toEqual(groups);
  });

  it("says which position a player is learning and what he returns to", async () => {
    renderTraining(plan);

    // He learns the out-of-possession position through a role there.
    expect(await row("Slot Winger")).toHaveTextContent(
      "AMR/MRInside Winger MRLearning MRFamiliarity 9 of 20 · then Inside Forward at AMRTracking Wide Midfielder",
    );
    // The in-possession position is the slot's own training.
    expect(await row("Youth Stopper")).toHaveTextContent(
      "DCRCentre-Back DCLearning DCNo familiarity yetStopping Centre-Back",
    );
    expect(screen.getByText("4 players · 2 learning a position")).toBeVisible();
  });

  it("sets the training of the ticked players in the chosen group", async () => {
    const user = userEvent.setup();
    const applied = renderTraining(plan, {
      applied: [77],
      failed: [
        {
          playerUid: 80,
          name: "Youth Stopper",
          reason: "FM did not take the focus (FM shows Passing)",
        },
      ],
      error: null,
    });
    await row("Slot Winger");

    expect(
      screen.getByRole("button", { name: "Apply training in FM (4)" }),
    ).toBeEnabled();
    await user.click(
      screen.getByRole("button", { name: "Learning a position 2" }),
    );
    expect(screen.queryByText("Veteran Keeper")).not.toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: "Apply training in FM (2)" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Apply training in FM",
    });
    await user.click(
      within(dialog).getByRole("button", {
        name: "Set training for 2 players",
      }),
    );

    expect(applied).toEqual([{ playerUids: [77, 80] }]);
    const status = await screen.findByRole("status");
    expect(status).toHaveTextContent(
      "FM took the training for 1 player. It did not take 1 player:",
    );
    expect(status).toHaveTextContent(
      "Youth Stopper: FM did not take the focus (FM shows Passing)",
    );
    // The Modal leaves after its exit transition.
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
  });

  it("leaves out the players the user unticks", async () => {
    const user = userEvent.setup();
    const applied = renderTraining(plan, {
      applied: [79, 77, 80],
      failed: [],
      error: null,
    });

    await user.click(
      await screen.findByRole("checkbox", {
        name: "Set training for Unplaced Prospect",
      }),
    );
    await user.click(
      screen.getByRole("button", { name: "Apply training in FM (3)" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Set training for 3 players" }),
    );

    expect(applied).toEqual([{ playerUids: [79, 77, 80] }]);
  });

  it("explains why the training cannot be set and blocks it", async () => {
    renderTraining({
      ...plan,
      unavailableReason:
        "Update the bridge plugin from Settings, restart FM and Load Data to set training in FM",
    });

    const apply = await screen.findByRole("button", {
      name: "Apply training in FM (4)",
    });
    expect(apply).toBeDisabled();
    expect(apply).toHaveAccessibleDescription(
      "Update the bridge plugin from Settings, restart FM and Load Data to set training in FM",
    );
  });

  it("says when only focuses are set", async () => {
    renderTraining({ players: [unplaced], unavailableReason: null });

    expect(
      await screen.findByText(
        "1 player · no one is in a Planner slot, so only focuses are set",
      ),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: /Learning a position/ }),
    ).not.toBeInTheDocument();
  });
});
