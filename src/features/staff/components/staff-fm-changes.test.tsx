import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type {
  StaffFmChangeResult,
  StaffFmChanges as StaffFmChangesPlan,
} from "../types/staff-fm-changes";
import { StaffFmChanges } from "./staff-fm-changes";

const context = {
  saveId: 1,
  saveContextToken: "save-token",
  snapshotId: 1,
  snapshotContextToken: "snapshot-token",
};

const plan: StaffFmChangesPlan = {
  changes: [
    {
      uid: 1,
      name: "Old Assistant",
      kind: "sack",
      currentJob: "Assistant Manager",
      currentTeamName: "First Team",
      currentClub: null,
      weeklyWageGbp: 4000,
      target: null,
    },
    {
      uid: 2,
      name: "Second Assistant",
      kind: "change",
      currentJob: "Assistant Manager",
      currentTeamName: "First Team",
      currentClub: null,
      weeklyWageGbp: 3000,
      target: {
        scope: "reserves",
        scopeDisplayName: "U21",
        jobLabel: "Assistant Manager",
        level: "reserve",
      },
    },
    {
      uid: 4,
      name: "Better Assistant",
      kind: "hire",
      currentJob: null,
      currentTeamName: null,
      currentClub: "Other FC",
      weeklyWageGbp: 9000,
      target: {
        scope: "senior",
        scopeDisplayName: "First Team",
        jobLabel: "Assistant Manager",
        level: "senior",
      },
    },
    {
      uid: 5,
      name: null,
      kind: "hire",
      currentJob: null,
      currentTeamName: null,
      currentClub: null,
      weeklyWageGbp: null,
      target: {
        scope: "club",
        scopeDisplayName: "Club",
        jobLabel: "Chief Scout",
        level: "senior",
      },
    },
  ],
  unavailableReason: null,
};

function renderChanges(
  changes: StaffFmChangesPlan,
  result: StaffFmChangeResult | null = null,
) {
  const applied: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "get_staff_fm_changes") return changes;
    if (cmd === "apply_staff_fm_changes") {
      applied.push(args);
      return result;
    }
    return null;
  });
  const onApplied = vi.fn();
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      <StaffFmChanges
        context={context}
        contextKey="1:1"
        open
        onClose={() => {}}
        onApplied={onApplied}
      />
    </QueryClientProvider>,
  );
  return { applied, onApplied };
}

function row(dialog: HTMLElement, name: string) {
  return within(dialog)
    .getByRole("rowheader", { name })
    .closest("tr") as HTMLElement;
}

describe("Apply staff changes in FM", () => {
  it("describes each change, applies the ticked ones, and reports FM's contracts", async () => {
    const user = userEvent.setup();
    const { applied, onApplied } = renderChanges(plan, {
      done: [
        {
          uid: 1,
          name: "Old Assistant",
          kind: "sack",
          jobLabel: null,
          teamLevel: null,
          weeklyWageGbp: null,
        },
        {
          uid: 4,
          name: "Better Assistant",
          kind: "hire",
          jobLabel: "Assistant Manager",
          teamLevel: "senior",
          weeklyWageGbp: 8500,
        },
      ],
      failed: [
        {
          uid: 5,
          name: null,
          kind: "hire",
          reason:
            "FM did not confirm the hire; a single-holder job may still be taken",
        },
      ],
      error: null,
    });

    const dialog = await screen.findByRole("dialog", {
      name: "Apply staff changes in FM",
    });
    const sack = await within(dialog).findByRole("rowheader", {
      name: "Old Assistant",
    });
    expect(sack.closest("tr")).toHaveTextContent(
      "SackFirst Team Assistant ManagerLeaves the club€4k a week",
    );
    expect(row(dialog, "Second Assistant")).toHaveTextContent(
      "New jobFirst Team Assistant ManagerU21 Assistant Manager",
    );
    expect(row(dialog, "Better Assistant")).toHaveTextContent(
      "HireOther FCFirst Team Assistant Manager",
    );
    expect(row(dialog, "Staff 5")).toHaveTextContent("UnemployedChief Scout");

    await user.click(
      within(dialog).getByRole("checkbox", {
        name: "New job Second Assistant",
      }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Apply 3 changes" }),
    );

    expect(applied).toEqual([
      {
        expectedSaveContextToken: "save-token",
        expectedSnapshotContextToken: "snapshot-token",
        staffUids: [1, 4, 5],
      },
    ]);
    expect(await within(dialog).findByRole("status")).toHaveTextContent(
      "FM made 2 changes.",
    );
    expect(
      within(dialog).getByText("Old Assistant: left the club"),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "Better Assistant: Assistant Manager, First team, €9k a week",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "Staff 5: FM did not confirm the hire; a single-holder job may still be taken",
      ),
    ).toBeInTheDocument();
    expect(onApplied).toHaveBeenCalled();
  });

  it("explains why the changes cannot be made and blocks them", async () => {
    renderChanges({
      ...plan,
      unavailableReason:
        "Load Data with FM open on this save before changing staff in FM",
    });

    const dialog = await screen.findByRole("dialog");

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Load Data with FM open on this save before changing staff in FM",
    );
    expect(
      within(dialog).getByRole("button", { name: "Apply 4 changes" }),
    ).toBeDisabled();
  });

  it("says so when FM already matches the recommendations", async () => {
    renderChanges({ changes: [], unavailableReason: null });

    const dialog = await screen.findByRole("dialog");

    expect(
      await within(dialog).findByText(
        "FM already matches the recommendations.",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Apply 0 changes" }),
    ).toBeDisabled();
  });
});
