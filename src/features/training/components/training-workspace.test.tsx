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
import { describe, expect, it, vi } from "vitest";
import type {
  FmTraining,
  FmTrainingLane,
  FmTrainingPlayer,
  FmTrainingResult,
} from "../types/fm-training";
import { TrainingWorkspace } from "./training-workspace";

const lanes: FmTrainingLane[] = [
  {
    laneId: "goalkeeper",
    ipPosition: "GK",
    oopPosition: "GK",
    trainsAs: "goalkeeper",
  },
  {
    laneId: "left_centre_back",
    ipPosition: "DCL",
    oopPosition: "DCL",
    trainsAs: "left_centre_back",
  },
  {
    laneId: "right_centre_back",
    ipPosition: "DCR",
    oopPosition: "DCR",
    trainsAs: "left_centre_back",
  },
  {
    laneId: "right_winger",
    ipPosition: "AMR",
    oopPosition: "MR",
    trainsAs: "right_winger",
  },
  {
    laneId: "centre_forward",
    ipPosition: "STC",
    oopPosition: "STC",
    trainsAs: "centre_forward",
  },
];

const wing = {
  laneId: "right_winger",
  ipPosition: "AMR",
  oopPosition: "MR",
  inPossessionRole: "Inside Forward",
  currentScore: 70,
  potentialScore: 75,
  playable: true,
  potentialTopPercent: 40,
};
const striker = {
  laneId: "centre_forward",
  ipPosition: "STC",
  oopPosition: "STC",
  inPossessionRole: "Centre Forward",
  currentScore: 60,
  potentialScore: 85,
  playable: true,
  potentialTopPercent: 3,
};

const keeper: FmTrainingPlayer = {
  playerUid: 79,
  name: "Veteran Keeper",
  age: 31,
  slot: {
    team: "senior",
    teamName: "Senior",
    stringName: "Starter",
    laneId: "goalkeeper",
    ipPosition: "GK",
    oopPosition: "GK",
    learning: null,
  },
  target: {
    laneId: "goalkeeper",
    ipPosition: "GK",
    oopPosition: "GK",
    inPossessionRole: "Goalkeeper",
    development: false,
  },
  position: "GK",
  inPossessionRole: "Goalkeeper",
  outOfPossessionRole: "Line-Holding Keeper",
  learning: null,
  development: null,
  focus: null,
};

/** A winger whose potential stands out up front. He plays the right wing. */
const winger: FmTrainingPlayer = {
  playerUid: 77,
  name: "Slot Winger",
  age: 21,
  slot: {
    team: "senior",
    teamName: "Senior",
    stringName: "Starter",
    laneId: "right_winger",
    ipPosition: "AMR",
    oopPosition: "MR",
    learning: { position: "MR", familiarity: 9, outOfPossession: true },
  },
  target: {
    laneId: "right_winger",
    ipPosition: "AMR",
    oopPosition: "MR",
    inPossessionRole: "Inside Forward",
    development: false,
  },
  position: "MR",
  inPossessionRole: "Inside Winger",
  outOfPossessionRole: "Tracking Wide Midfielder",
  learning: { position: "MR", familiarity: 9, outOfPossession: true },
  development: {
    laneId: "centre_forward",
    suggestedLaneId: "centre_forward",
    locked: false,
    options: [
      striker,
      wing,
      {
        laneId: "left_centre_back",
        ipPosition: "DCL",
        oopPosition: "DCL",
        inPossessionRole: "Centre-Back",
        currentScore: 40,
        potentialScore: 55,
        playable: false,
        potentialTopPercent: null,
      },
    ],
  },
  focus: "Ball Control",
};

const stopper: FmTrainingPlayer = {
  playerUid: 80,
  name: "Youth Stopper",
  age: 17,
  slot: {
    team: "youth",
    teamName: "Under 18s",
    stringName: "1st string",
    laneId: "right_centre_back",
    ipPosition: "DCR",
    oopPosition: "DCR",
    learning: { position: "DC", familiarity: 0, outOfPossession: false },
  },
  target: {
    laneId: "right_centre_back",
    ipPosition: "DCR",
    oopPosition: "DCR",
    inPossessionRole: "Centre-Back",
    development: false,
  },
  position: "DC",
  inPossessionRole: "Centre-Back",
  outOfPossessionRole: "Stopping Centre-Back",
  learning: { position: "DC", familiarity: 0, outOfPossession: false },
  development: {
    laneId: "right_centre_back",
    suggestedLaneId: "right_centre_back",
    locked: false,
    options: [
      {
        laneId: "right_centre_back",
        ipPosition: "DCR",
        oopPosition: "DCR",
        inPossessionRole: "Centre-Back",
        currentScore: 55,
        potentialScore: 80,
        playable: true,
        potentialTopPercent: 10,
      },
    ],
  },
  focus: "Tackling",
};

const unplaced: FmTrainingPlayer = {
  playerUid: 78,
  name: "Unplaced Prospect",
  age: 19,
  slot: null,
  target: null,
  position: null,
  inPossessionRole: null,
  outOfPossessionRole: null,
  learning: null,
  development: {
    laneId: "right_winger",
    suggestedLaneId: "right_winger",
    locked: false,
    options: [{ ...wing, currentScore: 58, potentialScore: 78 }],
  },
  focus: "Quickness",
};

/** Training by Planner slot, as a save starts. */
const plan: FmTraining = {
  players: [keeper, winger, stopper, unplaced],
  unavailableReason: null,
  basis: "planner",
  developmentMaxAge: 23,
  developmentPositionFamiliarity: true,
  lanes,
};

/** The same squad training by development slot. */
const developmentPlan: FmTraining = {
  ...plan,
  basis: "development",
  players: [
    keeper,
    {
      ...winger,
      target: {
        laneId: "centre_forward",
        ipPosition: "STC",
        oopPosition: "STC",
        inPossessionRole: "Centre Forward",
        development: true,
      },
      position: "ST",
      inPossessionRole: "Centre Forward",
      outOfPossessionRole: "Tracking Centre Forward",
      learning: { position: "ST", familiarity: 12, outOfPossession: false },
      focus: "Chance Conversion",
    },
    stopper,
    {
      ...unplaced,
      target: {
        laneId: "right_winger",
        ipPosition: "AMR",
        oopPosition: "MR",
        inPossessionRole: "Winger",
        development: true,
      },
      position: "AMR",
      inPossessionRole: "Winger",
      outOfPossessionRole: "Tracking Wide Midfielder",
    },
  ],
};

/**
 * Renders the workspace over `training`. A change to what training follows or
 * to a development slot is recorded, and the plan read after it is `changed`.
 */
function renderTraining(
  training: FmTraining,
  result: FmTrainingResult | null = null,
  changed: FmTraining = training,
) {
  const applied: unknown[] = [];
  const changes: unknown[] = [];
  const onDevelopmentChanged = vi.fn();
  mockIPC((cmd, args) => {
    if (cmd === "get_squad_fm_training") {
      return changes.length > 0 ? changed : training;
    }
    if (cmd === "apply_squad_fm_training") {
      applied.push(args);
      return result;
    }
    if (
      cmd === "set_training_basis" ||
      cmd === "set_development_max_age" ||
      cmd === "set_development_position_familiarity" ||
      cmd === "set_development_slot"
    ) {
      changes.push({ cmd, args });
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
          <TrainingWorkspace
            activeSaveId={1}
            disabled={false}
            onDevelopmentChanged={onDevelopmentChanged}
          />
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
  return { applied, changes, onDevelopmentChanged };
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
      "Veteran Keeper · 31GKOver 23Trains the slot he playsGoalkeeper GKLine-Holding KeeperCleared",
    );
    const prospect = await row("Unplaced Prospect");
    // His development slot is named by its position and the tactic's role.
    expect(prospect).toHaveTextContent(
      "Unplaced Prospect · 19No slotAMR/MR · Inside Forward58",
    );
    // By Planner slot, his development slot is shown but not trained.
    expect(prospect).toHaveTextContent(
      "Not used: training follows the Planner slotUnchangedUnchangedQuickness",
    );
    expect(
      screen.queryByRole("combobox", { name: /Development slot for/ }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Developing towards")).not.toBeInTheDocument();
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
      "Inside Winger MRLearning MRFamiliarity 9 of 20 · then Inside Forward at AMRTracking Wide Midfielder",
    );
    // The in-possession position is the slot's own training.
    expect(await row("Youth Stopper")).toHaveTextContent(
      "Centre-Back DCLearning DCNo familiarity yetStopping Centre-Back",
    );
    expect(screen.getByText("4 players · 2 learning a position")).toBeVisible();
  });

  it("sets the training of the ticked players in the chosen group", async () => {
    const user = userEvent.setup();
    const { applied } = renderTraining(plan, {
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
    const { applied } = renderTraining(plan, {
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
    renderTraining({ ...plan, players: [unplaced] });

    expect(
      await screen.findByText(
        "1 player · no one has a slot to train for, so only focuses are set",
      ),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: /Learning a position/ }),
    ).not.toBeInTheDocument();
  });

  it("trains a young player for the slot he is developing towards", async () => {
    renderTraining(developmentPlan);

    // He plays the wing and trains as a striker, which costs him the wing's
    // out-of-possession position.
    const wingerRow = await row("Slot Winger");
    expect(wingerRow).toHaveTextContent(
      "Centre Forward STAway from slotLearning STFamiliarity 12 of 20Plays AMR/MR and stops learning MR (9 of 20)Tracking Centre Forward",
    );
    expect(
      within(wingerRow).getByRole("combobox", {
        name: "Development slot for Slot Winger",
      }),
    ).toHaveValue("centre_forward");
    expect(
      within(wingerRow)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual([
      "STC · Centre Forward · suggested",
      "AMR/MR · Inside Forward",
      "DCL · Centre-Back",
    ]);
    expect(wingerRow).toHaveTextContent("Top 3% at potential");
    // A player in no squad trains his development slot too, and is not away
    // from any slot.
    const prospect = await row("Unplaced Prospect");
    expect(prospect).toHaveTextContent(
      "Winger AMRTracking Wide MidfielderQuickness",
    );
    expect(prospect).not.toHaveTextContent("Away from slot");
    expect(await row("Youth Stopper")).toHaveTextContent(
      "Top 10% at potential · same as his slot",
    );
    expect(await row("Veteran Keeper")).toHaveTextContent(
      "Over 23Trains the slot he plays",
    );
    expect(
      screen.getByText(
        "4 players · 1 trains away from his slot · 2 learning a position",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Away from slot 1" }),
    ).toBeVisible();
  });

  it("counts the players developing towards each slot and lists one slot's", async () => {
    const user = userEvent.setup();
    renderTraining(developmentPlan);
    await row("Slot Winger");

    // The two centre-back slots train the same, so they count as one.
    expect(
      screen.getByRole("button", { name: "DCL, DCR: 1 player" }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "GK: 0 players" })).toBeVisible();
    expect(
      screen.getByText("Nobody aged 23 or under is developing towards GK."),
    ).toBeVisible();

    await user.click(screen.getByRole("button", { name: "STC: 1 player" }));

    expect(screen.queryByText("Youth Stopper")).not.toBeInTheDocument();
    expect(screen.getByText("Slot Winger")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Apply training in FM (1)" }),
    ).toBeVisible();
    await user.click(screen.getByRole("button", { name: "STC: 1 player" }));
    expect(screen.getByText("Youth Stopper")).toBeVisible();
  });

  it("switches what training follows and up to what age", async () => {
    const user = userEvent.setup();
    const { changes, onDevelopmentChanged } = renderTraining(
      plan,
      null,
      developmentPlan,
    );
    await row("Slot Winger");
    expect(
      screen.getByRole("button", { name: "Planner slot" }),
    ).toHaveAttribute("aria-pressed", "true");

    await user.click(screen.getByRole("button", { name: "Development slot" }));

    expect(await screen.findByText("Developing towards")).toBeVisible();
    expect(changes).toEqual([
      { cmd: "set_training_basis", args: { basis: "development" } },
    ]);
    expect(onDevelopmentChanged).toHaveBeenCalledTimes(1);

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Develop players aged up to" }),
      "21",
    );
    await waitFor(() =>
      expect(changes[1]).toEqual({
        cmd: "set_development_max_age",
        args: { maxAge: 21 },
      }),
    );
  });

  it("suggests development slots in any position once familiarity is off", async () => {
    const user = userEvent.setup();
    // On role scores alone the stopper's potential stands out up front, a
    // position he has yet to learn.
    const anywhere: FmTraining = {
      ...developmentPlan,
      developmentPositionFamiliarity: false,
      players: developmentPlan.players.map((player) =>
        player.playerUid === 80 && player.development
          ? {
              ...player,
              development: {
                ...player.development,
                laneId: "centre_forward",
                suggestedLaneId: "centre_forward",
                options: [
                  { ...striker, playable: false, potentialTopPercent: 2 },
                  ...player.development.options,
                ],
              },
            }
          : player,
      ),
    };
    const { changes } = renderTraining(developmentPlan, null, anywhere);
    const toggle = await screen.findByRole("checkbox", {
      name: "Position familiarity",
    });
    expect(toggle).toBeChecked();
    expect(toggle).toHaveAccessibleDescription(
      /only among the positions a player can play/,
    );

    await user.click(toggle);

    await waitFor(() => expect(toggle).not.toBeChecked());
    expect(toggle).toHaveAccessibleDescription(/in any position/);
    expect(changes).toEqual([
      {
        cmd: "set_development_position_familiarity",
        args: { positionFamiliarity: false },
      },
    ]);
    const stopperRow = await row("Youth Stopper");
    expect(stopperRow).toHaveTextContent(
      "Top 2% at potential · a position to learn",
    );
    // He is ranked there, yet it stays listed as a position to learn.
    expect(
      within(stopperRow).getByRole("group", { name: "To learn" }),
    ).toHaveTextContent("STC · Centre Forward · suggested");
  });

  it("keeps the development slot the user chooses, and returns to the suggestion", async () => {
    const user = userEvent.setup();
    const chosen: FmTraining = {
      ...developmentPlan,
      players: developmentPlan.players.map((player) =>
        player.playerUid === 77 && player.development
          ? {
              ...player,
              development: {
                ...player.development,
                laneId: "right_winger",
                locked: true,
              },
            }
          : player,
      ),
    };
    const { changes } = renderTraining(developmentPlan, null, chosen);

    await user.selectOptions(
      await screen.findByRole("combobox", {
        name: "Development slot for Slot Winger",
      }),
      "right_winger",
    );

    const wingerRow = await row("Slot Winger");
    expect(await within(wingerRow).findByText("Your choice")).toBeVisible();
    await user.click(
      within(wingerRow).getByRole("button", { name: "Use suggested STC" }),
    );
    await waitFor(() =>
      expect(changes).toEqual([
        {
          cmd: "set_development_slot",
          args: { playerUid: 77, laneId: "right_winger" },
        },
        // The suggested slot is no choice of the user's.
        { cmd: "set_development_slot", args: { playerUid: 77, laneId: null } },
      ]),
    );
  });
});
