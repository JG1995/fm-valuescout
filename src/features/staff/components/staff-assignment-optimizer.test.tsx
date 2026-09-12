import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  fixtureStaffAssignmentOptimization,
  fixtureStaffAssignmentTargets,
  getLastStaffAssignmentOptimizerIpcArgs,
  getStaffAssignmentOptimizerIpcCallCount,
  resolvePendingStaffAssignmentOptimizationIpcMock,
  resolvePendingStaffAssignmentTargetsIpcMock,
  setStaffAssignmentOptimizationIpcMock,
  setStaffAssignmentOptimizerIpcMockMode,
  setStaffAssignmentTargetsIpcMockMode,
} from "@/testing/staff-ipc-mock";
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
  const result = render(
    <QueryClientProvider client={queryClient}>
      <StaffAssignmentOptimizer
        context={context}
        contextKey={contextKey}
        contextUnavailable={contextUnavailable}
        shortlistReady={true}
        onReviewShortlist={onReviewShortlist}
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
        name: "Save slots",
      }),
    );

    const actionRow = screen.getByTestId("assignment-action-row");
    expect(within(actionRow).queryByText("Slot counts saved.")).toBeNull();
    expect(await screen.findByText("Slot counts saved.")).toBeInTheDocument();
    expect(screen.getByTestId("assignment-status-region")).toHaveTextContent(
      "Slot counts saved.",
    );
  });
  it("disables Optimize assignments and explains how to configure zero slots", async () => {
    renderOptimizer(false, "assignment-context-a", true);

    const optimize = await screen.findByRole("button", {
      name: "Optimize assignments",
    });
    expect(optimize).toBeDisabled();
    const readiness = screen.getByText(
      "Configure staffing needs before optimizing assignments.",
    );
    expect(readiness).toBeInTheDocument();
    expect(optimize).toHaveAttribute(
      "aria-describedby",
      readiness.getAttribute("id"),
    );
  });

  it("sends only immutable tokens and renders Rust-provided recommendations and vacancies", async () => {
    const user = userEvent.setup();
    const onReviewShortlist = vi.fn();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({
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
            classification: "current_staff",
            score: 82,
            coachRequirement: "attacking_technical",
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
            classification: "recruitment",
            score: 79,
            coachRequirement: null,
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
            classification: "recruitment",
            score: 75,
            coachRequirement: null,
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
    expect(
      await screen.findByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).toBeInTheDocument();
    const currentStaffName = screen.getByText("Alex Coach");
    expect(currentStaffName).toHaveClass("font-medium", "text-info");
    expect(screen.getByRole("img", { name: "Current staff" })).toBeVisible();
    expect(screen.getByText("Riley Recruit")).not.toHaveClass("text-info");
    expect(
      screen.queryByRole("columnheader", { name: "Classification" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Recruitment")).not.toBeInTheDocument();
    expect(
      screen.getAllByRole("rowheader", { name: /Alpha Unit/ }),
    ).toHaveLength(2);
    expect(screen.getByText("Alpha Unit — 2 of 2 filled")).toBeInTheDocument();
    expect(
      screen.getByText("Club Services — 0 of 1 filled"),
    ).toBeInTheDocument();
    const resultRows = screen.getByRole("table").querySelectorAll("tbody tr");
    expect(Array.from(resultRows).map((row) => row.textContent)).toEqual([
      "Alpha Unit — 2 of 2 filled",
      expect.stringContaining("Alex Coach"),
      expect.stringContaining("Riley Recruit"),
      "Club Services — 0 of 1 filled",
      expect.stringContaining("Vacancy"),
      "Alpha Unit — 1 of 1 filled",
      expect.stringContaining("Taylor Coach"),
    ]);
    expect(screen.queryByText("senior")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: "Scope" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(/Coach requirement: Attacking Technical\./),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: /Coaches: 82, Excellent/i }),
    ).toBeInTheDocument();
    expect(screen.getByText("Vacancy")).toBeInTheDocument();
    expect(screen.getByText("Filled slots").parentElement).toHaveTextContent(
      "3",
    );
    expect(screen.getByText("Vacancies").parentElement).toHaveTextContent("1");
    expect(screen.getByText("Current staff").parentElement).toHaveTextContent(
      "1",
    );
    expect(screen.getByText("Recruits").parentElement).toHaveTextContent("2");
    expect(
      screen.getByText("5 joined shortlisted candidates; 4 configured slots."),
    ).toBeInTheDocument();
    const vacancyRow = screen.getByRole("row", { name: /Vacancy/ });
    expect(
      within(vacancyRow).getByText(
        "No eligible shortlisted candidate filled this slot.",
      ),
    ).toBeInTheDocument();
    expect(
      within(vacancyRow).getByText(/eligible scores;.*unavailable scores/i),
    ).not.toBeVisible();
    const evidence = within(vacancyRow).getByRole("group");
    expect(
      within(evidence).getByText("Show assignment evidence"),
    ).toBeInTheDocument();
    expect(evidence).not.toHaveAttribute("open");
    await user.click(within(evidence).getByText("Show assignment evidence"));
    expect(evidence).toHaveAttribute("open");
    expect(evidence).toHaveTextContent(
      "0 eligible scores; 2 unavailable scores; 2 joined shortlisted candidates.",
    );
    expect(
      within(vacancyRow).getByText("Coach requirement: Goalkeeping."),
    ).toBeInTheDocument();
    await user.click(within(evidence).getByText("Show assignment evidence"));
    expect(evidence).not.toHaveAttribute("open");
    expect(
      within(vacancyRow).getByText(/eligible scores;.*unavailable scores/i),
    ).not.toBeVisible();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
    expect(screen.getByText("Taylor Coach")).toBeInTheDocument();
    expect(screen.getByText(/unsupported Preferred Job/i)).toBeInTheDocument();
    expect(
      screen.getAllByText(/Preferred Job: Coach\. Eligible for this target\./),
    ).toHaveLength(2);

    const configure = screen.getByRole("button", {
      name: "Adjust staffing needs",
    });
    await user.click(configure);
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    await user.click(screen.getByRole("button", { name: "Review shortlist" }));
    expect(onReviewShortlist).toHaveBeenCalledOnce();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
  });

  it("discloses assignment evidence accessibly without optimizing again", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    const collapse = await screen.findByRole("button", {
      name: "Collapse assignment recommendations",
    });
    const bodyId = collapse.getAttribute("aria-controls");
    const body = bodyId ? document.getElementById(bodyId) : null;

    expect(collapse).toHaveAttribute("aria-expanded", "true");
    expect(body).toBeVisible();
    expect(body).toHaveTextContent("Alex Coach");
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);

    await user.click(collapse);

    const expand = screen.getByRole("button", {
      name: "Expand assignment recommendations",
    });
    expect(expand).toHaveAttribute("aria-expanded", "false");
    expect(expand).toHaveAttribute("aria-controls", bodyId);
    expect(body).not.toBeVisible();
    expect(body).toHaveTextContent("Alex Coach");
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);

    await user.click(expand);

    expect(
      screen.getByRole("button", {
        name: "Collapse assignment recommendations",
      }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(body).toBeVisible();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(1);
  });

  it("reopens a newly accepted result after the prior result was collapsed", async () => {
    const user = userEvent.setup();
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    await user.click(
      await screen.findByRole("button", {
        name: "Collapse assignment recommendations",
      }),
    );
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
            classification: "current_staff",
            score: 81,
            coachRequirement: null,
          },
        ],
      }),
    );

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    const collapse = await screen.findByRole("button", {
      name: "Collapse assignment recommendations",
    });
    expect(collapse).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Jordan Assistant")).toBeVisible();
    expect(getStaffAssignmentOptimizerIpcCallCount()).toBe(2);
  });

  it("renders canonical Fitness requirements without calculating eligibility", async () => {
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
            classification: "current_staff",
            score: 82,
            coachRequirement: "fitness",
          },
        ],
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    expect(
      await screen.findByText(/Coach requirement: Fitness\./),
    ).toBeInTheDocument();
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
            classification: "current_staff",
            score: 82,
            coachRequirement: null,
          },
        ],
      }),
    );
    renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );

    const row = await screen.findByRole("row", {
      name: /Assistant Manager.*Slot 1/i,
    });
    const person = within(row).getByText("—");
    expect(person).not.toHaveAttribute("title");
    expect(within(row).queryByText("0")).not.toBeInTheDocument();
    expect(
      within(row).queryByText(/Coach requirement:/),
    ).not.toBeInTheDocument();
  });

  it("suppresses a visible recommendation immediately while context is unavailable", async () => {
    const user = userEvent.setup();
    const { rerenderOptimizer } = renderOptimizer();

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(await screen.findByText("Alex Coach")).toBeInTheDocument();
    await user.click(
      screen.getByRole("button", {
        name: "Collapse assignment recommendations",
      }),
    );

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
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );
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
    await user.click(screen.getByRole("button", { name: "Save slots" }));
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
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );

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
      "Assignment context changed",
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
      "No current snapshot is available",
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
        screen.queryByText("Upload a Staff Shortlist"),
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
      "Upload a Staff Shortlist",
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
});
