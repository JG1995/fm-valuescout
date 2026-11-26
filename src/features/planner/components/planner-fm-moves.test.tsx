import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { PlannerFmMoveResult, PlannerFmMoves } from "../types/fm-moves";
import { PlannerFmMoves as PlannerFmMovesButton } from "./planner-fm-moves";

const plan: PlannerFmMoves = {
  moves: [
    {
      playerUid: 79,
      name: "Young Striker",
      age: 17,
      fromLevel: "youth",
      toTeam: "senior",
      toTeamName: "Senior",
      overAge: false,
    },
    {
      playerUid: 77,
      name: "Veteran Keeper",
      age: 25,
      fromLevel: "senior",
      toTeam: "reserves",
      toTeamName: "U21",
      overAge: true,
    },
    {
      playerUid: 78,
      name: "Squad Winger",
      age: 22,
      fromLevel: "senior",
      toTeam: "reserves",
      toTeamName: "U21",
      overAge: false,
    },
    {
      playerUid: 80,
      name: "Unread Youngster",
      age: 16,
      fromLevel: null,
      toTeam: "youth",
      toTeamName: "U18",
      overAge: false,
    },
  ],
  unavailableReason: null,
};

function renderMoves(
  fmMoves: PlannerFmMoves,
  result: PlannerFmMoveResult | null = null,
) {
  const applied: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "get_planner_fm_moves") return fmMoves;
    if (cmd === "apply_planner_fm_moves") {
      applied.push(args);
      return result;
    }
    return null;
  });
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      <PlannerFmMovesButton activeSaveId={1} disabled={false} />
    </QueryClientProvider>,
  );
  return applied;
}

describe("Apply squads in FM", () => {
  it("moves only the ticked players and reports what FM did", async () => {
    const user = userEvent.setup();
    const applied = renderMoves(plan, {
      moved: [79],
      unchanged: [],
      failed: [
        {
          playerUid: 77,
          name: "Veteran Keeper",
          reason: "FM did not confirm the move",
        },
      ],
      error: null,
    });

    await user.click(screen.getByRole("button", { name: "Apply in FM" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Apply squads in FM",
    });
    const keeperRow = within(dialog)
      .getByRole("rowheader", { name: "Veteran Keeper" })
      .closest("tr") as HTMLElement;
    expect(within(keeperRow).getByText("Over age")).toBeInTheDocument();
    expect(within(keeperRow).getByText("First team")).toBeInTheDocument();
    const unknownRow = within(dialog)
      .getByRole("rowheader", { name: "Unread Youngster" })
      .closest("tr") as HTMLElement;
    expect(within(unknownRow).getByText("Unknown")).toBeInTheDocument();

    await user.click(
      within(dialog).getByRole("checkbox", { name: "Move Squad Winger" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Move 3 players" }),
    );

    expect(applied).toEqual([{ playerUids: [79, 77, 80] }]);
    expect(await within(dialog).findByRole("status")).toHaveTextContent(
      "FM moved 1 player.",
    );
    expect(
      within(dialog).getByText("Veteran Keeper: FM did not confirm the move"),
    ).toBeInTheDocument();
  });

  it("explains why the moves cannot be made and blocks them", async () => {
    const user = userEvent.setup();
    const applied = renderMoves({
      ...plan,
      unavailableReason:
        "Load Data with FM open on this save before moving squads",
    });

    await user.click(screen.getByRole("button", { name: "Apply in FM" }));
    const dialog = await screen.findByRole("dialog");

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Load Data with FM open on this save before moving squads",
    );
    expect(
      within(dialog).getByRole("button", { name: "Move 4 players" }),
    ).toBeDisabled();
    expect(applied).toEqual([]);
  });

  it("says so when every placed player is already in his FM team", async () => {
    const user = userEvent.setup();
    renderMoves({ moves: [], unavailableReason: null });

    await user.click(screen.getByRole("button", { name: "Apply in FM" }));
    const dialog = await screen.findByRole("dialog");

    expect(
      await within(dialog).findByText("FM already matches the Planner."),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Move 0 players" }),
    ).toBeDisabled();
  });
});
