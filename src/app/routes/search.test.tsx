import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { academyKeys } from "@/features/academy/api/academy-keys";
import {
  DEFAULT_GRAPHICS_STATUS,
  setGraphicsStatusIpcMock,
} from "@/features/graphics/api/graphics-ipc-mock";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { useSavedFiltersStore } from "@/features/search/stores/use-saved-filters-store";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { routeTree } from "@/routeTree.gen";
import {
  playerShortlistSelectionKey,
  usePlayerShortlistSelectionStore,
} from "@/stores/use-player-shortlist-selection-store";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { useSearchViewMemory } from "@/stores/use-search-view-memory";
import {
  getLastRefreshSeasonStatsIpcArgs,
  getMoneyballLeagueAdjustmentChanges,
  resolvePendingRefreshSeasonStats,
  setMoneyballLeagueAdjustmentIpcMock,
  setRefreshSeasonStatsIpcMockPending,
  setRefreshSeasonStatsIpcMockResult,
} from "@/testing/moneyball-ipc-mock";
import {
  getPlannerTacticIpcMockCalls,
  getPlannerTacticOptionsIpcMockCalls,
  getPlannerTacticSaveIpcMockCalls,
  resolvePlannerTacticIpcMock,
  setManagedClubIpcMock,
  setPlannerTacticForContext,
} from "@/testing/planner-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastSearchPlayersArgs,
  getSearchPlayersCallCount,
  rejectPendingSearchPlayersPageIpcMock,
  resolvePendingSearchPlayersPageIpcMock,
  setSearchPlayersOverride,
  setSearchPlayersPageIpcMockMode,
} from "@/testing/search-ipc-mock";
import {
  resolveBusyLoadDataRequest,
  resolveCreateSaveIpcMock,
  resolveLoadDataIpcMock,
  resolveSetActiveSaveIpcMock,
  setLoadDataIpcMockMode,
  setSavePlayerShortlistsIpcMock,
} from "@/testing/snapshot-ipc-mock";

function renderSearchRoute(
  initialEntry = "/search",
  seedQueryClient?: (queryClient: QueryClient) => void,
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 60_000 },
    },
  });

  seedQueryClient?.(queryClient);

  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [initialEntry] }),
  });

  return {
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    ),
    router,
    queryClient,
  };
}

function playerNamed(name: string, ca: number): PlayerSummary {
  return {
    uid: ca,
    name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: "Test FC",
    currentClubUid: null,
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
  };
}

function manyPlayers(count: number): PlayerSummary[] {
  return Array.from({ length: count }, (_, index) => ({
    uid: index + 1,
    name: `Player ${String(index + 1).padStart(3, "0")}`,
    age: 20 + (index % 15),
    birthYear: 2000,
    birthDayOfYear: 1 + (index % 28),
    nationalities: ["ENG"],
    club: index % 3 === 0 ? null : `Club ${index % 10}`,
    currentClubUid: null,
    division: index % 3 === 0 ? null : "Premier Division",
    ca: 200 - index,
    pa: 200 - (index % 40),
    marketValueGbp: index % 5 === 0 ? null : (200 - index) * 50_000,
  }));
}

const ORDERED_TACTIC_LANE_IDS = [
  "goalkeeper",
  "right_back",
  "left_centre_back",
  "right_centre_back",
  "left_back",
  "defensive_midfielder",
  "left_central_midfielder",
  "right_central_midfielder",
  "right_winger",
  "left_winger",
  "centre_forward",
] as const;

function mockScrollerScrollTo(scroller: HTMLElement) {
  Object.defineProperty(scroller, "scrollTo", {
    configurable: true,
    value: (options: { top?: number }) => {
      scroller.scrollTop = options.top ?? scroller.scrollTop;
    },
  });
}

type HeldSearchRequest = {
  /** Answers from the shared IPC mock. */
  resolve: () => void;
};

/**
 * Holds every `search_players` request that names its table (a panel's
 * requested first page) until the test settles it. Other requests keep the
 * shared IPC mock, which the test setup registers again after each test.
 */
function holdNamedSearchRequests(): HeldSearchRequest[] {
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          cmd: string,
          args?: Record<string, unknown>,
          options?: unknown,
        ) => Promise<unknown>;
      };
    }
  ).__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  const held: HeldSearchRequest[] = [];
  internals.invoke = (cmd, args, options) => {
    if (cmd !== "search_players" || !args?.tableId) {
      return invoke(cmd, args, options);
    }
    return new Promise((resolve) => {
      held.push({ resolve: () => resolve(invoke(cmd, args, options)) });
    });
  };
  return held;
}

const TARGETS = { id: 1, name: "Targets" };
const LOANS = { id: 2, name: "Loans" };

/** The default save's stored shortlist selection. */
function storedShortlistSelection() {
  return usePlayerShortlistSelectionStore.getState().selections[
    playerShortlistSelectionKey({ saveId: 1, contextToken: "save-token-1" })
  ];
}

function selectStoredShortlist(shortlistId: number) {
  usePlayerShortlistSelectionStore
    .getState()
    .select({ saveId: 1, contextToken: "save-token-1" }, shortlistId);
}

/** Records the arguments of every `search_players` request. */
function recordSearchRequests(): Record<string, unknown>[] {
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          cmd: string,
          args?: Record<string, unknown>,
          options?: unknown,
        ) => Promise<unknown>;
      };
    }
  ).__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  const requests: Record<string, unknown>[] = [];
  internals.invoke = (cmd, args, options) => {
    if (cmd === "search_players" && args) {
      requests.push(args);
    }
    return invoke(cmd, args, options);
  };
  return requests;
}

function retainedRow(table: HTMLElement): HTMLElement {
  const row = within(table)
    .getAllByRole("row")
    .find((candidate) => candidate.hasAttribute("data-index"));
  if (!row) {
    throw new Error("expected a retained results row");
  }
  return row;
}

describe("search route", () => {
  beforeEach(() => {
    useSearchViewMemory.setState({ memory: null });
    useSavedFiltersStore.setState({ savedFilters: [], columnsChoice: null });
  });

  it("uses protocol URLs for visible exact-UID graphics and preserves rows", async () => {
    await resolveLoadDataIpcMock();
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 7,
      selected: true,
    });
    setSearchPlayersOverride([
      { ...playerNamed("Exact UID player", 160), uid: 42, currentClubUid: 9 },
      {
        ...playerNamed("Legacy name only", 150),
        uid: 43,
        currentClubUid: null,
      },
    ]);
    const { queryClient } = renderSearchRoute();
    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const row = within(table).getByText("Exact UID player").closest("tr");
    const legacy = within(table).getByText("Legacy name only").closest("tr");
    if (!row || !legacy) throw new Error("Expected graphics rows");
    await waitFor(() => expect(row.querySelectorAll("img")).toHaveLength(2));
    expect(
      Array.from(row.querySelectorAll("img")).map((image) =>
        image.getAttribute("src"),
      ),
    ).toEqual([
      "http://graphics.localhost/7/personPortrait/42",
      "http://graphics.localhost/7/clubLogo/9",
    ]);
    expect(
      legacy.querySelectorAll('img[src^="http://graphics.localhost/"]'),
    ).toHaveLength(1);
    expect(legacy.querySelector('img[src$="/personPortrait/43"]')).toBeTruthy();
    expect(row).toHaveStyle({ height: "40px" });
    expect(row).toHaveAttribute("data-index");
    queryClient.clear();
  });

  it("keeps Search identity and navigation geometry after native image errors", async () => {
    await resolveLoadDataIpcMock();
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 4,
      selected: true,
    });
    setSearchPlayersOverride([
      { ...playerNamed("Stateful graphics", 160), uid: 51, currentClubUid: 12 },
    ]);
    const { router } = renderSearchRoute();
    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const row = within(table).getByText("Stateful graphics").closest("tr");
    if (!row) throw new Error("Expected graphics row");
    const images = Array.from(
      row.querySelectorAll('img[src^="http://graphics.localhost/"]'),
    );
    for (const image of images) fireEvent.error(image);
    expect(row).toHaveStyle({ height: "40px" });
    expect(within(row).getByText("Stateful graphics")).toBeVisible();
    fireEvent.click(row);
    await waitFor(() =>
      expect(router.state.location.pathname).toBe("/players/51"),
    );
  });

  it("bounds Search graphics to rendered virtual rows", async () => {
    await resolveLoadDataIpcMock();
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 9,
      selected: true,
    });
    const players = manyPlayers(200).map((player) => ({
      ...player,
      currentClubUid: player.uid + 1000,
    }));
    setSearchPlayersOverride(players);
    const { queryClient } = renderSearchRoute();
    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    await waitFor(() =>
      expect(
        table.querySelectorAll("tr[data-index] img").length,
      ).toBeGreaterThan(0),
    );
    const images = Array.from(table.querySelectorAll("tr[data-index] img"));
    expect(images.length).toBeLessThan(players.length * 2);
    expect(images.length).toBeLessThan(100);
    expect(
      images.every((image) =>
        /\/(personPortrait|clubLogo)\/(\d+)$/.test(
          image.getAttribute("src") ?? "",
        ),
      ),
    ).toBe(true);
    queryClient.clear();
  });

  it("keeps Search controls mounted without a snapshot and does not touch tactic state", async () => {
    const user = userEvent.setup();
    const layoutBeforeRender = structuredClone(
      usePlayerTableStore.getState().layouts,
    );
    renderWithProviders();

    const searchLink = await screen.findByRole("link", {
      name: "Search",
    });
    await user.click(searchLink);

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(searchLink).toHaveAttribute("aria-current", "page");
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Add filter" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Moneyball Score)" }),
    ).toBeDisabled();
    expect(screen.getByText("No snapshot loaded: use Load Data")).toBeVisible();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(getPlannerTacticIpcMockCalls()).toEqual([]);
    expect(getPlannerTacticOptionsIpcMockCalls()).toEqual([]);
    expect(getPlannerTacticSaveIpcMockCalls()).toEqual([]);
    expect(usePlayerTableStore.getState().layouts).toEqual(layoutBeforeRender);
  });

  it("keeps Search controls mounted for a mismatched snapshot without tactic IPC", async () => {
    const loaded = await resolveLoadDataIpcMock();
    const layoutBeforeMismatch = structuredClone(
      usePlayerTableStore.getState().layouts,
    );
    const mismatchedSnapshot: SnapshotSummary = {
      ...loaded.effectiveSnapshot,
      saveId: loaded.effectiveSnapshot.saveId + 1,
    };
    renderSearchRoute("/search", (queryClient) => {
      queryClient.setQueryData(
        currentSnapshotQueryOptions.queryKey,
        mismatchedSnapshot,
      );
    });

    expect(
      await screen.findByRole("button", { name: "Add filter" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(getPlannerTacticIpcMockCalls()).toEqual([]);
    expect(getPlannerTacticOptionsIpcMockCalls()).toEqual([]);
    expect(getPlannerTacticSaveIpcMockCalls()).toEqual([]);
    expect(usePlayerTableStore.getState().layouts).toEqual(
      layoutBeforeMismatch,
    );
  });

  it("warms the table's exact first page from the loader", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore
      .getState()
      .addColumns("search", ["attr.Acceleration", "attr.Agility"]);
    const requests = recordSearchRequests();
    const { router } = renderSearchRoute("/settings");
    await waitFor(() => expect(router.state.status).toBe("idle"));

    // Hovering the Search link runs the loader.
    await act(() =>
      router.preloadRoute({
        to: "/search",
        search: { sort: "ca", dir: "desc", filters: [], combine: "and" },
      }),
    );
    expect(requests).toEqual([
      expect.objectContaining({
        offset: 0,
        sortBy: "ca",
        requestedFields: ["attr.Acceleration", "attr.Agility", "height"],
      }),
    ]);

    await act(() =>
      router.navigate({
        to: "/search",
        search: { sort: "ca", dir: "desc", filters: [], combine: "and" },
      }),
    );
    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    // The table reads the warmed page instead of requesting its own.
    expect(requests).toHaveLength(1);
  });

  it("paints the tactic controls ready on the first frame", async () => {
    await resolveLoadDataIpcMock();
    renderSearchRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Loading tactic…")).toBeNull();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    ).toBeEnabled();
  });

  it("keeps the results while a stale snapshot refetches unchanged", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Kept Player", 160)]);
    const { queryClient } = renderSearchRoute();
    expect(await screen.findByText("Kept Player")).toBeInTheDocument();

    const internals = (
      window as unknown as {
        __TAURI_INTERNALS__: {
          invoke: (
            cmd: string,
            args?: Record<string, unknown>,
            options?: unknown,
          ) => Promise<unknown>;
        };
      }
    ).__TAURI_INTERNALS__;
    const invoke = internals.invoke;
    let release = () => {};
    internals.invoke = (cmd, args, options) =>
      cmd === "get_current_snapshot"
        ? new Promise((resolve) => {
            release = () => resolve(invoke(cmd, args, options));
          })
        : invoke(cmd, args, options);

    // A newly mounted observer refetches data that is merely stale.
    const refetch = queryClient.refetchQueries({
      queryKey: snapshotKeys.current(),
    });
    await waitFor(() =>
      expect(queryClient.isFetching({ queryKey: snapshotKeys.current() })).toBe(
        1,
      ),
    );
    expect(screen.getByText("Kept Player")).toBeInTheDocument();
    expect(screen.queryByText("Loading player results…")).toBeNull();
    expect(screen.queryByText("Loading tactic…")).toBeNull();

    release();
    await act(() => refetch);
    expect(screen.getByText("Kept Player")).toBeInTheDocument();
  });

  it("holds the table's place with a skeleton while the first page loads", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    renderSearchRoute("/search?sort=value&dir=desc");

    const label = await screen.findByText("Loading player results…");
    expect(label.closest('[role="status"]')).toHaveAttribute(
      "aria-busy",
      "true",
    );
    expect(screen.queryByText("Loading players")).toBeNull();

    resolvePendingSearchPlayersPageIpcMock();
    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Loading player results…")).toBeNull();
  });

  it("atomically appends, interleaves, and re-compacts tactic groups", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderSearchRoute();

    await screen.findByRole("button", {
      name: "Add Tactic (Role Score)",
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
      ).toBeEnabled(),
    );
    const orderedLaneIds = ORDERED_TACTIC_LANE_IDS;
    const nonTacticIds = ["age", "nationality", "height", "ca", "pa", "value"];

    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...nonTacticIds,
      ...orderedLaneIds.map((laneId) => `tactic_current.${laneId}`),
    ]);
    expect(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    ).toHaveAttribute("aria-pressed", "true");

    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...nonTacticIds,
      ...orderedLaneIds.flatMap((laneId) => [
        `tactic_current.${laneId}`,
        `tactic_potential.${laneId}`,
      ]),
    ]);

    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...nonTacticIds,
      ...orderedLaneIds.map((laneId) => `tactic_potential.${laneId}`),
    ]);
    expect(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    ).toHaveAttribute("aria-pressed", "false");
    expect(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("shows compact tactic headers with keyboard-reachable full definitions", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderSearchRoute();

    await screen.findByRole("button", {
      name: "Add Tactic (Role Score)",
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
      ).toBeEnabled(),
    );
    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    );

    const full = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    // Current group only: each of the 11 lanes renders exactly once.
    const headers = within(table).getAllByRole("columnheader", {
      name: full,
    });
    expect(headers).toHaveLength(1);
    const header = headers[0];
    // Compact placement primary with smaller role context.
    expect(within(header).getByText("GK")).toBeVisible();
    expect(
      within(header).getByText("Goalkeeper / Line-Holding Keeper"),
    ).toBeVisible();
    // The focusable leaf carries the complete definition; keyboard focus
    // reveals a visible definition instead of relying on title alone.
    const leaf = within(header).getByRole("button", { name: full });
    expect(within(header).queryByRole("tooltip")).toBeNull();
    leaf.focus();
    expect(
      await within(header).findByRole("tooltip", { name: full }),
    ).toBeVisible();
    // No 12th lane: exactly the 11 canonical current lanes are present.
    const tacticIds = usePlayerTableStore
      .getState()
      .layouts.search.columnIds.filter((id) =>
        id.startsWith("tactic_current."),
      );
    expect(tacticIds).toHaveLength(11);
  });

  it("renders tactic scores and re-compacts the survivor when header removal resets sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const orderedLaneIds = ORDERED_TACTIC_LANE_IDS;
    const currentIds = orderedLaneIds.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    const potentialIds = orderedLaneIds.map(
      (laneId) => `tactic_potential.${laneId}`,
    );
    const interleavedIds = orderedLaneIds.flatMap((laneId) => [
      `tactic_current.${laneId}`,
      `tactic_potential.${laneId}`,
    ]);
    const store = usePlayerTableStore.getState();
    store.replaceLayout("search", ["ca", ...interleavedIds]);
    store.setColumnWidth("search", currentIds[0], 176);
    store.setColumnWidth("search", potentialIds[0], 184);
    setSearchPlayersOverride([
      {
        ...playerNamed("Available fit", 160),
        dynamicValues: {
          [currentIds[0]]: 88,
          [potentialIds[0]]: null,
        },
      },
      {
        ...playerNamed("Unavailable fit", 150),
        dynamicValues: {
          [currentIds[0]]: null,
          [potentialIds[0]]: null,
        },
      },
    ]);
    const { router } = renderSearchRoute(
      `/search?sort=${currentIds[0]}&dir=desc`,
    );

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const label = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
    const headers = within(table).getAllByRole("columnheader", { name: label });
    expect(headers).toHaveLength(2);
    const resizeHandles = within(table).getAllByRole("separator", {
      name: `Resize ${label} column`,
    });
    expect(resizeHandles[0]).toHaveAttribute("aria-valuenow", "176");
    expect(resizeHandles[1]).toHaveAttribute("aria-valuenow", "184");
    const rows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    expect(within(rows[0]).getByText("Available fit")).toBeInTheDocument();
    expect(
      within(rows[0]).getByRole("img", {
        name: `${label}: 88, Excellent`,
      }),
    ).toBeInTheDocument();
    expect(within(rows[0]).getAllByText("—")).not.toHaveLength(0);
    expect(within(rows[1]).getByText("Unavailable fit")).toBeInTheDocument();

    const siblingLabel = "DR (Full-Back) / DR (Holding Full-Back)";
    fireEvent.contextMenu(
      within(table).getAllByRole("columnheader", { name: siblingLabel })[0],
    );
    await user.click(
      screen.getByRole("menuitem", { name: `Remove ${siblingLabel}` }),
    );

    await waitFor(() => {
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
        "ca",
        ...potentialIds,
      ]);
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
    });
    expect(
      usePlayerTableStore.getState().layouts.search.widths[potentialIds[0]],
    ).toBe(184);
    expect(
      usePlayerTableStore.getState().layouts.search.widths[currentIds[0]],
    ).toBeUndefined();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
    ).toHaveAttribute("aria-pressed", "false");
    expect(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("keeps the surviving tactic group when a header is removed during tactic loading", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const potentialIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_potential.${laneId}`,
    );
    usePlayerTableStore
      .getState()
      .replaceLayout("search", [
        "ca",
        ...ORDERED_TACTIC_LANE_IDS.flatMap((laneId) => [
          `tactic_current.${laneId}`,
          `tactic_potential.${laneId}`,
        ]),
      ]);
    usePlayerTableStore.getState().moveColumn("search", potentialIds[0], 22);
    const { queryClient } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    // The tactic reloads from nothing, as after a context change, and stays
    // pending.
    const internals = (
      window as unknown as {
        __TAURI_INTERNALS__: {
          invoke: (
            cmd: string,
            args?: Record<string, unknown>,
            options?: unknown,
          ) => Promise<unknown>;
        };
      }
    ).__TAURI_INTERNALS__;
    const invoke = internals.invoke;
    internals.invoke = (cmd, args, options) =>
      cmd === "get_planner_tactic"
        ? new Promise<never>(() => undefined)
        : invoke(cmd, args, options);
    void queryClient.resetQueries({
      queryKey: plannerKeys.tactic({
        saveId: 1,
        contextToken: "save-token-1",
      }),
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
      ).toBeDisabled(),
    );
    fireEvent.contextMenu(
      within(table).getAllByRole("columnheader", { name: "goalkeeper" })[0],
    );
    await user.click(
      screen.getByRole("menuitem", { name: "Remove goalkeeper" }),
    );

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "ca",
      ...potentialIds.slice(1),
      potentialIds[0],
    ]);
    expect(
      usePlayerTableStore
        .getState()
        .layouts.search.columnIds.some((id) =>
          id.startsWith("tactic_current."),
        ),
    ).toBe(false);
  });

  it("preserves identity-only after tactic-only leaf removal", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const currentIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    usePlayerTableStore.getState().replaceLayout("search", currentIds);
    renderSearchRoute(`/search?sort=${currentIds[0]}&dir=desc`);

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const label = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: label }),
    );
    await user.click(screen.getByRole("menuitem", { name: `Remove ${label}` }));

    // Identity-only is valid: the emptied tactic group normalizes away
    // without rolling back to defaults.
    await waitFor(() => {
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
        [],
      );
    });
    expect(
      await screen.findByRole("table", {
        name: "Player search results",
      }),
    ).toBeInTheDocument();
  });

  it("falls back from a tactic sort when its active group is toggled off", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const currentIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    usePlayerTableStore
      .getState()
      .replaceLayout("search", ["ca", ...currentIds]);
    const { router } = renderSearchRoute(
      `/search?sort=${currentIds[0]}&dir=desc`,
    );

    const currentToggle = await screen.findByRole("button", {
      name: "Add Tactic (Role Score)",
    });
    await waitFor(() => expect(currentToggle).toBeEnabled());
    await user.click(currentToggle);

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "ca",
    ]);
  });

  it("keeps identity-only when its tactic-only group is toggled off", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const currentIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    usePlayerTableStore.getState().replaceLayout("search", currentIds);
    const { router } = renderSearchRoute(
      `/search?sort=${currentIds[0]}&dir=desc`,
    );

    const currentToggle = await screen.findByRole("button", {
      name: "Add Tactic (Role Score)",
    });
    await waitFor(() => expect(currentToggle).toBeEnabled());
    await user.click(currentToggle);

    await waitFor(() => {
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
        [],
      );
    });
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
    });
  });

  it("refreshes tactic labels for a new save without replacing its table layout", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const currentIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    const store = usePlayerTableStore.getState();
    store.replaceLayout("search", ["ca", ...currentIds]);
    store.setColumnWidth("search", currentIds[0], 176);
    const { queryClient } = renderSearchRoute();

    const originalLabel = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
    expect(
      await screen.findByRole("columnheader", { name: originalLabel }),
    ).toBeInTheDocument();

    const second = resolveCreateSaveIpcMock({ name: "Second save" });
    const secondTactic = resolvePlannerTacticIpcMock();
    secondTactic.lanes[0].ipRoleId = "ball_playing_goalkeeper_ip";
    setPlannerTacticForContext(
      { saveId: second.id, contextToken: second.contextToken },
      secondTactic,
    );
    await queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    const saveSelect = screen.getByRole("combobox", { name: "Active save" });
    await screen.findByRole("option", { name: "Second save" });
    await user.selectOptions(saveSelect, String(second.id));

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "ca",
      ...currentIds,
    ]);
    expect(
      usePlayerTableStore.getState().layouts.search.widths[currentIds[0]],
    ).toBe(176);

    await user.click(screen.getByRole("button", { name: "Load Data" }));
    expect(
      await screen.findByRole("columnheader", {
        name: "GK (Ball-Playing Goalkeeper) / GK (Line-Holding Keeper)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: originalLabel }),
    ).toBeNull();
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "ca",
      ...currentIds,
    ]);
    expect(
      usePlayerTableStore.getState().layouts.search.widths[currentIds[0]],
    ).toBe(176);
  });

  describe("Moneyball Score tactic columns", () => {
    const roleScoreIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    const potentialIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_potential.${laneId}`,
    );
    const moneyballIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_moneyball.${laneId}`,
    );
    const searchColumnIds = () =>
      usePlayerTableStore.getState().layouts.search.columnIds;

    async function enabledToggle(name: string) {
      const toggle = await screen.findByRole("button", { name });
      await waitFor(() => expect(toggle).toBeEnabled());
      return toggle;
    }

    it("offers Role Score, Potential, and Moneyball Score toggles, even without chosen metrics", async () => {
      await resolveLoadDataIpcMock();
      const tactic = resolvePlannerTacticIpcMock();
      for (const lane of tactic.lanes) {
        lane.customMetricKeys = [];
      }
      setPlannerTacticForContext(
        { saveId: 1, contextToken: "save-token-1" },
        tactic,
      );
      renderSearchRoute();

      await enabledToggle("Add Tactic (Role Score)");
      expect(
        screen.getByRole("button", { name: "Add Tactic (Potential)" }),
      ).toBeEnabled();
      // A slot without chosen metrics scores its roles, so the group can
      // always be added.
      expect(
        screen.getByRole("button", { name: "Add Tactic (Moneyball Score)" }),
      ).toBeEnabled();
      expect(
        screen.queryByRole("button", { name: /Custom|Roles\)/ }),
      ).toBeNull();
      expect(screen.queryByRole("link", { name: "Tactic editor" })).toBeNull();
    });

    it("interleaves each slot's Moneyball Score after its Role Score", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      renderSearchRoute();
      const nonTacticIds = searchColumnIds();

      const moneyball = await enabledToggle("Add Tactic (Moneyball Score)");
      await user.click(moneyball);
      expect(searchColumnIds()).toEqual([...nonTacticIds, ...moneyballIds]);
      expect(moneyball).toHaveAttribute("aria-pressed", "true");

      await user.click(
        screen.getByRole("button", { name: "Add Tactic (Role Score)" }),
      );
      expect(searchColumnIds()).toEqual([
        ...nonTacticIds,
        ...roleScoreIds.flatMap((id, index) => [id, moneyballIds[index]]),
      ]);

      await user.click(
        screen.getByRole("button", { name: "Add Tactic (Potential)" }),
      );
      expect(searchColumnIds()).toEqual([
        ...nonTacticIds,
        ...roleScoreIds.flatMap((id, index) => [
          id,
          potentialIds[index],
          moneyballIds[index],
        ]),
      ]);

      await user.click(moneyball);
      expect(searchColumnIds()).toEqual([
        ...nonTacticIds,
        ...roleScoreIds.flatMap((id, index) => [id, potentialIds[index]]),
      ]);
      expect(moneyball).toHaveAttribute("aria-pressed", "false");
    });

    it("names each Moneyball Score header by its slot's chosen metrics or its roles", async () => {
      await resolveLoadDataIpcMock();
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["age", ...moneyballIds]);
      renderSearchRoute();

      const table = await screen.findByRole("table", {
        name: "Player search results",
      });
      const goalkeeper = await within(table).findByRole("columnheader", {
        name: "GK Moneyball Score: Save %, Clean Sheets %",
      });
      expect(within(goalkeeper).getByText("GK")).toBeVisible();
      expect(within(goalkeeper).getByText("Moneyball Score")).toBeVisible();
      expect(
        within(table).getByRole("columnheader", {
          name: "DR Moneyball Score: Full-Back / Holding Full-Back",
        }),
      ).toBeInTheDocument();
    });
  });

  it("restores a persisted tactic sort only for the current view layout", async () => {
    const tacticSort = "tactic_current.goalkeeper";
    usePlayerTableStore.getState().replaceLayout("search", [tacticSort]);

    const { router } = renderSearchRoute(`/search?sort=${tacticSort}&dir=asc`);

    await screen.findByRole("heading", { level: 1, name: "Player Search" });
    expect(router.state.location.search).toMatchObject({
      sort: tacticSort,
      dir: "asc",
    });
  });

  it("falls back from a tactic sort whose column is not in the layout or no longer exists", async () => {
    usePlayerTableStore
      .getState()
      .replaceLayout("search", ["ca", "tactic_moneyball.goalkeeper"]);

    for (const tacticSort of [
      "tactic_current.goalkeeper",
      "tactic_custom.goalkeeper",
    ]) {
      const { router, unmount } = renderSearchRoute(
        `/search?sort=${tacticSort}`,
      );
      await screen.findByRole("heading", { level: 1, name: "Player Search" });
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
      unmount();
    }
  });

  it("shows Moneyball controls only while a season column, rule, or sort is in use", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Scout", 160)]);
    const { unmount } = renderSearchRoute();

    await screen.findByText("Scout");
    const toolbar = () =>
      screen.getByRole("toolbar", { name: "Player results toolbar" });
    expect(
      within(toolbar()).queryByRole("combobox", { name: "Percentiles vs" }),
    ).toBeNull();
    expect(screen.queryByRole("group", { name: "Scores" })).toBeNull();
    // Refresh stats stays a page action whatever the columns.
    expect(
      within(screen.getByTestId("search-page-header")).getByRole("button", {
        name: "Refresh stats",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ comparison: "own" });
    });
    expect(getLastSearchPlayersArgs()).not.toHaveProperty("searchView");

    // A season column shows them; removing it hides them again.
    act(() => {
      usePlayerTableStore
        .getState()
        .addColumns("search", ["moneyball.average_rating"]);
    });
    expect(
      await within(toolbar()).findByRole("combobox", {
        name: "Percentiles vs",
      }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("group", { name: "Scores" }),
    ).toBeInTheDocument();
    act(() => {
      usePlayerTableStore
        .getState()
        .removeColumn("search", "moneyball.average_rating");
    });
    await waitFor(() =>
      expect(
        within(toolbar()).queryByRole("combobox", { name: "Percentiles vs" }),
      ).toBeNull(),
    );
    unmount();

    // So does a season rule or sort with no season column.
    for (const query of [
      `filters=${encodeURIComponent(
        JSON.stringify([{ field: "moneyball.minutes", op: "gte", value: 900 }]),
      )}`,
      "sort=moneyball.average_rating",
    ]) {
      const rendered = renderSearchRoute(`/search?${query}`);
      expect(
        await screen.findByRole("combobox", { name: "Percentiles vs" }),
      ).toBeInTheDocument();
      rendered.unmount();
    }
  });

  it("mutes and dates season values borrowed from a kept snapshot", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore
      .getState()
      .addColumns("search", ["moneyball.average_rating"]);
    setSearchPlayersOverride([
      {
        ...playerNamed("Regular Scout", 160),
        dynamicValues: { "moneyball.average_rating": 7.25 },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
      {
        ...playerNamed("Borrowed Scout", 150),
        dynamicValues: { "moneyball.average_rating": 7.5 },
        moneyballPercentiles: { "moneyball.average_rating": 90 },
        moneyballStatsDate: "2026-05-20",
      },
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("img", { name: "Average Rating: 83, Excellent" }),
    ).toBeInTheDocument();
    // A borrowed sample's percentile is a full one, ranked in its season.
    expect(
      within(table).getByRole("img", { name: "Average Rating: 90, Excellent" }),
    ).toBeInTheDocument();
    const regular = within(table).getByText("7.25");
    expect(regular.closest("td")).toHaveAttribute("title", "7.25");
    expect(regular).not.toHaveClass("text-on-surface-variant");
    const borrowed = within(table).getByText("7.50");
    expect(borrowed).toHaveClass("text-on-surface-variant");
    expect(borrowed.closest("td")).toHaveAttribute(
      "title",
      "7.50, as of 20th May 2026: under 450 minutes this season",
    );
  });

  it("selects, requests, sorts, and renders Moneyball Score columns without changing raw percentiles", async () => {
    await resolveLoadDataIpcMock();
    const wingBackIp = "moneyball_role.wbl_wbr_wing_back_ip";
    const fullBackIp = "moneyball_role.dl_dr_wing_back_ip";
    usePlayerTableStore
      .getState()
      .addColumns("search", [
        "moneyball.average_rating",
        wingBackIp,
        fullBackIp,
      ]);
    setSearchPlayersOverride([
      {
        ...playerNamed("Role-fit Scout", 160),
        dynamicValues: {
          "moneyball.average_rating": 7.2,
          [wingBackIp]: 0,
          [fullBackIp]: null,
        },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", {
        name: "Moneyball Score · Wing-Back (IP · WBR/WBL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", {
        name: "Moneyball Score · Wing-Back (IP · DR/DL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("img", {
        name: "Moneyball Score · Wing-Back (IP · WBR/WBL): 0, Weak",
      }),
    ).toBeInTheDocument();
    expect(within(table).getAllByText("—").length).toBeGreaterThan(0);
    expect(
      within(table).getByRole("img", {
        name: "Average Rating: 83, Excellent",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        requestedFields: expect.arrayContaining([fullBackIp, wingBackIp]),
      });
    });

    await userEvent.setup().click(
      within(table).getByRole("button", {
        name: "Moneyball Score · Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(router.state.location.search).toMatchObject({
      sort: wingBackIp,
      dir: "desc",
    });
  });

  it("applies a Moneyball Score filter to the URL and comparison request", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const roleField = "moneyball_role.wbl_wbr_wing_back_ip";
    setSearchPlayersOverride([
      {
        ...playerNamed("Role-fit Scout", 160),
        dynamicValues: {
          "moneyball.average_rating": 7.2,
          [roleField]: 72,
        },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    const { router } = renderSearchRoute();

    await screen.findByText("Role-fit Scout");
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "wing-back",
    );
    await user.click(
      within(dialog).getByRole("option", {
        name: "Moneyball Score · Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(
      within(dialog).getByRole("combobox", { name: "Operator" }),
    ).toHaveAccessibleDescription(/^Scored among the role's position family/);
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "70" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        filters: [
          expect.objectContaining({ field: roleField, op: "gte", value: 70 }),
        ],
      });
      expect(getLastSearchPlayersArgs()).toMatchObject({
        filters: [{ field: roleField, op: "gte", value: 70 }],
        requestedFields: expect.arrayContaining([roleField]),
      });
    });
    expect(router.state.location.search).not.toHaveProperty("view");
    expect(
      screen.getByRole("button", {
        name: "Remove filter Moneyball Score · Wing-Back (IP · WBR/WBL) ≥ 70",
      }),
    ).toBeInTheDocument();

    // The season rule brings Percentiles vs into the toolbar.
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Percentiles vs" }),
      "Wing-backs",
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        comparison: "wing_back",
      });
      expect(getLastSearchPlayersArgs()).toMatchObject({
        comparison: "wing_back",
        filters: [{ field: roleField, op: "gte", value: 70 }],
      });
    });
  });

  it("drops an old view parameter and offers one Search destination", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Scout", 160)]);
    renderSearchRoute("/search?view=moneyball");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Search", current: "page" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Moneyball" })).toBeNull();
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ sortBy: "ca" });
    });
    expect(getLastSearchPlayersArgs()).not.toHaveProperty("searchView");
  });

  it("renders a virtualized page of basic columns via search_players", async () => {
    await resolveLoadDataIpcMock();
    const players = manyPlayers(80);
    players[0] = { ...players[0], dynamicValues: { height: 188 } };
    setSearchPlayersOverride(players);
    renderSearchRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", { name: "Player" }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "Height" }),
    ).toBeInTheDocument();
    expect(await within(table).findByText("Player 001")).toBeInTheDocument();
    expect(within(table).getByText("188 cm")).toBeInTheDocument();

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    expect(bodyRows.length).toBeGreaterThan(0);
    expect(bodyRows.length).toBeLessThan(80);
    const scroller = screen.getByTestId("search-results-scroller");
    expect(scroller).toHaveClass("h-full", "min-h-0", "overflow-auto");
    expect(scroller.parentElement).toHaveClass("relative", "min-h-0", "flex-1");
    expect(getLastSearchPlayersArgs()).toMatchObject({
      requestedFields: ["height"],
    });
  });

  it("bounds a wide grouped layout to fixed column widths in the same scroller", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore.getState().addColumns("search", ["wage", "reputation"]);
    setSearchPlayersOverride(manyPlayers(80));
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    // 280 identity + 144 age + 160 nationality + 88 height + 72 CA + 72 PA + 112 value
    // + 96 wage + 96 reputation. The bounded model fixes the table to
    // exactly this sum so readable minimums hold and overflow stays in
    // the same scroller instead of stretching cells.
    expect(table).toHaveStyle({
      minWidth: "1120px",
      maxWidth: "1120px",
      width: "1120px",
    });
    const cols = table.querySelectorAll("col");
    expect(cols).toHaveLength(9);
    expect(cols[0]).toHaveStyle({ width: "280px" });
    // Sticky identity plus the complete grouped + leaf header context.
    const headers = within(table).getAllByRole("columnheader");
    expect(headers[0]).toHaveTextContent("Player");
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toBeInTheDocument();
    const scroller = screen.getByTestId("search-results-scroller");
    expect(scroller).toHaveClass("h-full", "min-h-0", "overflow-auto");
  });

  it("renders every nationality flag in stored order", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Flagged Scout", 160),
        nationalities: ["England", "Wales", "South Korea"],
      },
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const flags = await within(table).findAllByRole("img");

    expect(flags.map((flag) => flag.getAttribute("aria-label"))).toEqual([
      "England",
      "Wales",
      "South Korea",
    ]);
  });

  it("renders current, potential, and Club DNA scores while leaving other dynamic metrics neutral", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore
      .getState()
      .addColumns("search", [
        "role.goalkeeper_ip",
        "potential_role.goalkeeper_ip",
        "club_dna",
        "attr.Acceleration",
      ]);
    setSearchPlayersOverride([
      ...[
        ["Weak fit", 164, 20],
        ["Average fit", 163, 50],
        ["Good fit", 162, 70],
        ["Excellent fit", 161, 90],
      ].map(([name, ca, score]) => ({
        ...playerNamed(String(name), Number(ca)),
        dynamicValues: {
          "role.goalkeeper_ip": Number(score),
          "potential_role.goalkeeper_ip": Number(score),
          club_dna: Number(score),
          "attr.Acceleration": 16,
        },
      })),
      {
        ...playerNamed("Missing fit", 160),
        dynamicValues: { "attr.Acceleration": 16, club_dna: null },
      },
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    for (const [score, tier, scoreClass] of [
      [20, "Weak", "text-score-1"],
      [50, "Average", "text-score-2"],
      [70, "Good", "text-score-3"],
      [90, "Excellent", "text-score-4"],
    ] as const) {
      expect(
        within(table).getByRole("img", {
          name: `Role Score · Goalkeeper (IP): ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
      expect(
        within(table).getByRole("img", {
          name: `Potential Role Score · Goalkeeper (IP): ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
      expect(
        within(table).getByRole("img", {
          name: `Club DNA: ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
    }

    const missingRow = within(table).getByText("Missing fit").closest("tr");
    if (!missingRow) {
      throw new Error("Expected the missing-score player row.");
    }
    expect(within(missingRow).queryAllByRole("img")).toHaveLength(0);
    // Height is a default column now, so the missing-score row also renders
    // its Height dash alongside the three score dashes.
    expect(within(missingRow).getAllByText("—")).toHaveLength(4);
    expect(within(missingRow).getByText("16")).not.toHaveAttribute(
      "role",
      "img",
    );
  });

  it("adds a metric from a header menu without changing the active sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Accelerating Scout", 160),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });
    const caSort = within(caHeader).getByRole("button", { name: "CA" });
    expect(
      within(table).queryByRole("button", { name: "Manage CA column" }),
    ).toBeNull();
    caSort.focus();
    fireEvent.keyDown(caSort, { key: "F10", shiftKey: true });
    await user.keyboard("{Escape}");
    expect(caSort).toHaveFocus();
    fireEvent.contextMenu(caHeader);
    expect(
      screen.getByRole("menu", { name: "CA column actions" }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "acceleration",
    );
    await user.click(screen.getByRole("option", { name: "Acceleration" }));

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "attr.Acceleration",
    );

    expect(
      await screen.findByRole("columnheader", { name: "Acceleration" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        requestedFields: ["attr.Acceleration", "height"],
      });
    });
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
  });

  it("never offers Suggested Training in Search columns or menus", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Search Scout", 160)]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).queryByRole("columnheader", {
        name: "Suggested Training",
      }),
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
      "training",
    );
    expect(
      screen.queryByRole("option", { name: "Suggested Training" }),
    ).toBeNull();
    await user.keyboard("{Escape}");
  });

  // The dialog lists every Search field (about 480), and jsdom's role-name
  // queries over it take a few seconds under a full parallel run.
  it("manages Search columns from the grouped Columns control without touching identity", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Search Scout", 160)]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", { name: "Market" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Columns" }));
    const dialog = screen.getByRole("dialog", { name: "Columns" });
    // Optional analysis columns grouped per the header map; identity absent.
    expect(within(dialog).getByText("Profile")).toBeInTheDocument();
    expect(within(dialog).getByText("Ability")).toBeInTheDocument();
    expect(within(dialog).getByText("Market")).toBeInTheDocument();
    expect(within(dialog).getByRole("checkbox", { name: "CA" })).toBeChecked();
    expect(within(dialog).queryByRole("checkbox", { name: "Name" })).toBeNull();
    expect(within(dialog).queryByRole("checkbox", { name: "Club" })).toBeNull();
    expect(
      within(dialog).queryByRole("checkbox", { name: "Division" }),
    ).toBeNull();

    await user.click(within(dialog).getByRole("checkbox", { name: "Value" }));
    expect(
      usePlayerTableStore.getState().layouts.search.columnIds,
    ).not.toContain("value");
    // The header run follows the same mapper: once reloaded, the emptied
    // Market group is gone along with its leaf.
    await waitFor(() => {
      const reloaded = screen.getByRole("table", {
        name: "Player search results",
      });
      expect(
        within(reloaded).queryByRole("columnheader", { name: "Value" }),
      ).toBeNull();
      expect(
        within(reloaded).queryByRole("columnheader", { name: "Market" }),
      ).toBeNull();
    });

    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reputation" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "reputation",
    );
    expect(
      await screen.findByRole("columnheader", { name: "Reputation" }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
  }, 15_000);

  it("never offers identity in the Search Add-column picker results", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Search Scout", 160)]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "CA" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "division",
    );
    expect(screen.queryByRole("option", { name: "Division" })).toBeNull();
    expect(screen.queryByRole("option", { name: "Club" })).toBeNull();
    expect(screen.queryByRole("option", { name: "Name" })).toBeNull();
    await user.keyboard("{Escape}");
  });

  it("reorders Search columns from the menu without changing its query, virtual row, or widths", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const store = usePlayerTableStore.getState();
    store.addColumns("search", ["attr.Acceleration", "attr.Agility"]);
    store.setColumnWidth("search", "attr.Acceleration", 216);
    setSearchPlayersOverride(
      manyPlayers(101).map((player) => ({
        ...player,
        dynamicValues: { "attr.Acceleration": 16, "attr.Agility": 15 },
      })),
    );
    const requests = recordSearchRequests();
    const { queryClient, router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const scroller = screen.getByTestId("search-results-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    // The visible window and its prefetched neighbour both settle before the
    // reorder, so any later request would come from the reorder itself.
    await waitFor(() => {
      for (const offset of [50, 100]) {
        expect(requests).toContainEqual(
          expect.objectContaining({
            offset,
            requestedFields: ["attr.Acceleration", "attr.Agility", "height"],
          }),
        );
      }
      expect(queryClient.isFetching()).toBe(0);
    });
    const focusedRow = await waitFor(() => {
      const row = scroller.querySelector<HTMLElement>('[data-index="49"]');
      if (!row) {
        throw new Error("Expected the loaded virtual row.");
      }
      return row;
    });
    focusedRow.focus();
    const callCountBeforeReorder = getSearchPlayersCallCount();
    const accelerationHeader = within(table).getByRole("columnheader", {
      name: "Acceleration",
    });
    fireEvent.contextMenu(accelerationHeader);
    await user.click(screen.getByRole("menuitem", { name: "Move right" }));

    await waitFor(() => {
      const headerLabels = within(table)
        .getAllByRole("columnheader")
        .filter((header) => header.getAttribute("scope") === "col")
        .map((header) => header.getAttribute("aria-label"));
      expect(headerLabels.indexOf("Agility")).toBeLessThan(
        headerLabels.indexOf("Acceleration"),
      );
    });
    const headerLabels = within(table)
      .getAllByRole("columnheader")
      .filter((header) => header.getAttribute("scope") === "col")
      .map((header) => header.getAttribute("aria-label"));
    const cellTexts = within(focusedRow)
      .getAllByRole("cell")
      .map((cell) => cell.textContent);
    expect(cellTexts[headerLabels.indexOf("Agility")]).toBe("15");
    expect(cellTexts[headerLabels.indexOf("Acceleration")]).toBe("16");
    expect(screen.getByRole("button", { name: "Acceleration" })).toHaveFocus();
    expect(scroller.scrollTop).toBe(1_950);
    expect(getSearchPlayersCallCount()).toBe(callCountBeforeReorder);
    expect(
      screen.getByRole("separator", { name: "Resize Acceleration column" }),
    ).toHaveAttribute("aria-valuenow", "216");
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
  });

  it("moves Search columns from the header menu with edge guards and focus restoration", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Movable menu", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });
    fireEvent.contextMenu(caHeader);
    await user.click(screen.getByRole("menuitem", { name: "Move left" }));

    expect(screen.getByRole("button", { name: "CA" })).toHaveFocus();
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "age",
      "nationality",
      "ca",
      "height",
      "pa",
      "value",
    ]);
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });

    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Age" }),
    );
    expect(screen.getByRole("menuitem", { name: "Move left" })).toBeDisabled();
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Value" }),
    );
    expect(screen.getByRole("menuitem", { name: "Move right" })).toBeDisabled();
  });

  it("closes a header menu after either pointer button is pressed outside", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Dismissible menu", 160)]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });

    fireEvent.contextMenu(caHeader);
    expect(
      screen.getByRole("menu", { name: "CA column actions" }),
    ).toBeInTheDocument();
    fireEvent.pointerDown(document.body, { button: 0 });
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();

    fireEvent.contextMenu(caHeader);
    expect(
      screen.getByRole("menu", { name: "CA column actions" }),
    ).toBeInTheDocument();
    fireEvent.pointerDown(document.body, { button: 2 });
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();

    fireEvent.contextMenu(caHeader);
    fireEvent.pointerDown(
      within(table).getByRole("separator", { name: "Resize CA column" }),
      { button: 0, pointerId: 1, clientX: 0 },
    );
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();
  });

  it("resizes Search columns without changing the active sort", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Resizable", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const handle = within(table).getByRole("separator", {
      name: "Resize CA column",
    });
    fireEvent.pointerDown(handle, { pointerId: 1, clientX: 20 });
    fireEvent.pointerMove(handle, { pointerId: 1, clientX: 60 });
    fireEvent.pointerUp(handle, { pointerId: 1 });

    await waitFor(() => {
      expect(handle).toHaveAttribute("aria-valuenow", "112");
    });
    fireEvent.keyDown(handle, { key: "ArrowRight" });
    fireEvent.keyDown(handle, { key: "End" });
    expect(handle).toHaveAttribute("aria-valuenow", "360");
    fireEvent.keyDown(handle, { key: "Home" });
    expect(handle).toHaveAttribute("aria-valuenow", "72");
    expect(usePlayerTableStore.getState().layouts.search.widths.ca).toBe(72);
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
  });

  it("keeps Search column widths when sorting", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Stable width", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const resizeCa = within(table).getByRole("separator", {
      name: "Resize CA column",
    });
    fireEvent.keyDown(resizeCa, { key: "ArrowRight" });
    expect(resizeCa).toHaveAttribute("aria-valuenow", "88");

    fireEvent.click(within(table).getByRole("button", { name: "CA" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "asc",
      });
    });
    expect(
      screen.getByRole("separator", { name: "Resize CA column" }),
    ).toHaveAttribute("aria-valuenow", "88");
  });

  it("resets the active Search sort when its visible column is removed", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Sort reset", 160)]);
    const { router } = renderSearchRoute("/search?sort=value&dir=desc");

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Value" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Remove Value" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
      expect(screen.queryByRole("columnheader", { name: "Value" })).toBeNull();
    });
  });

  it.each([
    { columns: "player", season: false },
    { columns: "season", season: true },
  ])(
    "stacks fixed-height player identity without duplicate columns beside $columns columns",
    async ({ season }) => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([
        {
          ...playerNamed("Identity Player", 250),
          club: "Test FC",
          division: "Premier Division",
          dynamicValues: { "moneyball.average_rating": 9.9 },
        },
        {
          ...playerNamed("No context", 249),
          club: null,
          division: null,
          dynamicValues: { "moneyball.average_rating": 9.8 },
        },
        {
          ...playerNamed("Club only", 248),
          club: "Test FC",
          division: null,
          dynamicValues: { "moneyball.average_rating": 9.7 },
        },
        {
          ...playerNamed("Division only", 247),
          club: null,
          division: "Premier Division",
          dynamicValues: { "moneyball.average_rating": 9.6 },
        },
        ...manyPlayers(99),
      ]);
      if (season) {
        usePlayerTableStore
          .getState()
          .addColumns("search", ["moneyball.average_rating"]);
      }
      const { router } = renderSearchRoute();

      const table = await screen.findByRole("table", {
        name: "Player search results",
      });
      expect(
        within(table).queryByRole("columnheader", { name: "Club" }),
      ).toBeNull();
      expect(
        within(table).queryByRole("columnheader", { name: "Division" }),
      ).toBeNull();
      const identityHeader = within(table).getAllByRole("columnheader")[0];
      expect(identityHeader).toHaveTextContent("Player");
      expect(identityHeader).toHaveAttribute("rowspan", "2");
      expect(identityHeader.className).toContain("sticky");
      expect(identityHeader.className).toContain("left-0");
      expect(table.querySelector("img")).toBeNull();
      const rows = within(table)
        .getAllByRole("row")
        .filter((row) => row.hasAttribute("data-index"));
      expect(rows.length).toBeGreaterThan(0);
      expect(rows.length).toBeLessThan(101);

      const identityRow = within(table)
        .getByText("Identity Player")
        .closest("tr");
      const missingContextRow = within(table)
        .getByText("No context")
        .closest("tr");
      const clubOnlyRow = within(table).getByText("Club only").closest("tr");
      const divisionOnlyRow = within(table)
        .getByText("Division only")
        .closest("tr");
      if (
        !identityRow ||
        !missingContextRow ||
        !clubOnlyRow ||
        !divisionOnlyRow
      ) {
        throw new Error("Expected stacked player identity rows.");
      }
      const identityCell = within(identityRow).getAllByRole("cell")[0];
      const missingContextCell =
        within(missingContextRow).getAllByRole("cell")[0];
      const clubOnlyCell = within(clubOnlyRow).getAllByRole("cell")[0];
      const divisionOnlyCell = within(divisionOnlyRow).getAllByRole("cell")[0];
      expect(identityRow).toHaveStyle({ height: "40px" });
      expect(identityCell.className).toContain("sticky");
      expect(identityCell).toHaveTextContent("Test FC · Premier Division");
      expect(missingContextCell).toHaveTextContent("No context");
      expect(missingContextCell).not.toHaveTextContent("—");
      expect(missingContextCell).not.toHaveTextContent(" · ");
      expect(clubOnlyCell).toHaveTextContent("Test FC");
      expect(clubOnlyCell).not.toHaveTextContent(" · ");
      expect(divisionOnlyCell).toHaveTextContent("Premier Division");
      expect(divisionOnlyCell).not.toHaveTextContent(" · ");
      identityRow.focus();
      expect(identityRow).toHaveFocus();

      await user.click(
        within(identityRow).getByText("Test FC · Premier Division"),
      );

      await waitFor(() => {
        expect(router.state.location.pathname).toBe("/players/250");
        expect(router.state.location.search).toEqual({});
      });
    },
  );

  it("navigates to /players/$uid when a results row is clicked", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Alex Morgan", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    await user.click(within(table).getByText("Alex Morgan"));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/160");
      expect(router.state.location.search).toEqual({});
    });
  });

  it("navigates to /players/$uid when Enter is pressed on a focused results row", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Alex Morgan", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const row = within(table)
      .getAllByRole("row")
      .find((candidate) => candidate.hasAttribute("data-index"));
    expect(row).toBeDefined();
    row?.focus();
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/160");
    });
  });

  it("restores Search sort after returning from a player profile", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara Scout", 160),
      playerNamed("Alex Scout", 145),
    ]);
    const { router } = renderSearchRoute("/search?sort=name&dir=asc");

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const sortedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!sortedRow) {
      throw new Error("expected a sorted virtualized body row");
    }
    expect(sortedRow).toHaveTextContent("Alex Scout");

    await user.click(sortedRow);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/145");
    });

    await router.history.back();

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/search");
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
      });
    });
    await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /^2 players · sorted by Name \(ascending\)$/,
    );
  });

  it("moves keyboard focus to the next results row on ArrowDown", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Alex Morgan", 160),
      playerNamed("Alexis Sanchez", 145),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const bodyRows = () =>
      within(table)
        .getAllByRole("row")
        .filter((row) => row.hasAttribute("data-index"));

    await waitFor(() => {
      expect(bodyRows().length).toBeGreaterThanOrEqual(2);
    });

    const first = bodyRows()[0];
    first.focus();
    expect(first).toHaveFocus();

    await user.keyboard("{ArrowDown}");

    await waitFor(() => {
      expect(bodyRows()[1]).toHaveFocus();
    });
  });

  it("moves keyboard focus to the previous results row on ArrowUp", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Alex Morgan", 160),
      playerNamed("Alexis Sanchez", 145),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const bodyRows = () =>
      within(table)
        .getAllByRole("row")
        .filter((row) => row.hasAttribute("data-index"));

    await waitFor(() => {
      expect(bodyRows().length).toBeGreaterThanOrEqual(2);
    });

    bodyRows()[1].focus();
    expect(bodyRows()[1]).toHaveFocus();

    await user.keyboard("{ArrowUp}");

    await waitFor(() => {
      expect(bodyRows()[0]).toHaveFocus();
    });
  });

  it("refetches already-cached search rows after Load Data", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Before Reload", 180)]);
    renderSearchRoute();

    expect(await screen.findByText("Before Reload")).toBeInTheDocument();

    setSearchPlayersOverride([playerNamed("After Reload", 190)]);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(await screen.findByText("After Reload")).toBeInTheDocument();
    expect(screen.queryByText("Before Reload")).not.toBeInTheDocument();
  });

  it("refetches cached search rows after switching active save", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Save One Star", 200)]);
    const { queryClient } = renderSearchRoute();

    expect(await screen.findByText("Save One Star")).toBeInTheDocument();

    const second = resolveCreateSaveIpcMock({ name: "Second save" });
    await queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });

    const saveSelect = screen.getByRole("combobox", { name: "Active save" });
    expect(
      await screen.findByRole("option", { name: "Second save" }),
    ).toBeInTheDocument();

    await user.selectOptions(saveSelect, String(second.id));
    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();

    setSearchPlayersOverride([playerNamed("Save Two Star", 150)]);
    await user.click(screen.getByRole("button", { name: "Load Data" }));
    expect(await screen.findByText("Save Two Star")).toBeInTheDocument();

    setSearchPlayersOverride([playerNamed("Save One Star", 200)]);
    await user.selectOptions(saveSelect, "1");
    expect(await screen.findByText("Save One Star")).toBeInTheDocument();
    expect(screen.queryByText("Save Two Star")).not.toBeInTheDocument();
  });

  it("deduplicates the initial Search page-zero IPC request", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Only once", 160)]);
    renderSearchRoute();

    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBe(1);
  });

  it("shows an initial Search failure and retries page zero", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Recovered Search", 160)]);
    setSearchPlayersPageIpcMockMode("rejectInitial");
    const user = userEvent.setup();
    renderSearchRoute();

    expect(
      await screen.findByText("Could not load players"),
    ).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBeGreaterThanOrEqual(1);

    setSearchPlayersPageIpcMockMode("success");
    const callsBeforeRetry = getSearchPlayersCallCount();
    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Recovered Search")).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBe(callsBeforeRetry + 1);
  });

  it("blocks Search results after an owner refresh fails", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Committed Search", 160)]);
    const { queryClient } = renderSearchRoute();

    expect(await screen.findByText("Committed Search")).toBeInTheDocument();
    const refresh = queryClient.fetchQuery({
      ...currentSnapshotQueryOptions,
      staleTime: 0,
      queryFn: () => Promise.reject(new Error("Snapshot refresh failed")),
    });
    await expect(refresh).rejects.toThrow("Snapshot refresh failed");

    expect(
      await screen.findByText("Loading player results…"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("table", { name: "Player search results" }),
    ).toBeNull();
  });

  it("defaults CA header to descending aria-sort", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High", 180),
      playerNamed("Low", 100),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: /CA/i });
    expect(caHeader).toHaveAttribute("aria-sort", "descending");
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /sorted by CA \(descending\)$/,
    );
  });

  it("writes sort into URL search params when a header is clicked", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    // CA desc → Zara first; CA asc → Alice first (orders must diverge).
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
      playerNamed("Bob", 150),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(await within(table).findByText("Zara")).toBeInTheDocument();

    const bodyRowsBefore = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstBefore = bodyRowsBefore[0];
    if (!firstBefore) {
      throw new Error("expected a virtualized body row before sort");
    }
    expect(within(firstBefore).getByText("Zara")).toBeInTheDocument();

    await user.click(within(table).getByRole("button", { name: "CA" }));

    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "asc",
    });
    const caHeader = within(table).getByRole("columnheader", {
      name: "CA",
    });
    await waitFor(() =>
      expect(caHeader).toHaveAttribute("aria-sort", "ascending"),
    );

    const bodyRowsAfter = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstAfter = bodyRowsAfter[0];
    if (!firstAfter) {
      throw new Error("expected a virtualized body row after sort");
    }
    expect(within(firstAfter).getByText("Alice")).toBeInTheDocument();
    expect(within(firstAfter).queryByText("Zara")).not.toBeInTheDocument();
  });

  it("keeps Search rows inactive with loading cells while a column change loads", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Projected Search", 160),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const callsBeforeProjection = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingProjection");
    act(() => {
      usePlayerTableStore
        .getState()
        .addColumns("search", ["attr.Acceleration"]);
    });

    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeProjection + 1),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading columns…");
    const row = retainedRow(table);
    expect(within(row).getByText("Projected Search")).toBeInTheDocument();
    const accelerationCell = () => {
      const headerLabels = within(table)
        .getAllByRole("columnheader")
        .filter((header) => header.getAttribute("scope") === "col")
        .map((header) => header.getAttribute("aria-label"));
      return within(retainedRow(table)).getAllByRole("cell")[
        headerLabels.indexOf("Acceleration")
      ];
    };
    // The committed row never requested Acceleration, so its cell waits.
    expect(accelerationCell()).toHaveTextContent("…");
    expect(row).not.toHaveAttribute("tabindex");
    fireEvent.click(row);
    fireEvent.keyDown(row, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/search");

    resolvePendingSearchPlayersPageIpcMock();
    await waitFor(() => expect(accelerationCell()).toHaveTextContent("16"));
    expect(screen.queryByRole("status")).toBeNull();
    act(() => {
      usePlayerTableStore
        .getState()
        .removeColumn("search", "attr.Acceleration");
    });
  });

  it("keeps Search rows while a filter loads, then shows the filtered rows from the top", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride(manyPlayers(120));
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const scroller = screen.getByTestId("search-results-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    expect(await within(table).findByText("Player 060")).toBeInTheDocument();
    const held = holdNamedSearchRequests();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "150" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() => expect(held).toHaveLength(1));
    expect(screen.getByRole("status")).toHaveTextContent("Filtering…");
    expect(screen.getByTestId("search-results-scroller")).toBe(scroller);
    expect(scroller.scrollTop).toBe(1_950);
    expect(within(table).getByText("Player 060")).toBeInTheDocument();
    expect(retainedRow(table)).not.toHaveAttribute("tabindex");
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /^120 players/,
    );

    act(() => held[0].resolve());
    await waitFor(() =>
      expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
        /^51 players/,
      ),
    );
    const filteredScroller = screen.getByTestId("search-results-scroller");
    expect(filteredScroller).not.toBe(scroller);
    expect(filteredScroller.scrollTop).toBe(0);
    expect(screen.getByText("Player 001")).toBeInTheDocument();
    expect(screen.queryByRole("status")).toBeNull();
    expect(router.state.location.search.filters).toEqual([
      expect.objectContaining({ field: "ca", op: "gte", value: 150 }),
    ]);
  });

  it("keeps Search rows through a shortlist selection, then shows its rows from the top", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [TARGETS]);
    setSearchPlayersOverride(manyPlayers(120));
    renderSearchRoute();

    const shortlistSelect = await screen.findByRole("combobox", {
      name: "Shortlist",
    });
    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const scroller = screen.getByTestId("search-results-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    expect(await within(table).findByText("Player 060")).toBeInTheDocument();
    const held = holdNamedSearchRequests();

    await user.selectOptions(shortlistSelect, "Targets");

    await waitFor(() => expect(held).toHaveLength(1));
    expect(screen.getByRole("status")).toHaveTextContent("Filtering…");
    expect(screen.getByTestId("search-results-scroller")).toBe(scroller);
    expect(within(table).getByText("Player 060")).toBeInTheDocument();
    expect(retainedRow(table)).not.toHaveAttribute("tabindex");

    act(() => held[0].resolve());
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    const switchedScroller = screen.getByTestId("search-results-scroller");
    expect(switchedScroller).not.toBe(scroller);
    expect(switchedScroller.scrollTop).toBe(0);
    expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: 1 });
  });

  it("clears Search rows when Load Data replaces the snapshot", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Before Reload", 180)]);
    renderSearchRoute();

    expect(await screen.findByText("Before Reload")).toBeInTheDocument();
    setSearchPlayersPageIpcMockMode("pendingProjection");
    setSearchPlayersOverride([playerNamed("After Reload", 190)]);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() =>
      expect(screen.queryByText("Before Reload")).not.toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("table", { name: "Player search results" }),
    ).toBeNull();

    resolvePendingSearchPlayersPageIpcMock();
    expect(await screen.findByText("After Reload")).toBeInTheDocument();
  });

  it("retains A until a stale cached Search sort refetch succeeds", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { queryClient, router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    await user.click(within(table).getByRole("button", { name: "Value" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    await user.click(within(table).getByRole("button", { name: "CA" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "CA" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    const cachedValueSort = queryClient
      .getQueryCache()
      .findAll({ queryKey: searchKeys.playerPages() })
      .find((query) => {
        const descriptor = query.queryKey.at(-1);
        return (
          typeof descriptor === "object" &&
          descriptor !== null &&
          (descriptor as { sortBy?: unknown }).sortBy === "value"
        );
      });
    if (!cachedValueSort) {
      throw new Error("expected a cached Search value sort");
    }
    await queryClient.invalidateQueries({ queryKey: cachedValueSort.queryKey });
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    await screen.findByRole("status");
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /^2 players · sorted by CA \(descending\)$/,
    );
    const retainedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!retainedRow) {
      throw new Error("expected a retained Search row");
    }
    fireEvent.click(retainedRow);
    fireEvent.keyDown(retainedRow, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/search");

    rejectPendingSearchPlayersPageIpcMock("Could not refresh sorted players.");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort players. Could not refresh sorted players.",
    );
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
  });

  it("retains committed Search rows and blocks activation while a replacement sort loads", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const callsBeforeSort = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeSort + 1),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Sorting…");
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    expect(within(table).getByText("Zara")).toBeInTheDocument();

    const row = within(table)
      .getAllByRole("row")
      .find((candidate) => candidate.hasAttribute("data-index"));
    if (!row) {
      throw new Error("expected a retained Search row");
    }
    expect(row).not.toHaveAttribute("tabindex");
    fireEvent.click(row);
    fireEvent.keyDown(row, { key: "ArrowDown" });
    fireEvent.keyDown(row, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/search");

    resolvePendingSearchPlayersPageIpcMock();
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    expect(within(table).getByText("Alice")).toBeInTheDocument();
  });

  it("falls back when removing a deferred requested dynamic Search sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    usePlayerTableStore.getState().addColumns("search", ["attr.Acceleration"]);
    setSearchPlayersOverride([
      {
        ...playerNamed("Fast Search", 160),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("pendingDynamicReplacement");
    await user.click(
      within(table).getByRole("button", { name: "Acceleration" }),
    );
    await screen.findByRole("status");

    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Acceleration" }),
    );
    await user.click(
      screen.getByRole("menuitem", { name: "Remove Acceleration" }),
    );

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
      expect(
        screen.queryByRole("columnheader", { name: "Acceleration" }),
      ).toBeNull();
      expect(screen.getByRole("columnheader", { name: "CA" })).toHaveAttribute(
        "aria-sort",
        "descending",
      );
    });
    expect(screen.queryByRole("status")).toBeNull();
    expect(screen.getByText("Fast Search")).toBeInTheDocument();

    resolvePendingSearchPlayersPageIpcMock();
    await Promise.resolve();
    expect(screen.getByRole("columnheader", { name: "CA" })).toHaveAttribute(
      "aria-sort",
      "descending",
    );
  });

  it("keeps committed Search rows after a failed sort and retries the replacement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("rejectReplacementOnce");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort players.",
    );
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    const retainedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!retainedRow) {
      throw new Error("expected a retained Search row after a failed sort");
    }
    expect(retainedRow).not.toHaveAttribute("tabindex");
    fireEvent.click(retainedRow);
    retainedRow.focus();
    await user.keyboard("{Enter}");
    expect(router.state.location.pathname).toBe("/search");

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("ignores a superseded Search sort when its deferred result resolves", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));
    await screen.findByRole("status");
    await user.click(within(table).getByRole("button", { name: "CA" }));

    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "CA" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    resolvePendingSearchPlayersPageIpcMock();
    await Promise.resolve();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "ascending");
    expect(within(table).getByText("Alice")).toBeInTheDocument();
  });

  it("names the Search table only on its requested first page", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(getLastSearchPlayersArgs()).not.toHaveProperty("tableId");

    const callsBeforeSort = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));
    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeSort + 1),
    );
    expect(getLastSearchPlayersArgs()).toMatchObject({
      offset: 0,
      sortBy: "value",
      tableId: "search",
    });
    resolvePendingSearchPlayersPageIpcMock();
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
  });

  it("refetches the committed Search page without naming the table", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Committed Star", 180)]);
    const { queryClient } = renderSearchRoute();

    expect(await screen.findByText("Committed Star")).toBeInTheDocument();
    const callsBeforeRefetch = getSearchPlayersCallCount();

    await queryClient.invalidateQueries({ queryKey: searchKeys.all });

    expect(getSearchPlayersCallCount()).toBe(callsBeforeRefetch + 1);
    expect(getLastSearchPlayersArgs()).toMatchObject({ offset: 0 });
    expect(getLastSearchPlayersArgs()).not.toHaveProperty("tableId");
  });

  it("never shows a superseded Search request as an error and requests it again", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const callsBeforeSort = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));
    await screen.findByRole("status");

    rejectPendingSearchPlayersPageIpcMock("Superseded by a newer request");

    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    expect(getSearchPlayersCallCount()).toBe(callsBeforeSort + 2);
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByText(/Superseded/)).toBeNull();
  });

  it("shows a superseded column change as loading rather than a failure", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Projected Search", 160)]);
    renderSearchRoute();

    expect(await screen.findByText("Projected Search")).toBeInTheDocument();
    const callsBeforeProjection = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingProjection");
    act(() => {
      usePlayerTableStore
        .getState()
        .addColumns("search", ["attr.Acceleration"]);
    });
    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeProjection + 1),
    );

    rejectPendingSearchPlayersPageIpcMock("Superseded by a newer request");
    setSearchPlayersPageIpcMockMode("pendingProjection");
    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeProjection + 2),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading columns…");
    expect(screen.getByText("Projected Search")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByText(/Superseded/)).toBeNull();

    resolvePendingSearchPlayersPageIpcMock();
    expect(await screen.findByText("Projected Search")).toBeInTheDocument();
    act(() => {
      usePlayerTableStore
        .getState()
        .removeColumn("search", "attr.Acceleration");
    });
  });

  it("defaults missing dir from the sort field for partial URLs", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { router } = renderSearchRoute("/search?sort=name");

    expect(router.state.location.search).toMatchObject({
      sort: "name",
      dir: "asc",
    });

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /^2 players · sorted by Name \(ascending\)$/,
    );

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstRow = bodyRows[0];
    if (!firstRow) {
      throw new Error("expected a virtualized body row");
    }
    expect(within(firstRow).getByText("Alice")).toBeInTheDocument();
  });

  it("keeps a drafted filter query-silent until Add, then applies it", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    const { router } = renderSearchRoute();

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    expect(screen.getByText("Low CA")).toBeInTheDocument();

    const callsBeforeEdit = getSearchPlayersCallCount();
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });

    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));

    const valueField = within(dialog).getByLabelText("Value");
    fireEvent.change(valueField, { target: { value: "150" } });

    expect(getSearchPlayersCallCount()).toBe(callsBeforeEdit);
    expect(router.state.location.search).toMatchObject({
      combine: "and",
      filters: [],
    });
    expect(screen.getByText("Low CA")).toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(getSearchPlayersCallCount()).toBeGreaterThan(callsBeforeEdit);
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gte", value: 150 },
      ]);
      expect(getLastSearchPlayersArgs()?.filterCombine).toBe("and");
    });
    expect(router.state.location.search).toMatchObject({
      combine: "and",
      filters: [
        expect.objectContaining({ field: "ca", op: "gte", value: 150 }),
      ],
    });
    expect(
      screen.getByRole("button", {
        name: /Remove filter CA ≥ 150/i,
      }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
    expect(screen.getByText("High CA")).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /Remove filter CA ≥ 150/i }),
    );
    expect(
      screen.queryByRole("button", { name: /Remove filter CA ≥ 150/i }),
    ).toBeNull();
    expect(await screen.findByText("Low CA")).toBeInTheDocument();
  });

  it("keeps a drafted potential role filter query-silent until Add", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Potential target", 180),
        dynamicValues: { "potential_role.goalkeeper_ip": 60 },
      },
    ]);
    renderSearchRoute();

    expect(await screen.findByText("Potential target")).toBeInTheDocument();
    const callsBeforeEdit = getSearchPlayersCallCount();
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "potential role score",
    );

    expect(
      within(dialog).getByRole("group", {
        name: "Potential role scores · Goalkeepers",
      }),
    ).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("option", {
        name: "Potential Role Score · Goalkeeper (IP)",
      }),
    );
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "50" },
    });

    expect(getSearchPlayersCallCount()).toBe(callsBeforeEdit);

    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "potential_role.goalkeeper_ip", op: "gte", value: 50 },
      ]);
    });
    expect(
      await screen.findByRole("columnheader", {
        name: "Potential Role Score · Goalkeeper (IP)",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: /Remove filter Potential Role Score · Goalkeeper \(IP\) ≥ 50/i,
      }),
    );
    // The retained table remounts once the unfiltered page commits.
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());

    expect(
      await screen.findByRole("columnheader", {
        name: "Potential Role Score · Goalkeeper (IP)",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        requestedFields: ["height", "potential_role.goalkeeper_ip"],
      });
    });
  });

  it("shows a no-matches empty state when filters exclude every player", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    renderSearchRoute();

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "250" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    expect(
      await screen.findByText("No players match these filters"),
    ).toBeInTheDocument();
    expect(screen.queryByText("High CA")).not.toBeInTheDocument();
  });

  it("keeps the committed no-matches empty state while a cleared filter loads", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    const encodedFilters = encodeURIComponent(
      JSON.stringify([{ id: "seed", field: "ca", op: "gte", value: 250 }]),
    );
    renderSearchRoute(
      `/search?sort=ca&dir=desc&combine=and&filters=${encodedFilters}`,
    );
    expect(
      await screen.findByText("No players match these filters"),
    ).toBeInTheDocument();
    const held = holdNamedSearchRequests();

    await user.click(
      screen.getByRole("button", { name: /Remove filter CA ≥ 250/i }),
    );

    await waitFor(() => expect(held).toHaveLength(1));
    expect(screen.getByRole("status")).toHaveTextContent("Filtering…");
    expect(
      screen.getByText("No players match these filters"),
    ).toBeInTheDocument();
    expect(screen.queryByText("No players in snapshot")).toBeNull();

    act(() => held[0].resolve());
    expect(await screen.findByText("High CA")).toBeInTheDocument();
  });

  it("shows the filtered empty state when a season-stat filter excludes every player", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([]);
    const encoded = encodeURIComponent(
      JSON.stringify([
        { field: "moneyball.average_rating", op: "gte", value: 7 },
      ]),
    );
    renderSearchRoute(`/search?filters=${encoded}`);

    expect(
      await screen.findByText("No players match these filters"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("No Moneyball stats for this snapshot"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Clear filters to see more players. Players under 450 minutes drop out of season-stat filters.",
      ),
    ).toBeInTheDocument();
  });

  it("filters Moneyball percentage stats on the displayed 0–100 scale", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Passer", 150)]);
    const encoded = encodeURIComponent(
      JSON.stringify([
        { field: "moneyball.pass_completion_ratio", op: "gte", value: 80 },
        {
          field: "moneyball.pass_completion_ratio",
          op: "pct_gte",
          value: 90,
        },
      ]),
    );
    renderSearchRoute(`/search?filters=${encoded}`);

    expect(
      await screen.findByRole("button", {
        name: "Remove filter Pass Completion % ≥ 80%",
      }),
    ).toBeInTheDocument();
    expect(getLastSearchPlayersArgs()?.filters).toEqual([
      { field: "moneyball.pass_completion_ratio", op: "gte", value: 0.8 },
      { field: "moneyball.pass_completion_ratio", op: "pct_gte", value: 90 },
    ]);
  });

  it("keeps focus in the filter value field while editing", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    renderSearchRoute();

    await user.click(await screen.findByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));

    const valueField = within(dialog).getByLabelText("Value");
    await user.click(valueField);
    expect(valueField).toHaveFocus();

    await user.clear(valueField);
    await user.type(valueField, "1");
    expect(valueField).toHaveFocus();
    await user.type(valueField, "50");
    expect(valueField).toHaveFocus();
  });

  it("writes filters and combine into URL search params and restores them", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    const seeded = encodeURIComponent(
      JSON.stringify([{ id: "seed", field: "ca", op: "gte", value: 160 }]),
    );
    const { router } = renderSearchRoute(`/search?filters=${seeded}`);

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "150" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Add" }));
    await user.click(
      within(await screen.findByRole("group", { name: "Match" })).getByRole(
        "button",
        { name: "Any" },
      ),
    );

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
        combine: "or",
        filters: [
          expect.objectContaining({ field: "ca", op: "gte", value: 160 }),
          expect.objectContaining({ field: "ca", op: "gte", value: 150 }),
        ],
      });
      const href = decodeURIComponent(router.state.location.href);
      expect(href).toContain('"value":150');
      expect(href).not.toContain('"type":"integer"');
    });

    await router.navigate({ to: "/" });
    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();

    await router.history.back();

    expect(
      await screen.findByRole("button", {
        name: /Remove filter CA ≥ 150/i,
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gte", value: 160 },
        { field: "ca", op: "gte", value: 150 },
      ]);
      expect(getLastSearchPlayersArgs()?.filterCombine).toBe("or");
    });
    expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
  });

  it("restores filters from the initial URL search string", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    const encodedFilters = encodeURIComponent(
      JSON.stringify([{ id: "seed", field: "ca", op: "gte", value: 150 }]),
    );
    renderSearchRoute(
      `/search?sort=ca&dir=desc&combine=and&filters=${encodedFilters}`,
    );

    expect(
      await screen.findByRole("button", {
        name: /Remove filter CA ≥ 150/i,
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gte", value: 150 },
      ]);
    });
    expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
  });

  it("stops adding filter rules once the UI cap is reached", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    const rules = Array.from({ length: 32 }, (_, index) => ({
      field: "ca",
      op: "gte",
      value: index,
    }));
    renderSearchRoute(
      `/search?filters=${encodeURIComponent(JSON.stringify(rules))}`,
    );

    expect(
      await screen.findByRole("button", { name: "Add filter" }),
    ).toBeDisabled();
    expect(
      screen.getAllByRole("button", { name: /^Remove filter CA/ }),
    ).toHaveLength(32);
  });

  it("sorts and starts a filter from a column's menu", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High", 180),
      playerNamed("Low", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const paHeader = within(table).getByRole("columnheader", { name: "PA" });
    fireEvent.contextMenu(paHeader);
    await user.click(screen.getByRole("menuitem", { name: "Sort ascending" }));
    expect(router.state.location.search).toMatchObject({
      sort: "pa",
      dir: "asc",
    });

    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "PA" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Filter by PA" }));
    const editor = screen.getByRole("form", { name: "Add filter" });
    expect(
      within(editor).getByRole("button", { name: "Field: PA" }),
    ).toBeInTheDocument();
    expect(within(editor).getByLabelText("Value")).toHaveFocus();
  });

  it("sorts by any field from the summary and shows its column", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High", 180),
      playerNamed("Low", 100),
    ]);
    const { router } = renderSearchRoute();

    const summary = await screen.findByTestId("search-results-summary");
    await user.click(
      within(summary).getByRole("button", { name: "Sort by: CA" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search fields" }),
      "pace",
    );
    await user.keyboard("{Enter}");

    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({
        sort: "attr.Pace",
        dir: "desc",
      }),
    );
    expect(
      await screen.findByRole("columnheader", { name: "Pace" }),
    ).toBeInTheDocument();

    await user.click(
      within(screen.getByTestId("search-results-summary")).getByRole("button", {
        name: "Sort direction: descending",
      }),
    );
    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({
        sort: "attr.Pace",
        dir: "asc",
      }),
    );
  });

  it("toggles CA from default descending to ascending on header click", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High", 180),
      playerNamed("Low", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caButton = await within(table).findByRole("button", {
      name: /^CA$/i,
    });

    await user.click(caButton);
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "asc",
    });
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: /CA/i }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstRow = bodyRows[0];
    if (!firstRow) {
      throw new Error("expected a virtualized body row");
    }
    expect(within(firstRow).getByText("Low")).toBeInTheDocument();
  });

  it("returns to its last filters, sort, and comparison through the Search link", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    const encodedFilters = encodeURIComponent(
      JSON.stringify([{ field: "ca", op: "gte", value: 150 }]),
    );
    const { router } = renderSearchRoute(
      `/search?sort=pa&dir=asc&combine=or&comparison=striker&filters=${encodedFilters}`,
    );
    await screen.findByText("High CA");
    const state = {
      sort: "pa",
      dir: "asc",
      combine: "or",
      comparison: "striker",
      filters: [expect.objectContaining({ field: "ca" })],
    };
    expect(router.state.location.search).toMatchObject(state);

    await user.click(screen.getByRole("link", { name: "Settings" }));
    await waitFor(() =>
      expect(router.state.location.pathname).toBe("/settings"),
    );
    await user.click(screen.getByRole("link", { name: "Search" }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/search");
      expect(router.state.location.search).toMatchObject(state);
    });
    expect(router.state.location.search).not.toHaveProperty("view");
  });

  it("keeps a Name sort from a direct URL", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Name Sorted", 160)]);
    const { router } = renderSearchRoute("/search?sort=name&dir=asc");

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
      });
    });
    await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(screen.getByTestId("search-results-summary")).toHaveTextContent(
      /^1 players · sorted by Name \(ascending\)$/,
    );
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        sortBy: "name",
        sortDir: "asc",
      });
    });
  });

  it("keeps active non-basic filter fields hidden until added to the layout", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Role Fit", 160),
        dynamicValues: {
          "role.deep_lying_playmaker_ip": 82,
          "attr.Acceleration": 16,
        },
      },
    ]);

    const filters = encodeURIComponent(
      JSON.stringify([
        { field: "role.deep_lying_playmaker_ip", op: "gte", value: 70 },
        { field: "attr.Acceleration", op: "gte", value: 12 },
      ]),
    );
    renderSearchRoute(
      `/search?sort=ca&dir=desc&combine=and&filters=${filters}`,
    );

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).queryByRole("columnheader", {
        name: /Role Score · Deep-Lying Playmaker \(IP\)/i,
      }),
    ).toBeNull();
    expect(
      within(table).queryByRole("columnheader", { name: /Acceleration/i }),
    ).toBeNull();
    expect(getLastSearchPlayersArgs()).toMatchObject({
      requestedFields: ["height"],
      filters: [
        { field: "role.deep_lying_playmaker_ip", op: "gte", value: 70 },
        { field: "attr.Acceleration", op: "gte", value: 12 },
      ],
    });
  });

  it("renders Search without local view tabs or a Player Shortlist tab", async () => {
    await resolveLoadDataIpcMock();
    renderSearchRoute("/search");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "General" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Moneyball" })).toBeNull();
    expect(
      screen.queryByRole("tab", { name: "Shortlist" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Search", current: "page" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Moneyball" })).toBeNull();
    expect(screen.queryByText("No shortlist yet")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Go to Moneyball" }),
    ).not.toBeInTheDocument();
    expect(usePlayerTableStore.getState().layouts).not.toHaveProperty(
      "shortlist",
    );
  });

  it("lists No shortlist and each of the save's shortlists, and starts with none", async () => {
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [
      LOANS,
      TARGETS,
      { id: 7, name: "(Default)" },
    ]);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    renderSearchRoute("/search");

    const select = await screen.findByRole("combobox", { name: "Shortlist" });
    expect(
      within(select)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["No shortlist", "Loans", "Targets", "(Default)"]);
    expect(select).toHaveValue("");
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: null });
    });
  });

  it("hides the shortlist dropdown when the save has no shortlists", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    renderSearchRoute("/search");

    await screen.findByText("General Scout");
    expect(screen.queryByRole("combobox", { name: "Shortlist" })).toBeNull();
  });

  it("restricts Search to the selected shortlist and starts each save context with none", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [LOANS, TARGETS]);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    usePlayerTableStore
      .getState()
      .addColumns("search", ["moneyball.average_rating"]);
    const { router, queryClient } = renderSearchRoute("/search");

    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Shortlist" }),
      "Targets",
    );
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: 1 });
    });
    expect(router.state.location.search).not.toHaveProperty("shortlistOnly");

    // The dropdown sits with Percentiles vs in the toolbar.
    const toolbar = screen.getByRole("toolbar", {
      name: "Player results toolbar",
    });
    expect(
      within(toolbar).getByRole("combobox", { name: "Shortlist" }),
    ).toHaveValue("1");
    expect(
      within(toolbar).getByRole("combobox", { name: "Percentiles vs" }),
    ).toBeInTheDocument();

    await user.selectOptions(
      within(toolbar).getByRole("combobox", { name: "Shortlist" }),
      "No shortlist",
    );
    // The unrestricted first page is already cached, so no request follows.
    await waitFor(() => expect(storedShortlistSelection()).toBeUndefined());
    expect(screen.getByRole("combobox", { name: "Shortlist" })).toHaveValue("");
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Shortlist" }),
      "Loans",
    );

    // Another save context starts with no shortlist selected.
    await resolveCreateSaveIpcMock({ name: "Save B" });
    await resolveSetActiveSaveIpcMock({ saveId: 2 });
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(2, [{ ...LOANS, id: 7 }]);
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
      await queryClient.invalidateQueries({ queryKey: searchKeys.all });
    });

    await waitFor(() =>
      expect(screen.getByRole("combobox", { name: "Shortlist" })).toHaveValue(
        "",
      ),
    );
    expect(storedShortlistSelection()).toBe(LOANS.id);
  });

  it("selects nothing for a stored shortlist the save does not list and clears one Search no longer finds", async () => {
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [TARGETS]);
    selectStoredShortlist(9);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    const { queryClient } = renderSearchRoute("/search");

    const select = await screen.findByRole("combobox", { name: "Shortlist" });
    expect(select).toHaveValue("");
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: null });
    });
    // A stored ID is never cleared merely because the save does not list it.
    expect(storedShortlistSelection()).toBe(9);

    // The shortlist was removed elsewhere, but the saves list is stale.
    act(() => selectStoredShortlist(TARGETS.id));
    await waitFor(() => expect(select).toHaveValue("1"));
    setSavePlayerShortlistsIpcMock(1, []);
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: searchKeys.all });
    });

    await waitFor(() => expect(storedShortlistSelection()).toBeUndefined());
  });

  it("keeps the selection when Search answers for another save's snapshot", async () => {
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [LOANS]);
    selectStoredShortlist(LOANS.id);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    const { queryClient } = renderSearchRoute("/search");
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        shortlistId: LOANS.id,
      });
    });

    // Another save becomes current before this panel's views refresh, so
    // Search checks the shortlist against that save.
    await resolveCreateSaveIpcMock({ name: "Save B" });
    await resolveSetActiveSaveIpcMock({ saveId: 2 });
    await resolveLoadDataIpcMock();
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: searchKeys.all });
    });
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
    });

    expect(storedShortlistSelection()).toBe(LOANS.id);
  });

  it.each([
    ["a legacy shortlistOnly URL", "/search?shortlistOnly=true"],
    ["a legacy shortlist view", "/search?view=shortlist"],
  ])(
    "opens %s with no shortlist selected and a clean URL",
    async (_case, url) => {
      await resolveLoadDataIpcMock();
      setSavePlayerShortlistsIpcMock(1, [TARGETS]);
      selectStoredShortlist(TARGETS.id);
      setSearchPlayersOverride([playerNamed("General Scout", 160)]);
      const { router } = renderSearchRoute(url);

      await waitFor(() => {
        expect(router.state.location.href).not.toContain("shortlist");
      });
      expect(router.history.canGoBack()).toBe(false);
      expect(storedShortlistSelection()).toBeUndefined();
      expect(
        await screen.findByRole("heading", { name: "Player Search" }),
      ).toBeInTheDocument();
      expect(
        await screen.findByRole("combobox", { name: "Shortlist" }),
      ).toHaveValue("");
      await waitFor(() => {
        expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: null });
      });
      expect(router.state.location.search).not.toHaveProperty("view");
    },
  );

  it("composes the selected shortlist with a filter in one fetch", async () => {
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [TARGETS]);
    selectStoredShortlist(TARGETS.id);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    const encoded = encodeURIComponent(
      JSON.stringify([{ field: "ca", op: "gte", value: 150 }]),
    );
    renderSearchRoute(`/search?filters=${encoded}`);

    await screen.findByText("General Scout");
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        shortlistId: 1,
        filters: [expect.objectContaining({ field: "ca" })],
      });
    });
  });

  it.each([
    {
      state: "no shortlisted player in the snapshot",
      route: "/search",
      filters: "",
      title: "No shortlisted player in this snapshot",
      body: "Select No shortlist to browse every player.",
    },
    {
      state: "no shortlisted player matching the filters",
      route: "/search",
      filters: `?filters=${encodeURIComponent(
        JSON.stringify([{ field: "ca", op: "gte", value: 150 }]),
      )}`,
      title: "No shortlisted players match these filters",
      body: "Clear filters, or select No shortlist.",
    },
  ])(
    "explains a selected shortlist with $state",
    async ({ route, filters, title, body }) => {
      await resolveLoadDataIpcMock();
      setSavePlayerShortlistsIpcMock(1, [TARGETS]);
      selectStoredShortlist(TARGETS.id);
      setSearchPlayersOverride([]);
      renderSearchRoute(`${route}${filters}`);

      expect(await screen.findByText(title)).toBeInTheDocument();
      expect(screen.getByText(body)).toBeInTheDocument();
      expect(screen.getByRole("combobox", { name: "Shortlist" })).toHaveValue(
        "1",
      );
    },
  );

  it("refreshes season stats for the save context and reloads their as-of date", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
    setRefreshSeasonStatsIpcMockResult({
      status: "attached",
      matchedPlayers: 1_200,
      unmatchedPlayers: 3,
      gameDate: "2026-06-30",
      shortlists: { status: "notRead" },
      staffShortlist: { status: "notRead" },
      elapsedMs: 40,
    });
    // The as-of date shows beside Percentiles vs while a season column is in use.
    usePlayerTableStore
      .getState()
      .addColumns("search", ["moneyball.average_rating"]);
    const { queryClient } = renderSearchRoute();
    await screen.findByText("Moneyball Scout");
    const toolbar = screen.getByRole("toolbar", {
      name: "Player results toolbar",
    });
    expect(within(toolbar).queryByText(/Stats as of/)).toBeNull();
    const invalidate = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(screen.getByRole("button", { name: "Refresh stats" }));

    expect(
      await screen.findByText(
        "Attached season stats for 1,200 players from the save dated 30th June 2026.",
      ),
    ).toBeInTheDocument();
    expect(getLastRefreshSeasonStatsIpcArgs()).toEqual({
      saveId: 1,
      contextToken: "save-token-1",
    });
    for (const queryKey of [
      snapshotKeys.all,
      searchKeys.all,
      moneyballKeys.all,
      playerKeys.all,
      academyKeys.all,
    ]) {
      expect(invalidate).toHaveBeenCalledWith({ queryKey });
    }
    expect(
      await within(
        screen.getByRole("toolbar", { name: "Player results toolbar" }),
      ).findByText("Stats as of 30th June 2026 · Snapshot dated 1st July 2026"),
    ).toBeInTheDocument();

    const second = resolveCreateSaveIpcMock({ name: "Second save" });
    await queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    await screen.findByRole("option", { name: "Second save" });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      String(second.id),
    );
    await screen.findByText("No data loaded for this save");
    expect(
      screen.queryByText(/Attached season stats for 1,200 players/),
    ).toBeNull();
  });

  it("reloads Staff queries when Refresh stats reads the staff shortlist without attaching stats", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
    setRefreshSeasonStatsIpcMockResult({
      status: "noMatches",
      unmatchedPlayers: 3,
      gameDate: "2026-06-30",
      shortlists: { status: "notRead" },
      staffShortlist: { status: "read", count: 13, unread: 0 },
      elapsedMs: 40,
    });
    const { queryClient } = renderSearchRoute();
    await screen.findByText("Moneyball Scout");
    const invalidate = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(screen.getByRole("button", { name: "Refresh stats" }));

    expect(
      await screen.findByText(/Read 13 shortlisted staff from the save\./),
    ).toBeInTheDocument();
    expect(invalidate).toHaveBeenCalledWith({ queryKey: staffKeys.all });
    expect(invalidate).not.toHaveBeenCalledWith({ queryKey: searchKeys.all });
  });

  it("keeps Refresh stats unavailable while a refresh or Load Data runs", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
    renderSearchRoute();
    await screen.findByText("Moneyball Scout");
    const refresh = screen.getByRole("button", { name: "Refresh stats" });

    setRefreshSeasonStatsIpcMockPending();
    await user.click(refresh);
    await waitFor(() => expect(refresh).toBeDisabled());
    resolvePendingRefreshSeasonStats();
    expect(
      await screen.findByText(
        "No FM save file is linked, so no season stats were read. Link one in Settings.",
      ),
    ).toBeInTheDocument();
    expect(refresh).toBeEnabled();

    setLoadDataIpcMockMode("busy");
    await user.click(screen.getByRole("button", { name: "Load Data" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Refresh stats" }),
      ).toBeDisabled(),
    );
    await act(async () => {
      await resolveBusyLoadDataRequest();
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Refresh stats" }),
      ).toBeEnabled(),
    );

    // Load Data started elsewhere, such as in another window, is reported by the command.
    setRefreshSeasonStatsIpcMockResult("loadDataRunning");
    await user.click(screen.getByRole("button", { name: "Refresh stats" }));
    expect(
      await screen.findByText(
        "Load Data is running. Refresh stats after it finishes.",
      ),
    ).toBeInTheDocument();
  });

  describe("saved filters", () => {
    async function openSavedFilters(user: ReturnType<typeof userEvent.setup>) {
      await user.click(screen.getByRole("button", { name: "Saved filters" }));
      return screen.getByRole("dialog", { name: "Saved filters" });
    }

    it("saves the applied filters under a name and applies them again", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([
        {
          ...playerNamed("High CA", 180),
          dynamicValues: { "moneyball.average_rating": 7.4 },
        },
        playerNamed("Low CA", 100),
      ]);
      const rules = [
        { field: "ca", op: "gte", value: 150 },
        { field: "moneyball.average_rating", op: "gte", value: 7 },
      ];
      const { router } = renderSearchRoute(
        `/search?combine=or&comparison=striker&filters=${encodeURIComponent(
          JSON.stringify(rules),
        )}`,
      );
      await screen.findByText("High CA");

      let dialog = await openSavedFilters(user);
      expect(within(dialog).getByText("No saved filters")).toBeInTheDocument();
      const name = within(dialog).getByLabelText("Save current filters as");
      expect(
        within(dialog).getByRole("button", { name: "Save" }),
      ).toBeDisabled();
      await user.type(name, "Scouting");
      await user.click(within(dialog).getByRole("button", { name: "Save" }));
      expect(useSavedFiltersStore.getState().savedFilters).toEqual([
        expect.objectContaining({
          name: "Scouting",
          combine: "or",
          comparison: "striker",
          filters: rules.map((rule) => expect.objectContaining(rule)),
        }),
      ]);
      // Saving under the same name offers to replace it.
      await user.type(name, "scouting");
      expect(
        within(dialog).getByRole("button", { name: "Replace" }),
      ).toBeEnabled();
      await user.keyboard("{Escape}");

      // A clean Search, with no season column in the layout.
      await act(async () => {
        await router.navigate({
          to: "/search",
          search: { sort: "ca", dir: "desc", filters: [], combine: "and" },
        });
      });
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({
          filters: [],
          combine: "and",
        });
      });
      expect(
        usePlayerTableStore.getState().layouts.search.columnIds,
      ).not.toContain("moneyball.average_rating");

      dialog = await openSavedFilters(user);
      await user.click(
        within(dialog).getByRole("button", { name: "Scouting" }),
      );
      const prompt = await screen.findByRole("dialog", {
        name: "Replace the columns?",
      });
      await user.click(
        within(prompt).getByRole("button", { name: "Keep columns" }),
      );

      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({
          combine: "or",
          comparison: "striker",
          filters: rules.map((rule) => expect.objectContaining(rule)),
        });
        expect(getLastSearchPlayersArgs()).toMatchObject({
          comparison: "striker",
          filterCombine: "or",
          filters: rules,
        });
      });
      expect(
        screen.queryByRole("dialog", { name: "Saved filters" }),
      ).toBeNull();
      // Keeping the columns adds the rules' columns, as applying one filter does.
      await waitFor(() =>
        expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
          [
            "age",
            "nationality",
            "height",
            "ca",
            "pa",
            "value",
            "moneyball.average_rating",
          ],
        ),
      );
      expect(
        await screen.findByRole("combobox", { name: "Percentiles vs" }),
      ).toHaveValue("striker");
    });

    it("replaces the columns with a saved filter's layout and stops asking once told", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      useSavedFiltersStore.getState().save({
        name: "Inside forwards",
        filters: [
          { field: "potential_role.inside_forward_ip", op: "gte", value: 70 },
          { field: "height", op: "gte", value: 180 },
        ],
        combine: "and",
        comparison: "own",
      });
      const { router } = renderSearchRoute();
      await screen.findByRole("table", { name: "Player search results" });
      const replaced = [
        "age",
        "nationality",
        "height",
        "value",
        "role.inside_forward_ip",
        "potential_role.inside_forward_ip",
      ];
      const columnIds = () =>
        usePlayerTableStore.getState().layouts.search.columnIds;
      const promptClosed = () =>
        waitFor(() =>
          expect(
            screen.queryByRole("dialog", { name: "Replace the columns?" }),
          ).toBeNull(),
        );
      const applyInsideForwards = async () => {
        const dialog = await openSavedFilters(user);
        await user.click(
          within(dialog).getByRole("button", { name: "Inside forwards" }),
        );
      };

      await applyInsideForwards();
      let prompt = await screen.findByRole("dialog", {
        name: "Replace the columns?",
      });
      await user.click(
        within(prompt).getByRole("button", { name: "Replace columns" }),
      );
      await promptClosed();
      await waitFor(() => expect(columnIds()).toEqual(replaced));
      expect(router.state.location.search).toMatchObject({
        filters: [
          expect.objectContaining({
            field: "potential_role.inside_forward_ip",
          }),
          expect.objectContaining({ field: "height" }),
          // Without a managed club, the age limit is the only default.
          expect.objectContaining({ field: "age", op: "lte", value: 28 }),
        ],
      });

      // The columns already match, so there is nothing to ask.
      await applyInsideForwards();
      expect(
        screen.queryByRole("dialog", { name: "Replace the columns?" }),
      ).toBeNull();

      act(() => usePlayerTableStore.getState().addColumns("search", ["ca"]));
      await applyInsideForwards();
      prompt = await screen.findByRole("dialog", {
        name: "Replace the columns?",
      });
      await user.click(
        within(prompt).getByRole("checkbox", {
          name: "Don't ask again this session",
        }),
      );
      await user.click(
        within(prompt).getByRole("button", { name: "Replace columns" }),
      );
      await promptClosed();
      await waitFor(() => expect(columnIds()).toEqual(replaced));

      act(() => usePlayerTableStore.getState().addColumns("search", ["ca"]));
      await applyInsideForwards();
      await waitFor(() => expect(columnIds()).toEqual(replaced));
      expect(
        screen.queryByRole("dialog", { name: "Replace the columns?" }),
      ).toBeNull();
    });

    it("renames and deletes a saved filter", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      useSavedFiltersStore.getState().save({
        name: "Wonderkids",
        filters: [{ field: "age", op: "lte", value: 19 }],
        combine: "and",
        comparison: "own",
      });
      renderSearchRoute();
      await screen.findByRole("table", { name: "Player search results" });

      const dialog = await openSavedFilters(user);
      await user.click(
        within(dialog).getByRole("button", { name: "Rename Wonderkids" }),
      );
      const rename = within(dialog).getByRole("textbox", {
        name: "Rename Wonderkids",
      });
      await user.clear(rename);
      await user.type(rename, "Teen prospects");
      const renameForm = rename.closest("form");
      if (!renameForm) throw new Error("Expected the rename form");
      await user.click(
        within(renameForm).getByRole("button", { name: "Save" }),
      );
      expect(
        within(dialog).getByRole("button", { name: "Teen prospects" }),
      ).toBeInTheDocument();

      await user.click(
        within(dialog).getByRole("button", { name: "Delete Teen prospects" }),
      );
      expect(useSavedFiltersStore.getState().savedFilters).toEqual([]);
      expect(within(dialog).getByText("No saved filters")).toBeInTheDocument();
    });

    it("applies a tactic slot's positions, recruitment filters, and recruiting defaults, ranked among the slot's two families", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setManagedClubIpcMock({
        clubName: "Stevenage",
        clubUid: 22,
        status: "available",
        unclassifiedPlayerCount: 0,
      });
      const tactic = resolvePlannerTacticIpcMock();
      const winger = tactic.lanes.find((lane) => lane.laneId === "left_winger");
      if (!winger) throw new Error("Expected the left winger slot");
      winger.recruitmentFilters = [
        { field: "height", op: "gte", value: 180 },
        { field: "moneyball.progressive_passes", op: "pct_gte", value: 25 },
      ];
      setPlannerTacticForContext(
        { saveId: 1, contextToken: "save-token-1" },
        tactic,
      );
      setSearchPlayersOverride([playerNamed("Wide Scout", 160)]);
      const { router } = renderSearchRoute();
      await screen.findByText("Wide Scout");
      const realistic = screen.getByRole("switch", {
        name: "Only realistic signings",
      });
      await waitFor(() => expect(realistic).toBeEnabled());

      const dialog = await openSavedFilters(user);
      const slots = await within(dialog).findByRole("region", {
        name: "Tactic slot filters",
      });
      // Only slots with recruitment filters are offered.
      expect(within(slots).getAllByRole("button")).toHaveLength(1);
      await user.click(
        within(slots).getByRole("button", { name: /^AML\/ML slot: / }),
      );
      const prompt = await screen.findByRole("dialog", {
        name: "Replace the columns?",
      });
      await user.click(
        within(prompt).getByRole("button", { name: "Replace columns" }),
      );

      // The slot only scores a player rated 12 or higher at both positions.
      const expected = [
        { field: "pos.AML", op: "gte", value: 12 },
        { field: "pos.ML", op: "gte", value: 12 },
        { field: "height", op: "gte", value: 180 },
        { field: "moneyball.progressive_passes", op: "pct_gte", value: 25 },
        { field: "realistic_signing", op: "is", value: true },
        { field: "age", op: "lte", value: 28 },
      ];
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({
          combine: "and",
          comparison: "winger+wide_midfielder",
          filters: expected.map((rule) => expect.objectContaining(rule)),
        });
        expect(getLastSearchPlayersArgs()).toMatchObject({
          comparison: "winger+wide_midfielder",
          filters: expected,
        });
      });
      expect(
        await screen.findByRole("switch", {
          name: "Only realistic signings",
        }),
      ).toBeChecked();
      const comparison = await screen.findByRole("combobox", {
        name: "Percentiles vs",
      });
      expect(comparison).toHaveValue("winger+wide_midfielder");
      expect(
        within(comparison).getByRole("option", {
          name: "Wingers + Wide midfielders",
          selected: true,
        }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("button", {
          name: "Edit filter Position · AML suitability ≥ 12",
        }),
      ).toBeInTheDocument();
      // The slot's own Role Score and Potential replace the other columns.
      await waitFor(() =>
        expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
          [
            "age",
            "nationality",
            "height",
            "value",
            "pos.AML",
            "pos.ML",
            "moneyball.progressive_passes",
            "tactic_current.left_winger",
            "tactic_potential.left_winger",
          ],
        ),
      );
    });
  });

  describe("search table toolbar", () => {
    it("associates summary, chips, edit, columns, and dataset toggles in one table toolbar", async () => {
      await resolveLoadDataIpcMock();
      setSavePlayerShortlistsIpcMock(1, [TARGETS]);
      setSearchPlayersOverride([playerNamed("High CA", 180)]);
      renderSearchRoute();

      await screen.findByText("High CA");
      const toolbar = screen.getByRole("toolbar", {
        name: "Player results toolbar",
      });
      expect(
        within(toolbar).getByText(/players · sorted by/),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("button", { name: "Add filter" }),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("button", { name: "Columns" }),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("combobox", { name: "Shortlist" }),
      ).toBeInTheDocument();
      const table = screen.getByRole("table", {
        name: "Player search results",
      });
      expect(
        toolbar.compareDocumentPosition(table) &
          Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
    });

    it("keeps Add Tactic outside the generic toolbar", async () => {
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("High CA", 180)]);
      renderSearchRoute();

      await screen.findByText("High CA");
      const toolbar = screen.getByRole("toolbar", {
        name: "Player results toolbar",
      });
      const header = screen.getByTestId("search-page-header");
      expect(
        within(header).getByRole("heading", {
          level: 1,
          name: "Player Search",
        }),
      ).toBeInTheDocument();
      expect(
        within(header).getByTestId("search-page-actions"),
      ).toBeInTheDocument();
      for (const name of [
        "Add Tactic (Role Score)",
        "Add Tactic (Potential)",
        "Add Tactic (Moneyball Score)",
        "Refresh stats",
      ]) {
        expect(screen.getByRole("button", { name })).toBeInTheDocument();
        expect(
          within(header).getByRole("button", { name }),
        ).toBeInTheDocument();
        expect(within(toolbar).queryByRole("button", { name })).toBeNull();
      }
    });

    it("removes chips and clears all through the toolbar", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([
        playerNamed("High CA", 180),
        playerNamed("Low CA", 100),
      ]);
      const { router } = renderSearchRoute(
        `/search?sort=ca&dir=desc&combine=and&filters=${encodeURIComponent(
          JSON.stringify([
            { field: "ca", op: "gte", value: 150 },
            { field: "pa", op: "gte", value: 150 },
          ]),
        )}`,
      );

      await screen.findByText("High CA");
      const getToolbar = () =>
        screen.getByRole("toolbar", {
          name: "Player results toolbar",
        });
      const toolbar = getToolbar();
      expect(
        within(toolbar).getAllByRole("button", { name: /Remove filter/i }),
      ).toHaveLength(2);
      expect(
        within(toolbar).getByRole("button", { name: "Clear all" }),
      ).toBeInTheDocument();

      await user.click(
        within(getToolbar()).getByRole("button", {
          name: /Remove filter CA ≥ 150/i,
        }),
      );
      await waitFor(() => {
        expect(router.state.location.search.filters).toHaveLength(1);
      });
      await waitFor(() => {
        expect(
          within(getToolbar()).queryByRole("button", {
            name: /Remove filter CA ≥ 150/i,
          }),
        ).toBeNull();
      });

      await user.click(
        within(getToolbar()).getByRole("button", { name: "Clear all" }),
      );
      await waitFor(() => {
        expect(router.state.location.search.filters).toEqual([]);
      });
      await waitFor(() => {
        expect(
          within(getToolbar()).queryByRole("button", {
            name: /Remove filter/i,
          }),
        ).toBeNull();
      });
      expect(await screen.findByText("Low CA")).toBeInTheDocument();
    });

    it("hosts the comparison select in the toolbar dataset slot", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
      usePlayerTableStore
        .getState()
        .addColumns("search", ["moneyball.average_rating"]);
      const { router } = renderSearchRoute();

      await screen.findByText("Moneyball Scout");
      const toolbar = screen.getByRole("toolbar", {
        name: "Player results toolbar",
      });
      const comparison = within(toolbar).getByRole("combobox", {
        name: "Percentiles vs",
      });
      expect(comparison).toHaveValue("own");
      expect(
        within(comparison)
          .getAllByRole("option")
          .map((option) => option.textContent),
      ).toEqual([
        "Own position",
        "Goalkeepers",
        "Centre-backs",
        "Full-backs",
        "Wing-backs",
        "Defensive midfielders",
        "Central midfielders",
        "Wide midfielders",
        "Attacking midfielders",
        "Wingers",
        "Strikers",
      ]);
      expect(
        within(toolbar).getByRole("button", { name: "Add filter" }),
      ).toBeInTheDocument();
      const header = screen.getByTestId("search-page-header");
      expect(
        within(header).getByRole("heading", {
          level: 1,
          name: "Player Search",
        }),
      ).toBeInTheDocument();
      for (const name of ["Add Tactic (Moneyball Score)", "Refresh stats"]) {
        expect(
          within(header).getByRole("button", { name }),
        ).toBeInTheDocument();
        expect(within(toolbar).queryByRole("button", { name })).toBeNull();
      }

      await user.selectOptions(comparison, "Centre-backs");
      await waitFor(() => {
        expect(getLastSearchPlayersArgs()).toMatchObject({
          comparison: "central_defender",
        });
        expect(router.state.location.search).toMatchObject({
          comparison: "central_defender",
        });
      });
      // Own position is the default and stays out of the URL.
      await user.selectOptions(
        within(
          screen.getByRole("toolbar", { name: "Player results toolbar" }),
        ).getByRole("combobox", { name: "Percentiles vs" }),
        "Own position",
      );
      await waitFor(() => {
        expect(router.state.location.search).not.toHaveProperty("comparison");
      });
    });

    it("saves the league adjustment choice and reloads the results it changes", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
      usePlayerTableStore
        .getState()
        .addColumns("search", ["moneyball_role.wbl_wbr_wing_back_ip"]);
      renderSearchRoute();

      await screen.findByText("Moneyball Scout");
      const scores = screen.getByRole("group", { name: "Scores" });
      expect(scores).toHaveAccessibleDescription(
        "League-adjusted scores show how a player would score in Premier Division.",
      );
      expect(
        within(scores).getByRole("button", { name: "League-adjusted" }),
      ).toHaveAttribute("aria-pressed", "true");
      const requestsBefore = getSearchPlayersCallCount();

      await user.click(within(scores).getByRole("button", { name: "Raw" }));

      await waitFor(() => {
        expect(getMoneyballLeagueAdjustmentChanges()).toEqual([false]);
        expect(screen.getByRole("button", { name: "Raw" })).toHaveAttribute(
          "aria-pressed",
          "true",
        );
        expect(getSearchPlayersCallCount()).toBeGreaterThan(requestsBefore);
      });
      expect(await screen.findByText("Moneyball Scout")).toBeInTheDocument();
    });

    it("offers the league choice only with season fields in use and only once leagues are measured", async () => {
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("Scout", 160)]);
      const { unmount } = renderSearchRoute();
      await screen.findByText("Scout");
      expect(screen.queryByRole("group", { name: "Scores" })).toBeNull();
      unmount();

      setMoneyballLeagueAdjustmentIpcMock({
        enabled: true,
        referenceDivision: null,
      });
      renderSearchRoute("/search?sort=moneyball.average_rating");
      const scores = await screen.findByRole("group", { name: "Scores" });
      expect(scores).toBeDisabled();
      expect(scores).toHaveAccessibleDescription(
        "League strength is measured by the next Load Data.",
      );
    });
  });
});
