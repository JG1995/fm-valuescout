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
import {
  DEFAULT_GRAPHICS_STATUS,
  setGraphicsStatusIpcMock,
} from "@/features/graphics/api/graphics-ipc-mock";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  playerShortlistSelectionKey,
  usePlayerShortlistSelectionStore,
} from "@/stores/use-player-shortlist-selection-store";
import {
  PLAYER_TABLE_LAYOUT_STORAGE_KEY,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import { useSearchViewMemory } from "@/stores/use-search-view-memory";
import {
  getLastRefreshSeasonStatsIpcArgs,
  resolvePendingRefreshSeasonStats,
  setRefreshSeasonStatsIpcMockPending,
  setRefreshSeasonStatsIpcMockResult,
} from "@/testing/moneyball-ipc-mock";
import {
  getPlannerTacticIpcMockCalls,
  getPlannerTacticOptionsIpcMockCalls,
  getPlannerTacticSaveIpcMockCalls,
  resolvePlannerTacticIpcMock,
  setPlannerTacticForContext,
} from "@/testing/planner-ipc-mock";
import {
  getLastChoosePlayerShortlistIpcArgs,
  getLastRemovePlayerShortlistIpcArgs,
  setPlayerShortlistChoiceIpcMockError,
  setPlayerShortlistRemoveIpcMockError,
} from "@/testing/player-shortlist-ipc-mock";
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

const { openFileDialog } = vi.hoisted(() => ({
  openFileDialog: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openFileDialog }));

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

/**
 * Holds every `list_saves` request until the returned release runs. Other
 * requests keep the shared IPC mock.
 */
function holdSaveListRequests(): () => void {
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
  const held: (() => void)[] = [];
  let holding = true;
  internals.invoke = (cmd, args, options) => {
    if (cmd !== "list_saves" || !holding) {
      return invoke(cmd, args, options);
    }
    return new Promise((resolve) => {
      held.push(() => resolve(invoke(cmd, args, options)));
    });
  };
  return () => {
    holding = false;
    for (const release of held) release();
  };
}

const TARGETS = { id: 1, name: "Targets", fileName: "Targets.fmf" };
const LOANS = { id: 2, name: "Loans", fileName: "Loans.fmf" };

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
    openFileDialog.mockReset();
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
    useSearchViewMemory.setState({ views: {} });
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
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
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

  it("atomically appends, interleaves, and re-compacts tactic groups", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderSearchRoute();

    await screen.findByRole("button", {
      name: "Add Tactic (Current)",
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add Tactic (Current)" }),
      ).toBeEnabled(),
    );
    const orderedLaneIds = ORDERED_TACTIC_LANE_IDS;
    const nonTacticIds = ["age", "nationality", "height", "ca", "pa", "value"];

    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...nonTacticIds,
      ...orderedLaneIds.map((laneId) => `tactic_current.${laneId}`),
    ]);
    expect(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
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
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
    );
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...nonTacticIds,
      ...orderedLaneIds.map((laneId) => `tactic_potential.${laneId}`),
    ]);
    expect(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
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
      name: "Add Tactic (Current)",
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add Tactic (Current)" }),
      ).toBeEnabled(),
    );
    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
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
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
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
    renderSearchRoute("/search", (queryClient) => {
      void queryClient.fetchQuery({
        queryKey: plannerKeys.tactic({
          saveId: 1,
          contextToken: "save-token-1",
        }),
        queryFn: () => new Promise<never>(() => undefined),
      });
    });

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
    ).toBeDisabled();
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

  it.each([
    {
      tableId: "search",
      entry: (sort: string) => `/search?sort=${sort}&dir=desc`,
    },
    {
      tableId: "moneyball-search",
      entry: (sort: string) => `/search?view=moneyball&sort=${sort}&dir=desc`,
    },
  ] as const)(
    "preserves identity-only after tactic-only leaf removal ($tableId)",
    async ({ tableId, entry }) => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      const currentIds = ORDERED_TACTIC_LANE_IDS.map(
        (laneId) => `tactic_current.${laneId}`,
      );
      usePlayerTableStore.getState().replaceLayout(tableId, currentIds);
      renderSearchRoute(entry(currentIds[0]));

      const table = await screen.findByRole("table", {
        name: "Player search results",
      });
      const label = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
      fireEvent.contextMenu(
        within(table).getByRole("columnheader", { name: label }),
      );
      await user.click(
        screen.getByRole("menuitem", { name: `Remove ${label}` }),
      );

      // Identity-only is valid: the emptied tactic group normalizes away
      // without rolling back to defaults.
      await waitFor(() => {
        expect(
          usePlayerTableStore.getState().layouts[tableId].columnIds,
        ).toEqual([]);
      });
      expect(
        await screen.findByRole("table", {
          name: "Player search results",
        }),
      ).toBeInTheDocument();
    },
  );

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
      name: "Add Tactic (Current)",
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

  it.each([
    {
      tableId: "search",
      entry: (sort: string) => `/search?sort=${sort}&dir=desc`,
      expectedSort: "ca",
      toggleName: "Add Tactic (Current)",
    },
    {
      tableId: "moneyball-search",
      entry: (sort: string) => `/search?view=moneyball&sort=${sort}&dir=desc`,
      expectedSort: "moneyball.average_rating",
      toggleName: "Add Tactic (Roles)",
    },
  ] as const)(
    "keeps identity-only when its tactic-only group is toggled off ($tableId)",
    async ({ tableId, entry, expectedSort, toggleName }) => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      const currentIds = ORDERED_TACTIC_LANE_IDS.map(
        (laneId) => `tactic_current.${laneId}`,
      );
      usePlayerTableStore.getState().replaceLayout(tableId, currentIds);
      const { router } = renderSearchRoute(entry(currentIds[0]));

      const currentToggle = await screen.findByRole("button", {
        name: toggleName,
      });
      await waitFor(() => expect(currentToggle).toBeEnabled());
      await user.click(currentToggle);

      await waitFor(() => {
        expect(
          usePlayerTableStore.getState().layouts[tableId].columnIds,
        ).toEqual([]);
      });
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({
          sort: expectedSort,
          dir: "desc",
        });
      });
    },
  );

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

  it("stores tactic toggles in the moneyball table layout without touching search", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderSearchRoute("/search?view=moneyball");

    const toggle = await screen.findByRole("button", {
      name: "Add Tactic (Roles)",
    });
    await waitFor(() => expect(toggle).toBeEnabled());
    await user.click(toggle);

    expect(
      usePlayerTableStore.getState().layouts["moneyball-search"].columnIds,
    ).toContain("tactic_current.goalkeeper");
    expect(
      usePlayerTableStore.getState().layouts.search.columnIds,
    ).not.toContain("tactic_current.goalkeeper");
  });

  describe("Moneyball custom tactic columns", () => {
    const rolesIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    const customIds = ORDERED_TACTIC_LANE_IDS.map(
      (laneId) => `tactic_custom.${laneId}`,
    );
    const moneyballColumnIds = () =>
      usePlayerTableStore.getState().layouts["moneyball-search"].columnIds;

    async function enabledToggle(name: string) {
      const toggle = await screen.findByRole("button", { name });
      await waitFor(() => expect(toggle).toBeEnabled());
      return toggle;
    }

    function storeTacticWithoutMetrics() {
      const tactic = resolvePlannerTacticIpcMock();
      for (const lane of tactic.lanes) {
        lane.customMetricKeys = [];
      }
      setPlannerTacticForContext(
        { saveId: 1, contextToken: "save-token-1" },
        tactic,
      );
    }

    it("offers Roles and Custom in place of Current and Potential", async () => {
      await resolveLoadDataIpcMock();
      renderSearchRoute("/search?view=moneyball");

      await enabledToggle("Add Tactic (Roles)");
      expect(
        screen.getByRole("button", { name: "Add Tactic (Custom)" }),
      ).toBeEnabled();
      expect(
        screen.queryByRole("button", { name: "Add Tactic (Current)" }),
      ).toBeNull();
      expect(screen.queryByRole("button", { name: /Potential/ })).toBeNull();
      expect(screen.queryByRole("link", { name: "Tactic editor" })).toBeNull();
    });

    it("adds and removes all 11 Custom columns, interleaved after Roles", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      renderSearchRoute("/search?view=moneyball");
      const nonTacticIds = moneyballColumnIds();

      const custom = await enabledToggle("Add Tactic (Custom)");
      await user.click(custom);
      expect(moneyballColumnIds()).toEqual([...nonTacticIds, ...customIds]);
      expect(custom).toHaveAttribute("aria-pressed", "true");

      await user.click(
        screen.getByRole("button", { name: "Add Tactic (Roles)" }),
      );
      expect(moneyballColumnIds()).toEqual([
        ...nonTacticIds,
        ...rolesIds.flatMap((rolesId, index) => [rolesId, customIds[index]]),
      ]);

      await user.click(custom);
      expect(moneyballColumnIds()).toEqual([...nonTacticIds, ...rolesIds]);
      expect(custom).toHaveAttribute("aria-pressed", "false");
    });

    it("names each Custom header by its slot and metrics in order", async () => {
      await resolveLoadDataIpcMock();
      usePlayerTableStore
        .getState()
        .replaceLayout("moneyball-search", ["age", ...customIds]);
      renderSearchRoute("/search?view=moneyball");

      const table = await screen.findByRole("table", {
        name: "Player search results",
      });
      const goalkeeper = await within(table).findByRole("columnheader", {
        name: "GK custom: Save %, Clean Sheets %",
      });
      expect(within(goalkeeper).getByText("GK")).toBeVisible();
      expect(within(goalkeeper).getByText("Custom")).toBeVisible();
      expect(
        within(table).getByRole("columnheader", {
          name: "DR custom: no metrics chosen",
        }),
      ).toBeInTheDocument();
    });

    it("cannot add Custom columns while no slot has metrics and links to the Tactic editor", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      storeTacticWithoutMetrics();
      const { router } = renderSearchRoute("/search?view=moneyball");
      const before = moneyballColumnIds();

      await enabledToggle("Add Tactic (Roles)");
      const custom = screen.getByRole("button", {
        name: "Add Tactic (Custom)",
      });
      expect(custom).toBeDisabled();
      await user.click(custom);
      expect(moneyballColumnIds()).toEqual(before);

      const link = screen.getByRole("link", { name: "Tactic editor" });
      expect(link).toHaveAttribute("href", "/my-club?view=tactic");
      await user.click(link);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe("/my-club");
      });
      expect(router.state.location.search).toMatchObject({ view: "tactic" });
    });

    it("still removes shown Custom columns while no slot has metrics", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      storeTacticWithoutMetrics();
      usePlayerTableStore
        .getState()
        .replaceLayout("moneyball-search", ["age", ...customIds]);
      renderSearchRoute("/search?view=moneyball");

      const custom = await enabledToggle("Add Tactic (Custom)");
      expect(custom).toHaveAttribute("aria-pressed", "true");
      expect(screen.queryByRole("link", { name: "Tactic editor" })).toBeNull();
      await user.click(custom);

      expect(moneyballColumnIds()).toEqual(["age"]);
      expect(custom).toBeDisabled();
      expect(
        screen.getByRole("link", { name: "Tactic editor" }),
      ).toBeInTheDocument();
    });

    it("falls back to the default sort for a stored Potential tactic sort", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              "moneyball-search": {
                columnIds: [
                  "age",
                  ...rolesIds.flatMap((rolesId) => [
                    rolesId,
                    rolesId.replace("tactic_current.", "tactic_potential."),
                  ]),
                ],
                widths: {},
              },
            },
          },
          version: 8,
        }),
      );
      await usePlayerTableStore.persist.rehydrate();
      expect(moneyballColumnIds()).toEqual(["age", ...rolesIds]);

      const { router } = renderSearchRoute(
        "/search?view=moneyball&sort=tactic_potential.goalkeeper",
      );

      await screen.findByRole("heading", { level: 1, name: "Moneyball" });
      expect(router.state.location.search).toMatchObject({
        sort: "moneyball.average_rating",
        dir: "desc",
      });
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

  it("falls back when a tactic sort exists only in another table layout", async () => {
    const tacticSort = "tactic_custom.goalkeeper";
    usePlayerTableStore
      .getState()
      .replaceLayout("moneyball-search", [tacticSort]);

    const { router } = renderSearchRoute(`/search?sort=${tacticSort}`);

    await screen.findByRole("heading", { level: 1, name: "Player Search" });
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
  });

  it("opens the opt-in Moneyball workspace with its own query view and pool", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Moneyball Scout", 160),
        dynamicValues: { "moneyball.average_rating": 7.25 },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    renderSearchRoute("/search?view=moneyball");

    const moneyballLink = await screen.findByRole("link", {
      name: "Moneyball",
    });
    expect(moneyballLink).toHaveAttribute("aria-current", "page");
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Refresh stats" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Upload Moneyball CSV" }),
    ).not.toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        comparisonPool: "filtered",
      });
    });
  });

  it("selects, requests, sorts, and renders Moneyball role columns without changing raw percentiles", async () => {
    await resolveLoadDataIpcMock();
    const wingBackIp = "moneyball_role.wbl_wbr_wing_back_ip";
    const fullBackIp = "moneyball_role.dl_dr_wing_back_ip";
    usePlayerTableStore
      .getState()
      .addColumns("moneyball-search", [wingBackIp, fullBackIp]);
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
    const { router } = renderSearchRoute("/search?view=moneyball");

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", {
        name: "Wing-Back (IP · DR/DL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("img", {
        name: "Moneyball role · Wing-Back (IP · WBR/WBL): 0, Weak",
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
        searchView: "moneyball",
        requestedFields: expect.arrayContaining([fullBackIp, wingBackIp]),
      });
    });

    await userEvent.setup().click(
      within(table).getByRole("button", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(router.state.location.search).toMatchObject({
      sort: wingBackIp,
      dir: "desc",
    });
  });

  it("applies a Moneyball role filter to the URL and comparison-pool request", async () => {
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
    const { router } = renderSearchRoute("/search?view=moneyball");

    await screen.findByText("Role-fit Scout");
    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const dialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "wing-back",
    );
    await user.click(
      within(dialog).getByRole("option", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(
      within(dialog).getByText(
        /Scored against the players matching your other filters/,
      ),
    ).toBeInTheDocument();
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "70" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
        filters: [
          expect.objectContaining({ field: roleField, op: "gte", value: 70 }),
        ],
      });
      expect(getLastSearchPlayersArgs()).toMatchObject({
        filters: [{ field: roleField, op: "gte", value: 70 }],
        requestedFields: expect.arrayContaining([roleField]),
      });
    });

    await user.click(screen.getByRole("button", { name: "All players" }));
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        comparisonPool: "fullCsv",
        filters: [{ field: roleField, op: "gte", value: 70 }],
      });
    });
  });

  it("uses the saved default only when Search has no explicit view", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);

    renderSearchRoute();

    expect(
      await screen.findByRole("link", {
        name: "Moneyball",
        current: "page",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        sortBy: "moneyball.average_rating",
        comparisonPool: "filtered",
      });
    });
  });

  it("keeps an explicit General Search view above the saved default", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });

    renderSearchRoute("/search?view=general");

    expect(
      await screen.findByRole("link", {
        name: "Search",
        current: "page",
      }),
    ).toBeInTheDocument();
  });

  it("makes General explicit when selected from an implicit Moneyball default", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    const { router } = renderSearchRoute();

    await user.click(await screen.findByRole("link", { name: "Search" }));

    await waitFor(() => {
      expect(router.state.location.search.view).toBe("general");
      expect(
        screen.getByRole("link", { name: "Search", current: "page" }),
      ).toBeInTheDocument();
    });
  });

  it("renders Search views without a local view tablist", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    const { router } = renderSearchRoute("/search?view=general");

    expect(
      await screen.findByRole("link", {
        name: "Search",
        current: "page",
      }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();

    await user.click(screen.getByRole("link", { name: "Moneyball" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
      });
    });
    expect(
      screen.getByRole("link", { name: "Moneyball", current: "page" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("button", {
        name: "Refresh stats",
      }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
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
          name: `Role · Goalkeeper (IP): ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
      expect(
        within(table).getByRole("img", {
          name: `Potential role · Goalkeeper (IP): ${score}, ${tier}`,
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
  });

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
    { view: "general" as const, entry: "/search?view=general" },
    { view: "moneyball" as const, entry: "/search?view=moneyball" },
  ])(
    "stacks fixed-height player identity without duplicate columns in $view Search",
    async ({ view, entry }) => {
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
      const { router } = renderSearchRoute(entry);

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
        expect(router.state.location.search).toEqual({ view });
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
      expect(router.state.location.search).toEqual({ view: "general" });
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
    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Recovered Search")).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBe(2);
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
      "potential role",
    );

    expect(
      within(dialog).getByRole("group", {
        name: "Potential role scores · Goalkeepers",
      }),
    ).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("option", {
        name: "Potential role · Goalkeeper (IP)",
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
        name: "Potential role · Goalkeeper (IP)",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: /Remove filter Potential role · Goalkeeper \(IP\) ≥ 50/i,
      }),
    );
    // The retained table remounts once the unfiltered page commits.
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());

    expect(
      await screen.findByRole("columnheader", {
        name: "Potential role · Goalkeeper (IP)",
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

  it("shows the filtered empty state for Moneyball when filters exclude every player", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([]);
    const encoded = encodeURIComponent(
      JSON.stringify([
        { field: "moneyball.average_rating", op: "gte", value: 7 },
      ]),
    );
    renderSearchRoute(`/search?view=moneyball&filters=${encoded}`);

    expect(
      await screen.findByText("No players match these filters"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("No Moneyball stats for this snapshot"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(/Adjust or clear filters in the toolbar above/),
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
    renderSearchRoute(`/search?view=moneyball&filters=${encoded}`);

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

  it("points an empty Moneyball cohort at the linked save and Refresh stats", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([]);
    renderSearchRoute("/search?view=moneyball");

    expect(
      await screen.findByText("No Moneyball stats for this snapshot"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Moneyball stats come from the FM save file linked to this save. Use Refresh stats to read it, or link a save file in Settings.",
      ),
    ).toBeInTheDocument();
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

  it("returns each view to its own filters and sort when navigation switches views", async () => {
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
      `/search?view=general&sort=pa&dir=asc&combine=or&filters=${encodedFilters}`,
    );
    await screen.findByText("High CA");
    const generalState = {
      view: "general",
      sort: "pa",
      dir: "asc",
      combine: "or",
      filters: [expect.objectContaining({ field: "ca" })],
    };
    expect(router.state.location.search).toMatchObject(generalState);

    // A first visit starts from the view's defaults; nothing carries across.
    await user.click(screen.getByRole("link", { name: "Moneyball" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
        sort: "moneyball.average_rating",
        dir: "desc",
        combine: "and",
        filters: [],
        comparisonPool: "filtered",
      });
    });

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    const moneyballDialog = screen.getByRole("form", { name: "Add filter" });
    await user.type(
      within(moneyballDialog).getByRole("combobox", { name: "Search fields" }),
      "average rating",
    );
    await user.click(
      within(moneyballDialog).getByRole("option", { name: "Average Rating" }),
    );
    fireEvent.change(within(moneyballDialog).getByLabelText("Value"), {
      target: { value: "7" },
    });
    await user.click(
      within(moneyballDialog).getByRole("button", { name: "Add" }),
    );
    await waitFor(() =>
      expect(router.state.location.search.filters).toHaveLength(1),
    );

    await user.click(screen.getByRole("link", { name: "Search" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject(generalState);
    });

    await user.click(screen.getByRole("link", { name: "Moneyball" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
        filters: [
          expect.objectContaining({
            field: "moneyball.average_rating",
            op: "gte",
            value: 7,
          }),
        ],
      });
    });
    expect(screen.queryByRole("tablist", { name: "Search view" })).toBeNull();
  });

  it("keeps Moneyball Name sort from direct URL rather than normalizing to Average Rating", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Name Sorted", 160)]);
    const { router } = renderSearchRoute(
      "/search?view=moneyball&sort=name&dir=asc",
    );

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
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
        name: /Role · Deep-Lying Playmaker \(IP\)/i,
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
    expect(screen.getByRole("link", { name: "Moneyball" })).toBeInTheDocument();
    expect(screen.queryByText("No shortlist yet")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Go to Moneyball" }),
    ).not.toBeInTheDocument();
    expect(usePlayerTableStore.getState().layouts).not.toHaveProperty(
      "shortlist",
    );
  });

  it("lists No shortlist and each shortlist, adds file names to shared names, and starts with none", async () => {
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [
      LOANS,
      { id: 3, name: "Targets", fileName: "First.fmf" },
      { id: 4, name: "Targets", fileName: "Second.fmf" },
      { id: 5, name: "", fileName: "Unnamed.fmf" },
      // Read from the linked FM save, so they have no file.
      { id: 6, name: "Targets", fileName: null },
      { id: 7, name: "(Default)", fileName: null },
    ]);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    renderSearchRoute("/search");

    const select = await screen.findByRole("combobox", { name: "Shortlist" });
    expect(
      within(select)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual([
      "No shortlist",
      "Loans",
      "Targets (First.fmf)",
      "Targets (Second.fmf)",
      "Unnamed.fmf",
      "Targets (FM save)",
      "(Default)",
    ]);
    expect(select).toHaveValue("");
    expect(
      screen.queryByRole("button", { name: "Remove shortlist" }),
    ).toBeNull();
    // A list read from the FM save has no file to name.
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "general",
        shortlistId: null,
      });
    });
  });

  it("offers no remove action for a shortlist read from the FM save", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [
      LOANS,
      { id: 7, name: "(Default)", fileName: null },
    ]);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    renderSearchRoute("/search");
    const select = await screen.findByRole("combobox", { name: "Shortlist" });

    await user.selectOptions(select, "(Default)");

    await waitFor(() => expect(select).toHaveValue("7"));
    expect(
      screen.queryByRole("button", { name: "Remove shortlist" }),
    ).toBeNull();

    await user.selectOptions(select, "Loans");

    expect(
      await screen.findByRole("button", { name: "Remove shortlist" }),
    ).toBeInTheDocument();
  });

  it("hides the shortlist dropdown when the save has no shortlists", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    renderSearchRoute("/search");

    await screen.findByText("General Scout");
    expect(screen.queryByRole("combobox", { name: "Shortlist" })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Add FM Shortlist" }),
    ).toBeEnabled();
  });

  it("restricts both views to the selected shortlist and starts each save context with none", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [LOANS, TARGETS]);
    setSearchPlayersOverride([playerNamed("General Scout", 160)]);
    const { router, queryClient } = renderSearchRoute("/search");

    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Shortlist" }),
      "Targets",
    );
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "general",
        shortlistId: 1,
      });
    });
    expect(router.state.location.search).not.toHaveProperty("shortlistOnly");

    // Moneyball keeps the selection, with the dropdown and remove beside
    // Percentiles vs and only page actions in the header.
    await user.click(screen.getByRole("link", { name: "Moneyball" }));
    const toolbar = await screen.findByRole("toolbar", {
      name: "Player results toolbar",
    });
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        shortlistId: 1,
      });
    });
    expect(
      within(toolbar).getByRole("combobox", { name: "Shortlist" }),
    ).toHaveValue("1");
    expect(
      within(toolbar).getByRole("button", { name: "Remove shortlist" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("group", { name: "Percentiles vs" }),
    ).toBeInTheDocument();
    const header = screen.getByTestId("search-page-header");
    expect(
      within(header).queryByRole("button", { name: "Remove shortlist" }),
    ).toBeNull();

    await user.selectOptions(
      within(toolbar).getByRole("combobox", { name: "Shortlist" }),
      "No shortlist",
    );
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        shortlistId: null,
      });
    });
    expect(storedShortlistSelection()).toBeUndefined();
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
    "opens %s as General with no shortlist selected and a clean URL",
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
        expect(getLastSearchPlayersArgs()).toMatchObject({
          searchView: "general",
          shortlistId: null,
        });
      });
    },
  );

  it("adds a chosen FM shortlist, selects it once the save lists it, and refreshes the save and General", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [LOANS]);
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    openFileDialog.mockResolvedValue("C:\\FM\\shortlists\\Targets.fmf");
    const { queryClient } = renderSearchRoute("/search");
    await screen.findByText("High CA");
    const invalidate = vi.spyOn(queryClient, "invalidateQueries");
    const releaseSaves = holdSaveListRequests();

    await user.click(screen.getByRole("button", { name: "Add FM Shortlist" }));

    // The selection is stored while the saves list still lacks the new
    // shortlist, and it survives until the refetched list contains it.
    await waitFor(() => expect(storedShortlistSelection()).toBe(3));
    expect(openFileDialog).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "FM shortlist", extensions: ["fmf"] }],
    });
    expect(getLastChoosePlayerShortlistIpcArgs()).toEqual({
      saveId: 1,
      contextToken: "save-token-1",
      path: "C:\\FM\\shortlists\\Targets.fmf",
    });

    act(() => releaseSaves());
    expect(
      await screen.findByText("Targets: 9 players, 7 in this snapshot."),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(screen.getByRole("combobox", { name: "Shortlist" })).toHaveValue(
        "3",
      ),
    );
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ shortlistId: 3 });
    });
    for (const queryKey of [snapshotKeys.saves(), searchKeys.playerPages()]) {
      expect(invalidate).toHaveBeenCalledWith({ queryKey });
    }
    expect(
      screen.getByRole("button", { name: "Remove shortlist" }),
    ).toBeInTheDocument();
  });

  it("clears the add confirmation when the current snapshot changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    openFileDialog.mockResolvedValue("C:\\FM\\shortlists\\Targets.fmf");
    renderSearchRoute("/search");
    await screen.findByText("High CA");

    await user.click(screen.getByRole("button", { name: "Add FM Shortlist" }));
    const confirmation = await screen.findByText(
      "Targets: 9 players, 7 in this snapshot.",
    );

    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() => expect(confirmation).not.toBeInTheDocument());
  });

  it("states why a chosen file was rejected and keeps every shortlist and the selection", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [TARGETS]);
    selectStoredShortlist(TARGETS.id);
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    openFileDialog.mockResolvedValue("C:\\FM\\views\\Scouting.fmf");
    setPlayerShortlistChoiceIpcMockError({
      kind: "not_player_shortlist",
      fileName: "Scouting.fmf",
    });
    renderSearchRoute("/search");
    await screen.findByText("High CA");

    await user.click(screen.getByRole("button", { name: "Add FM Shortlist" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Scouting.fmf is not a player shortlist.",
    );
    const select = screen.getByRole("combobox", { name: "Shortlist" });
    expect(select).toHaveValue("1");
    expect(
      within(select)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["No shortlist", "Targets"]);
  });

  it("does nothing when the shortlist picker is cancelled", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    openFileDialog.mockResolvedValue(null);
    renderSearchRoute("/search");
    await screen.findByText("High CA");

    await user.click(screen.getByRole("button", { name: "Add FM Shortlist" }));

    await waitFor(() => expect(openFileDialog).toHaveBeenCalled());
    expect(getLastChoosePlayerShortlistIpcArgs()).toBeUndefined();
    expect(screen.queryByRole("combobox", { name: "Shortlist" })).toBeNull();
  });

  it("removes the selected shortlist, selects none, clears the add confirmation, and refreshes the save and General", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [LOANS]);
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    openFileDialog.mockResolvedValue("C:\\FM\\shortlists\\Targets.fmf");
    const { queryClient } = renderSearchRoute("/search");
    await screen.findByText("High CA");
    await user.click(screen.getByRole("button", { name: "Add FM Shortlist" }));
    const confirmation = await screen.findByText(
      "Targets: 9 players, 7 in this snapshot.",
    );
    const shortlistSelect = () =>
      screen.getByRole("combobox", { name: "Shortlist" });
    await waitFor(() => expect(shortlistSelect()).toHaveValue("3"));
    const invalidate = vi.spyOn(queryClient, "invalidateQueries");

    await user.click(screen.getByRole("button", { name: "Remove shortlist" }));

    expect(getLastRemovePlayerShortlistIpcArgs()).toEqual({
      saveId: 1,
      contextToken: "save-token-1",
      shortlistId: 3,
    });
    await waitFor(() =>
      expect(
        within(shortlistSelect())
          .getAllByRole("option")
          .map((option) => option.textContent),
      ).toEqual(["No shortlist", "Loans"]),
    );
    expect(shortlistSelect()).toHaveValue("");
    expect(storedShortlistSelection()).toBeUndefined();
    expect(confirmation).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Remove shortlist" }),
    ).toBeNull();
    for (const queryKey of [snapshotKeys.saves(), searchKeys.playerPages()]) {
      expect(invalidate).toHaveBeenCalledWith({ queryKey });
    }

    // Removing the last shortlist hides the dropdown.
    await user.selectOptions(shortlistSelect(), "Loans");
    await user.click(
      await screen.findByRole("button", { name: "Remove shortlist" }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("combobox", { name: "Shortlist" })).toBeNull(),
    );
  });

  it("states that a shortlist could not be removed and keeps it selected", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSavePlayerShortlistsIpcMock(1, [TARGETS]);
    selectStoredShortlist(TARGETS.id);
    setPlayerShortlistRemoveIpcMockError({ kind: "database", fileName: null });
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    renderSearchRoute("/search");
    await screen.findByText("High CA");

    await user.click(
      await screen.findByRole("button", { name: "Remove shortlist" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The shortlist could not be removed. Try again.",
    );
    expect(screen.getByRole("combobox", { name: "Shortlist" })).toHaveValue(
      "1",
    );
    expect(storedShortlistSelection()).toBe(TARGETS.id);
  });

  it("composes the selected shortlist with a General filter in one fetch", async () => {
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
        searchView: "general",
        shortlistId: 1,
        filters: [expect.objectContaining({ field: "ca" })],
      });
    });
  });

  it.each([
    {
      state: "no shortlisted player in the General snapshot",
      route: "/search?view=general",
      filters: "",
      title: "No shortlisted player in this snapshot",
      body: "None of the shortlist's players are in this snapshot. Select No shortlist to browse every snapshot player.",
    },
    {
      state: "no shortlisted player with season stats",
      route: "/search?view=moneyball",
      filters: "",
      title: "No shortlisted player has season stats",
      body: "None of the shortlist's players have season stats in this snapshot. Select No shortlist to compare every player with season stats.",
    },
    {
      state: "no shortlisted player matching the filters",
      route: "/search?view=moneyball",
      filters: `&filters=${encodeURIComponent(
        JSON.stringify([{ field: "ca", op: "gte", value: 150 }]),
      )}`,
      title: "No shortlisted players match these filters",
      body: "Adjust or clear filters in the toolbar above, or select No shortlist to widen the result set.",
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

  it("refreshes Moneyball stats for the save context and reloads their as-of date", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
    setRefreshSeasonStatsIpcMockResult({
      status: "attached",
      matchedPlayers: 1_200,
      unmatchedPlayers: 3,
      gameDate: "2026-06-30",
      shortlists: { status: "notRead" },
      elapsedMs: 40,
    });
    const { queryClient } = renderSearchRoute("/search?view=moneyball");
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

  it("keeps Refresh stats unavailable while a refresh or Load Data runs", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
    renderSearchRoute("/search?view=moneyball");
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

  it("keeps Add FM Shortlist unavailable without a current snapshot", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    const { queryClient } = renderSearchRoute("/search");
    await screen.findByText("High CA");
    expect(
      screen.getByRole("button", { name: "Add FM Shortlist" }),
    ).toBeEnabled();

    await act(async () => {
      queryClient.setQueryData(currentSnapshotQueryOptions.queryKey, null);
    });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add FM Shortlist" }),
      ).toBeDisabled(),
    );
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

    it("keeps Add Tactic and Add FM Shortlist outside the generic toolbar", async () => {
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("High CA", 180)]);
      renderSearchRoute("/search?view=general");

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
        "Add Tactic (Current)",
        "Add Tactic (Potential)",
        "Add FM Shortlist",
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

    it("hosts the comparison-pool toggle in the toolbar dataset slot", async () => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);
      renderSearchRoute("/search?view=moneyball");

      await screen.findByText("Moneyball Scout");
      const toolbar = screen.getByRole("toolbar", {
        name: "Player results toolbar",
      });
      expect(
        within(toolbar).getByRole("button", { name: "Filtered cohort" }),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("button", { name: "All players" }),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("group", { name: "Percentiles vs" }),
      ).toBeInTheDocument();
      expect(
        within(toolbar).getByRole("button", { name: "Add filter" }),
      ).toBeInTheDocument();
      const header = screen.getByTestId("search-page-header");
      expect(
        within(header).getByRole("heading", {
          level: 1,
          name: "Moneyball",
        }),
      ).toBeInTheDocument();
      for (const name of [
        "Add Tactic (Roles)",
        "Add Tactic (Custom)",
        "Refresh stats",
      ]) {
        expect(
          within(header).getByRole("button", { name }),
        ).toBeInTheDocument();
        expect(within(toolbar).queryByRole("button", { name })).toBeNull();
      }

      await user.click(
        within(toolbar).getByRole("button", { name: "All players" }),
      );
      await waitFor(() => {
        expect(getLastSearchPlayersArgs()).toMatchObject({
          comparisonPool: "fullCsv",
        });
      });
    });
  });
});
