import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticOptionsIpcMock,
} from "@/testing/planner-ipc-mock";
import type { PlannerSlotCandidate } from "../types/depth";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";

const candidate: PlannerSlotCandidate = {
  playerUid: 77,
  name: "Reserve Keeper",
  currentClub: "Barcelona",
  age: 24,
  overAge: false,
  ipScore: 85,
  oopScore: 75,
  combinedScore: 80,
  expectedCombinedScore: 82,
  positionFit: "suited",
  footFit: "matches",
  assignmentLocation: null,
};

function renderPicker(
  occupant: PlannerSlotTarget["occupant"],
  onClose = vi.fn(),
) {
  const depth = resolvePlannerDepthIpcMock();
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      <PlannerSlotFitPicker
        activeSaveId={1}
        open
        target={{
          team: "reserves",
          stringId: 2,
          stringOrder: 0,
          laneId: "goalkeeper",
          laneName: "IP: GK / OOP: GK",
          occupant,
        }}
        tactic={depth.tactic}
        options={resolvePlannerTacticOptionsIpcMock()}
        teamLabels={{ senior: "Senior", reserves: "Reserves" }}
        onClose={onClose}
        onMutationError={vi.fn()}
      />
    </QueryClientProvider>,
  );
  return onClose;
}

describe("Planner slot picker", () => {
  it("replaces the cell's player with an unassigned pick in one step", async () => {
    const user = userEvent.setup();
    const depth = resolvePlannerDepthIpcMock();
    const calls: Array<[string, unknown]> = [];
    mockIPC((cmd, args) => {
      if (cmd === "get_planner_slot_candidates")
        return [
          candidate,
          { ...candidate, playerUid: 78, name: "Current Keeper" },
        ];
      calls.push([cmd, args]);
      return depth;
    });
    const onClose = renderPicker({
      playerUid: 78,
      name: "Current Keeper",
      pinned: false,
    });

    expect(
      screen.getByRole("dialog", { name: "Replace Current Keeper" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("option", { name: /Current Keeper/ }),
    ).toBeDisabled();
    await user.click(screen.getByRole("option", { name: /Reserve Keeper/ }));

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(calls).toEqual([
      [
        "assign_planner_player",
        { stringId: 2, laneId: "goalkeeper", playerUid: 77 },
      ],
    ]);
  });

  it("pins the cell's player, or hands the cell back to Optimize", async () => {
    const user = userEvent.setup();
    const depth = resolvePlannerDepthIpcMock();
    const calls: Array<[string, unknown]> = [];
    mockIPC((cmd, args) => {
      if (cmd === "get_planner_slot_candidates") return [];
      calls.push([cmd, args]);
      return depth;
    });
    const onClose = renderPicker({
      playerUid: 78,
      name: "Current Keeper",
      pinned: true,
    });

    expect(
      screen.getByRole("region", { name: "Current player" }),
    ).toHaveTextContent("Pinned: Optimize keeps him here.");
    expect(screen.queryByRole("button", { name: "Pin" })).toBeNull();
    await user.click(screen.getByRole("button", { name: "Unpin" }));

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(calls).toEqual([
      [
        "set_planner_assignment_pinned",
        { stringId: 2, laneId: "goalkeeper", pinned: false },
      ],
    ]);
  });

  it("shows each candidate's age, expected score, and what Optimize holds against him", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_planner_slot_candidates")
        return [
          candidate,
          {
            ...candidate,
            playerUid: 79,
            name: "Awkward Keeper",
            overAge: true,
            expectedCombinedScore: null,
            positionFit: "unfamiliar",
            footFit: "excluded",
          },
          {
            ...candidate,
            playerUid: 80,
            name: "Outfield Player",
            positionFit: "unsuited",
            footFit: "mismatch",
          },
        ];
      throw new Error(`Unexpected IPC: ${cmd}`);
    });
    renderPicker(null);

    const suited = await screen.findByRole("option", {
      name: /Reserve Keeper/,
    });
    expect(suited).toHaveTextContent("Reserve Keeper · Age 24");
    expect(
      within(suited).getByRole("img", {
        name: /Expected combined role score: 82/,
      }),
    ).toBeInTheDocument();
    expect(suited).not.toHaveTextContent(/position|foot|age limit/i);
    expect(
      screen.getByRole("option", { name: /Awkward Keeper/ }),
    ).toHaveTextContent(
      "Unfamiliar position · Wrong foot: slot is strict · Over age limit",
    );
    expect(
      screen.getByRole("option", { name: /Outfield Player/ }),
    ).toHaveTextContent("Cannot play this position · Wrong foot");
  });
});

describe("Planner slot mutation locks", () => {
  for (const action of ["assign", "move", "clear"] as const) {
    it(`keeps the ${action} dialog open and prevents another request until settlement`, async () => {
      const user = userEvent.setup();
      const depth = resolvePlannerDepthIpcMock();
      let settle: (() => void) | undefined;
      const pending = new Promise<typeof depth>((resolve) => {
        settle = () => resolve(depth);
      });
      const commands: string[] = [];
      const command =
        action === "clear"
          ? "clear_planner_assignment"
          : `${action}_planner_player`;
      mockIPC((cmd) => {
        if (cmd === "get_planner_slot_candidates")
          return [
            {
              ...candidate,
              assignmentLocation:
                action === "move"
                  ? {
                      team: "reserves",
                      stringId: 2,
                      stringOrder: 0,
                      laneId: "goalkeeper",
                    }
                  : null,
            },
          ];
        if (cmd === command) {
          commands.push(cmd);
          return pending;
        }
        throw new Error(`Unexpected IPC: ${cmd}`);
      });
      const onClose = vi.fn();
      render(
        <QueryClientProvider
          client={
            new QueryClient({ defaultOptions: { queries: { retry: false } } })
          }
        >
          <PlannerSlotFitPicker
            activeSaveId={1}
            open
            target={{
              team: "senior",
              stringId: 1,
              stringOrder: 0,
              laneId: "goalkeeper",
              laneName: "IP: GK / OOP: GK",
              occupant:
                action === "clear"
                  ? { playerUid: 77, name: "Reserve Keeper", pinned: true }
                  : null,
            }}
            tactic={depth.tactic}
            options={resolvePlannerTacticOptionsIpcMock()}
            teamLabels={{ senior: "Senior", reserves: "Reserves" }}
            onClose={onClose}
            onMutationError={vi.fn()}
          />
        </QueryClientProvider>,
      );
      if (action !== "clear") {
        await user.click(
          await screen.findByRole("option", { name: /Reserve Keeper/ }),
        );
      }
      if (action !== "assign") {
        await user.click(
          screen.getByRole("button", {
            name: action === "move" ? "Confirm move" : "Clear slot",
          }),
        );
      }
      await waitFor(() => expect(commands).toHaveLength(1));
      if (action === "assign") {
        const search = screen.getByRole("combobox", {
          name: "Search squad candidates",
        });
        expect(search).toBeDisabled();
        await user.keyboard("{Enter}");
      }
      await user.keyboard("{Escape}");
      await user.click(screen.getByRole("button", { name: "Close dialog" }));
      expect(onClose).not.toHaveBeenCalled();
      expect(commands).toEqual([command]);
      if (!settle) throw new Error("Expected a pending Planner mutation");
      settle();
      await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    });
  }
});
