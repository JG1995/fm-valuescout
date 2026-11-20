import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type {
  SquadFmTraining,
  SquadFmTrainingResult,
} from "../types/squad-fm-training";
import { SquadFmTraining as SquadFmTrainingButton } from "./squad-fm-training";

const plan: SquadFmTraining = {
  players: [
    {
      playerUid: 77,
      name: "Slot Winger",
      position: "AMR",
      inPossessionRole: "Winger",
      outOfPossessionRole: "Tracking Wide Midfielder",
      focus: "Ball Control",
    },
    {
      playerUid: 78,
      name: "Unplaced Prospect",
      position: null,
      inPossessionRole: null,
      outOfPossessionRole: null,
      focus: "Quickness",
    },
    {
      playerUid: 79,
      name: "Veteran Keeper",
      position: "GK",
      inPossessionRole: "Goalkeeper",
      outOfPossessionRole: "Line-Holding Keeper",
      focus: null,
    },
  ],
  unavailableReason: null,
};

function renderTraining(
  training: SquadFmTraining,
  result: SquadFmTrainingResult | null = null,
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
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      <SquadFmTrainingButton activeSaveId={1} disabled={false} />
    </QueryClientProvider>,
  );
  return applied;
}

describe("Apply training in FM", () => {
  it("lists each player's training and sets only the ticked players'", async () => {
    const user = userEvent.setup();
    const applied = renderTraining(plan, {
      applied: [77],
      failed: [
        {
          playerUid: 79,
          name: "Veteran Keeper",
          reason:
            "FM did not take the out-of-possession role (FM shows Goalkeeper)",
        },
      ],
      error: null,
    });

    await user.click(
      screen.getByRole("button", { name: "Apply training in FM" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Apply training in FM",
    });
    const row = (name: string) =>
      within(dialog)
        .getByRole("rowheader", { name })
        .closest("tr") as HTMLElement;
    expect(row("Slot Winger")).toHaveTextContent(
      "Winger AMRTracking Wide MidfielderBall Control",
    );
    expect(
      within(row("Unplaced Prospect")).getAllByText("Unchanged"),
    ).toHaveLength(2);
    expect(
      within(row("Veteran Keeper")).getByText("Cleared"),
    ).toBeInTheDocument();

    await user.click(
      within(dialog).getByRole("checkbox", {
        name: "Set training for Unplaced Prospect",
      }),
    );
    await user.click(
      within(dialog).getByRole("button", {
        name: "Set training for 2 players",
      }),
    );

    expect(applied).toEqual([{ playerUids: [77, 79] }]);
    expect(await within(dialog).findByRole("status")).toHaveTextContent(
      "FM took the training for 1 player.",
    );
    expect(
      within(dialog).getByText(
        "Veteran Keeper: FM did not take the out-of-possession role (FM shows Goalkeeper)",
      ),
    ).toBeInTheDocument();
  });

  it("explains why the training cannot be set and blocks it", async () => {
    const user = userEvent.setup();
    const applied = renderTraining({
      ...plan,
      unavailableReason:
        "Update the bridge plugin from Settings, restart FM and Load Data to set training in FM",
    });

    await user.click(
      screen.getByRole("button", { name: "Apply training in FM" }),
    );
    const dialog = await screen.findByRole("dialog");

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Update the bridge plugin from Settings",
    );
    expect(
      within(dialog).getByRole("button", {
        name: "Set training for 3 players",
      }),
    ).toBeDisabled();
    expect(applied).toEqual([]);
  });
});
