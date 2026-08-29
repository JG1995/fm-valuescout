import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixtureStaffAssignmentOptimization,
  getLastStaffAssignmentOptimizerIpcArgs,
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
) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const result = render(
    <QueryClientProvider client={queryClient}>
      <StaffAssignmentOptimizer
        context={context}
        contextKey={contextKey}
        contextUnavailable={contextUnavailable}
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
          />
        </QueryClientProvider>,
      );
    },
  };
}

describe("StaffAssignmentOptimizer", () => {
  it("sends only immutable tokens and renders Rust-provided recommendations and vacancies", async () => {
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
            name: "Alex Coach",
            preferredJob: "Coach",
            classification: "current_staff",
            score: 82,
            coachRequirement: "attacking_technical",
          },
          {
            kind: "vacancy",
            scope: "club",
            scopeDisplayName: "Club",
            jobId: "coaches",
            jobLabel: "Coaches",
            slotNumber: 2,
            coachRequirement: "goalkeeping",
            evidence: {
              jobId: "coaches",
              joinedCandidateCount: 2,
              eligibleScoreCount: 0,
              unavailableScoreCount: 2,
            },
          },
        ],
      }),
    );
    renderOptimizer();

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
    expect(screen.getByText("Current staff")).toBeInTheDocument();
    expect(screen.getByText("First Team")).toBeInTheDocument();
    expect(screen.getByText("Club")).toBeInTheDocument();
    expect(screen.queryByText("senior")).not.toBeInTheDocument();
    expect(
      screen.getByText(/Coach requirement: Attacking Technical\./),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: /Coaches: 82, Excellent/i }),
    ).toBeInTheDocument();
    expect(screen.getByText("Vacancy")).toBeInTheDocument();
    expect(
      screen.getByText(
        /Coach requirement: Goalkeeping\. 0 eligible scores; 2 unavailable scores/i,
      ),
    ).toBeInTheDocument();
    expect(screen.getByText(/unsupported Preferred Job/i)).toBeInTheDocument();
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
      name: /First Team.*Assistant Manager.*Slot 1/i,
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
    await user.click(screen.getByRole("button", { name: "Configure slots" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Configure assignment slots",
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
      await screen.findByRole("button", { name: "Configure slots" }),
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
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Configure assignment slots",
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
