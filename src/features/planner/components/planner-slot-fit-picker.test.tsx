import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticOptionsIpcMock,
} from "@/testing/planner-ipc-mock";
import { PlannerSlotFitPicker } from "./planner-slot-fit-picker";

const candidate = {
  playerUid: 77,
  name: "Reserve Keeper",
  currentClub: "Barcelona",
  ipScore: 85,
  oopScore: 75,
  combinedScore: 80,
  assignmentLocation: null,
};

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
              occupantName: action === "clear" ? "Reserve Keeper" : null,
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
