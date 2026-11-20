import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  fixtureStaffAssignmentOptimization,
  fixtureStaffAssignmentTargets,
  getLastStaffAssignmentOptimizerIpcArgs,
  getStaffAssignmentOptimizerIpcCallCount,
  getStaffAssignmentSkipIpcCalls,
  resolvePendingStaffAssignmentOptimizationIpcMock,
  resolvePendingStaffAssignmentSkipIpcMock,
  resolvePendingStaffAssignmentTargetsIpcMock,
  setStaffAssignmentOptimizationIpcMock,
  setStaffAssignmentOptimizerIpcMockMode,
  setStaffAssignmentSkipIpcError,
  setStaffAssignmentSkipIpcPending,
  setStaffAssignmentTargetsIpcMock,
  setStaffAssignmentTargetsIpcMockMode,
} from "@/testing/staff-ipc-mock";
import type { StaffAssignmentRecommendation } from "../types/staff-assignment";
import { StaffAssignmentOptimizer } from "./staff-assignment-optimizer";

const context = {
  saveId: 1,
  saveContextToken: "save-token-1",
  snapshotId: 1,
  snapshotContextToken: "snapshot-token-1",
};

function renderOptimizer(
  contextUnavailable = false,
  contextKey = "assignment-context-a",
  zeroSlots = false,
  onReviewShortlist = vi.fn(),
  onOpenStaff = vi.fn(),
) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const targets = fixtureStaffAssignmentTargets();
  targets.targets = targets.targets.map((target) => ({
    ...target,
    slotCount: zeroSlots ? 0 : 1,
  }));
  queryClient.setQueryData(
    ["staff", "assignment-targets", contextKey],
    targets,
  );
  setStaffAssignmentTargetsIpcMock(targets);
  const result = render(
    <QueryClientProvider client={queryClient}>
      <StaffAssignmentOptimizer
        context={context}
        contextKey={contextKey}
        contextUnavailable={contextUnavailable}
        shortlistReady={true}
        onReviewShortlist={onReviewShortlist}
        onOpenStaff={onOpenStaff}
      />
    </QueryClientProvider>,
  );

  return {
    ...result,
    rerenderOptimizer(
      nextUnavailable: boolean,
      nextContextKey = contextKey,
      nextContext = context,
    ) {
      result.rerender(
        <QueryClientProvider client={queryClient}>
          <StaffAssignmentOptimizer
            context={nextContext}
            contextKey={nextContextKey}
            contextUnavailable={nextUnavailable}
            shortlistReady={true}
            onReviewShortlist={onReviewShortlist}
            onOpenStaff={onOpenStaff}
          />
        </QueryClientProvider>,
      );
    },
  };
}

describe("StaffAssignmentOptimizer", () => {
  it("keeps modal feedback in the stable status region outside the action row", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Configure staffing needs" }),
    );
    await user.click(
      within(await screen.findByRole("dialog")).getByRole("button", {
        name: "Save",
      }),
    );

    const actionRow = screen.getByTestId("assignment-action-row");
    expect(within(actionRow).queryByText("Staffing needs saved.")).toBeNull();
    expect(
      await screen.findByText("Staffing needs saved."),
    ).toBeInTheDocument();
    expect(screen.getByTestId("assignment-status-region")).toHaveTextContent(
      "Staffing needs saved.",
    );
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(0);
  });

  it("describes the saved staffing needs beside Optimize", async () => {
    renderOptimizer();

    const optimize = await screen.findByRole("button", {
      name: "Optimize assignments",
    });
    expect(optimize).toBeEnabled();
    expect(optimize).toHaveAccessibleDescription("Staffing needs: 28 slots.");
  });

  it("opens staffing needs from Optimize when none are set, then optimizes after saving", async () => {
    const user = userEvent.setup();
    renderOptimizer(false, "assignment-context-a", true);

    const optimize = await screen.findByRole("button", {
      name: "Optimize assignments",
    });
    expect(optimize).toBeEnabled();
    expect(optimize).toHaveAccessibleDescription("No staffing needs set yet.");
    await user.click(optimize);
    const staffing = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(0);
    const assistantManager = within(staffing).getByRole("spinbutton", {
      name: "Senior Assistant Manager",
    });
    await user.clear(assistantManager);
    await user.type(assistantManager, "1");
    await user.click(within(staffing).getByRole("button", { name: "Save" }));

    expect(
      await screen.findByRole("dialog", { name: "Assignment recommendations" }),
    ).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
  });

  it("does not optimize when staffing needs opened from Optimize are saved as zero", async () => {
    const user = userEvent.setup();
    renderOptimizer(false, "assignment-context-a", true);

    await user.click(
      await screen.findByRole("button", { name: "Optimize assignments" }),
    );
    const staffing = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    await user.click(within(staffing).getByRole("button", { name: "Save" }));

    expect(
      await screen.findByText("Staffing needs saved."),
    ).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(0);
    expect(
      screen.queryByRole("dialog", { name: "Assignment recommendations" }),
    ).toBeNull();
  });

  it("sends only immutable tokens and says what each recommendation needs", async () => {
    const user = userEvent.setup();
    const onReviewShortlist = vi.fn();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        joinedCandidateCount: 6,
        slots: [
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "Alpha Unit",
            jobId: "coaches",
            jobLabel: "Coaches",
            slotNumber: 1,
            uid: 101,
            name: "Alex Coach",
            preferredJob: "Coach",
            jobSource: "preferred_job",
            action: { kind: "keep" },
            score: 82,
            coachRequirement: "attacking_technical",
            replaces: null,
          },
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "Alpha Unit",
            jobId: "coaches",
            jobLabel: "Coaches",
            slotNumber: 2,
            uid: 102,
            name: "Riley Recruit",
            preferredJob: "Coach",
            jobSource: "preferred_job",
            action: { kind: "recruit", club: "Valencia" },
            score: 79,
            coachRequirement: null,
            replaces: null,
          },
          {
            kind: "vacancy",
            scope: "club",
            scopeDisplayName: "Club Services",
            jobId: "coaches",
            jobLabel: "Coaches",
            slotNumber: 3,
            coachRequirement: "goalkeeping",
            evidence: {
              jobId: "coaches",
              joinedCandidateCount: 2,
              eligibleScoreCount: 0,
              unavailableScoreCount: 2,
            },
          },
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "Alpha Unit",
            jobId: "assistant_manager",
            jobLabel: "Assistant Manager",
            slotNumber: 4,
            uid: 103,
            name: "Taylor Coach",
            preferredJob: "Assistant Manager",
            jobSource: "preferred_job",
            action: {
              kind: "change_job",
              currentJob: "Coach",
              currentTeam: null,
            },
            score: 75,
            coachRequirement: null,
            replaces: null,
          },
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "Alpha Unit",
            jobId: "physio",
            jobLabel: "Physio",
            slotNumber: 1,
            uid: 104,
            name: "Jo Free",
            preferredJob: "Physio",
            jobSource: "preferred_job",
            action: { kind: "recruit", club: null },
            score: 70,
            coachRequirement: null,
            replaces: null,
          },
        ],
      }),
    );
    renderOptimizer(false, "assignment-context-a", false, onReviewShortlist);

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    await waitFor(() =>
      expect(getLastStaffAssignmentOptimizerIpcArgs()).toEqual({
        expectedSaveContextToken: "save-token-1",
        expectedSnapshotContextToken: "snapshot-token-1",
      }),
    );
    const table = await screen.findByRole("table", {
      name: "Staff assignment recommendations and vacancies",
    });
    expect(
      within(table)
        .getAllByRole("columnheader")
        .map((header) => header.textContent),
    ).toEqual(["Slot", "Person", "Score", "Action", "Skip"]);
    const resultRows = table.querySelectorAll("tbody tr");
    expect(Array.from(resultRows).map((row) => row.textContent)).toEqual([
      "Alpha Unit: 2 of 2 filled",
      expect.stringContaining("Alex Coach"),
      expect.stringContaining("Riley Recruit"),
      "Club Services: 0 of 1 filled",
      expect.stringContaining("Unfilled"),
      "Alpha Unit: 2 of 2 filled",
      expect.stringContaining("Taylor Coach"),
      expect.stringContaining("Jo Free"),
    ]);
    const row = (name: RegExp) => within(table).getByRole("row", { name });
    expect(row(/Alex Coach/)).toHaveTextContent(
      /^Coaches 1Attacking TechnicalAlex Coach82Keep/,
    );
    expect(row(/Riley Recruit/)).toHaveTextContent(
      /^Coaches 2Riley Recruit79RecruitFrom Valencia/,
    );
    expect(row(/Taylor Coach/)).toHaveTextContent(
      /^Assistant ManagerTaylor Coach75Change jobCurrently Coach/,
    );
    expect(row(/Jo Free/)).toHaveTextContent(/RecruitUnemployed/);
    expect(row(/Unfilled/)).toHaveTextContent(
      "CoachesGoalkeepingUnfilled2 people fit this job, but their ratings are missing.",
    );
    expect(
      screen.getByRole("img", { name: /Coaches: 82, Excellent/i }),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Eligible|eligible score|evidence/i)).toBeNull();
    expect(
      screen
        .getAllByRole("term")
        .map((term) => [
          term.textContent,
          term.nextElementSibling?.textContent,
        ]),
    ).toEqual([
      ["Filled", "4 of 5"],
      ["Your staff", "2"],
      ["Recruit", "2"],
      ["Unfilled", "1"],
    ]);
    expect(
      screen.getByText("Picked from 6 people on your shortlist and staff."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "1 slot has no suitable person. Lower your staffing needs, or shortlist more staff in FM.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "1 person on your shortlist wants a job that no slot here covers, so they were left out.",
      ),
    ).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);

    await user.click(
      screen.getByRole("button", { name: "Adjust staffing needs" }),
    );
    const staffingDialog = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Assignment recommendations" }),
      ).toBeNull(),
    );
    expect(staffingDialog).toContainElement(
      document.activeElement as HTMLElement,
    );
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Configure staffing needs" }),
      ).toHaveFocus(),
    );

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Review shortlist" }),
    );
    expect(onReviewShortlist).toHaveBeenCalledOnce();
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Assignment recommendations" }),
      ).toBeNull(),
    );
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(2);
  });

  it("opens recommendations in a dialog that returns focus to Optimize when closed", async () => {
    const user = userEvent.setup();
    renderOptimizer();
    const optimize = screen.getByRole("button", {
      name: "Optimize assignments",
    });

    await user.click(optimize);
    const dialog = await screen.findByRole("dialog", {
      name: "Assignment recommendations",
    });
    expect(within(dialog).getByText("Alex Coach")).toBeVisible();
    expect(
      within(dialog).getByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).toBeInTheDocument();

    await user.keyboard("{Escape}");

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(optimize).toHaveFocus());
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
  });

  it("opens a newly accepted result after the prior dialog closed", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await user.click(
      within(
        await screen.findByRole("dialog", {
          name: "Assignment recommendations",
        }),
      ).getByRole("button", { name: "Close" }),
    );
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "Senior",
            jobId: "assistant_manager",
            jobLabel: "Assistant Manager",
            slotNumber: 1,
            uid: 102,
            name: "Jordan Assistant",
            preferredJob: "Assistant Manager",
            jobSource: "preferred_job",
            action: { kind: "keep" },
            score: 81,
            coachRequirement: null,
            replaces: null,
          },
        ],
      }),
    );

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Assignment recommendations",
    });
    expect(within(dialog).getByText("Jordan Assistant")).toBeVisible();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(2);
  });

  it("names a coach slot's canonical requirement under the job", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "First Team",
            jobId: "coaches",
            jobLabel: "Coaches",
            slotNumber: 1,
            uid: 101,
            name: "Fit Coach",
            preferredJob: "Fitness Coach",
            jobSource: "preferred_job",
            action: { kind: "keep" },
            score: 82,
            coachRequirement: "fitness",
            replaces: null,
          },
        ],
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    const row = await screen.findByRole("row", { name: /Fit Coach/ });
    expect(within(row).getAllByRole("cell")[0]).toHaveTextContent(
      "CoachesFitness",
    );
  });

  it("renders an em dash instead of a blank person when a recommendation name is missing", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [
          {
            kind: "recommendation",
            scope: "senior",
            scopeDisplayName: "First Team",
            jobId: "assistant_manager",
            jobLabel: "Assistant Manager",
            slotNumber: 1,
            uid: 101,
            name: null,
            preferredJob: "Assistant Manager",
            jobSource: "preferred_job",
            action: { kind: "keep" },
            score: 82,
            coachRequirement: null,
            replaces: null,
          },
        ],
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    const row = await screen.findByRole("row", {
      name: /^Assistant Manager/,
    });
    expect(within(row).getByText("—")).not.toHaveAttribute("title");
    expect(within(row).queryByRole("button", { name: "—" })).toBeNull();
    expect(within(row).queryByText("0")).not.toBeInTheDocument();
  });

  it("suppresses a visible recommendation immediately while context is unavailable", async () => {
    const user = userEvent.setup();
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(await screen.findByText("Alex Coach")).toBeInTheDocument();

    rerenderOptimizer(true);

    expect(screen.queryByText("Alex Coach")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeDisabled();
  });

  it("clears a recommendation after saving slot targets", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(
      await screen.findByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).toBeInTheDocument();

    setStaffAssignmentOptimizerIpcMockMode("pending");
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    setStaffAssignmentTargetsIpcMockMode("pending");
    await user.click(
      screen.getByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    resolvePendingStaffAssignmentOptimizationIpcMock();

    await waitFor(() =>
      expect(
        screen.queryByRole("table", {
          name: "Staff assignment recommendations and vacancies",
        }),
      ).not.toBeInTheDocument(),
    );
    resolvePendingStaffAssignmentTargetsIpcMock();
  });

  it("keeps Optimize blocked when an old-context target save settles after a current one starts", async () => {
    const user = userEvent.setup();
    setStaffAssignmentTargetsIpcMockMode("pending");
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeDisabled();

    rerenderOptimizer(false, "assignment-context-b", {
      ...context,
      snapshotContextToken: "snapshot-token-b",
    });
    await user.click(
      await screen.findByRole("button", { name: "Configure staffing needs" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    await user.click(within(dialog).getByRole("button", { name: "Save" }));

    resolvePendingStaffAssignmentTargetsIpcMock();

    await waitFor(() =>
      expect(
        within(dialog).getByRole("button", { name: "Saving…" }),
      ).toBeDisabled(),
    );
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeDisabled();

    resolvePendingStaffAssignmentTargetsIpcMock();
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Optimize assignments" }),
      ).not.toBeDisabled(),
    );
  });

  it("rejects a response for a different immutable snapshot token", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        snapshotContextToken: "replacement-token",
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    await waitFor(() =>
      expect(getLastStaffAssignmentOptimizerIpcArgs()).toEqual(
        expect.objectContaining({
          expectedSnapshotContextToken: "snapshot-token-1",
        }),
      ),
    );
    expect(screen.queryByText("Alex Coach")).not.toBeInTheDocument();
  });

  it("rejects no-managed-club guidance with mismatched returned tokens", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "no_managed_club",
        saveContextToken: "replacement-save-token",
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    await waitFor(() =>
      expect(
        screen.queryByText(
          "Choose a managed club before optimizing assignments.",
        ),
      ).not.toBeInTheDocument(),
    );
  });

  it("presents a Rust stale-context response without its result rows", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "stale_context",
        saveContextToken: "replacement-save-token",
        snapshotId: null,
        snapshotContextToken: null,
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Your save changed while optimizing. Optimize again.",
    );
    expect(
      screen.queryByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).not.toBeInTheDocument();
  });

  it("accepts no-current-snapshot guidance only for its current save and no snapshot token", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "no_current_snapshot",
        snapshotId: null,
        snapshotContextToken: null,
      }),
    );
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "This save has no data yet. Use Load Data first.",
    );
    expect(
      screen.queryByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).not.toBeInTheDocument();

    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "no_current_snapshot",
        saveContextToken: "replacement-save-token",
        snapshotId: null,
        snapshotContextToken: null,
      }),
    );
    rerenderOptimizer(false, "assignment-context-b");
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByText("No current snapshot is available"),
      ).not.toBeInTheDocument(),
    );

    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "no_current_snapshot",
        snapshotContextToken: "replacement-token",
      }),
    );
    rerenderOptimizer(false, "assignment-context-c");
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByText("No current snapshot is available"),
      ).not.toBeInTheDocument(),
    );
  });

  it("rejects a delayed result after a same-ID replacement token", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizerIpcMockMode("pending");
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    rerenderOptimizer(false, "assignment-context-replacement", {
      ...context,
      snapshotContextToken: "snapshot-token-replacement",
    });
    resolvePendingStaffAssignmentOptimizationIpcMock();

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Optimize assignments" }),
      ).not.toBeDisabled(),
    );
    expect(
      screen.queryByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).not.toBeInTheDocument();
  });

  it("shows a Rust setup state and mutation error without a prior result", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        state: "no_shortlist",
        snapshotContextToken: "replacement-token",
      }),
    );
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByText(/Shortlist staff in FM/),
      ).not.toBeInTheDocument(),
    );

    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({ state: "no_shortlist" }),
    );
    rerenderOptimizer(false, "assignment-context-b");
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Shortlist staff in FM, save the game, then use Load Data.",
    );

    rerenderOptimizer(false, "assignment-context-c");
    setStaffAssignmentOptimizerIpcMockMode("error");
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not optimize staff assignments",
    );
  });

  function reservesManager(
    uid: number,
    name: string,
    overrides: Partial<StaffAssignmentRecommendation> = {},
  ): StaffAssignmentRecommendation {
    return {
      kind: "recommendation",
      scope: "reserves",
      scopeDisplayName: "Reserves",
      jobId: "manager",
      jobLabel: "Manager",
      slotNumber: 1,
      uid,
      name,
      preferredJob: "Manager",
      jobSource: "preferred_job",
      action: { kind: "recruit", club: null },
      score: 90,
      coachRequirement: null,
      replaces: null,
      ...overrides,
    };
  }

  it("opens a recommended person's Staff Profile and closes the results", async () => {
    const user = userEvent.setup();
    const onOpenStaff = vi.fn();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [
          reservesManager(102, "Darren Fletcher", {
            action: { kind: "on_staff" },
          }),
        ],
      }),
    );
    renderOptimizer(false, "assignment-context-a", false, vi.fn(), onOpenStaff);

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    const row = await screen.findByRole("row", { name: /Darren Fletcher/ });
    expect(row).toHaveTextContent("Your staff");
    await user.click(
      within(row).getByRole("button", { name: "Darren Fletcher" }),
    );

    expect(onOpenStaff).toHaveBeenCalledWith(102);
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("names each newcomer's team move and whom they replace", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        teamNames: { senior: "First Team", reserves: "U21", youth: "U18" },
        slots: [
          reservesManager(301, "Mover Manager", {
            action: { kind: "move_team", currentTeam: "youth" },
            replaces: {
              uid: 303,
              name: "Sacked Manager",
              movesTo: null,
            },
          }),
          reservesManager(302, "Recruited Manager", {
            scope: "youth",
            scopeDisplayName: "U18",
            replaces: {
              uid: 301,
              name: "Mover Manager",
              movesTo: { scope: "reserves", job: "Manager" },
            },
          }),
          reservesManager(304, "Scout Turned Manager", {
            scope: "senior",
            scopeDisplayName: "First Team",
            action: {
              kind: "change_job",
              currentJob: "Coach",
              currentTeam: "reserves",
            },
          }),
        ],
        surplus: [
          {
            uid: 303,
            name: "Sacked Manager",
            currentJob: "Manager",
            currentTeam: "reserves",
            replacedBy: { uid: 301, name: "Mover Manager" },
          },
        ],
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    const rowOf = (name: string) =>
      screen.getByRole("button", { name }).closest("tr");

    await screen.findByRole("button", { name: "Mover Manager" });
    expect(rowOf("Mover Manager")).toHaveTextContent(
      "Move teamCurrently U18Replaces Sacked Manager",
    );
    expect(rowOf("Recruited Manager")).toHaveTextContent(
      "UnemployedReplaces Mover Manager, who moves to U21 Manager",
    );
    expect(rowOf("Scout Turned Manager")).toHaveTextContent(
      "Change jobCurrently U21 CoachAdds a place",
    );
    expect(
      screen.getByRole("region", { name: "Staff to sack (1)" }),
    ).toHaveTextContent("Sacked ManagerU21 ManagerReplaced by Mover Manager");
  });

  it("lists the club staff to sack and opens their Staff Profiles", async () => {
    const user = userEvent.setup();
    const onOpenStaff = vi.fn();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        surplus: [
          {
            uid: 201,
            name: "Spare Scout",
            currentJob: "Scout",
            currentTeam: null,
            replacedBy: null,
          },
          {
            uid: 202,
            name: null,
            currentJob: null,
            currentTeam: null,
            replacedBy: null,
          },
        ],
      }),
    );
    renderOptimizer(false, "assignment-context-a", false, vi.fn(), onOpenStaff);

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    const surplus = await screen.findByRole("region", {
      name: "Staff to sack (2)",
    });
    expect(
      within(surplus)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["Spare ScoutScout", "Staff 202"]);
    await user.click(
      within(surplus).getByRole("button", { name: "Spare Scout" }),
    );

    expect(onOpenStaff).toHaveBeenCalledWith(201);
  });

  it("optimizes again after staffing needs are adjusted from the results", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Adjust staffing needs" }),
    );
    const staffing = await screen.findByRole("dialog", {
      name: "Configure staffing needs",
    });
    await user.click(within(staffing).getByRole("button", { name: "Save" }));

    expect(
      await screen.findByRole("dialog", { name: "Assignment recommendations" }),
    ).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(2);
  });

  it("skips a recommendation, shows the next best, and restores the skipped person", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [reservesManager(101, "Fatih Terim")],
      }),
    );
    renderOptimizer();
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await screen.findByText("Fatih Terim");

    setStaffAssignmentOptimizerIpcMockMode("pending");
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [reservesManager(103, "Next Best", { score: 80 })],
        skipped: [{ uid: 101, name: "Fatih Terim" }],
      }),
    );
    await user.click(
      screen.getByRole("button", {
        name: "Skip Fatih Terim for Reserves Manager",
      }),
    );

    expect(getStaffAssignmentSkipIpcCalls()).toEqual([
      { expectedSaveContextToken: "save-token-1", uid: 101, skipped: true },
    ]);
    await waitFor(() =>
      expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(2),
    );
    expect(screen.getByText("Fatih Terim")).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Skip Fatih Terim for Reserves Manager",
      }),
    ).toBeDisabled();

    resolvePendingStaffAssignmentOptimizationIpcMock();

    expect(await screen.findByText("Next Best")).toBeInTheDocument();
    const skipped = screen.getByRole("region", { name: "Skipped staff" });
    expect(within(skipped).getByText("Fatih Terim")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Skip Fatih Terim/ }),
    ).toBeNull();

    setStaffAssignmentOptimizerIpcMockMode("success");
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [reservesManager(101, "Fatih Terim")],
      }),
    );
    await user.click(
      within(skipped).getByRole("button", { name: "Restore Fatih Terim" }),
    );

    expect(getStaffAssignmentSkipIpcCalls()).toEqual([
      { expectedSaveContextToken: "save-token-1", uid: 101, skipped: true },
      { expectedSaveContextToken: "save-token-1", uid: 101, skipped: false },
    ]);
    await waitFor(() =>
      expect(
        screen.queryByRole("region", { name: "Skipped staff" }),
      ).toBeNull(),
    );
    expect(screen.getByText("Fatih Terim")).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(3);
  });

  it("reports a failed skip without optimizing again", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [reservesManager(101, "Fatih Terim")],
      }),
    );
    renderOptimizer();
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await screen.findByText("Fatih Terim");

    setStaffAssignmentSkipIpcError("Save changed or no longer exists");
    await user.click(
      screen.getByRole("button", {
        name: "Skip Fatih Terim for Reserves Manager",
      }),
    );

    const dialog = screen.getByRole("dialog", {
      name: "Assignment recommendations",
    });
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Save changed or no longer exists",
    );
    expect(screen.getAllByRole("alert")).toHaveLength(1);
    expect(screen.getByText("Fatih Terim")).toBeInTheDocument();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
  });

  it("does not optimize again when a skip settles after the context refreshed", async () => {
    const user = userEvent.setup();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
        slots: [reservesManager(101, "Fatih Terim")],
      }),
    );
    const { rerenderOptimizer } = renderOptimizer();
    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await screen.findByText("Fatih Terim");

    setStaffAssignmentSkipIpcPending(true);
    await user.click(
      screen.getByRole("button", {
        name: "Skip Fatih Terim for Reserves Manager",
      }),
    );
    rerenderOptimizer(true);
    rerenderOptimizer(false);
    resolvePendingStaffAssignmentSkipIpcMock();

    await waitFor(() =>
      expect(getStaffAssignmentSkipIpcCalls()).toHaveLength(1),
    );
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
    expect(screen.queryByText("Fatih Terim")).toBeNull();
  });
});
