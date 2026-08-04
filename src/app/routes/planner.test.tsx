import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
import type { PlannerTactic } from "@/features/planner/types/tactic";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import {
  getPlannerAddStringIpcMockCalls,
  getPlannerClearTeamIpcMockCalls,
  getPlannerClubFamilySaveCalls,
  getPlannerDepthIpcMockCalls,
  getPlannerOptimizeIpcMockCalls,
  resolvePlannerClubFamilyIpcMock,
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticIpcMock,
  setPlannerAddStringError,
  setPlannerAddStringPending,
  setPlannerAssignmentError,
  setPlannerAvailableClubs,
  setPlannerClearTeamError,
  setPlannerClearTeamPending,
  setPlannerDepthIpcMock,
  setPlannerOptimizeDepth,
  setPlannerOptimizeError,
  setPlannerOptimizePending,
  setPlannerSlotCandidates,
  setPlannerTacticSaveError,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";

function renderPlannerRoute({ staleTime = 0 }: { staleTime?: number } = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: ["/planner"] }),
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return { queryClient };
}

describe("planner route", () => {
  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderPlannerRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Squad Planner" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("persists a separate Reserves club in the club family", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona", "Barca Athletic", "Barcelona U19"]);
    renderPlannerRoute();

    const primaryClub = await screen.findByRole("combobox", {
      name: "Primary club",
    });
    await user.type(primaryClub, "Barcelona");
    await user.click(await screen.findByRole("option", { name: "Barcelona" }));
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    const addReserves = await screen.findByRole("button", {
      name: "Add Reserves source",
    });
    await user.click(addReserves);
    await user.selectOptions(
      screen.getByLabelText("Reserves club 1"),
      "Barca Athletic",
    );
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    expect(
      (await screen.findAllByRole("option", { name: "Barca Athletic" })).length,
    ).toBeGreaterThan(0);
    await user.click(screen.getByRole("button", { name: "Add Youth source" }));
    expect(screen.getByLabelText("Youth club 1")).toBeInTheDocument();
    expect(
      screen.queryByLabelText("Youth player level 1"),
    ).not.toBeInTheDocument();
    expect(
      resolvePlannerClubFamilyIpcMock().sources.some(
        (source) =>
          source.clubName === "Barca Athletic" &&
          source.team === "reserves" &&
          source.teamLevel === null,
      ),
    ).toBe(true);
    expect(
      within(screen.getByRole("main")).getByText(/Associated clubs/),
    ).toBeInTheDocument();
    expect(screen.getByText("11 linked lanes")).toBeInTheDocument();
  });

  it("filters primary clubs and selects a club from the results", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["FC København", "Brøndby IF", "Copenhagen City"]);
    renderPlannerRoute();

    const primaryClub = await screen.findByRole("combobox", {
      name: "Primary club",
    });
    await user.type(primaryClub, "en");

    const suggestions = await screen.findByRole("listbox", {
      name: "Club suggestions",
    });
    expect(
      within(suggestions).getByRole("option", { name: "FC København" }),
    ).toBeInTheDocument();
    expect(
      within(suggestions).getByRole("option", { name: "Copenhagen City" }),
    ).toBeInTheDocument();
    expect(
      within(suggestions).queryByRole("option", { name: "Brøndby IF" }),
    ).not.toBeInTheDocument();

    await user.click(
      within(suggestions).getByRole("option", { name: "FC København" }),
    );

    expect(primaryClub).toHaveValue("FC København");
    expect(
      screen.getByRole("button", { name: "Save club family" }),
    ).toBeEnabled();
  });

  it("does not submit a stale club family for an unmatched primary-club search", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const primaryClub = await screen.findByRole("combobox", {
      name: "Primary club",
    });
    await user.type(primaryClub, "Barcelona");
    await user.click(await screen.findByRole("option", { name: "Barcelona" }));
    await user.click(screen.getByRole("button", { name: "Save club family" }));
    await waitFor(() => expect(getPlannerClubFamilySaveCalls()).toBe(1));

    await user.clear(primaryClub);
    await user.type(primaryClub, "No matching club");
    await user.keyboard("{Enter}");

    expect(getPlannerClubFamilySaveCalls()).toBe(1);
    expect(
      screen.getByRole("button", { name: "Save club family" }),
    ).toBeDisabled();
  });

  it("selects the highlighted primary club with the keyboard", async () => {
    const scrollIntoView = vi.fn();
    const scrollDescriptor = Object.getOwnPropertyDescriptor(
      HTMLElement.prototype,
      "scrollIntoView",
    );
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    try {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setPlannerAvailableClubs(["Barcelona", "Barca Athletic"]);
      renderPlannerRoute();

      const primaryClub = await screen.findByRole("combobox", {
        name: "Primary club",
      });
      await user.type(primaryClub, "bar");
      scrollIntoView.mockClear();
      await user.keyboard("{ArrowDown}");

      expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" });
      await user.keyboard("{Enter}");
      expect(primaryClub).toHaveValue("Barca Athletic");
    } finally {
      if (scrollDescriptor) {
        Object.defineProperty(
          HTMLElement.prototype,
          "scrollIntoView",
          scrollDescriptor,
        );
      } else {
        Reflect.deleteProperty(HTMLElement.prototype, "scrollIntoView");
      }
    }
  });

  it("renders first-use club-family setup before downstream planner panels", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const clubSetup = await screen.findByRole("heading", {
      level: 2,
      name: "Set up your club family",
    });
    const tacticEditor = screen.getByRole("heading", {
      level: 2,
      name: "Tactic editor",
    });
    const squadDepth = screen.getByRole("heading", {
      level: 2,
      name: "Squad depth",
    });

    expect(
      clubSetup.compareDocumentPosition(tacticEditor) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).not.toBe(0);
    expect(
      clubSetup.compareDocumentPosition(squadDepth) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).not.toBe(0);
  });

  it("edits linked IP and OOP lanes with filtered roles and weight control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });

    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });
    const bothView = within(viewGroup).getByRole("button", { name: "Both" });
    expect(bothView).toHaveAttribute("aria-pressed", "true");
    bothView.focus();
    await user.keyboard("{ArrowLeft}");
    expect(
      within(viewGroup).getByRole("button", { name: "OOP" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.queryByRole("heading", { level: 3, name: "In-Possession" }),
    ).not.toBeInTheDocument();
    await user.click(bothView);

    const firstIpLane = screen.getByRole("button", {
      name: "IP lane 1: GK, Goalkeeper",
    });
    firstIpLane.focus();
    await user.keyboard("{Enter}");

    const ipPosition = screen.getByRole("combobox", {
      name: "IP lane 1 position",
    });
    await user.selectOptions(ipPosition, "DL");

    const ipRole = screen.getByRole("combobox", { name: "IP lane 1 role" });
    expect(ipRole).toHaveValue("");
    expect(
      within(ipRole).queryByRole("option", { name: "Goalkeeper" }),
    ).not.toBeInTheDocument();
    await user.selectOptions(ipRole, "full_back_ip");

    const oopPosition = screen.getByRole("combobox", {
      name: "OOP lane 1 position",
    });
    await user.selectOptions(oopPosition, "DL");
    const oopRole = screen.getByRole("combobox", { name: "OOP lane 1 role" });
    expect(oopRole).toHaveValue("");
    await user.selectOptions(oopRole, "holding_full_back_oop");

    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    expect(
      screen.getAllByRole("slider", { name: "Lane 1 IP score weight" }),
    ).toHaveLength(1);
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");
    expect(screen.getByText("IP 55% / OOP 45%")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(resolvePlannerTacticIpcMock().lanes[0]).toMatchObject({
      ipWeight: 0.55,
      ipPosition: "DL",
      ipRoleId: "full_back_ip",
      oopPosition: "DL",
      oopRoleId: "holding_full_back_oop",
    });
    expect(resolvePlannerTacticIpcMock().lanes[1].ipWeight).toBe(0.5);
    await waitFor(() =>
      expect(getPlannerDepthIpcMockCalls()).toBeGreaterThan(1),
    );
  });

  it("retains the edited tactic draft when save fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerTacticSaveError("Tactic save failed");
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Tactic save failed",
    );
    expect(weight).toHaveValue("55");
    expect(resolvePlannerTacticIpcMock().lanes[0].ipWeight).toBe(0.5);
  });

  it("saves only the selected lane score weight", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    await user.click(
      screen.getByRole("button", { name: "IP lane 2: DL, Full-Back" }),
    );
    const weight = screen.getByRole("slider", {
      name: "Lane 2 IP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(resolvePlannerTacticIpcMock().lanes[0].ipWeight).toBe(0.5);
    expect(resolvePlannerTacticIpcMock().lanes[1].ipWeight).toBe(0.51);
  });

  it("saves and reloads the selected lane importance rank", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    await user.click(
      screen.getByRole("button", { name: "IP lane 2: DL, Full-Back" }),
    );
    const rank = screen.getByRole("combobox", {
      name: "Lane 2 importance rank",
    });
    await user.selectOptions(rank, "3");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(resolvePlannerTacticIpcMock().lanes[0].importanceRank).toBeNull();
    expect(resolvePlannerTacticIpcMock().lanes[1].importanceRank).toBe(3);
    expect(rank).toHaveValue("3");
  });

  it("saves the selected lane foot rule and disables its mode for Either", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const preferredFoot = screen.getByRole("combobox", {
      name: "Lane 1 preferred foot",
    });
    const footPreference = screen.getByRole("combobox", {
      name: "Lane 1 foot preference",
    });
    expect(footPreference).toBeDisabled();

    await user.selectOptions(preferredFoot, "both");
    expect(footPreference).toBeEnabled();
    await user.selectOptions(footPreference, "strict");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(resolvePlannerTacticIpcMock().lanes[0]).toMatchObject({
      preferredFoot: "both",
      footPreference: "strict",
    });
  });

  it("retains an edited foot rule after a failed save", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerTacticSaveError("Tactic save failed");
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const preferredFoot = screen.getByRole("combobox", {
      name: "Lane 1 preferred foot",
    });
    await user.selectOptions(preferredFoot, "left");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Tactic save failed",
    );
    expect(preferredFoot).toHaveValue("left");
    expect(resolvePlannerTacticIpcMock().lanes[0].preferredFoot).toBe("any");
  });

  it("shows duplicate importance ranks inline and retains them after a failed save", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerTacticSaveError("Tactic save failed");
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Lane 1 importance rank" }),
      "1",
    );
    await user.click(
      screen.getByRole("button", { name: "IP lane 2: DL, Full-Back" }),
    );
    const duplicateRank = screen.getByRole("combobox", {
      name: "Lane 2 importance rank",
    });
    await user.selectOptions(duplicateRank, "1");

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Importance rank 1 is already used.",
    );
    expect(screen.getByRole("button", { name: "Save tactic" })).toBeDisabled();

    await user.selectOptions(duplicateRank, "2");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Tactic save failed",
    );
    expect(duplicateRank).toHaveValue("2");
    expect(resolvePlannerTacticIpcMock().lanes[1].importanceRank).toBeNull();
  });

  it("refreshes 60-second cached candidates after saving a tactic", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Old tactic fit",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
      }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    const cell = await screen.findByRole("button", {
      name: /Senior, 1st string, Goalkeeper, Empty/,
    });
    await user.click(cell);
    expect(
      await screen.findByRole("option", { name: /Old tactic fit/ }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Updated tactic fit",
        currentClub: "Barcelona",
        ipScore: 90,
        oopScore: 80,
        combinedScore: 85,
      }),
    ]);
    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    await user.click(cell);
    expect(
      await screen.findByRole("option", { name: /Updated tactic fit/ }),
    ).toBeInTheDocument();
  });

  it("resets a dirty tactic draft when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");

    queryClient.setQueryData(plannerKeys.tactic(), {
      ...resolvePlannerTacticIpcMock(),
      lanes: resolvePlannerTacticIpcMock().lanes.map((lane, index) =>
        index === 0 ? { ...lane, ipWeight: 0.2 } : lane,
      ),
    });
    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      snapshotKeys.current(),
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot in the planner query");
    }
    queryClient.setQueryData<SnapshotSummary | null>(
      snapshotKeys.current(),
      () => ({ ...snapshot, saveId: 2 }),
    );

    await waitFor(() =>
      expect(
        screen.getByRole("slider", { name: "Lane 1 IP score weight" }),
      ).toHaveValue("20"),
    );
  });

  it("blocks tactic saves while active-save data refreshes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");

    let resolveRefresh!: (tactic: PlannerTactic) => void;
    const refresh = new Promise<PlannerTactic>((resolve) => {
      resolveRefresh = resolve;
    });
    const refreshRequest = queryClient.fetchQuery({
      queryKey: plannerKeys.tactic(),
      queryFn: () => refresh,
    });

    const saveButton = screen.getByRole("button", { name: "Save tactic" });
    await waitFor(() => expect(saveButton).toBeDisabled());
    expect(screen.getByRole("status")).toHaveTextContent(
      "Refreshing active save",
    );
    await user.click(saveButton);
    expect(resolvePlannerTacticIpcMock().lanes[0].ipWeight).toBe(0.5);

    resolveRefresh(resolvePlannerTacticIpcMock());
    await refreshRequest;
    await waitFor(() => expect(saveButton).toBeEnabled());
  });

  it("keeps tactic saves blocked after an active-save refresh fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", {
      name: "Lane 1 IP score weight",
    });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );

    const refreshRequest = queryClient.fetchQuery({
      queryKey: plannerKeys.tactic(),
      queryFn: () => Promise.reject(new Error("Tactic refresh failed")),
    });
    await expect(refreshRequest).rejects.toThrow("Tactic refresh failed");

    const saveButton = screen.getByRole("button", { name: "Save tactic" });
    await waitFor(() => expect(saveButton).toBeDisabled());
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Could not refresh the active save",
    );
    await user.click(saveButton);
    expect(resolvePlannerTacticIpcMock().lanes[0].ipWeight).toBe(0.5);
  });

  it("renders shared lanes, ordered strings, keyboard tabs, and truthful assignment states", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = resolvePlannerDepthIpcMock();
    setPlannerDepthIpcMock(withDepthAssignments(depth));
    renderPlannerRoute();

    const matrix = await screen.findByRole("region", {
      name: "Senior squad depth matrix",
    });
    expect(matrix).toHaveClass("overflow-x-auto");
    expect(
      within(matrix).getByRole("columnheader", { name: "1st string" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("columnheader", { name: "2nd string" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("row", { name: /Goalkeeper/ }),
    ).toBeInTheDocument();
    expect(within(matrix).getByText("IP: GK · Goalkeeper")).toBeInTheDocument();
    expect(
      within(matrix).getByText("OOP: GK · Line-Holding Keeper"),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("img", { name: /Combined role score: 82/ }),
    ).toBeInTheDocument();
    expect(within(matrix).getByText("Outside pool")).toBeInTheDocument();
    expect(within(matrix).getByText("Unresolved")).toBeInTheDocument();
    expect(
      within(matrix).getByRole("button", {
        name: /Missing Centre-Back/,
      }),
    ).toBeInTheDocument();
    const unavailableCell = within(matrix).getByRole("button", {
      name: /No Score Player, Resolved, score —/,
    });
    expect(unavailableCell).not.toBeDisabled();
    unavailableCell.focus();
    expect(document.activeElement).toBe(unavailableCell);
    expect(within(matrix).getAllByText("—").length).toBeGreaterThan(0);

    const seniorTab = screen.getByRole("tab", { name: "Senior" });
    seniorTab.focus();
    await user.keyboard("{ArrowRight}");
    const reservesTab = screen.getByRole("tab", { name: "Reserves" });
    expect(reservesTab).toHaveAttribute("aria-selected", "true");
    expect(document.activeElement).toBe(reservesTab);
    expect(
      screen.getByRole("region", { name: "Reserves squad depth matrix" }),
    ).toBeInTheDocument();

    const cell = screen.getAllByRole("button", {
      name: /Reserves, 1st string, Goalkeeper, Empty/,
    })[0];
    expect(cell).not.toBeDisabled();
    cell.focus();
    expect(document.activeElement).toBe(cell);
  });

  it("opens a slot-fit picker from an empty matrix cell", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const cell = await screen.findByRole("button", {
      name: /Senior, 1st string, Goalkeeper, Empty/,
    });
    await user.click(cell);

    expect(
      screen.getByRole("dialog", { name: "Find a player for Goalkeeper" }),
    ).toBeInTheDocument();
  });

  it("searches null-score candidates and assigns the keyboard selection", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const scrollIntoView = vi.fn();
    const scrollDescriptor = Object.getOwnPropertyDescriptor(
      HTMLElement.prototype,
      "scrollIntoView",
    );
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    try {
      const user = userEvent.setup({
        advanceTimers: vi.advanceTimersByTime,
      });
      await resolveLoadDataIpcMock();
      setPlannerAvailableClubs(["Barcelona"]);
      setPlannerSlotCandidates([
        slotCandidate({
          playerUid: 77,
          name: "First Keeper",
          currentClub: "Barcelona",
          ipScore: 90,
          oopScore: 80,
          combinedScore: 85,
        }),
        slotCandidate({
          playerUid: 78,
          name: "B Team Keeper",
          currentClub: "Barca Athletic",
          ipScore: null,
          oopScore: 70,
          combinedScore: null,
        }),
      ]);
      renderPlannerRoute();

      const cell = await screen.findByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      });
      await user.click(cell);
      const search = screen.getByRole("combobox", {
        name: "Search squad candidates",
      });
      await user.type(search, "Keeper");
      const bTeam = await screen.findByRole("option", {
        name: /B Team Keeper/,
      });
      expect(bTeam).toHaveTextContent("IP — · OOP 70");
      expect(bTeam).toHaveTextContent("—");
      expect(
        screen.getByRole("option", { name: /First Keeper/ }),
      ).toBeInTheDocument();
      scrollIntoView.mockClear();
      await user.keyboard("{ArrowDown}");
      expect(bTeam).toHaveAttribute("aria-selected", "true");
      await vi.advanceTimersByTimeAsync(200);
      expect(bTeam).toHaveAttribute("aria-selected", "true");
      await waitFor(() =>
        expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" }),
      );

      await user.keyboard("{Enter}");

      await waitFor(() =>
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
      );
      expect(
        screen.getByRole("button", { name: /B Team Keeper, Resolved/ }),
      ).toBeInTheDocument();
      await waitFor(() => expect(document.activeElement).toBe(cell));
    } finally {
      vi.useRealTimers();
      if (scrollDescriptor) {
        Object.defineProperty(
          HTMLElement.prototype,
          "scrollIntoView",
          scrollDescriptor,
        );
      } else {
        Reflect.deleteProperty(HTMLElement.prototype, "scrollIntoView");
      }
    }
  });

  it("refreshes 60-second cached candidates after assigning a player", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondSeniorString(resolvePlannerDepthIpcMock()),
    );
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Cache Keeper",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
      }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    await user.click(
      await screen.findByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      }),
    );
    await user.click(
      await screen.findByRole("option", { name: /Cache Keeper/ }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(
      screen.getByRole("button", {
        name: /Senior, 2nd string, Goalkeeper, Empty/,
      }),
    );
    const cacheKeeper = await screen.findByRole("option", {
      name: /Cache Keeper/,
    });
    expect(cacheKeeper).toHaveTextContent(
      "Assigned: Senior · 1st string · Goalkeeper",
    );
    await user.click(cacheKeeper);

    expect(
      screen.getByRole("dialog", { name: "Move Cache Keeper?" }),
    ).toHaveTextContent(
      "Move Cache Keeper from Senior · 1st string · Goalkeeper to Senior · 2nd string · Goalkeeper?",
    );
  });

  it("refreshes 60-second cached candidates after removing a populated string", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = withSecondSeniorString(resolvePlannerDepthIpcMock());
    depth.teams = depth.teams.map((team) =>
      team.team === "senior"
        ? {
            ...team,
            strings: team.strings.map((plannerString, index) =>
              index === 0
                ? {
                    ...plannerString,
                    assignments: [
                      {
                        id: 201,
                        laneId: "goalkeeper",
                        playerUid: 77,
                        lastKnownName: "String Keeper",
                        currentName: "String Keeper",
                        state: "resolved",
                        combinedScore: 82,
                      },
                    ],
                  }
                : plannerString,
            ),
          }
        : team,
    );
    setPlannerDepthIpcMock(depth);
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "String Keeper",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
      }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    await user.click(
      await screen.findByRole("button", {
        name: /Senior, 2nd string, Goalkeeper, Empty/,
      }),
    );
    expect(
      await screen.findByRole("option", { name: /String Keeper/ }),
    ).toHaveTextContent("Assigned: Senior · 1st string · Goalkeeper");
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(screen.getByRole("button", { name: "Manage 1st string" }));
    await user.click(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Remove string" }),
    );
    await user.click(screen.getByRole("button", { name: "Remove string" }));

    await user.click(
      screen.getByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      }),
    );
    expect(
      await screen.findByRole("option", { name: /String Keeper/ }),
    ).toHaveTextContent("Unassigned");
  });

  it("requires confirmation before clearing an occupied slot", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondReserveString(
        withReserveGoalkeeper(resolvePlannerDepthIpcMock()),
      ),
    );
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Reserve Keeper",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
      }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    await user.click(await screen.findByRole("tab", { name: "Reserves" }));
    const occupiedCell = screen.getByRole("button", {
      name: /Reserves, 1st string, Goalkeeper, Reserve Keeper, Resolved/,
    });
    const emptyCell = screen.getByRole("button", {
      name: /Reserves, 2nd string, Goalkeeper, Empty/,
    });

    await user.click(emptyCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent("Assigned: Reserves · 1st string · Goalkeeper");
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(occupiedCell);
    const clearDialog = screen.getByRole("dialog", {
      name: "Clear Reserve Keeper?",
    });
    expect(clearDialog).toHaveTextContent(
      "Reserve Keeper is assigned to Reserves · 1st string · Goalkeeper. It must be cleared before assigning or moving a player.",
    );
    expect(within(clearDialog).queryByRole("combobox")).not.toBeInTheDocument();
    expect(within(clearDialog).queryByRole("listbox")).not.toBeInTheDocument();
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(document.activeElement).toBe(occupiedCell));
    expect(occupiedCell).toHaveTextContent("Reserve Keeper");

    occupiedCell.focus();
    await user.keyboard("{Enter}");
    setPlannerAssignmentError("Clear failed");
    await user.click(screen.getByRole("button", { name: "Clear slot" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Clear failed");
    await waitFor(() => expect(document.activeElement).toBe(occupiedCell));
    expect(occupiedCell).toHaveTextContent("Reserve Keeper");

    setPlannerAssignmentError(null);
    await user.keyboard("{Enter}");
    await user.click(screen.getByRole("button", { name: "Clear slot" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(occupiedCell).toHaveAccessibleName(
      /Reserves, 1st string, Goalkeeper, Empty/,
    );
    await waitFor(() => expect(document.activeElement).toBe(occupiedCell));

    await user.click(emptyCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent("Unassigned");
  });

  it("confirms moves for assigned players before reconciling the depth matrix", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = withReserveGoalkeeper(resolvePlannerDepthIpcMock());
    setPlannerDepthIpcMock(depth);
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Reserve Keeper",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
        assignmentLocation: {
          team: "reserves",
          stringId: 2,
          stringOrder: 0,
          laneId: "goalkeeper",
        },
      }),
    ]);
    renderPlannerRoute();

    await user.click(
      await screen.findByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      }),
    );
    await user.click(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    );

    expect(
      screen.getByRole("dialog", { name: "Move Reserve Keeper?" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("dialog", { name: "Move Reserve Keeper?" }),
    ).toHaveTextContent(
      "Move Reserve Keeper from Reserves · 1st string · Goalkeeper to Senior · 1st string · Goalkeeper?",
    );
    const depthFetchesBeforeMove = getPlannerDepthIpcMockCalls();
    await user.click(screen.getByRole("button", { name: "Confirm move" }));

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    await waitFor(() =>
      expect(getPlannerDepthIpcMockCalls()).toBeGreaterThan(
        depthFetchesBeforeMove,
      ),
    );
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", {
        name: /Reserves, 1st string, Goalkeeper, Empty/,
      }),
    ).toBeInTheDocument();
  });

  it("cancels and fails without changing assignments, then restores the origin focus", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(withReserveGoalkeeper(resolvePlannerDepthIpcMock()));
    setPlannerSlotCandidates([
      slotCandidate({
        playerUid: 77,
        name: "Reserve Keeper",
        currentClub: "Barcelona",
        ipScore: 85,
        oopScore: 75,
        combinedScore: 80,
        assignmentLocation: {
          team: "reserves",
          stringId: 2,
          stringOrder: 0,
          laneId: "goalkeeper",
        },
      }),
    ]);
    renderPlannerRoute();

    const cell = await screen.findByRole("button", {
      name: /Senior, 1st string, Goalkeeper, Empty/,
    });
    cell.focus();
    await user.keyboard("{Enter}");
    await user.keyboard("{Escape}");
    await waitFor(() => expect(document.activeElement).toBe(cell));

    setPlannerAssignmentError("Move failed");
    await user.keyboard("{Enter}");
    await user.click(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    );
    await user.click(screen.getByRole("button", { name: "Confirm move" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Move failed");
    await waitFor(() => expect(document.activeElement).toBe(cell));
    expect(
      screen.getByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
  });

  it("manages ordered strings from equivalent header menus", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = withSecondSeniorString(resolvePlannerDepthIpcMock());
    depth.teams = depth.teams.map((team) =>
      team.team === "senior"
        ? {
            ...team,
            strings: team.strings.map((plannerString, index) =>
              index === 0
                ? {
                    ...plannerString,
                    assignments: [
                      {
                        id: 201,
                        laneId: "goalkeeper",
                        playerUid: 77,
                        lastKnownName: "Senior Keeper",
                        currentName: "Senior Keeper",
                        state: "resolved",
                        combinedScore: 82,
                      },
                    ],
                  }
                : plannerString,
            ),
          }
        : team,
    );
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute();

    const firstHeader = await screen.findByRole("button", {
      name: "Manage 1st string",
    });
    firstHeader.focus();
    await user.keyboard("{Enter}");
    const firstMenu = screen.getByRole("menu", { name: "1st string actions" });
    await user.click(
      within(firstMenu).getByRole("menuitem", { name: "Add string" }),
    );
    expect(
      await screen.findByRole("columnheader", { name: "3rd string" }),
    ).toBeInTheDocument();

    const secondHeader = screen.getByRole("button", {
      name: "Manage 2nd string",
    });
    await user.pointer({ target: secondHeader, keys: "[MouseRight]" });
    const secondMenu = screen.getByRole("menu", { name: "2nd string actions" });
    await user.click(
      within(secondMenu).getByRole("menuitem", { name: "Remove string" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole("columnheader", { name: "3rd string" }),
      ).not.toBeInTheDocument(),
    );
    expect(
      screen.getByRole("columnheader", { name: "2nd string" }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(document.activeElement).toHaveAccessibleName("Manage 2nd string"),
    );

    await user.click(firstHeader);
    await user.click(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Remove string" }),
    );
    const confirmation = screen.getByRole("dialog", {
      name: "Remove 1st string?",
    });
    expect(confirmation).toHaveTextContent("Senior Keeper");
    await user.click(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(document.activeElement).toBe(firstHeader));
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerAssignmentError("Remove failed");
    await user.click(firstHeader);
    await user.click(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Remove string" }),
    );
    await user.click(screen.getByRole("button", { name: "Remove string" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Remove failed");
    await waitFor(() => expect(document.activeElement).toBe(firstHeader));
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerAssignmentError(null);
    await user.click(firstHeader);
    await user.click(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Remove string" }),
    );
    await user.click(screen.getByRole("button", { name: "Remove string" }));
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: /Senior Keeper, Resolved/ }),
      ).not.toBeInTheDocument(),
    );
    await user.click(screen.getByRole("button", { name: "Manage 1st string" }));
    expect(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Remove string" }),
    ).toBeDisabled();
  });

  it("opens the string menu from the whole header context menu", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const header = await screen.findByRole("columnheader", {
      name: "1st string",
    });

    expect(fireEvent.contextMenu(header)).toBe(false);
    expect(
      screen.getByRole("menu", { name: "1st string actions" }),
    ).toBeInTheDocument();
  });

  it("keeps focus on the originating header when adding a string fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondSeniorString(resolvePlannerDepthIpcMock()),
    );
    setPlannerAddStringError("Add failed");
    renderPlannerRoute();

    const firstHeader = await screen.findByRole("button", {
      name: "Manage 1st string",
    });
    await user.click(firstHeader);
    await user.click(
      within(
        screen.getByRole("menu", { name: "1st string actions" }),
      ).getByRole("menuitem", { name: "Add string" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent("Add failed");
    await waitFor(() => expect(document.activeElement).toBe(firstHeader));
    expect(
      screen.getByRole("columnheader", { name: "2nd string" }),
    ).toBeInTheDocument();
  });

  it("starts one add mutation while the header action is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerAddStringPending(true);
    renderPlannerRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage 1st string" }),
    );
    const addButton = screen.getByRole("menuitem", { name: "Add string" });
    await user.click(addButton);
    await user.click(addButton);

    expect(getPlannerAddStringIpcMockCalls()).toBe(1);
  });

  it("confirms clearing only the selected squad and reconciles its candidates", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = withSecondSeniorString(
      withReserveGoalkeeper(resolvePlannerDepthIpcMock()),
    );
    depth.teams = depth.teams.map((team) =>
      team.team === "senior"
        ? {
            ...team,
            strings: team.strings.map((plannerString, index) => ({
              ...plannerString,
              assignments:
                index === 0
                  ? [
                      {
                        id: 78,
                        laneId: "goalkeeper",
                        playerUid: 78,
                        lastKnownName: "Senior Keeper",
                        currentName: "Senior Keeper",
                        state: "resolved",
                        combinedScore: 80,
                      },
                    ]
                  : [],
            })),
          }
        : team,
    );
    setPlannerDepthIpcMock(depth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 78, name: "Senior Keeper" }),
      slotCandidate({ playerUid: 77, name: "Reserve Keeper" }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    const secondSeniorCell = await screen.findByRole("button", {
      name: /Senior, 2nd string, Goalkeeper, Empty/,
    });
    await user.click(secondSeniorCell);
    expect(
      await screen.findByRole("option", { name: /Senior Keeper/ }),
    ).toHaveTextContent("Assigned: Senior · 1st string · Goalkeeper");
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    const clearButton = await screen.findByRole("button", {
      name: "Clear Senior squad",
    });
    clearButton.focus();
    await user.keyboard("{Enter}");
    const confirmation = screen.getByRole("dialog", {
      name: "Clear Senior squad?",
    });
    expect(confirmation).toHaveTextContent(
      "This clears every assignment from the Senior squad.",
    );
    await user.click(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(document.activeElement).toBe(clearButton));
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerClearTeamError("Clear squad failed");
    await user.click(clearButton);
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Clear Senior squad?" }),
      ).getByRole("button", { name: "Clear Senior squad" }),
    );
    expect(
      await within(
        screen.getByRole("dialog", { name: "Clear Senior squad?" }),
      ).findByRole("alert"),
    ).toHaveTextContent("Clear squad failed");
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerClearTeamError(null);
    const confirmButton = within(
      screen.getByRole("dialog", { name: "Clear Senior squad?" }),
    ).getByRole("button", { name: "Clear Senior squad" });
    await user.click(confirmButton);
    expect(getPlannerClearTeamIpcMockCalls()).toBe(2);
    expect(
      await screen.findByText("Senior squad cleared."),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: /Senior Keeper, Resolved/ }),
      ).not.toBeInTheDocument(),
    );
    await user.click(secondSeniorCell);
    expect(
      await screen.findByRole("option", { name: /Senior Keeper/ }),
    ).toHaveTextContent("Unassigned");
    await user.keyboard("{Escape}");
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
  });

  it("prevents duplicate selected-squad clears while confirmation is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerClearTeamPending(true);
    renderPlannerRoute();

    await user.click(
      await screen.findByRole("button", { name: "Clear Senior squad" }),
    );
    const confirmButton = within(
      screen.getByRole("dialog", { name: "Clear Senior squad?" }),
    ).getByRole("button", { name: "Clear Senior squad" });
    await user.click(confirmButton);
    await user.click(confirmButton);

    expect(getPlannerClearTeamIpcMockCalls()).toBe(1);
    expect(confirmButton).toBeDisabled();
    expect(confirmButton).toHaveAccessibleName("Clearing…");
  });

  it("optimizes every squad and reconciles depth and candidates", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const optimizedDepth = withReserveGoalkeeper(resolvePlannerDepthIpcMock());
    setPlannerOptimizeDepth(optimizedDepth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Reserve Keeper" }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    const seniorCell = await screen.findByRole("button", {
      name: /Senior, 1st string, Goalkeeper, Empty/,
    });
    await user.click(seniorCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent("Unassigned");
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    const optimizeButton = await screen.findByRole("button", {
      name: "Optimize squads",
    });
    await user.click(optimizeButton);

    await waitFor(() =>
      expect(screen.getByText("Squads optimized.")).toBeInTheDocument(),
    );
    expect(getPlannerOptimizeIpcMockCalls()).toBe(1);
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Senior" }));
    await user.click(seniorCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent("Assigned: Reserves · 1st string · Goalkeeper");
  });

  it("keeps the depth unchanged and reports optimizer errors", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerOptimizeError("Optimize failed");
    renderPlannerRoute();

    await user.click(
      await screen.findByRole("button", { name: "Optimize squads" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Optimize failed",
    );
    expect(
      screen.getByRole("button", {
        name: /Senior, 1st string, Goalkeeper, Empty/,
      }),
    ).toBeInTheDocument();
  });

  it("prevents duplicate optimizer runs while pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerOptimizePending(true);
    renderPlannerRoute();

    const optimizeButton = await screen.findByRole("button", {
      name: "Optimize squads",
    });
    await user.click(optimizeButton);
    await user.click(optimizeButton);

    expect(getPlannerOptimizeIpcMockCalls()).toBe(1);
    expect(optimizeButton).toBeDisabled();
    expect(optimizeButton).toHaveAccessibleName("Optimizing…");
  });
});

function slotCandidate(
  candidate: Partial<PlannerSlotCandidate> &
    Pick<PlannerSlotCandidate, "playerUid" | "name">,
): PlannerSlotCandidate {
  return {
    playerUid: candidate.playerUid,
    name: candidate.name,
    currentClub: candidate.currentClub ?? "Barcelona",
    ipScore: candidate.ipScore ?? null,
    oopScore: candidate.oopScore ?? null,
    combinedScore: candidate.combinedScore ?? null,
    assignmentLocation: candidate.assignmentLocation ?? null,
  };
}

function withReserveGoalkeeper(depth: PlannerDepth): PlannerDepth {
  return {
    ...depth,
    teams: depth.teams.map((team) =>
      team.team === "reserves"
        ? {
            ...team,
            strings: [
              {
                ...team.strings[0],
                assignments: [
                  {
                    id: 201,
                    laneId: "goalkeeper",
                    playerUid: 77,
                    lastKnownName: "Reserve Keeper",
                    currentName: "Reserve Keeper",
                    state: "resolved",
                    combinedScore: 80,
                  },
                ],
              },
            ],
          }
        : team,
    ),
  };
}

function withSecondSeniorString(depth: PlannerDepth): PlannerDepth {
  return {
    ...depth,
    teams: depth.teams.map((team) =>
      team.team === "senior"
        ? {
            ...team,
            strings: [
              ...team.strings,
              { id: 4, stringOrder: 1, assignments: [] },
            ],
          }
        : team,
    ),
  };
}

function withSecondReserveString(depth: PlannerDepth): PlannerDepth {
  return {
    ...depth,
    teams: depth.teams.map((team) =>
      team.team === "reserves"
        ? {
            ...team,
            strings: [
              ...team.strings,
              { id: 4, stringOrder: 1, assignments: [] },
            ],
          }
        : team,
    ),
  };
}

function withDepthAssignments(depth: PlannerDepth): PlannerDepth {
  return {
    ...depth,
    teams: depth.teams.map((team) =>
      team.team === "senior"
        ? {
            ...team,
            strings: [
              {
                ...team.strings[0],
                assignments: [
                  {
                    id: 101,
                    laneId: "goalkeeper",
                    playerUid: 77,
                    lastKnownName: "Alex Keeper",
                    currentName: "Alex Keeper",
                    state: "resolved",
                    combinedScore: 82,
                  },
                  {
                    id: 102,
                    laneId: "left_back",
                    playerUid: 78,
                    lastKnownName: "Outside Full-Back",
                    currentName: "Outside Full-Back",
                    state: "outside_pool",
                    combinedScore: 61,
                  },
                  {
                    id: 103,
                    laneId: "left_centre_back",
                    playerUid: 79,
                    lastKnownName: "Missing Centre-Back",
                    currentName: null,
                    state: "unresolved",
                    combinedScore: null,
                  },
                  {
                    id: 104,
                    laneId: "right_back",
                    playerUid: 80,
                    lastKnownName: "No Score Player",
                    currentName: "No Score Player",
                    state: "resolved",
                    combinedScore: null,
                  },
                ],
              },
              { id: 4, stringOrder: 1, assignments: [] },
            ],
          }
        : team,
    ),
  };
}
