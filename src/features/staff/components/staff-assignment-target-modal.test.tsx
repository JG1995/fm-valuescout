import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixtureStaffAssignmentTargets,
  getLastStaffAssignmentTargetsIpcArgs,
  setStaffAssignmentTargetsIpcMock,
  setStaffAssignmentTargetsIpcMockMode,
} from "@/testing/staff-ipc-mock";
import { StaffAssignmentTargetModal } from "./staff-assignment-target-modal";

const contextA = {
  saveId: 1,
  saveContextToken: "save-token-a",
  snapshotId: 1,
  snapshotContextToken: "snapshot-token-a",
};
const contextB = {
  ...contextA,
  snapshotContextToken: "snapshot-token-b",
};

function renderModal(context = contextA, contextKey = "context-a") {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const result = render(
    <QueryClientProvider client={queryClient}>
      <StaffAssignmentTargetModal context={context} contextKey={contextKey} />
    </QueryClientProvider>,
  );

  return {
    ...result,
    rerenderModal(nextContext = contextB, nextContextKey = "context-b") {
      result.rerender(
        <QueryClientProvider client={queryClient}>
          <StaffAssignmentTargetModal
            context={nextContext}
            contextKey={nextContextKey}
          />
        </QueryClientProvider>,
      );
    },
  };
}

function needsTables(dialog: HTMLElement) {
  return {
    teamJobs: within(dialog).getByRole("table", { name: "Team jobs" }),
    clubJobs: within(dialog).getByRole("table", { name: "Club-wide jobs" }),
  };
}

describe("StaffAssignmentTargetModal", () => {
  it("lays out team jobs by team, club-wide jobs once, and saves the complete payload", async () => {
    const user = userEvent.setup();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams[1] = { ...targets.teams[1], displayName: "B Squad" };
    setStaffAssignmentTargetsIpcMock(targets);
    renderModal();

    const trigger = await screen.findByRole("button", {
      name: "Configure staffing needs",
    });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    const { teamJobs, clubJobs } = needsTables(dialog);

    expect(
      within(teamJobs)
        .getAllByRole("columnheader")
        .map((header) => header.textContent),
    ).toEqual(["Job", "Senior", "B Squad", "Youth"]);
    expect(
      within(teamJobs)
        .getAllByRole("rowheader")
        .map((header) => header.textContent),
    ).toEqual([
      "Coaching",
      "Manager",
      "Assistant Manager",
      "Coaches",
      "Performance Analyst",
      "Medical",
      "Physio",
      "Sports Scientist",
      "Total",
    ]);
    expect(
      within(teamJobs).getByRole("spinbutton", {
        name: "Senior Assistant Manager",
      }),
    ).toHaveAttribute("max", "50");
    expect(
      within(teamJobs).getByRole("spinbutton", { name: "B Squad Coaches" }),
    ).toBeInTheDocument();
    expect(
      within(teamJobs).queryByRole("spinbutton", { name: "Senior Manager" }),
    ).toBeNull();
    expect(
      within(teamJobs).getByTitle("You manage the Senior."),
    ).toHaveTextContent("You");
    expect(
      within(clubJobs)
        .getAllByRole("rowheader")
        .filter((header) =>
          ["Coaching", "Recruitment", "Medical"].includes(
            header.textContent ?? "",
          ),
        ),
    ).toHaveLength(3);
    expect(
      within(clubJobs).getByRole("spinbutton", {
        name: "Head of Youth Development",
      }),
    ).toHaveAttribute("max", "1");
    expect(
      within(clubJobs).getByRole("spinbutton", { name: "Set Piece Coach" }),
    ).toHaveAttribute("max", "1");
    expect(
      within(clubJobs).getByRole("spinbutton", {
        name: "Recruitment Analyst",
      }),
    ).toHaveAttribute("max", "50");
    const chiefScout = within(clubJobs).getByRole("spinbutton", {
      name: "Chief Scout",
    });
    const scout = within(clubJobs).getByRole("spinbutton", { name: "Scout" });
    expect(
      chiefScout.compareDocumentPosition(scout) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).not.toBe(0);
    expect(
      within(dialog).queryByRole("spinbutton", { name: /Doctor/ }),
    ).toBeNull();
    expect(within(dialog).getAllByRole("spinbutton")).toHaveLength(28);

    const headOfYouthDevelopment = within(clubJobs).getByRole("spinbutton", {
      name: "Head of Youth Development",
    });
    const coaches = within(teamJobs).getByRole("spinbutton", {
      name: "Senior Coaches",
    });
    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "1");
    await user.clear(coaches);
    await user.type(coaches, "50");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));

    await waitFor(() => {
      const request = getLastStaffAssignmentTargetsIpcArgs() as {
        targets?: { scope: string; jobId: string; slotCount: number }[];
      };
      expect(request).toEqual({
        expectedSaveContextToken: "save-token-a",
        targets: expect.arrayContaining([
          {
            scope: "club",
            jobId: "head_of_youth_development",
            slotCount: 1,
          },
          { scope: "senior", jobId: "coaches", slotCount: 50 },
        ]),
      });
      expect(request.targets).toHaveLength(28);
      expect(
        new Set(
          request.targets?.map((target) => `${target.scope}:${target.jobId}`),
        ).size,
      ).toBe(28);
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Staffing needs saved.",
    );
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("shows only the enabled teams when Senior is disabled", async () => {
    const user = userEvent.setup();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams = targets.teams.filter(({ team }) => team !== "senior");
    targets.targets = targets.targets.filter(({ scope }) => scope !== "senior");
    setStaffAssignmentTargetsIpcMock(targets);
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const { teamJobs, clubJobs } = needsTables(
      await screen.findByRole("dialog"),
    );

    expect(
      within(teamJobs)
        .getAllByRole("columnheader")
        .map((header) => header.textContent),
    ).toEqual(["Job", "Reserves", "Youth"]);
    expect(
      within(clubJobs).getAllByRole("spinbutton", {
        name: "Head of Youth Development",
      }),
    ).toHaveLength(1);
  });

  it("steps counts within each maximum and totals every team", async () => {
    const user = userEvent.setup();
    setStaffAssignmentTargetsIpcMock(
      fixtureStaffAssignmentTargets({
        targets: fixtureStaffAssignmentTargets().targets.map((target) =>
          target.jobId === "assistant_manager" && target.scope === "senior"
            ? { ...target, slotCount: 2 }
            : target,
        ),
      }),
    );
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog");
    const { teamJobs, clubJobs } = needsTables(dialog);
    const boundedInput = within(clubJobs).getByRole("spinbutton", {
      name: "Head of Youth Development",
    });
    const boundedDecrease = within(clubJobs).getByRole("button", {
      name: "Decrease Head of Youth Development",
    });
    const boundedIncrease = within(clubJobs).getByRole("button", {
      name: "Increase Head of Youth Development",
    });
    expect(boundedIncrease).toBeDisabled();
    await user.click(boundedDecrease);
    expect(boundedInput).toHaveValue(0);
    expect(boundedDecrease).toBeDisabled();
    await user.click(boundedIncrease);
    expect(boundedInput).toHaveValue(1);

    const assistantManager = within(teamJobs).getByRole("spinbutton", {
      name: "Senior Assistant Manager",
    });
    await user.click(
      within(teamJobs).getByRole("button", {
        name: "Increase Senior Assistant Manager",
      }),
    );
    expect(assistantManager).toHaveValue(3);
    const teamTotals = () =>
      within(within(teamJobs).getByRole("row", { name: /^Total/ }))
        .getAllByRole("cell")
        .map((cell) => cell.textContent);
    expect(teamTotals()).toEqual(["7", "6", "6"]);
    expect(
      within(within(clubJobs).getByRole("row", { name: /^Total/ })).getByRole(
        "cell",
      ),
    ).toHaveTextContent("11");

    await user.clear(assistantManager);
    await user.type(assistantManager, "not-a-number");
    expect(teamTotals()).toEqual(["—", "6", "6"]);
    expect(assistantManager).toHaveAccessibleDescription(
      "Senior Assistant Manager: Enter a whole number from 0 to 50.",
    );
  });

  it("shows FM's staff limit beside each need that differs from it", async () => {
    const user = userEvent.setup();
    setStaffAssignmentTargetsIpcMock(
      fixtureStaffAssignmentTargets({
        fmLimits: [
          { scope: "senior", jobId: "coaches", limit: 1 },
          { scope: "senior", jobId: "physio", limit: 4 },
        ],
      }),
    );
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog");
    const { teamJobs } = needsTables(dialog);
    expect(
      within(teamJobs).getByRole("button", { name: "About FM's staff limits" }),
    ).toHaveAccessibleDescription(
      "Load Data resets a job to FM's staff limit whenever that limit changes.",
    );
    expect(
      within(teamJobs).getByRole("spinbutton", { name: "Senior Physio" }),
    ).toHaveAccessibleDescription("FM's staff limit is 4.");
    const coaches = within(teamJobs).getByRole("spinbutton", {
      name: "Senior Coaches",
    });
    expect(coaches).not.toHaveAccessibleDescription();
    expect(within(teamJobs).getAllByText(/^FM \d+$/)).toHaveLength(1);

    await user.click(
      within(teamJobs).getByRole("button", { name: "Increase Senior Coaches" }),
    );
    expect(coaches).toHaveAccessibleDescription("FM's staff limit is 1.");
    expect(within(teamJobs).getAllByText(/^FM \d+$/)).toHaveLength(2);
  });

  it("uses each Rust maximum for local validation and shows a Rust save error", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog");
    const headOfYouthDevelopment = within(dialog).getByRole("spinbutton", {
      name: "Head of Youth Development",
    });
    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "2");

    expect(headOfYouthDevelopment).toHaveAttribute("aria-invalid", "true");
    expect(
      within(dialog).getByText(
        "Head of Youth Development: Enter a whole number from 0 to 1.",
      ),
    ).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Save" })).toBeDisabled();

    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "1");
    setStaffAssignmentTargetsIpcMockMode("error");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Could not save assignment targets",
    );
  });

  it("locks the form while a save is pending", async () => {
    const user = userEvent.setup();
    setStaffAssignmentTargetsIpcMockMode("pending");
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));

    expect(
      within(dialog).getByRole("button", { name: "Saving…" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toBeDisabled();
    expect(within(dialog).getAllByRole("spinbutton")[0]).toBeDisabled();
    for (const control of within(dialog).getAllByRole("button", {
      name: /^(Decrease|Increase) /,
    })) {
      expect(control).toBeDisabled();
    }
  });

  it("closes and discards a draft when the immutable context changes", async () => {
    const user = userEvent.setup();
    const { rerenderModal } = renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog");
    const assistantManager = within(dialog).getByRole("spinbutton", {
      name: "Senior Assistant Manager",
    });
    await user.clear(assistantManager);
    await user.type(assistantManager, "12");

    rerenderModal();
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    expect(
      screen.getByRole("spinbutton", { name: "Senior Assistant Manager" }),
    ).toHaveValue(1);
  });
});
