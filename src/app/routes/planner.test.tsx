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
import { academyKeys } from "@/features/academy/api/academy-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type { PlannerWorkspace } from "@/features/planner/components/planner-workspace-tabs";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
import type { PlannerTactic } from "@/features/planner/types/tactic";
import { phasePositionLabel } from "@/features/planner/utils/tactic-editor";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import type { SquadPlayer } from "@/features/squad/types/squad-player";
import { routeTree } from "@/routeTree.gen";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  getPlannerAddStringIpcMockCalls,
  getPlannerClearAllIpcMockCalls,
  getPlannerDepthIpcMockCalls,
  getPlannerOptimizeIpcMockBases,
  getPlannerOptimizeIpcMockCalls,
  getPlannerSlotCandidateFetchCount,
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticIpcMock,
  resolveSavePlannerClubFamilyIpcMock,
  setPlannerAddStringError,
  setPlannerAddStringPending,
  setPlannerAssignmentError,
  setPlannerAvailableClubs,
  setPlannerClearAllError,
  setPlannerClearAllPending,
  setPlannerDepthIpcMock,
  setPlannerOptimizeDepth,
  setPlannerOptimizeError,
  setPlannerOptimizePending,
  setPlannerSlotCandidates,
  setPlannerTacticIpcMock,
  setPlannerTacticSaveError,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  getLastSquadPlayersArgs,
  getSquadCurrentAbilityBoostIpcMockCalls,
  getSquadPlayersCallCount,
  getSquadWonderkidMentalityBoostIpcMockCalls,
  resolvePendingSquadCurrentAbilityBoostIpcMock,
  resolvePendingSquadPlayersPageIpcMock,
  resolvePendingSquadWonderkidMentalityBoostIpcMock,
  setSquadCurrentAbilityBoostIpcMockMode,
  setSquadPlayersOverride,
  setSquadPlayersPageIpcMockMode,
  setSquadWonderkidMentalityBoostIpcMockMode,
} from "@/testing/squad-ipc-mock";

function renderPlannerRoute({
  staleTime = 0,
  initialEntry = "/planner?view=planner",
}: {
  staleTime?: number;
  initialEntry?: string;
} = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime } },
  });
  const history = createMemoryHistory({ initialEntries: [initialEntry] });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history,
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return { history, queryClient, router };
}

async function openPlannerWorkspace(
  user: ReturnType<typeof userEvent.setup>,
  workspace: PlannerWorkspace,
) {
  const labels: Record<PlannerWorkspace, string> = {
    squad: "Squad",
    planner: "Planner",
    tactic: "Tactic",
  };
  await user.click(await screen.findByRole("tab", { name: labels[workspace] }));
}

const KEEPER_POSITION = "IP: GK · Goalkeeper / OOP: GK · Line-Holding Keeper";
const SENIOR_FIRST_KEEPER = `Senior · 1st string · ${KEEPER_POSITION}`;
const SENIOR_SECOND_KEEPER = `Senior · 2nd string · ${KEEPER_POSITION}`;
const RESERVES_FIRST_KEEPER = `Reserves · 1st string · ${KEEPER_POSITION}`;

function squadPlayerNamed(name: string, uid: number, ca = 160): SquadPlayer {
  return {
    uid,
    name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: "Metro FC",
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
  };
}

function manySquadPlayers(count: number): SquadPlayer[] {
  return Array.from({ length: count }, (_, index) =>
    squadPlayerNamed(
      `Squad player ${String(index + 1).padStart(3, "0")}`,
      index + 1,
      200 - index,
    ),
  );
}

function withSecondStringForEveryTeam(depth: PlannerDepth): PlannerDepth {
  let nextStringId =
    Math.max(
      ...depth.teams.flatMap((team) =>
        team.strings.map((plannerString) => plannerString.id),
      ),
    ) + 1;

  return {
    ...depth,
    teams: depth.teams.map((team) => ({
      ...team,
      strings: [
        ...team.strings,
        {
          id: nextStringId++,
          stringOrder: team.strings.length,
          assignments: [],
        },
      ],
    })),
  };
}

async function setPlannerMatrixWidth(width: number) {
  const matrixContainer = await screen.findByTestId(
    "planner-depth-matrix-container",
  );
  Object.defineProperty(matrixContainer, "clientWidth", {
    configurable: true,
    value: width,
  });
  fireEvent(window, new Event("resize"));
}

function mockScrollerScrollTo(scroller: HTMLElement) {
  Object.defineProperty(scroller, "scrollTo", {
    configurable: true,
    value: (options: { top?: number }) => {
      scroller.scrollTop = options.top ?? scroller.scrollTop;
    },
  });
}

describe("planner route", () => {
  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderPlannerRoute({ initialEntry: "/planner" });

    expect(
      await screen.findByRole("heading", { level: 1, name: "Squad" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("defaults to Squad and keeps Planner and Tactic mounted", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("link", { name: "Open Club Setup" });
    expect(screen.getByRole("tab", { name: "Squad" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("tab", { name: "Planner" })).toHaveAttribute(
      "aria-selected",
      "false",
    );
    expect(screen.queryByRole("tab", { name: "Club Setup" })).toBeNull();
    expect(
      screen.getByRole("link", { name: "Open Club Setup" }),
    ).toHaveAttribute("href", "/#club-setup");
    const tacticPanel = document.getElementById(
      "planner-workspace-panel-tactic",
    );
    const plannerPanel = document.getElementById(
      "planner-workspace-panel-planner",
    );
    expect(tacticPanel).toBeInTheDocument();
    expect(plannerPanel).toBeInTheDocument();
    expect(tacticPanel).toHaveAttribute("hidden");
    expect(plannerPanel).toHaveAttribute("hidden");
    expect(
      within(tacticPanel as HTMLElement).getByRole("region", {
        name: "Tactic controls",
        hidden: true,
      }),
    ).toBeInTheDocument();
    expect(
      within(plannerPanel as HTMLElement).getByRole("heading", {
        level: 2,
        name: "Squad depth",
        hidden: true,
      }),
    ).toBeInTheDocument();

    await openPlannerWorkspace(user, "tactic");
    const tacticEditor = screen.getByRole("region", {
      name: "Tactic controls",
    });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await openPlannerWorkspace(user, "planner");
    await openPlannerWorkspace(user, "tactic");
    expect(tacticEditor).toBeInTheDocument();
    expect(weight).toHaveValue("51");
  });

  it("shows a sortable overview for a configured squad", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    for (const column of [
      "Name",
      "Age / DOB",
      "Nationality",
      "Club",
      "Division",
      "CA",
      "PA",
      "Value",
    ]) {
      expect(
        within(table).getByRole("columnheader", { name: column }),
      ).toBeInTheDocument();
    }
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    expect(
      within(table).getByRole("link", { name: "Alex Scout" }),
    ).toHaveAttribute("href", "/players/42?tab=technical");
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "1 player · sorted by CA (descending)",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Edit filters" })).toBeNull();
  });

  it("renders every nationality flag in the squad overview", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Flagged Squad", 42),
        nationalities: ["England", "Wales", "South Korea"],
      },
    ]);
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    const flags = await within(table).findAllByRole("img");

    expect(flags.map((flag) => flag.getAttribute("aria-label"))).toEqual([
      "England",
      "Wales",
      "South Korea",
    ]);
  });

  it("keeps the Squad layout independent while querying added columns", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    usePlayerTableStore.getState().addColumns("search", ["attr.Acceleration"]);
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Accelerating Squad", 42),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    expect(
      within(table).queryByRole("columnheader", { name: "Acceleration" }),
    ).toBeNull();
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "CA" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "acceleration",
    );
    await user.click(screen.getByRole("option", { name: "Acceleration" }));

    expect(
      await screen.findByRole("columnheader", { name: "Acceleration" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        requestedFields: ["attr.Acceleration"],
      });
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "attr.Acceleration",
    );
  });

  it("reorders Squad columns by drag without changing its query, virtual row, or widths", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    const store = usePlayerTableStore.getState();
    store.addColumns("squad", ["attr.Acceleration", "attr.Agility"]);
    store.setColumnWidth("squad", "attr.Acceleration", 216);
    setSquadPlayersOverride(
      manySquadPlayers(101).map((player) => ({
        ...player,
        dynamicValues: { "attr.Acceleration": 16, "attr.Agility": 15 },
      })),
    );
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 50,
        requestedFields: ["attr.Acceleration", "attr.Agility"],
      });
    });
    const focusedRow = await waitFor(() => {
      const row = scroller.querySelector<HTMLElement>('[data-index="49"]');
      if (!row) {
        throw new Error("Expected the loaded virtual row.");
      }
      return row;
    });
    focusedRow.focus();
    const callCountBeforeReorder = getSquadPlayersCallCount();
    const accelerationHeader = within(table).getByRole("columnheader", {
      name: "Acceleration",
    });
    const agilityHeader = within(table).getByRole("columnheader", {
      name: "Agility",
    });
    Object.defineProperty(agilityHeader, "getBoundingClientRect", {
      configurable: true,
      value: () => ({ left: 0, right: 100, top: 0, bottom: 32, width: 100 }),
    });

    fireEvent.dragStart(
      within(accelerationHeader).getByRole("button", { name: "Acceleration" }),
      { dataTransfer: { effectAllowed: "move", setData: () => {} } },
    );
    fireEvent.dragOver(agilityHeader, { clientX: 90 });
    fireEvent.drop(agilityHeader, { clientX: 90 });
    fireEvent.dragEnd(
      within(accelerationHeader).getByRole("button", { name: "Acceleration" }),
    );

    await waitFor(() => {
      const headerLabels = within(table)
        .getAllByRole("columnheader")
        .map((header) => header.getAttribute("aria-label"));
      expect(headerLabels.indexOf("Agility")).toBeLessThan(
        headerLabels.indexOf("Acceleration"),
      );
    });
    const headerLabels = within(table)
      .getAllByRole("columnheader")
      .map((header) => header.getAttribute("aria-label"));
    const cellTexts = within(focusedRow)
      .getAllByRole("cell")
      .map((cell) => cell.textContent);
    expect(cellTexts[headerLabels.indexOf("Agility")]).toBe("15");
    expect(cellTexts[headerLabels.indexOf("Acceleration")]).toBe("16");
    expect(focusedRow).toHaveFocus();
    expect(scroller.scrollTop).toBe(1_950);
    expect(getSquadPlayersCallCount()).toBe(callCountBeforeReorder);
    expect(
      screen.getByRole("separator", { name: "Resize Acceleration column" }),
    ).toHaveAttribute("aria-valuenow", "216");
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
  });

  it("opens distinct format-bound CSV import modals from Squad", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    expect(
      screen.getByRole("button", { name: "Upload Moneyball CSV" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Upload Youth Academy CSV" }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Upload Moneyball CSV" }),
    );
    expect(
      await screen.findByRole("dialog", { name: "Upload Moneyball CSV" }),
    ).toHaveTextContent("Only a Moneyball export can be imported");
    await user.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Upload Moneyball CSV" }),
      ).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: "Upload Moneyball CSV" }),
    ).toHaveFocus();

    await user.click(
      screen.getByRole("button", { name: "Upload Youth Academy CSV" }),
    );
    expect(
      await screen.findByRole("dialog", {
        name: "Upload Youth Academy CSV",
      }),
    ).toHaveTextContent("Only a Youth Academy export can be imported");
  });

  it("confirms a Squad CA boost, locks the action, and refreshes affected views", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("pending");
    const { queryClient } = renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));

    const dialog = await screen.findByRole("dialog", {
      name: "Boost all CA?",
    });
    expect(dialog).toHaveTextContent(
      "Players aged 20 or younger receive +5 CA.",
    );
    expect(dialog).toHaveTextContent(
      "Players aged 21 through 28 receive +10 CA.",
    );
    expect(dialog).toHaveTextContent("Players aged 29 or older are skipped.");
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );

    expect(
      within(dialog).getByRole("button", { name: "Boosting…" }),
    ).toBeDisabled();
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toEqual([{}]);

    resolvePendingSquadCurrentAbilityBoostIpcMock();

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Updated 2 players. Skipped 1. Failed 0.",
    );
    await waitFor(() => {
      expect(invalidateQueries).toHaveBeenCalledTimes(5);
    });
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: snapshotKeys.all,
    });
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: searchKeys.all,
    });
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: playerKeys.all,
    });
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: plannerKeys.all,
    });
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: academyKeys.all,
    });
  });

  it("reports when a Squad CA boost needs Load Data before another attempt", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("recoveryRequired");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Boost all CA?" }),
      ).getByRole("button", { name: "Boost all CA" }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Updated 1 player. Skipped 2. Failed 1.",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Load Data is required before another boost.",
    );
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    const action = screen.getByRole("button", { name: "Boost all CA" });
    expect(action).toBeDisabled();
    await user.click(action);
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toEqual([{}]);
  });

  it("confirms the Squad Wonderkid action before applying it", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Make all Wonderkids?",
    });
    expect(dialog).toHaveTextContent(
      "Known Ambition, Professionalism, and Determination values at 10 or below can change.",
    );
    expect(dialog).toHaveTextContent(
      "Unknown and higher values are unchanged.",
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Make all Wonderkids" }),
    );
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toEqual([{}]);
  });

  it("locks both Squad actions while Wonderkid Mentality is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadWonderkidMentalityBoostIpcMockMode("pending");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Make all Wonderkids?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Make all Wonderkids" }),
    );

    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Applying…" }),
    ).toBeDisabled();
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toEqual([{}]);

    resolvePendingSquadWonderkidMentalityBoostIpcMock();
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Updated 2 players. Skipped 1. Failed 0.",
    );
  });

  it("requires Load Data before either Squad action after Wonderkid recovery", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadWonderkidMentalityBoostIpcMockMode("recoveryRequired");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    );
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Make all Wonderkids?" }),
      ).getByRole("button", { name: "Make all Wonderkids" }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Load Data is required before another boost.",
    );
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeDisabled();
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toEqual([{}]);
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toEqual([]);
  });

  it("sorts the Squad table through the URL and backend query", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara Scout", 1, 160),
      squadPlayerNamed("Alex Scout", 2, 150),
    ]);
    const { router } = renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByRole("button", { name: "Name" }));

    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        sort: "name",
        dir: "asc",
      });
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 0,
        limit: 50,
        sortBy: "name",
        sortDir: "asc",
        requestedFields: [],
      });
    });
    expect(
      within(screen.getByRole("table", { name: "Squad overview" })).getByRole(
        "columnheader",
        { name: "Name" },
      ),
    ).toHaveAttribute("aria-sort", "ascending");
    expect(screen.getByText("Alex Scout")).toBeInTheDocument();
  });

  it("moves focus between Squad rows with the arrow keys", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Alex Scout", 42, 160),
      squadPlayerNamed("Zara Scout", 43, 150),
    ]);
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    const rows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    rows[0].focus();
    await user.keyboard("{ArrowDown}");

    await waitFor(() => {
      expect(rows[1]).toHaveFocus();
    });
  });

  it("loads bounded virtual Squad pages without pagination controls", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    const scroller = screen.getByTestId("squad-overview-scroller");
    expect(scroller).toHaveClass("h-full", "min-h-0", "overflow-auto");
    expect(scroller.parentElement).toHaveClass("relative", "min-h-0", "flex-1");
    expect(
      screen.queryByRole("navigation", { name: "Squad overview pages" }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: "Previous page" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Next page" })).toBeNull();
    const virtualRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    expect(virtualRows.length).toBeGreaterThan(0);
    expect(virtualRows.length).toBeLessThan(101);

    fireEvent.scroll(scroller, { target: { scrollTop: 2_000 } });

    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 50,
        limit: 50,
      });
    });
    expect(await screen.findByText("Squad player 051")).toBeInTheDocument();

    fireEvent.scroll(scroller, { target: { scrollTop: 4_000 } });

    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 100,
        limit: 50,
      });
    });
    expect(await screen.findByText("Squad player 101")).toBeInTheDocument();
  });

  it("keeps ArrowDown focus pending while a virtual Squad page loads", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("pendingSecondPage");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({ offset: 50 });
    });

    const boundaryRow = await waitFor(() => {
      const row = scroller.querySelector<HTMLElement>('[data-index="49"]');
      if (!row) {
        throw new Error("Expected the loaded page boundary row.");
      }
      return row;
    });
    boundaryRow.focus();
    await user.keyboard("{ArrowDown}");
    expect(boundaryRow).toHaveFocus();

    resolvePendingSquadPlayersPageIpcMock();

    await waitFor(() => {
      expect(
        scroller.querySelector<HTMLElement>('[data-index="50"]'),
      ).toHaveFocus();
    });
  });

  it("does not reclaim focus after a pending virtual Squad page loses it", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("pendingSecondPage");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({ offset: 50 });
    });

    const boundaryRow = await waitFor(() => {
      const row = scroller.querySelector<HTMLElement>('[data-index="49"]');
      if (!row) {
        throw new Error("Expected the loaded page boundary row.");
      }
      return row;
    });
    boundaryRow.focus();
    await user.keyboard("{ArrowDown}");

    const plannerTab = screen.getByRole("tab", { name: "Planner" });
    plannerTab.focus();
    expect(plannerTab).toHaveFocus();

    resolvePendingSquadPlayersPageIpcMock();

    expect(await screen.findByText("Squad player 051")).toBeInTheDocument();
    expect(plannerTab).toHaveFocus();
  });

  it("offers a retry when a visible virtual Squad page fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("rejectSecondPageOnce");
    renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 2_000 } });

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Couldn't load this part of the table.",
    );
    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByText("Squad player 051")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    });
  });

  it("clamps the virtual Squad range after its data shrinks", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    const { queryClient } = renderPlannerRoute({ initialEntry: "/planner" });

    await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    let scrollHeight = 4_072;
    Object.defineProperties(scroller, {
      clientHeight: { configurable: true, value: 400 },
      scrollHeight: {
        configurable: true,
        get: () => scrollHeight,
      },
      scrollTop: { configurable: true, value: 3_672, writable: true },
    });
    fireEvent.scroll(scroller, { target: { scrollTop: 4_000 } });
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({ offset: 100 });
    });

    setSquadPlayersOverride(manySquadPlayers(11));
    scrollHeight = 472;
    await queryClient.invalidateQueries({ queryKey: plannerKeys.all });

    expect(await screen.findByText("Squad player 011")).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 0,
        limit: 50,
      });
      expect(scroller.scrollTop).toBe(72);
      expect(scroller.scrollTop + scroller.clientHeight).toBe(
        scroller.scrollHeight,
      );
    });
  });

  it("opens a Squad player from a non-name cell", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { router } = renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByText("Metro FC"));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/42");
    });
  });

  it("opens a focused Squad row with Enter and restores its sort on back", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara Scout", 42),
      squadPlayerNamed("Alex Scout", 43),
    ]);
    const { router } = renderPlannerRoute({ initialEntry: "/planner" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByRole("button", { name: "Name" }));
    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        sort: "name",
        dir: "asc",
      });
    });
    const sortedRow = await waitFor(() => {
      const currentTable = screen.getByRole("table", {
        name: "Squad overview",
      });
      const row = within(currentTable)
        .getAllByRole("row")
        .find((candidate) => candidate.hasAttribute("data-index"));
      if (!row) {
        throw new Error("expected a sorted virtualized Squad row");
      }
      expect(row).toHaveTextContent("Alex Scout");
      return row;
    });
    sortedRow.focus();
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/43");
    });

    await router.history.back();

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/planner");
      expect(router.state.location.search).toEqual({
        sort: "name",
        dir: "asc",
      });
    });
    const restoredTable = await screen.findByRole("table", {
      name: "Squad overview",
    });
    expect(
      within(restoredTable).getByRole("columnheader", { name: "Name" }),
    ).toHaveAttribute("aria-sort", "ascending");
  });

  it("uses the Squad default and replaces workspace URL state", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Barcelona",
      sources: [],
    });
    const { router } = renderPlannerRoute({ initialEntry: "/planner" });

    expect(
      await screen.findByText("Primary club: Barcelona"),
    ).toBeInTheDocument();
    const squadTab = screen.getByRole("tab", { name: "Squad" });
    expect(squadTab).toHaveAttribute("aria-selected", "true");
    squadTab.focus();
    await user.keyboard("{End}");

    const tacticTab = screen.getByRole("tab", { name: "Tactic" });
    expect(tacticTab).toHaveAttribute("aria-selected", "true");
    expect(tacticTab).toHaveFocus();
    expect(tacticTab).toHaveAttribute("tabIndex", "0");
    expect(squadTab).toHaveAttribute("tabIndex", "-1");
    expect(router.state.location.search).toEqual({ view: "tactic" });
    await user.keyboard("{ArrowLeft}");
    const plannerTab = screen.getByRole("tab", { name: "Planner" });
    expect(plannerTab).toHaveAttribute("aria-selected", "true");
    expect(plannerTab).toHaveFocus();
    expect(plannerTab).toHaveAttribute("tabIndex", "0");
    expect(tacticTab).toHaveAttribute("tabIndex", "-1");
    expect(router.state.location.search).toEqual({ view: "planner" });
    plannerTab.focus();
    await user.keyboard("{Home}");
    expect(squadTab).toHaveAttribute("aria-selected", "true");
    expect(squadTab).toHaveFocus();
    expect(squadTab).toHaveAttribute("tabIndex", "0");
    expect(plannerTab).toHaveAttribute("tabIndex", "-1");
    expect(router.state.location.search).toEqual({ view: "squad" });

    router.history.back();
    await waitFor(() =>
      expect(router.state.location.search).toEqual({ view: "squad" }),
    );
    expect(router.history.canGoBack()).toBe(false);
  });

  it("lets an explicit Planner workspace override the default", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Barcelona",
      sources: [],
    });
    renderPlannerRoute({ initialEntry: "/planner?view=planner" });

    expect(await screen.findByRole("tab", { name: "Planner" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByRole("region", { name: "Senior squad depth matrix" }),
    ).toBeVisible();
  });

  it("uses the Squad default for the retired Club Setup workspace", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=clubs" });

    expect(await screen.findByRole("tab", { name: "Squad" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("link", { name: "Open Club Setup" })).toBeVisible();
  });

  it("edits linked IP and OOP lanes with filtered roles and weight control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });

    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });
    const bothView = within(viewGroup).getByRole("button", { name: "Both" });
    expect(bothView).toHaveAttribute("aria-pressed", "true");
    const inspectors = screen.getAllByRole("region", {
      name: "Selected position settings",
    });
    expect(inspectors).toHaveLength(1);
    const inspector = inspectors[0];
    expect(
      within(inspector).getByRole("combobox", {
        name: "IP GK position",
      }),
    ).toBeInTheDocument();
    expect(
      within(inspector).getByRole("combobox", {
        name: "OOP GK position",
      }),
    ).toBeInTheDocument();
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
      name: "IP: GK · Goalkeeper",
    });
    firstIpLane.focus();
    await user.keyboard("{Enter}");

    const ipPosition = screen.getByRole("combobox", {
      name: "IP GK position",
    });
    await user.selectOptions(ipPosition, "DL");

    const ipRole = screen.getByRole("combobox", { name: "IP DL role" });
    expect(ipRole).toHaveValue("");
    expect(
      within(ipRole).queryByRole("option", { name: "Goalkeeper" }),
    ).not.toBeInTheDocument();
    await user.selectOptions(ipRole, "full_back_ip");

    const oopPosition = screen.getByRole("combobox", {
      name: "OOP GK position",
    });
    await user.selectOptions(oopPosition, "DL");
    const oopRole = screen.getByRole("combobox", {
      name: "OOP DL role",
    });
    expect(oopRole).toHaveValue("");
    await user.selectOptions(oopRole, "holding_full_back_oop");

    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    expect(
      screen.getAllByRole("slider", { name: "IP/OOP score weight" }),
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

  it("orders the tactic command bar, pitches, and one settings shelf", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    const commandBar = await screen.findByRole("region", {
      name: "Tactic controls",
    });
    const pitches = screen.getAllByRole("group", { name: /pitch$/ });
    const settings = screen.getByRole("region", {
      name: "Selected position settings",
    });

    expect(
      within(commandBar).queryByRole("heading", { name: "Tactic editor" }),
    ).not.toBeInTheDocument();
    expect(
      within(commandBar).getByRole("group", {
        name: "Tactic phase views",
      }),
    ).toBeInTheDocument();
    expect(
      within(commandBar).queryByText(
        "IP: GK · Goalkeeper / OOP: GK · Line-Holding Keeper",
      ),
    ).not.toBeInTheDocument();
    expect(
      within(commandBar).queryByText("11 linked positions"),
    ).not.toBeInTheDocument();
    expect(
      within(commandBar).getByRole("button", { name: "Save tactic" }),
    ).toBeInTheDocument();
    expect(pitches).toHaveLength(2);
    expect(
      commandBar.compareDocumentPosition(pitches[0]) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      pitches[1].compareDocumentPosition(settings) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      within(settings).getAllByRole("slider", {
        name: "IP/OOP score weight",
      }),
    ).toHaveLength(1);
    expect(
      within(settings).getAllByRole("combobox", {
        name: "Importance rank",
      }),
    ).toHaveLength(1);
    expect(
      within(settings).getByRole("group", {
        name: "In-Possession settings",
      }),
    ).toBeInTheDocument();
    expect(
      within(settings).getByRole("group", {
        name: "Out-of-Possession settings",
      }),
    ).toBeInTheDocument();
  });

  it("renders every tactic pitch from attack to goalkeeper", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });

    for (const view of ["Both", "IP", "OOP"] as const) {
      await user.click(within(viewGroup).getByRole("button", { name: view }));

      const pitches = screen.getAllByRole("group", { name: /pitch$/ });
      expect(pitches).toHaveLength(view === "Both" ? 2 : 1);

      for (const pitch of pitches) {
        const positionButtons = within(pitch).getAllByRole("button");
        expect(positionButtons[0]).toHaveAccessibleName(/: STC · /);
        expect(
          positionButtons[positionButtons.length - 1],
        ).toHaveAccessibleName(/: GK · /);
      }
    }
  });

  it("presents current linked positions without lane terminology", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes[8] = {
      ...tactic.lanes[8],
      ipPosition: "AMC",
      ipRoleId: "winger_ip",
    };
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    const ipButton = await screen.findByRole("button", {
      name: /IP: AMC · Winger/,
    });
    const oopButton = screen.getByRole("button", {
      name: /OOP: ML · Tracking Wide Midfielder/,
    });
    expect(screen.queryByText("11 linked positions")).not.toBeInTheDocument();
    expect(screen.queryByText("Left winger")).not.toBeInTheDocument();
    expect(screen.queryByText(/linked lanes/i)).not.toBeInTheDocument();

    fireEvent.focus(ipButton);
    await waitFor(() => expect(ipButton).toHaveClass("ring-2"));
    expect(oopButton).toHaveClass("ring-2");

    await user.click(ipButton);
    const weight = screen.getByRole("slider", { name: "IP/OOP score weight" });
    weight.focus();
    expect(ipButton).toHaveClass("ring-2");
    expect(oopButton).toHaveClass("ring-2");

    const alternateIpButton = screen.getByRole("button", {
      name: "IP: DL · Full-Back",
    });
    alternateIpButton.focus();
    weight.focus();
    expect(oopButton).toHaveClass("ring-2");

    await openPlannerWorkspace(user, "planner");
    const matrix = screen.getByRole("region", {
      name: "Senior squad depth matrix",
    });
    expect(within(matrix).getByText("IP: AMC · Winger")).toBeInTheDocument();
    expect(within(matrix).queryByText("Left winger")).not.toBeInTheDocument();
  });

  it("keeps repeated positions distinguishable without numeric labels", () => {
    const tactic = resolvePlannerTacticIpcMock();
    const lanes = tactic.lanes.map((lane, index) =>
      index < 5 ? { ...lane, ipPosition: "AMC", ipRoleId: "winger_ip" } : lane,
    );
    const labels = lanes
      .slice(0, 5)
      .map((lane) => phasePositionLabel(lane, "ip", lanes));

    expect(new Set(labels).size).toBe(5);
    expect(labels.every((label) => !label.includes("additional"))).toBe(true);
  });

  it("arranges repeated positions in stable central slots regardless of role", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes = tactic.lanes.map((lane, index) => {
      if (index === 0) {
        return {
          ...lane,
          ipPosition: "MC",
          ipRoleId: "central_midfielder_ip",
          oopPosition: "MC",
          oopRoleId: "pressing_central_midfielder_oop",
        };
      }
      if (index === 1) {
        return {
          ...lane,
          ipPosition: "MC",
          ipRoleId: "advanced_playmaker_ip",
          oopPosition: "MC",
          oopRoleId: "pressing_central_midfielder_oop",
        };
      }
      if (index === 2) {
        return {
          ...lane,
          ipPosition: "MC",
          ipRoleId: "box_to_box_midfielder_ip",
          oopPosition: "MC",
          oopRoleId: "pressing_central_midfielder_oop",
        };
      }
      if (index === 3) {
        return {
          ...lane,
          ipPosition: "DC",
          ipRoleId: "centre_back_ip",
          oopPosition: "DC",
          oopRoleId: "covering_centre_back_oop",
        };
      }
      if (index === 4) {
        return {
          ...lane,
          ipPosition: "DC",
          ipRoleId: "ball_playing_centre_back_ip",
          oopPosition: "DC",
          oopRoleId: "stopping_centre_back_oop",
        };
      }
      if (index === 6) {
        return {
          ...lane,
          ipPosition: "ML",
          ipRoleId: "wide_midfielder_ip",
          oopPosition: "ML",
          oopRoleId: "tracking_wide_midfielder_oop",
        };
      }
      if (index === 7) {
        return {
          ...lane,
          ipPosition: "MR",
          ipRoleId: "wide_midfielder_ip",
          oopPosition: "MR",
          oopRoleId: "tracking_wide_midfielder_oop",
        };
      }
      return lane;
    });
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    const rightMc = await screen.findByRole("button", {
      name: "IP: MCR · Central Midfielder",
    });
    const centreMc = screen.getByRole("button", {
      name: "IP: MC · Advanced Playmaker",
    });
    const leftMc = screen.getByRole("button", {
      name: "IP: MCL · Box-to-Box Midfielder",
    });
    const mcGroup = rightMc.closest('[data-position-group="MC"]');
    expect(mcGroup).not.toBeNull();
    expect(mcGroup).toContainElement(centreMc);
    expect(mcGroup).toContainElement(leftMc);
    expect(mcGroup).toHaveAttribute("data-position-slot-count", "3");
    for (const pitch of await screen.findAllByRole("group", {
      name: /pitch$/,
    })) {
      expect(pitch).toHaveAttribute("data-pitch-slot-count", "5");
    }
    expect(rightMc.parentElement).toHaveStyle({
      gridColumn: "5 / span 2",
      gridRow: "1",
    });
    expect(centreMc.parentElement).toHaveStyle({
      gridColumn: "3 / span 2",
      gridRow: "1",
    });
    expect(leftMc.parentElement).toHaveStyle({
      gridColumn: "1 / span 2",
      gridRow: "1",
    });
    expect(mcGroup).toHaveStyle({ gridColumn: "3 / span 6" });
    expect(mcGroup).toHaveClass("bg-surface-container-high");

    const rightDc = screen.getByRole("button", {
      name: "IP: DCR · Centre-Back",
    });
    const leftDc = screen.getByRole("button", {
      name: "IP: DCL · Ball-Playing Centre-Back",
    });
    expect(rightDc.parentElement).toHaveStyle({
      gridColumn: "3 / span 2",
      gridRow: "1",
    });
    expect(leftDc.parentElement).toHaveStyle({
      gridColumn: "1 / span 2",
      gridRow: "1",
    });
    const dcGroup = rightDc.closest('[data-position-group="DC"]');
    expect(dcGroup).not.toBeNull();
    expect(dcGroup).toContainElement(leftDc);
    expect(dcGroup).toHaveStyle({ gridColumn: "4 / span 4" });
    const defensiveMidfielder = screen.getByRole("button", {
      name: "IP: DM · Defensive Midfielder",
    });
    expect(defensiveMidfielder).toBeInTheDocument();
    expect(defensiveMidfielder.parentElement).toHaveStyle({
      gridColumn: "1 / span 2",
      gridRow: "1",
    });
    expect(
      defensiveMidfielder.closest('[data-position-group="DM"]'),
    ).toHaveStyle({ gridColumn: "5 / span 2" });
    const leftMidfielder = screen.getByRole("button", {
      name: "IP: ML · Wide Midfielder",
    });
    const rightMidfielder = screen.getByRole("button", {
      name: "IP: MR · Wide Midfielder",
    });
    expect(leftMidfielder.parentElement).toHaveStyle({
      gridColumn: "1 / span 2",
      gridRow: "1",
    });
    expect(rightMidfielder.parentElement).toHaveStyle({
      gridColumn: "1 / span 2",
      gridRow: "1",
    });
    expect(leftMidfielder.closest('[data-position-group="ML"]')).toHaveStyle({
      gridColumn: "1 / span 2",
    });
    expect(rightMidfielder.closest('[data-position-group="MR"]')).toHaveStyle({
      gridColumn: "9 / span 2",
    });

    expect(
      screen.getByRole("button", {
        name: "OOP: DCR · Covering Centre-Back",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "OOP: DCL · Stopping Centre-Back",
      }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /^IP:/ })).toHaveLength(11);
    expect(
      screen.getByRole("button", { name: "IP: STC · Centre Forward" }),
    ).toBeInTheDocument();
  });

  it("keeps every position button when a base position has more than three lanes", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes = tactic.lanes.map((lane, index) =>
      index < 5 ? { ...lane, ipPosition: "AMC", ipRoleId: "winger_ip" } : lane,
    );
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    for (const pitch of await screen.findAllByRole("group", {
      name: /pitch$/,
    })) {
      expect(pitch).toHaveAttribute("data-pitch-slot-count", "5");
    }
    const secondRowRight = await screen.findByRole("button", {
      name: "IP: AMCR (row 2) · Winger",
    });
    const secondRowLeft = screen.getByRole("button", {
      name: "IP: AMCL (row 2) · Winger",
    });
    expect(secondRowRight).toBeInTheDocument();
    expect(secondRowRight.parentElement).toHaveStyle({
      gridColumn: "3 / span 2",
      gridRow: "1",
    });
    const secondRowGroup = secondRowRight.closest(
      '[data-position-group="AMC"]',
    );
    expect(secondRowGroup).not.toBeNull();
    expect(secondRowGroup).toContainElement(secondRowLeft);
    expect(secondRowGroup).toHaveStyle({ gridColumn: "4 / span 4" });
    expect(secondRowGroup).toHaveAttribute("data-position-slot-count", "2");
    expect(screen.getAllByRole("button", { name: /^IP:/ })).toHaveLength(11);
  });

  it("follows visible row order when a wide position overflows", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes = tactic.lanes.map((lane, index) => {
      if (index < 2) {
        return { ...lane, ipPosition: "AML", ipRoleId: "winger_ip" };
      }
      if (index === 2) {
        return { ...lane, ipPosition: "AMC", ipRoleId: "winger_ip" };
      }
      if (index === 3) {
        return { ...lane, ipPosition: "AMR", ipRoleId: "winger_ip" };
      }
      if (lane.ipPosition === "AML" || lane.ipPosition === "AMR") {
        return {
          ...lane,
          ipPosition: "MC",
          ipRoleId: "central_midfielder_ip",
        };
      }
      return lane;
    });
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    const ipPitch = (
      await screen.findAllByRole("group", { name: /pitch$/ })
    )[0];
    const attackMidfieldRows = ipPitch.querySelectorAll(
      '[data-pitch-band="attack-midfield"]',
    );
    expect(attackMidfieldRows).toHaveLength(2);
    expect(
      within(attackMidfieldRows[0] as HTMLElement)
        .getAllByRole("button")
        .map((button) => button.getAttribute("aria-label")),
    ).toEqual(["IP: AML · Winger", "IP: AMC · Winger", "IP: AMR · Winger"]);
    expect(
      within(attackMidfieldRows[1] as HTMLElement)
        .getAllByRole("button")
        .map((button) => button.getAttribute("aria-label")),
    ).toEqual(["IP: AML (row 2) · Winger"]);
  });

  it("keeps a three-slot minimum when every tactic row has at most two positions", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const tactic = resolvePlannerTacticIpcMock();
    const compactRows = [
      ["ST", "centre_forward_ip", "central_outlet_centre_forward_oop"],
      ["ST", "centre_forward_ip", "central_outlet_centre_forward_oop"],
      ["AML", "winger_ip", "tracking_winger_oop"],
      ["AMR", "winger_ip", "tracking_winger_oop"],
      ["MC", "central_midfielder_ip", "pressing_central_midfielder_oop"],
      ["MC", "central_midfielder_ip", "pressing_central_midfielder_oop"],
      ["DM", "defensive_midfielder_ip", "screening_defensive_midfielder_oop"],
      ["DM", "defensive_midfielder_ip", "screening_defensive_midfielder_oop"],
      ["DC", "centre_back_ip", "covering_centre_back_oop"],
      ["DC", "centre_back_ip", "covering_centre_back_oop"],
      ["GK", "goalkeeper_ip", "line_holding_keeper_oop"],
    ] as const;
    tactic.lanes = tactic.lanes.map((lane, index) => {
      const [position, ipRoleId, oopRoleId] = compactRows[index];
      return {
        ...lane,
        ipPosition: position,
        ipRoleId,
        oopPosition: position,
        oopRoleId,
      };
    });
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    for (const pitch of await screen.findAllByRole("group", {
      name: /pitch$/,
    })) {
      expect(pitch).toHaveAttribute("data-pitch-slot-count", "3");
    }
    expect(screen.getAllByRole("button", { name: /^IP:/ })).toHaveLength(11);
  });

  it("retains the edited tactic draft when save fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerTacticSaveError("Tactic save failed");
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
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

  it("keeps phase controls in one inspector for each tactic view", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });

    for (const [view, phase, hiddenPhase] of [
      ["IP", "IP", "OOP"],
      ["OOP", "OOP", "IP"],
      ["Both", "IP", ""],
    ] as const) {
      await user.click(within(viewGroup).getByRole("button", { name: view }));
      const inspectors = screen.getAllByRole("region", {
        name: "Selected position settings",
      });
      expect(inspectors).toHaveLength(1);
      const inspector = inspectors[0];
      expect(
        within(inspector).getByRole("combobox", {
          name: `${phase} GK position`,
        }),
      ).toBeInTheDocument();
      if (hiddenPhase) {
        expect(
          within(inspector).queryByRole("combobox", {
            name: `${hiddenPhase} GK position`,
          }),
        ).not.toBeInTheDocument();
      }
    }
  });

  it("saves only the selected lane score weight", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    await user.click(
      screen.getByRole("button", { name: "IP: DL · Full-Back" }),
    );
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
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
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    await user.click(
      screen.getByRole("button", { name: "IP: DL · Full-Back" }),
    );
    const rank = screen.getByRole("combobox", {
      name: "Importance rank",
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
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const preferredFoot = screen.getByRole("combobox", {
      name: "Preferred foot",
    });
    const footPreference = screen.getByRole("combobox", {
      name: "Foot preference",
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
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const preferredFoot = screen.getByRole("combobox", {
      name: "Preferred foot",
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
    renderPlannerRoute({ initialEntry: "/planner?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Importance rank" }),
      "1",
    );
    await user.click(
      screen.getByRole("button", { name: "IP: DL · Full-Back" }),
    );
    const duplicateRank = screen.getByRole("combobox", {
      name: "Importance rank",
    });
    await user.selectOptions(duplicateRank, "1");

    expect(screen.getByRole("alert")).toHaveTextContent(
      "IP: DL · Full-Back / OOP: DL · Holding Full-Back cannot use importance rank 1; it is already used.",
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
      name: /Senior, 1st string, IP: GK .* Empty/,
    });
    await user.click(cell);
    expect(
      await screen.findByRole("option", { name: /Old tactic fit/ }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    await openPlannerWorkspace(user, "tactic");

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
      name: "IP/OOP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    await openPlannerWorkspace(user, "planner");
    await user.click(cell);
    expect(
      await screen.findByRole("option", { name: /Updated tactic fit/ }),
    ).toBeInTheDocument();
  });

  it("resets a dirty tactic draft when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute({
      initialEntry: "/planner?view=tactic",
    });

    await screen.findByRole("region", { name: "Tactic controls" });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
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
        screen.getByRole("slider", { name: "IP/OOP score weight" }),
      ).toHaveValue("20"),
    );
  });

  it("blocks tactic saves while active-save data refreshes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute({
      initialEntry: "/planner?view=tactic",
    });

    await screen.findByRole("region", { name: "Tactic controls" });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
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
    const { queryClient } = renderPlannerRoute({
      initialEntry: "/planner?view=tactic",
    });

    await screen.findByRole("region", { name: "Tactic controls" });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
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
    expect(matrix).toHaveClass("overflow-auto");
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
      within(matrix).getByRole("img", {
        name: /Current combined role score: 82/,
      }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("img", {
        name: /Potential combined role score: 91/,
      }),
    ).toBeInTheDocument();
    expect(within(matrix).getByText("Outside pool")).toBeInTheDocument();
    expect(within(matrix).getByText("Unresolved")).toBeInTheDocument();
    expect(
      within(matrix).getByRole("button", {
        name: /Missing Centre-Back/,
      }),
    ).toBeInTheDocument();
    const unavailableCell = within(matrix).getByRole("button", {
      name: /No Score Player, Resolved, current score —, potential score —/,
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
      name: /Reserves, 1st string, IP: GK .* Empty/,
    })[0];
    expect(cell).not.toBeDisabled();
    cell.focus();
    expect(document.activeElement).toBe(cell);
  });

  it("groups squad actions above a bounded compact matrix", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(withDepthAssignments(resolvePlannerDepthIpcMock()));
    renderPlannerRoute();

    const toolbar = await screen.findByRole("group", {
      name: "Squad controls",
    });
    expect(
      within(toolbar).getByRole("tablist", { name: "Squad planner teams" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Optimize squads" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Clear all" }),
    ).toBeInTheDocument();

    const matrix = screen.getByRole("region", {
      name: "Senior squad depth matrix",
    });
    expect(matrix).toHaveClass("max-h-[min(70vh,720px)]");
    expect(matrix).toHaveClass("overflow-auto");
    expect(within(matrix).getByRole("row", { name: /Goalkeeper/ })).toHaveClass(
      "h-table-row-height-two-line",
    );
  });

  it("groups all teams in one semantic table when the matrix fits", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondStringForEveryTeam(resolvePlannerDepthIpcMock()),
    );
    renderPlannerRoute();
    await setPlannerMatrixWidth(1600);

    const matrix = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    expect(
      within(matrix).getByRole("columnheader", { name: "Senior squad" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("columnheader", { name: "Reserves squad" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("columnheader", { name: "Youth squad" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getAllByRole("columnheader", { name: "1st string" }),
    ).toHaveLength(3);
    expect(
      within(matrix).getByRole("button", {
        name: /Youth, 2nd string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    expect(
      within(matrix)
        .getByRole("button", { name: /Reserves, 1st string, IP: GK .* Empty/ })
        .closest("td"),
    ).toHaveAttribute(
      "headers",
      expect.stringContaining("planner-team-reserves"),
    );
    expect(
      screen.queryByRole("tab", { name: "Senior" }),
    ).not.toBeInTheDocument();

    const clearAll = screen.getByRole("button", { name: "Clear all" });
    expect(
      within(matrix).queryByRole("button", { name: /Clear .* squad/ }),
    ).toBeNull();
    await user.click(clearAll);
    const confirmation = screen.getByRole("dialog", {
      name: "Clear all squads?",
    });
    expect(confirmation).toHaveTextContent("Senior, Reserves, and Youth");
    await user.click(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(document.activeElement).toBe(clearAll));
  });

  it("keeps the selected team and string focus across responsive mode changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondStringForEveryTeam(resolvePlannerDepthIpcMock()),
    );
    renderPlannerRoute();
    await setPlannerMatrixWidth(1200);

    await user.click(await screen.findByRole("tab", { name: "Reserves" }));
    const reservesTab = screen.getByRole("tab", { name: "Reserves" });
    reservesTab.focus();
    await setPlannerMatrixWidth(1600);
    const combinedFromTab = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    await waitFor(() =>
      expect(document.activeElement).toBe(
        within(combinedFromTab).getAllByRole("button", {
          name: "Manage 1st string",
        })[1],
      ),
    );
    await setPlannerMatrixWidth(1200);
    const constrainedAfterTab = await screen.findByRole("region", {
      name: "Reserves squad depth matrix",
    });
    await waitFor(() =>
      expect(
        within(constrainedAfterTab).getByRole("button", {
          name: "Manage 1st string",
        }),
      ).toHaveFocus(),
    );

    const constrainedHeader = within(
      screen.getByRole("region", { name: "Reserves squad depth matrix" }),
    ).getByRole("button", { name: "Manage 1st string" });
    constrainedHeader.focus();

    await setPlannerMatrixWidth(1600);
    const combined = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    const combinedHeaders = within(combined).getAllByRole("button", {
      name: "Manage 1st string",
    });
    await waitFor(() =>
      expect(document.activeElement).toBe(combinedHeaders[1]),
    );

    await setPlannerMatrixWidth(1200);
    const constrained = await screen.findByRole("region", {
      name: "Reserves squad depth matrix",
    });
    await waitFor(() =>
      expect(document.activeElement).toBe(
        within(constrained).getByRole("button", { name: "Manage 1st string" }),
      ),
    );

    const constrainedCell = within(constrained).getByRole("button", {
      name: /Reserves, 1st string, IP: GK .* Empty/,
    });
    constrainedCell.focus();
    await setPlannerMatrixWidth(1600);
    await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    await setPlannerMatrixWidth(1200);
    const constrainedFromCell = await screen.findByRole("region", {
      name: "Reserves squad depth matrix",
    });
    await waitFor(() =>
      expect(document.activeElement).toBe(
        within(constrainedFromCell).getByRole("button", {
          name: /Reserves, 1st string, IP: GK .* Empty/,
        }),
      ),
    );

    await setPlannerMatrixWidth(1600);
    const combinedFromClear = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    await waitFor(() =>
      expect(
        within(combinedFromClear).getByRole("button", {
          name: /Reserves, 1st string, IP: GK .* Empty/,
        }),
      ).toHaveFocus(),
    );
    const clearAll = screen.getByRole("button", { name: "Clear all" });
    clearAll.focus();
    await setPlannerMatrixWidth(1200);
    await screen.findByRole("region", {
      name: "Reserves squad depth matrix",
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Clear all" })).toHaveFocus(),
    );
  });

  it("keeps the acted-on team visible when adding a string crosses the fit threshold", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();
    await setPlannerMatrixWidth(900);

    const combined = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    within(combined).getByRole("columnheader", {
      name: "Senior squad",
    });
    const seniorHeader = within(combined).getAllByRole("columnheader", {
      name: "1st string",
    })[0];
    await user.click(
      within(seniorHeader).getByRole("button", { name: "Manage 1st string" }),
    );
    await user.click(
      within(seniorHeader).getByRole("menuitem", { name: "Add string" }),
    );

    const constrained = await screen.findByRole("region", {
      name: "Senior squad depth matrix",
    });
    const addedHeader = await within(constrained).findByRole("columnheader", {
      name: "2nd string",
    });
    expect(addedHeader).toBeInTheDocument();
    await waitFor(() =>
      expect(document.activeElement).toHaveAccessibleName("Manage 2nd string"),
    );
  });

  it("announces only the latest successful squad action", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerOptimizeDepth(
      withReserveGoalkeeper(resolvePlannerDepthIpcMock()),
    );
    renderPlannerRoute();

    await user.click(
      await screen.findByRole("button", { name: "Optimize squads" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Squads optimized by current scores.",
    );

    await user.click(screen.getByRole("button", { name: "Clear all" }));
    const confirmation = screen.getByRole("dialog", {
      name: "Clear all squads?",
    });
    await user.click(
      within(confirmation).getByRole("button", { name: "Clear all" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "All squads cleared.",
      ),
    );
    expect(screen.getAllByRole("status")).toHaveLength(1);
  });

  it("opens a slot-fit picker from an empty matrix cell", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const cell = await screen.findByRole("button", {
      name: /Senior, 1st string, IP: GK .* Empty/,
    });
    await user.click(cell);

    expect(
      screen.getByRole("dialog", {
        name: `Find a player for ${KEEPER_POSITION}`,
      }),
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
        name: /Senior, 1st string, IP: GK .* Empty/,
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
        name: /Senior, 1st string, IP: GK .* Empty/,
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
        name: /Senior, 2nd string, IP: GK .* Empty/,
      }),
    );
    const cacheKeeper = await screen.findByRole("option", {
      name: /Cache Keeper/,
    });
    expect(cacheKeeper).toHaveTextContent(`Assigned: ${SENIOR_FIRST_KEEPER}`);
    await user.click(cacheKeeper);

    expect(
      screen.getByRole("dialog", { name: "Move Cache Keeper?" }),
    ).toHaveTextContent(
      `Move Cache Keeper from ${SENIOR_FIRST_KEEPER} to ${SENIOR_SECOND_KEEPER}?`,
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
                        potentialCombinedScore: null,
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
        name: /Senior, 2nd string, IP: GK .* Empty/,
      }),
    );
    expect(
      await screen.findByRole("option", { name: /String Keeper/ }),
    ).toHaveTextContent(`Assigned: ${SENIOR_FIRST_KEEPER}`);
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
        name: /Senior, 1st string, IP: GK .* Empty/,
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
      name: /Reserves, 1st string, IP: GK .* Reserve Keeper, Resolved/,
    });
    const emptyCell = screen.getByRole("button", {
      name: /Reserves, 2nd string, IP: GK .* Empty/,
    });

    await user.click(emptyCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent(`Assigned: ${RESERVES_FIRST_KEEPER}`);
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(occupiedCell);
    const clearDialog = screen.getByRole("dialog", {
      name: "Clear Reserve Keeper?",
    });
    expect(clearDialog).toHaveTextContent(
      `Reserve Keeper is assigned to ${RESERVES_FIRST_KEEPER}. It must be cleared before assigning or moving a player.`,
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
      /Reserves, 1st string, IP: GK .* Empty/,
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
        name: /Senior, 1st string, IP: GK .* Empty/,
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
      `Move Reserve Keeper from ${RESERVES_FIRST_KEEPER} to ${SENIOR_FIRST_KEEPER}?`,
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
        name: /Reserves, 1st string, IP: GK .* Empty/,
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
      name: /Senior, 1st string, IP: GK .* Empty/,
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
        name: /Senior, 1st string, IP: GK .* Empty/,
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
                        potentialCombinedScore: null,
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

  it("returns focus to the string trigger when Escape closes its menu", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const trigger = await screen.findByRole("button", {
      name: "Manage 1st string",
    });
    await user.click(trigger);
    screen.getByRole("menuitem", { name: "Add string" }).focus();

    await user.keyboard("{Escape}");

    expect(
      screen.queryByRole("menu", { name: "1st string actions" }),
    ).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it("closes a keyboard-opened string menu with Escape from its trigger", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    const trigger = await screen.findByRole("button", {
      name: "Manage 1st string",
    });
    trigger.focus();
    await user.keyboard("{Enter}{Escape}");

    expect(
      screen.queryByRole("menu", { name: "1st string actions" }),
    ).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
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

  it("confirms clearing every squad and reconciles all candidates", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const depth = withAllTeamDepthAssignments(resolvePlannerDepthIpcMock());
    setPlannerDepthIpcMock(depth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Senior Keeper" }),
      slotCandidate({ playerUid: 79, name: "Reserve Keeper" }),
      slotCandidate({ playerUid: 80, name: "Youth Keeper" }),
    ]);
    renderPlannerRoute({ staleTime: 60_000 });

    const secondSeniorCell = await screen.findByRole("button", {
      name: /Senior, 2nd string, IP: GK .* Empty/,
    });
    await user.click(secondSeniorCell);
    expect(
      await screen.findByRole("option", { name: /Senior Keeper/ }),
    ).toHaveTextContent(`Assigned: ${SENIOR_FIRST_KEEPER}`);
    const candidateFetchesBeforeClear = getPlannerSlotCandidateFetchCount();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    const clearButton = await screen.findByRole("button", {
      name: "Clear all",
    });
    clearButton.focus();
    await user.keyboard("{Enter}");
    const confirmation = screen.getByRole("dialog", {
      name: "Clear all squads?",
    });
    expect(confirmation).toHaveTextContent(
      "This clears every assignment from Senior, Reserves, and Youth.",
    );
    await user.click(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(document.activeElement).toBe(clearButton));
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerClearAllError("Clear all failed");
    await user.click(clearButton);
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Clear all squads?" }),
      ).getByRole("button", { name: "Clear all" }),
    );
    expect(
      await within(
        screen.getByRole("dialog", { name: "Clear all squads?" }),
      ).findByRole("alert"),
    ).toHaveTextContent("Clear all failed");
    expect(
      screen.getByRole("button", { name: /Senior Keeper, Resolved/ }),
    ).toBeInTheDocument();

    setPlannerClearAllError(null);
    const confirmButton = within(
      screen.getByRole("dialog", { name: "Clear all squads?" }),
    ).getByRole("button", { name: "Clear all" });
    await user.click(confirmButton);
    expect(getPlannerClearAllIpcMockCalls()).toBe(2);
    expect(await screen.findByText("All squads cleared.")).toBeInTheDocument();
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: /Senior Keeper, Resolved/ }),
      ).not.toBeInTheDocument(),
    );
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", {
        name: /Reserves, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Youth" }));
    expect(
      screen.getByRole("button", {
        name: /Youth, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Senior" }));
    await user.click(secondSeniorCell);
    expect(
      await screen.findByRole("option", { name: /Senior Keeper/ }),
    ).toHaveTextContent("Unassigned");
    expect(getPlannerSlotCandidateFetchCount()).toBe(
      candidateFetchesBeforeClear + 1,
    );
    await user.keyboard("{Escape}");
  });

  it("prevents duplicate clear-all requests while confirmation is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerClearAllPending(true);
    renderPlannerRoute();

    await user.click(await screen.findByRole("button", { name: "Clear all" }));
    const confirmButton = within(
      screen.getByRole("dialog", { name: "Clear all squads?" }),
    ).getByRole("button", { name: "Clear all" });
    await user.click(confirmButton);
    await user.click(confirmButton);

    expect(getPlannerClearAllIpcMockCalls()).toBe(1);
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
      name: /Senior, 1st string, IP: GK .* Empty/,
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
      expect(
        screen.getByText("Squads optimized by current scores."),
      ).toBeInTheDocument(),
    );
    expect(getPlannerOptimizeIpcMockCalls()).toBe(1);
    expect(getPlannerOptimizeIpcMockBases()).toEqual(["current"]);
    await user.click(screen.getByRole("tab", { name: "Reserves" }));
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Senior" }));
    await user.click(seniorCell);
    expect(
      await screen.findByRole("option", { name: /Reserve Keeper/ }),
    ).toHaveTextContent(`Assigned: ${RESERVES_FIRST_KEEPER}`);
    await user.keyboard("{Escape}");
    await user.click(
      screen.getByRole("button", { name: "Optimize by potential" }),
    );
    await waitFor(() =>
      expect(
        screen.getByText("Squads optimized by potential."),
      ).toBeInTheDocument(),
    );
    expect(getPlannerOptimizeIpcMockBases()).toEqual(["current", "potential"]);
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
        name: /Senior, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Optimize by potential" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Potential optimization failed: Optimize failed",
    );
    expect(getPlannerOptimizeIpcMockBases()).toEqual(["current", "potential"]);
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
    const potentialButton = screen.getByRole("button", {
      name: "Optimize by potential",
    });
    await user.click(optimizeButton);
    await user.click(optimizeButton);
    await user.click(potentialButton);

    expect(getPlannerOptimizeIpcMockCalls()).toBe(1);
    expect(optimizeButton).toBeDisabled();
    expect(potentialButton).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear all" })).toBeDisabled();
    expect(optimizeButton).toHaveAccessibleName("Optimizing current…");
    expect(getPlannerOptimizeIpcMockBases()).toEqual(["current"]);
  });

  it("identifies a pending potential optimization", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerOptimizePending(true);
    renderPlannerRoute();

    const potentialButton = await screen.findByRole("button", {
      name: "Optimize by potential",
    });
    await user.click(potentialButton);

    expect(getPlannerOptimizeIpcMockCalls()).toBe(1);
    expect(potentialButton).toHaveAccessibleName("Optimizing potential…");
    expect(
      screen.getByRole("button", { name: "Optimize squads" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear all" })).toBeDisabled();
    expect(getPlannerOptimizeIpcMockBases()).toEqual(["potential"]);
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
                    potentialCombinedScore: null,
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
                    potentialCombinedScore: 91,
                  },
                  {
                    id: 102,
                    laneId: "left_back",
                    playerUid: 78,
                    lastKnownName: "Outside Full-Back",
                    currentName: "Outside Full-Back",
                    state: "outside_pool",
                    combinedScore: 61,
                    potentialCombinedScore: 70,
                  },
                  {
                    id: 103,
                    laneId: "left_centre_back",
                    playerUid: 79,
                    lastKnownName: "Missing Centre-Back",
                    currentName: null,
                    state: "unresolved",
                    combinedScore: null,
                    potentialCombinedScore: null,
                  },
                  {
                    id: 104,
                    laneId: "right_back",
                    playerUid: 80,
                    lastKnownName: "No Score Player",
                    currentName: "No Score Player",
                    state: "resolved",
                    combinedScore: null,
                    potentialCombinedScore: null,
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

function withAllTeamDepthAssignments(depth: PlannerDepth): PlannerDepth {
  return {
    ...depth,
    teams: depth.teams.map((team) => {
      const assignment =
        team.team === "senior"
          ? {
              id: 101,
              laneId: "goalkeeper",
              playerUid: 77,
              lastKnownName: "Senior Keeper",
              currentName: "Senior Keeper",
              state: "resolved" as const,
              combinedScore: 82,
              potentialCombinedScore: null,
            }
          : team.team === "reserves"
            ? {
                id: 102,
                laneId: "goalkeeper",
                playerUid: 79,
                lastKnownName: "Reserve Keeper",
                currentName: "Reserve Keeper",
                state: "resolved" as const,
                combinedScore: 80,
                potentialCombinedScore: null,
              }
            : {
                id: 103,
                laneId: "goalkeeper",
                playerUid: 80,
                lastKnownName: "Youth Keeper",
                currentName: "Youth Keeper",
                state: "resolved" as const,
                combinedScore: 78,
                potentialCombinedScore: null,
              };

      return {
        ...team,
        strings:
          team.team === "senior"
            ? [
                { ...team.strings[0], assignments: [assignment] },
                { id: 4, stringOrder: 1, assignments: [] },
              ]
            : team.strings.map((plannerString) => ({
                ...plannerString,
                assignments: [assignment],
              })),
      };
    }),
  };
}
