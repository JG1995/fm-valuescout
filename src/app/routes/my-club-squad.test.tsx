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
import { describe, expect, it, vi } from "vitest";
import type { RouterContext } from "@/app/router-context";
import type { MyClubWorkspace } from "@/app/routes/my-club";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import {
  DEFAULT_GRAPHICS_STATUS,
  getGraphicsIpcMockCalls,
  getPendingGraphicsResultIpcMockCount,
  resolveAllPendingGraphicsResultsIpcMock,
  setGraphicsResultIpcMockForCall,
  setGraphicsResultIpcMockMode,
  setGraphicsStatusIpcMock,
} from "@/features/graphics/api/graphics-ipc-mock";
import { graphicsStatusQueryOptions } from "@/features/graphics/api/graphics-query-options";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
import type { PlannerRoleReference } from "@/features/planner/types/role-reference";
import type { PlannerTactic } from "@/features/planner/types/tactic";
import { validateTacticDraft } from "@/features/planner/utils/tactic-editor";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SaveSummary } from "@/features/snapshot/types/save";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { squadKeys } from "@/features/squad/api/squad-keys";
import type { SquadPlayer } from "@/features/squad/types/squad-player";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { routeTree } from "@/routeTree.gen";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  rejectBusyClubDnaRemoveRequest,
  resolveBusyClubDnaSetRequest,
  setClubDnaGetIpcMockMode,
  setClubDnaIpcMockDefinition,
  setClubDnaRemoveIpcMockMode,
  setClubDnaSetIpcMockMode,
} from "@/testing/club-dna-ipc-mock";
import {
  getLastCsvImportIpcArgs,
  setCsvImportIpcMockResult,
} from "@/testing/csv-import-ipc-mock";
import {
  getLastManagedClubSaveArgs,
  getPlannerClearAllIpcMockCalls,
  getPlannerDepthIpcMockCalls,
  getPlannerOptimizeIpcMockBases,
  getPlannerOptimizeIpcMockCalls,
  getPlannerRoleReferenceCalls,
  getPlannerSlotCandidateFetchCount,
  getPlannerTacticIpcMockCalls,
  getPlannerTacticOptionsIpcMockCalls,
  getPlannerTacticSaveIpcMockCalls,
  getPlannerTeamSaveIpcMockCalls,
  observeManagedClubSaveCall,
  resolvePendingManagedClubSave,
  resolvePendingPlannerTeamRemovalImpact,
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
  resolveSavePlannerClubFamilyIpcMock,
  setManagedClubIpcMock,
  setManagedClubOptionsError,
  setManagedClubSavePending,
  setPlannerAssignmentError,
  setPlannerAvailableClubs,
  setPlannerClearAllError,
  setPlannerClearAllPending,
  setPlannerDepthIpcMock,
  setPlannerOptimizeDepth,
  setPlannerOptimizeError,
  setPlannerOptimizePending,
  setPlannerRoleReference,
  setPlannerRoleReferenceError,
  setPlannerSlotCandidates,
  setPlannerTacticForContext,
  setPlannerTacticIpcMock,
  setPlannerTacticSaveError,
  setPlannerTeamRemovalImpactPending,
  setPlannerTeamRemovalImpacts,
  setPlannerTeamSaveError,
  setPlannerTeamSavePending,
} from "@/testing/planner-ipc-mock";
import {
  resolveCreateSaveIpcMock,
  resolveGetCurrentSnapshotIpcMock,
  resolveLoadDataIpcMock,
} from "@/testing/snapshot-ipc-mock";
import {
  getLastSquadCurrentAbilityBoostProgress,
  getLastSquadPlayersArgs,
  getLastSquadWonderkidMentalityBoostProgress,
  getSquadCurrentAbilityBoostIpcMockCalls,
  getSquadPlayersCallCount,
  getSquadWonderkidMentalityBoostIpcMockCalls,
  rejectPendingSquadPlayersPageIpcMock,
  resolvePendingSquadCurrentAbilityBoostIpcMock,
  resolvePendingSquadPlayersPageIpcMock,
  resolvePendingSquadWonderkidMentalityBoostIpcMock,
  sendPendingSquadCurrentAbilityBoostProgressIpcMock,
  sendPendingSquadWonderkidMentalityBoostProgressIpcMock,
  setSquadCurrentAbilityBoostIpcMockMode,
  setSquadPlayersOverride,
  setSquadPlayersPageIpcMockMode,
  setSquadWonderkidMentalityBoostIpcMockMode,
} from "@/testing/squad-ipc-mock";

const { openCsvDialog } = vi.hoisted(() => ({ openCsvDialog: vi.fn() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openCsvDialog }));

function renderMyClubRoute({
  staleTime = 0,
  initialEntry = "/my-club?view=planner",
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime } },
  }),
}: {
  staleTime?: number;
  initialEntry?: string;
  queryClient?: QueryClient;
} = {}) {
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

async function openMyClubWorkspace(
  user: ReturnType<typeof userEvent.setup>,
  workspace: MyClubWorkspace,
) {
  const labels: Record<MyClubWorkspace, string> = {
    squad: "Squad",
    planner: "Planner",
    tactic: "Tactic",
  };
  const navigation = screen.getByRole("navigation", { name: "Primary" });
  await user.click(
    within(navigation).getByRole("link", { name: labels[workspace] }),
  );
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
    currentClubUid: null,
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
    suggestedTraining: null,
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
          displayName: `${team.strings.length + 1}${
            team.strings.length + 1 === 2 ? "nd" : "th"
          } string`,
          assignments: [],
        },
      ],
    })),
  };
}

function mockScrollerScrollTo(scroller: HTMLElement) {
  Object.defineProperty(scroller, "scrollTo", {
    configurable: true,
    value: (options: { top?: number }) => {
      scroller.scrollTop = options.top ?? scroller.scrollTop;
    },
  });
}

const CLUB_DNA_CONTEXT = { saveId: 1, contextToken: "save-token-1" };
const SECOND_SAVE: SaveSummary = {
  id: 2,
  contextToken: "save-token-2",
  name: "Second save",
  isActive: true,
  createdAtUtc: "2026-07-28T16:00:00.000Z",
  updatedAtUtc: "2026-07-28T16:00:00.000Z",
};

function savesFor(activeSaveId: number): SaveSummary[] {
  return [
    {
      id: CLUB_DNA_CONTEXT.saveId,
      contextToken: CLUB_DNA_CONTEXT.contextToken,
      name: "Default save",
      isActive: activeSaveId === 1,
      createdAtUtc: "2026-07-28T12:00:00.000Z",
      updatedAtUtc: "2026-07-28T12:00:00.000Z",
    },
    ...(activeSaveId === SECOND_SAVE.id ? [SECOND_SAVE] : []),
  ];
}

function switchToSecondSave(
  queryClient: QueryClient,
  updateSnapshot: (snapshot: SnapshotSummary) => SnapshotSummary = (
    snapshot,
  ) => ({
    ...snapshot,
    saveId: SECOND_SAVE.id,
  }),
) {
  const snapshot = queryClient.getQueryData<SnapshotSummary>(
    snapshotKeys.current(),
  );
  if (!snapshot) {
    throw new Error("Expected a current snapshot in the planner query");
  }
  const created = resolveCreateSaveIpcMock({ name: SECOND_SAVE.name });
  if (
    created.id !== SECOND_SAVE.id ||
    created.contextToken !== SECOND_SAVE.contextToken
  ) {
    throw new Error("Expected the second save context");
  }
  queryClient.setQueryData(
    savesQueryOptions.queryKey,
    savesFor(SECOND_SAVE.id),
  );
  queryClient.setQueryData<SnapshotSummary | null>(
    snapshotKeys.current(),
    updateSnapshot(snapshot),
  );
}

describe("My Club route", () => {
  it("requests exact UID graphics in visible Squad rows and keeps null-club fallbacks", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setGraphicsStatusIpcMock({
      generation: 11,
      selected: true,
      candidate: { available: true, source: "documents" },
      summary: {
        configs: 1,
        mappings: 2,
        truncated: false,
        diagnostics: {
          configLimit: 10000,
          entryLimit: 1000000,
          depthLimit: 32,
          mappingLimit: 500000,
          configTooLarge: 0,
          configUnreadable: 0,
          malformedConfig: 0,
          invalidMapping: 0,
          sourceUnreadable: 0,
        },
      },
    });
    setGraphicsResultIpcMockForCall("personPortrait", 42, {
      status: "available",
      mime: "image/png",
      bytes: [137, 80, 78, 71],
    });
    setGraphicsResultIpcMockForCall("clubLogo", 7, {
      status: "available",
      mime: "image/png",
      bytes: [137, 80, 78, 71],
    });
    setSquadPlayersOverride([
      { ...squadPlayerNamed("Exact Squad", 42), currentClubUid: 7 },
      { ...squadPlayerNamed("Name only Squad", 43), currentClubUid: null },
    ]);
    setGraphicsResultIpcMockMode("pending");
    renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    const row = within(table).getByText("Exact Squad").closest("tr");
    const legacy = within(table).getByText("Name only Squad").closest("tr");
    if (!row || !legacy) throw new Error("Expected Squad graphics rows");
    await waitFor(() =>
      expect(getGraphicsIpcMockCalls().length).toBeGreaterThanOrEqual(2),
    );
    setGraphicsResultIpcMockMode("available");
    resolveAllPendingGraphicsResultsIpcMock();
    await waitFor(() => expect(row.querySelectorAll("img")).toHaveLength(2));
    expect(legacy.querySelectorAll("img")).toHaveLength(0);
    expect(row).toHaveStyle({ height: "40px" });
    expect(row).toHaveAttribute("data-index");
    expect(getGraphicsIpcMockCalls()).toEqual(
      expect.arrayContaining([
        { kind: "personPortrait", uid: 42 },
        { kind: "clubLogo", uid: 7 },
      ]),
    );
    expect(getGraphicsIpcMockCalls()).not.toContainEqual({
      kind: "clubLogo",
      uid: 43,
    });

    setGraphicsResultIpcMockMode("error");
  });
  it("renders the selected managed-club logo by exact UID", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 42,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 7,
      selected: true,
      candidate: { available: true, source: "documents" },
    });
    setGraphicsResultIpcMockForCall("clubLogo", 42, {
      status: "available",
      mime: "image/png",
      bytes: [137, 80, 78, 71],
    });

    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(await screen.findByText("Managed club: Barcelona")).toBeVisible();
    await waitFor(() =>
      expect(getGraphicsIpcMockCalls()).toContainEqual({
        kind: "clubLogo",
        uid: 42,
      }),
    );
    expect(
      screen
        .getByText("Managed club: Barcelona")
        .parentElement?.querySelector("img"),
    ).toBeTruthy();
  });

  it("does not look up a persisted UID when managed club status is missing", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 42,
      status: "missing",
      unclassifiedPlayerCount: 0,
    });
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 8,
      selected: true,
    });
    setGraphicsResultIpcMockForCall("clubLogo", 42, {
      status: "available",
      mime: "image/png",
      bytes: [137, 80, 78, 71],
    });

    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    const label = await screen.findByText("Managed club: Barcelona");
    await waitFor(() => {
      expect(
        queryClient.getQueryState(graphicsStatusQueryOptions.queryKey),
      ).toMatchObject({
        status: "success",
        data: expect.objectContaining({ selected: true }),
      });
    });
    expect(getGraphicsIpcMockCalls()).not.toContainEqual({
      kind: "clubLogo",
      uid: 42,
    });
    expect(label.parentElement?.querySelector("img")).toBeNull();
    expect(
      label.parentElement?.querySelector('svg[aria-hidden="true"]'),
    ).toBeTruthy();
  });

  it.each(["pending", "missing", "error"] as const)(
    "keeps the managed-club shield fallback for %s graphics",
    async (mode) => {
      await resolveLoadDataIpcMock();
      setManagedClubIpcMock({
        clubName: "Barcelona",
        clubUid: 42,
        status: "available",
        unclassifiedPlayerCount: 0,
      });
      setGraphicsStatusIpcMock({
        ...DEFAULT_GRAPHICS_STATUS,
        generation: 8,
        selected: true,
      });
      setGraphicsResultIpcMockForCall("clubLogo", 42, { status: "missing" });
      setGraphicsResultIpcMockMode(mode);
      renderMyClubRoute({ initialEntry: "/my-club" });

      const label = await screen.findByText("Managed club: Barcelona");
      await waitFor(() =>
        expect(label.parentElement?.querySelector("img")).toBeNull(),
      );
      expect(
        label.parentElement?.querySelector('svg[aria-hidden="true"]'),
      ).toBeTruthy();
      setGraphicsResultIpcMockMode("error");
    },
  );

  it("does not look up a logo for a legacy name-only selection", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: null,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 9,
      selected: true,
    });
    renderMyClubRoute({ initialEntry: "/my-club" });

    const label = await screen.findByText("Managed club: Barcelona");
    await waitFor(() =>
      expect(label.parentElement?.querySelector("img")).toBeNull(),
    );
    expect(getGraphicsIpcMockCalls()).not.toContainEqual({
      kind: "clubLogo",
      uid: 0,
    });
    expect(
      label.parentElement?.querySelector('svg[aria-hidden="true"]'),
    ).toBeTruthy();
  });

  it.each(["pending", "missing", "error"] as const)(
    "keeps Squad marks and navigation geometry for %s graphics",
    async (mode) => {
      await resolveLoadDataIpcMock();
      resolveSavePlannerClubFamilyIpcMock({
        primaryClub: { clubName: "Metro FC", clubUid: 1 },
        sources: [],
      });
      setGraphicsStatusIpcMock({
        generation: 12,
        selected: true,
        candidate: { available: true, source: "documents" },
        summary: {
          configs: 1,
          mappings: 1,
          truncated: false,
          diagnostics: {
            configLimit: 10000,
            entryLimit: 1000000,
            depthLimit: 32,
            mappingLimit: 500000,
            configTooLarge: 0,
            configUnreadable: 0,
            malformedConfig: 0,
            invalidMapping: 0,
            sourceUnreadable: 0,
          },
        },
      });
      setGraphicsResultIpcMockMode(mode);
      setSquadPlayersOverride([
        { ...squadPlayerNamed("Stateful Squad", 51), currentClubUid: 12 },
      ]);
      const { router } = renderMyClubRoute({ initialEntry: "/my-club" });
      const table = await screen.findByRole("table", {
        name: "Squad overview",
      });
      const row = within(table).getByText("Stateful Squad").closest("tr");
      if (!row) throw new Error("Expected Squad graphics row");
      expect(row).toHaveStyle({ height: "40px" });
      expect(within(row).getByText("Stateful Squad")).toBeVisible();
      expect(row).toHaveAttribute("tabindex", "0");
      fireEvent.click(row);
      await waitFor(() =>
        expect(router.state.location.pathname).toBe("/players/51"),
      );
      if (mode === "pending") {
        setGraphicsResultIpcMockMode("missing");
        resolveAllPendingGraphicsResultsIpcMock();
        await waitFor(() =>
          expect(getPendingGraphicsResultIpcMockCount()).toBe(0),
        );
      }
    },
  );

  it("bounds Squad graphics requests to rendered virtual rows", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 13,
      selected: true,
    });
    const players = manySquadPlayers(200).map((player) => ({
      ...player,
      currentClubUid: player.uid + 1000,
    }));
    setSquadPlayersOverride(players);
    setGraphicsResultIpcMockMode("pending");
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });
    const table = await screen.findByRole("table", { name: "Squad overview" });
    await waitFor(() =>
      expect(getGraphicsIpcMockCalls().length).toBeGreaterThan(0),
    );
    const calls = getGraphicsIpcMockCalls() as Array<{
      kind: string;
      uid: number;
    }>;
    expect(calls.length).toBeLessThan(players.length);
    expect(calls.length).toBeLessThan(100);
    expect(
      calls.every((call) =>
        players.some(
          (player) =>
            call.uid === player.uid || call.uid === player.currentClubUid,
        ),
      ),
    ).toBe(true);
    expect(table.querySelectorAll("tr[data-index]").length).toBeGreaterThan(0);
    setGraphicsResultIpcMockMode("missing");
    resolveAllPendingGraphicsResultsIpcMock();
    await waitFor(() => expect(getPendingGraphicsResultIpcMockCount()).toBe(0));
    queryClient.clear();
  });

  it("selects each Club workspace from navigation with no local tabs", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const user = userEvent.setup();
    const { history, router } = renderMyClubRoute({
      initialEntry: "/my-club?view=squad&squadSort=name&squadDir=asc",
    });

    const navigation = await screen.findByRole("navigation", {
      name: "Primary",
    });
    const workspaceLink = (name: string) =>
      within(navigation).getByRole("link", { name });
    expect(workspaceLink("Squad")).toHaveAttribute("aria-current", "page");
    expect(workspaceLink("Planner")).not.toHaveAttribute("aria-current");
    expect(workspaceLink("Tactic")).not.toHaveAttribute("aria-current");
    expect(
      screen.queryByRole("tablist", { name: "My Club workspaces" }),
    ).toBeNull();
    expect(screen.queryByRole("tab", { name: "Squad" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Planner" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Tactic" })).toBeNull();
    expect(document.querySelector('[id^="my-club-workspace-"]')).toBeNull();
    expect(
      document.querySelector('[aria-labelledby^="my-club-workspace-"]'),
    ).toBeNull();
    const squadTable = await screen.findByRole("table", {
      name: "Squad overview",
    });
    expect(squadTable).toBeVisible();
    expect(
      within(squadTable).getByRole("columnheader", { name: "Player" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        name: "Squad depth",
        hidden: true,
      }),
    ).not.toBeVisible();
    expect(
      screen.getByRole("region", {
        name: "Tactic controls",
        hidden: true,
      }),
    ).not.toBeVisible();

    await user.click(workspaceLink("Planner"));
    await waitFor(() =>
      expect(router.state.location.search).toEqual({
        view: "planner",
        squadSort: "name",
        squadDir: "asc",
      }),
    );
    expect(workspaceLink("Planner")).toHaveAttribute("aria-current", "page");
    expect(workspaceLink("Squad")).not.toHaveAttribute("aria-current");
    expect(
      await screen.findByRole("heading", { name: "Squad depth" }),
    ).toBeVisible();
    expect(squadTable).not.toBeVisible();

    await user.click(workspaceLink("Tactic"));
    await waitFor(() =>
      expect(router.state.location.search).toEqual({
        view: "tactic",
        squadSort: "name",
        squadDir: "asc",
      }),
    );
    expect(workspaceLink("Tactic")).toHaveAttribute("aria-current", "page");
    expect(workspaceLink("Planner")).not.toHaveAttribute("aria-current");
    expect(
      await screen.findByRole("region", { name: "Tactic controls" }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", {
        name: "Squad depth",
        hidden: true,
      }),
    ).not.toBeVisible();

    await user.click(workspaceLink("Squad"));
    await waitFor(() =>
      expect(router.state.location.search).toEqual({
        view: "squad",
        squadSort: "name",
        squadDir: "asc",
      }),
    );
    expect(workspaceLink("Squad")).toHaveAttribute("aria-current", "page");
    expect(workspaceLink("Tactic")).not.toHaveAttribute("aria-current");
    expect(history.canGoBack()).toBe(true);
    expect(
      within(squadTable).getByRole("columnheader", { name: "Player" }),
    ).toBeInTheDocument();
  });

  it.each([
    ["null", null],
    [
      "owned by another save",
      { saveId: SECOND_SAVE.id } as Partial<SnapshotSummary>,
    ],
  ])(
    "does not mount tactic IPC when the snapshot is %s",
    async (_description, snapshotOverride) => {
      await resolveLoadDataIpcMock();
      const currentSnapshot = resolveGetCurrentSnapshotIpcMock();
      if (!currentSnapshot) {
        throw new Error("Expected a current snapshot fixture");
      }
      const queryClient = new QueryClient({
        defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
      });
      queryClient.setQueryData(savesQueryOptions.queryKey, savesFor(1));
      queryClient.setQueryData(
        snapshotKeys.current(),
        snapshotOverride === null
          ? null
          : { ...currentSnapshot, ...snapshotOverride },
      );

      renderMyClubRoute({ queryClient });

      if (snapshotOverride === null) {
        expect(
          await screen.findByText("No data loaded for this save"),
        ).toBeInTheDocument();
      } else {
        const navigation = await screen.findByRole("navigation", {
          name: "Primary",
        });
        expect(
          within(navigation).getByRole("link", { name: "Planner" }),
        ).toBeInTheDocument();
      }
      expect(getPlannerTacticIpcMockCalls()).toEqual([]);
      expect(getPlannerTacticOptionsIpcMockCalls()).toEqual([]);
      expect(getPlannerTacticSaveIpcMockCalls()).toEqual([]);
    },
  );

  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByRole("heading", { level: 1, name: "My Club" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("selects one managed club and invalidates membership consumers", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 42 }]);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });
    queryClient.setQueryData(staffKeys.all, []);
    const searchPage = searchKeys.players(0, 50);
    const squadPage = squadKeys.players(0, 50);
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    let mutationWasVisible = false;
    observeManagedClubSaveCall(() => {
      expect(queryClient.getQueryData(searchPage)).toBeUndefined();
      expect(queryClient.getQueryData(squadPage)).toBeUndefined();
      mutationWasVisible =
        queryClient.isMutating({
          mutationKey: playerResultContextMutationKey,
        }) > 0;
    });

    const managedClub = await screen.findByRole("combobox", {
      name: "Managed club",
    });
    await user.type(managedClub, "Bar");
    await user.click(screen.getByRole("option", { name: "Barcelona" }));
    await user.click(screen.getByRole("button", { name: "Save managed club" }));

    await waitFor(() => {
      expect(queryClient.getQueryState(staffKeys.all)?.isInvalidated).toBe(
        true,
      );
      expect(mutationWasVisible).toBe(true);
      expect(getLastManagedClubSaveArgs()).toEqual({
        clubName: "Barcelona",
        clubUid: 42,
      });
    });
  });

  it("disables Club DNA until a managed club is selected", async () => {
    await resolveLoadDataIpcMock();
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByRole("button", { name: "Define DNA" }),
    ).toBeDisabled();
  });

  it("keeps a disabled Define DNA placeholder while saves initially load", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    let resolveSaves!: (saves: SaveSummary[]) => void;
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: 0 } },
    });
    const pendingSaves = queryClient.fetchQuery({
      ...savesQueryOptions,
      queryFn: () =>
        new Promise<SaveSummary[]>((resolve) => {
          resolveSaves = resolve;
        }),
    });

    renderMyClubRoute({ initialEntry: "/my-club", queryClient });

    expect(
      await screen.findByRole("button", { name: "Define DNA" }),
    ).toBeDisabled();
    expect(
      screen.queryByRole("dialog", { name: "Define Club DNA" }),
    ).toBeNull();

    resolveSaves(savesFor(CLUB_DNA_CONTEXT.saveId));
    await pendingSaves;
  });

  it("keeps a disabled Define DNA placeholder after an initial saves error", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, refetchOnMount: false, staleTime: 0 },
      },
    });
    const savesError = queryClient.fetchQuery({
      ...savesQueryOptions,
      queryFn: () => Promise.reject(new Error("Could not load saves")),
    });
    await expect(savesError).rejects.toThrow("Could not load saves");

    renderMyClubRoute({ initialEntry: "/my-club", queryClient });

    expect(
      await screen.findByRole("button", { name: "Define DNA" }),
    ).toBeDisabled();
    expect(
      screen.queryByRole("dialog", { name: "Define Club DNA" }),
    ).toBeNull();
  });

  it("keeps the visible Define DNA action disabled after a Club DNA query error", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setClubDnaGetIpcMockMode("error");
    renderMyClubRoute({ initialEntry: "/my-club" });

    const trigger = await screen.findByRole("button", { name: "Define DNA" });
    await waitFor(() => expect(trigger).toBeDisabled());
    expect(
      screen.queryByRole("dialog", { name: "Define Club DNA" }),
    ).toBeNull();
  });

  it("places Club DNA beside the managed-club save and appends it on creation", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const { queryClient, router } = renderMyClubRoute({
      initialEntry: "/my-club?squadSort=club_dna&squadDir=asc",
    });

    const controls = await screen.findByRole("group", {
      name: "Managed club controls",
    });
    const saveButton = within(controls).getByRole("button", {
      name: "Save managed club",
    });
    const defineButton = within(controls).getByRole("button", {
      name: "Define DNA",
    });
    expect(saveButton.compareDocumentPosition(defineButton)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    // The disabled placeholder swaps for the definition trigger once saves
    // settle, so re-query inside the wait instead of holding a stale node.
    await waitFor(() =>
      expect(
        within(controls).getByRole("button", { name: "Define DNA" }),
      ).toBeEnabled(),
    );

    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");
    await user.click(
      within(controls).getByRole("button", { name: "Define DNA" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Acceleration" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save Club DNA" }),
    );

    await waitFor(() => {
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
        "club_dna",
      );
      expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
        "club_dna",
      );
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: clubDnaKeys.definition(CLUB_DNA_CONTEXT),
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: squadKeys.all,
      });
    });

    const store = usePlayerTableStore.getState();
    invalidateQueries.mockClear();
    store.removeColumn("search", "club_dna");
    store.removeColumn("squad", "club_dna");
    await user.click(
      within(controls).getByRole("button", { name: "Define DNA" }),
    );
    const editDialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(editDialog).getByRole("button", { name: "Save Club DNA" }),
    );
    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Define Club DNA" }),
      ).toBeNull();
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: clubDnaKeys.definition(CLUB_DNA_CONTEXT),
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: squadKeys.all,
      });
    });
    expect(
      usePlayerTableStore.getState().layouts.search.columnIds,
    ).not.toContain("club_dna");
    expect(
      usePlayerTableStore.getState().layouts.squad.columnIds,
    ).not.toContain("club_dna");

    store.addColumns("search", ["club_dna"]);
    store.addColumns("squad", ["club_dna"]);
    const layoutsBeforeRemoval = {
      search: [...usePlayerTableStore.getState().layouts.search.columnIds],
      squad: [...usePlayerTableStore.getState().layouts.squad.columnIds],
    };
    expect(layoutsBeforeRemoval.search).toContain("club_dna");
    expect(layoutsBeforeRemoval.squad).toContain("club_dna");

    invalidateQueries.mockClear();
    await user.click(
      within(controls).getByRole("button", { name: "Define DNA" }),
    );
    const removeDialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(removeDialog).getByRole("button", { name: "Remove Club DNA" }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Remove Club DNA?" }),
      ).getByRole("button", { name: "Remove definition" }),
    );
    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Remove Club DNA?" }),
      ).toBeNull();
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: clubDnaKeys.definition(CLUB_DNA_CONTEXT),
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: squadKeys.all,
      });
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
      layoutsBeforeRemoval.search,
    );
    expect(usePlayerTableStore.getState().layouts.squad.columnIds).toEqual(
      layoutsBeforeRemoval.squad,
    );
    expect(router.state.location.search).toEqual({
      squadSort: "club_dna",
      squadDir: "asc",
    });

    store.removeColumn("search", "club_dna");
    store.removeColumn("squad", "club_dna");

    await user.click(
      within(controls).getByRole("button", { name: "Define DNA" }),
    );
    const recreateDialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(recreateDialog).getByRole("checkbox", { name: "Acceleration" }),
    );
    await user.click(
      within(recreateDialog).getByRole("button", { name: "Save Club DNA" }),
    );
    await waitFor(() => {
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
        "club_dna",
      );
      expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
        "club_dna",
      );
    });
  });

  it("keeps the visible Define DNA action disabled while its mounted definition query is pending", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setClubDnaGetIpcMockMode("busy");
    renderMyClubRoute({ initialEntry: "/my-club" });

    const trigger = await screen.findByRole("button", { name: "Define DNA" });
    expect(trigger).toBeDisabled();
    await user.click(trigger);
    expect(
      screen.queryByRole("dialog", { name: "Define Club DNA" }),
    ).toBeNull();
  });

  it("blocks a stale A create from appending or invalidating during a saves refresh", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });
    await screen.findByRole("button", { name: "Define DNA" });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Define DNA" })).toBeEnabled(),
    );

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Acceleration" }),
    );
    setClubDnaSetIpcMockMode("busy");
    await user.click(
      within(dialog).getByRole("button", { name: "Save Club DNA" }),
    );
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");

    let resolveSaves!: (saves: SaveSummary[]) => void;
    const savesRefresh = queryClient.fetchQuery({
      ...savesQueryOptions,
      queryFn: () =>
        new Promise<SaveSummary[]>((resolve) => {
          resolveSaves = resolve;
        }),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Define DNA" })).toBeDisabled(),
    );
    expect(
      screen.queryByRole("dialog", { name: "Define Club DNA" }),
    ).toBeNull();

    resolveBusyClubDnaSetRequest(CLUB_DNA_CONTEXT);
    await waitFor(() => {
      expect(
        usePlayerTableStore.getState().layouts.search.columnIds,
      ).not.toContain("club_dna");
      expect(
        usePlayerTableStore.getState().layouts.squad.columnIds,
      ).not.toContain("club_dna");
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: clubDnaKeys.definition(CLUB_DNA_CONTEXT),
      });
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: squadKeys.all,
      });
    });

    resolveSaves(savesFor(SECOND_SAVE.id));
    await savesRefresh;
    expect(screen.getByRole("button", { name: "Define DNA" })).toBeDisabled();
  });

  it("blocks stale A remove feedback after a failed saves refresh", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setManagedClubIpcMock({
      clubName: "Barcelona",
      clubUid: 1,
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setClubDnaIpcMockDefinition(CLUB_DNA_CONTEXT, ["attr.Acceleration"]);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("button", { name: "Define DNA" });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Define DNA" })).toBeEnabled(),
    );
    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Define Club DNA",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove Club DNA" }),
    );
    setClubDnaRemoveIpcMockMode("busy");
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Remove Club DNA?" }),
      ).getByRole("button", { name: "Remove definition" }),
    );
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");

    const savesRefresh = queryClient.fetchQuery({
      ...savesQueryOptions,
      queryFn: () => Promise.reject(new Error("Could not refresh saves")),
    });
    await expect(savesRefresh).rejects.toThrow("Could not refresh saves");
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Define DNA" })).toBeDisabled(),
    );

    rejectBusyClubDnaRemoveRequest(
      CLUB_DNA_CONTEXT,
      new Error("Could not remove Club DNA"),
    );
    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Remove Club DNA?" }),
      ).toBeNull();
      expect(screen.queryByText("Could not remove Club DNA")).toBeNull();
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: clubDnaKeys.definition(CLUB_DNA_CONTEXT),
      });
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).not.toHaveBeenCalledWith({
        queryKey: squadKeys.all,
      });
    });
  });

  it("groups managed-club controls above feedback while retaining save states", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setManagedClubIpcMock({
      clubName: "Legacy FC",
      clubUid: null,
      status: "missing",
      unclassifiedPlayerCount: 0,
    });
    setPlannerAvailableClubs([
      { clubName: "Legacy FC", clubUid: 1 },
      { clubName: "Barcelona", clubUid: 2 },
    ]);
    setManagedClubSavePending(true);
    renderMyClubRoute({ initialEntry: "/my-club" });

    const picker = await screen.findByRole("combobox", {
      name: "Managed club",
    });
    const saveButton = screen.getByRole("button", {
      name: "Save managed club",
    });
    const warning = screen.getByText(
      "Legacy FC is not in the latest snapshot. The saved selection remains active until you replace it.",
    );
    const controls = screen.getByRole("group", {
      name: "Managed club controls",
    });

    expect(controls).toContainElement(picker);
    expect(controls).toContainElement(saveButton);
    expect(
      picker.compareDocumentPosition(saveButton) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    expect(controls).toHaveClass("flex", "flex-wrap");
    expect(controls.closest("form")).toHaveClass("max-w-2xl");
    expect(controls).not.toContainElement(warning);
    expect(controls.compareDocumentPosition(warning)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(saveButton).toBeDisabled();

    await user.clear(picker);
    await user.type(picker, "Bar");
    await user.click(screen.getByRole("option", { name: "Barcelona" }));
    expect(saveButton).toBeEnabled();

    await user.click(saveButton);
    expect(saveButton).toBeDisabled();

    resolvePendingManagedClubSave();
  });

  it("retains a missing managed club without exposing team-level diagnostics", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubIpcMock({
      clubName: "Legacy FC",
      clubUid: null,
      status: "missing",
      unclassifiedPlayerCount: 2,
    });

    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByRole("combobox", { name: "Managed club" }),
    ).toHaveValue("Legacy FC");
    expect(
      screen.getByText(
        "Legacy FC is not in the latest snapshot. The saved selection remains active until you replace it.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/supported FM team level/i),
    ).not.toBeInTheDocument();
  });

  it("keeps managed-club option failures inside the selector boundary", async () => {
    await resolveLoadDataIpcMock();
    setManagedClubOptionsError("Managed club options are unavailable.");
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByText("Managed club options are unavailable."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();

    setManagedClubOptionsError(null);
    await userEvent
      .setup()
      .click(screen.getByRole("button", { name: "Retry" }));

    expect(
      await screen.findByRole("combobox", { name: "Managed club" }),
    ).toBeInTheDocument();
    expect(queryClient.getQueryState(managedClubKeys.options())?.status).toBe(
      "success",
    );
  });

  it("does not restore a late managed-club result after context invalidation", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setManagedClubSavePending(true);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });
    const picker = await screen.findByRole("combobox", {
      name: "Managed club",
    });
    await user.type(picker, "Bar");
    await user.click(screen.getByRole("option", { name: "Barcelona" }));

    vi.useFakeTimers();
    fireEvent.blur(picker);
    fireEvent.click(screen.getByRole("button", { name: "Save managed club" }));

    try {
      setManagedClubIpcMock({
        clubName: "Second FC",
        clubUid: 2,
        status: "available",
        unclassifiedPlayerCount: 0,
      });
      setPlannerAvailableClubs([
        { clubName: "Second FC", clubUid: 1 },
        { clubName: "Barcelona", clubUid: 1 },
      ]);
      await act(async () => {
        const invalidation = queryClient.invalidateQueries({
          queryKey: managedClubKeys.all,
        });
        await vi.advanceTimersByTimeAsync(0);
        await invalidation;
      });
      expect(picker).toHaveValue("Second FC");

      act(() => vi.advanceTimersByTime(150));
      expect(picker).toHaveValue("Second FC");
    } finally {
      vi.useRealTimers();
    }

    resolvePendingManagedClubSave();

    await waitFor(() => {
      expect(picker).toHaveValue("Second FC");
      expect(queryClient.getQueryData(managedClubKeys.status())).toEqual({
        clubName: "Second FC",
        clubUid: 2,
        status: "available",
        unclassifiedPlayerCount: 0,
      });
    });
  });

  it("defaults to Squad and keeps Planner and Tactic mounted", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderMyClubRoute({ initialEntry: "/my-club" });

    const navigation = await screen.findByRole("navigation", {
      name: "Primary",
    });
    const workspaceLink = (name: string) =>
      within(navigation).getByRole("link", { name });
    await screen.findByRole("link", { name: "Open Managed Club" });
    expect(workspaceLink("Squad")).toHaveAttribute("aria-current", "page");
    expect(workspaceLink("Planner")).not.toHaveAttribute("aria-current");
    expect(workspaceLink("Tactic")).not.toHaveAttribute("aria-current");
    expect(
      screen.queryByRole("tablist", { name: "My Club workspaces" }),
    ).toBeNull();
    expect(screen.queryByRole("tab", { name: "Squad" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Planner" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Tactic" })).toBeNull();
    expect(document.querySelector('[id^="my-club-workspace-"]')).toBeNull();
    expect(
      document.querySelector('[aria-labelledby^="my-club-workspace-"]'),
    ).toBeNull();
    expect(
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toHaveAttribute("href", "/my-club#managed-club");
    const tacticRegion = await screen.findByRole("region", {
      name: "Tactic controls",
      hidden: true,
    });
    const depthHeading = await screen.findByRole("heading", {
      level: 2,
      name: "Squad depth",
      hidden: true,
    });
    expect(tacticRegion.closest("[hidden]")).not.toBeNull();
    expect(depthHeading.closest("[hidden]")).not.toBeNull();
    expect(tacticRegion).not.toBeVisible();
    expect(depthHeading).not.toBeVisible();

    await openMyClubWorkspace(user, "tactic");
    const tacticEditor = screen.getByRole("region", {
      name: "Tactic controls",
    });
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await openMyClubWorkspace(user, "planner");
    await openMyClubWorkspace(user, "tactic");
    expect(tacticEditor).toBeInTheDocument();
    expect(weight).toHaveValue("51");
  });

  it("shows a sortable overview for a configured squad", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    for (const column of [
      "Player",
      "Age",
      "Nationality",
      "Height",
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
    expect(within(table).getByText("Alex Scout")).toBeInTheDocument();
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "1 player · sorted by CA (descending)",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Edit filters" })).toBeNull();
  });

  it("blocks the Squad controller through a managed-club owner refresh", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    let resolveSaves!: (value: SaveSummary[]) => void;
    const refreshedSaves = queryClient.fetchQuery({
      ...savesQueryOptions,
      queryFn: () =>
        new Promise<SaveSummary[]>((resolve) => {
          resolveSaves = resolve;
        }),
    });

    expect(await screen.findByText("Loading squad overview…")).toBeVisible();
    expect(screen.queryByRole("table", { name: "Squad overview" })).toBeNull();
    resolveSaves(savesFor(1));
    await refreshedSaves;
    expect(
      await screen.findByRole("table", { name: "Squad overview" }),
    ).toBeInTheDocument();
  });

  it("describes an empty configured Squad as one managed club", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByText(
        "No current-snapshot players match your managed club.",
      ),
    ).toBeInTheDocument();
  });

  it("renders every nationality flag in the squad overview", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Flagged Squad", 42),
        nationalities: ["England", "Wales", "South Korea"],
      },
    ]);
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    usePlayerTableStore.getState().addColumns("search", ["attr.Acceleration"]);
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Accelerating Squad", 42),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    renderMyClubRoute({ initialEntry: "/my-club" });

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
        requestedFields: ["attr.Acceleration", "height"],
      });
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "attr.Acceleration",
    );
  });

  it("renders nullable Club DNA scores through ScoreBadge", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    usePlayerTableStore.getState().addColumns("squad", ["club_dna"]);
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("DNA fit", 42),
        dynamicValues: { club_dna: 82 },
      },
      {
        ...squadPlayerNamed("DNA unavailable", 43),
        dynamicValues: { club_dna: null },
      },
    ]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    expect(
      within(table).getByRole("img", {
        name: "Club DNA: 82, Excellent",
      }),
    ).toBeInTheDocument();
    const unavailableRow = within(table)
      .getByText("DNA unavailable")
      .closest("tr");
    if (!unavailableRow) {
      throw new Error("Expected the unavailable-score player row.");
    }
    expect(
      within(unavailableRow).getByText("—", { selector: "span" }),
    ).toBeInTheDocument();
  });

  it("manages Squad columns from the grouped Columns control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Squad Scout", 160)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Columns" }));
    const dialog = screen.getByRole("dialog", { name: "Columns" });
    expect(within(dialog).getByText("Development")).toBeInTheDocument();
    expect(
      within(dialog).getByRole("checkbox", {
        name: "Suggested Training",
      }),
    ).toBeChecked();
    expect(within(dialog).queryByRole("checkbox", { name: "Name" })).toBeNull();

    await user.click(
      within(dialog).getByRole("checkbox", { name: "Suggested Training" }),
    );
    expect(
      usePlayerTableStore.getState().layouts.squad.columnIds,
    ).not.toContain("suggested_training");
    await waitFor(() => {
      const reloaded = screen.getByRole("table", {
        name: "Squad overview",
      });
      expect(
        within(reloaded).queryByRole("columnheader", {
          name: "Development",
        }),
      ).toBeNull();
    });

    await user.click(
      within(dialog).getByRole("checkbox", { name: "Suggested Training" }),
    );
    expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
      "suggested_training",
    );
    expect(
      await screen.findByRole("columnheader", {
        name: "Suggested Training",
      }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
  });

  it("reorders Squad columns from the menu without changing its query, virtual row, or widths", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
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
    renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    const scroller = screen.getByTestId("squad-overview-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 50,
        requestedFields: ["attr.Acceleration", "attr.Agility", "height"],
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
    expect(getSquadPlayersCallCount()).toBe(callCountBeforeReorder);
    expect(
      screen.getByRole("separator", { name: "Resize Acceleration column" }),
    ).toHaveAttribute("aria-valuenow", "216");
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
  });

  it("uploads a Moneyball CSV from Squad and refreshes its consumers", async () => {
    const user = userEvent.setup();
    openCsvDialog.mockResolvedValue("C:\\Users\\Jonas\\private-squad.csv");
    setCsvImportIpcMockResult({
      format: "moneyball",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");
    expect(
      screen.getByRole("button", { name: "Upload Squad CSV" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Upload Youth Academy CSV" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Upload Squad CSV" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Upload Moneyball CSV",
    });
    expect(dialog).toHaveTextContent("Only a Moneyball export can be imported");
    await user.click(
      within(dialog).getByRole("button", { name: "Browse files" }),
    );

    await waitFor(() => {
      expect(getLastCsvImportIpcArgs()).toEqual({
        path: "C:\\Users\\Jonas\\private-squad.csv",
        expectedFormat: "moneyball",
      });
    });
    expect(
      await within(dialog).findByText(/Moneyball imported/i),
    ).toBeInTheDocument();
    expect(dialog).not.toHaveTextContent("C:\\Users\\Jonas\\private-squad.csv");
    await waitFor(() => {
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: searchKeys.all,
      });
      expect(invalidateQueries).toHaveBeenCalledWith({
        queryKey: moneyballKeys.all,
      });
    });
    expect(invalidateQueries).not.toHaveBeenCalledWith({
      queryKey: squadKeys.all,
    });
  });

  it("confirms a Squad CA boost, locks the action, and refreshes affected views", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("pending");
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

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
    expect(dialog).toHaveTextContent("0 of 2 players processed.");
    sendPendingSquadCurrentAbilityBoostProgressIpcMock();
    await waitFor(() =>
      expect(dialog).toHaveTextContent("1 of 2 players processed."),
    );
    const progressbar = within(dialog).getByRole("progressbar", {
      name: "Squad boost progress",
    });
    expect(progressbar).toHaveAttribute("max", "2");
    expect(progressbar).toHaveAttribute("value", "1");
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toHaveLength(1);
    expect(getSquadCurrentAbilityBoostIpcMockCalls()[0]).toHaveProperty(
      "onProgress",
    );

    resolvePendingSquadCurrentAbilityBoostIpcMock();

    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );
    expect(getLastSquadCurrentAbilityBoostProgress()).toEqual({
      processed: 2,
      total: 2,
      updated: 2,
      skipped: 0,
      failed: 0,
    });
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

  it("shows zero-total Squad boost progress without a progress bar", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("pendingEmpty");
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Boost all CA?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );

    expect(dialog).toHaveTextContent("0 of 0 players processed.");
    expect(
      within(dialog).queryByRole("progressbar", {
        name: "Squad boost progress",
      }),
    ).toBeNull();

    resolvePendingSquadCurrentAbilityBoostIpcMock();
    expect(await screen.findByRole("status")).toHaveTextContent(
      "0 processed — 0 updated, 0 skipped, 0 failed.",
    );
  });

  it("shows only the latest Squad boost outcome in the shared feedback region", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Boost all CA?" }),
      ).getByRole("button", { name: "Boost all CA" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );

    await user.click(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    );
    const wonderkidDialog = await screen.findByRole("dialog", {
      name: "Make all Wonderkids?",
    });
    expect(screen.queryByRole("status")).toBeNull();
    await user.click(
      within(wonderkidDialog).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(screen.queryByRole("status")).toBeNull();

    await user.click(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    );
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Make all Wonderkids?" }),
      ).getByRole("button", { name: "Make all Wonderkids" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );
    expect(screen.getAllByRole("status")).toHaveLength(1);
  });

  it("keeps a Squad boost error in the Modal before moving it to shared feedback", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("error");
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Boost all CA?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );
    expect(within(dialog).getByRole("alert")).toHaveTextContent(
      "Could not boost the squad.",
    );
    expect(
      within(screen.getByTestId("squad-boost-feedback")).queryByRole("alert"),
    ).toBeNull();

    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(
      within(screen.getByTestId("squad-boost-feedback")).getByRole("alert"),
    ).toHaveTextContent("Could not boost the squad.");
  });

  it("clears shared Squad feedback when the current snapshot is replaced", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Boost all CA?" }),
      ).getByRole("button", { name: "Boost all CA" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );

    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot in the planner query");
    }
    queryClient.setQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
      { ...snapshot, id: snapshot.id + 1 },
    );

    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
  });

  it("drops pending Squad progress when the current snapshot is replaced", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("pending");
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Boost all CA?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );
    expect(dialog).toHaveTextContent("0 of 2 players processed.");

    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot in the planner query");
    }
    queryClient.setQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
      { ...snapshot, id: snapshot.id + 1 },
    );

    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Boost all CA?" }),
      ).toBeNull();
      expect(screen.queryByText("0 of 2 players processed.")).toBeNull();
    });
  });

  it("reports when a Squad CA boost needs Load Data before another attempt", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadCurrentAbilityBoostIpcMockMode("recoveryRequired");
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("table", { name: "Squad overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Boost all CA?" }),
      ).getByRole("button", { name: "Boost all CA" }),
    );

    expect(await screen.findByRole("status")).toHaveTextContent(
      "4 processed — 1 updated, 2 skipped, 1 failed.",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Stopped before all players were processed. Load Data is required before another boost.",
    );
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
      expect(screen.getByTestId("squad-boost-feedback")).toHaveFocus();
    });
    const action = screen.getByRole("button", { name: "Boost all CA" });
    expect(action).toBeDisabled();
    await user.click(action);
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toHaveLength(1);
  });

  it("confirms the Squad Wonderkid action before applying it", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

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
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toHaveLength(1);
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()[0]).toHaveProperty(
      "onProgress",
    );
  });

  it("locks both Squad actions while Wonderkid Mentality is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadWonderkidMentalityBoostIpcMockMode("pending");
    renderMyClubRoute({ initialEntry: "/my-club" });

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
    expect(dialog).toHaveTextContent("0 of 2 players processed.");
    sendPendingSquadWonderkidMentalityBoostProgressIpcMock();
    await waitFor(() =>
      expect(dialog).toHaveTextContent("1 of 2 players processed."),
    );
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toHaveLength(1);

    resolvePendingSquadWonderkidMentalityBoostIpcMock();
    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );
    expect(getLastSquadWonderkidMentalityBoostProgress()).toEqual({
      processed: 2,
      total: 2,
      updated: 2,
      skipped: 0,
      failed: 0,
    });
  });

  it("requires Load Data before either Squad action after Wonderkid recovery", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    setSquadWonderkidMentalityBoostIpcMockMode("recoveryRequired");
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      "4 processed — 1 updated, 2 skipped, 1 failed.",
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Stopped before all players were processed. Load Data is required before another boost.",
    );
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
      expect(screen.getByTestId("squad-boost-feedback")).toHaveFocus();
    });
    expect(
      screen.getByRole("button", { name: "Make all Wonderkids" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeDisabled();
    expect(getSquadWonderkidMentalityBoostIpcMockCalls()).toHaveLength(1);
    expect(getSquadCurrentAbilityBoostIpcMockCalls()).toEqual([]);
  });

  it("deduplicates the initial Squad page-zero IPC request", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Only once", 1)]);
    renderMyClubRoute({
      initialEntry: "/my-club",
      staleTime: 60_000,
    });

    expect(
      await screen.findByRole("table", { name: "Squad overview" }),
    ).toBeInTheDocument();
    expect(getSquadPlayersCallCount()).toBe(1);
  });

  it("shows an initial Squad failure and retries page zero", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Recovered Squad", 1)]);
    setSquadPlayersPageIpcMockMode("rejectInitial");
    const user = userEvent.setup();
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(await screen.findByText("Could not load squad")).toBeInTheDocument();
    const callsAfterFailure = getSquadPlayersCallCount();

    setSquadPlayersPageIpcMockMode("success");
    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(
      await screen.findByRole("table", { name: "Squad overview" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Recovered Squad")).toBeInTheDocument();
    expect(getSquadPlayersCallCount()).toBeGreaterThan(callsAfterFailure);
  });

  it("clears Squad rows while a visible-field projection loads", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Projected Squad", 1)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(await screen.findByText("Projected Squad")).toBeInTheDocument();
    const callsBeforeProjection = getSquadPlayersCallCount();
    setSquadPlayersPageIpcMockMode("pendingProjection");
    act(() => {
      usePlayerTableStore.getState().addColumns("squad", ["attr.Acceleration"]);
    });

    await waitFor(() =>
      expect(getSquadPlayersCallCount()).toBe(callsBeforeProjection + 1),
    );
    expect(screen.getByText("Loading squad overview…")).toBeInTheDocument();
    expect(screen.queryByRole("table", { name: "Squad overview" })).toBeNull();
    expect(screen.queryByText("Projected Squad")).toBeNull();

    resolvePendingSquadPlayersPageIpcMock();
    expect(await screen.findByText("Projected Squad")).toBeInTheDocument();
    act(() => {
      usePlayerTableStore.getState().removeColumn("squad", "attr.Acceleration");
    });
  });

  it("sorts the Squad table through the URL and backend query", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara Scout", 1, 160),
      squadPlayerNamed("Alex Scout", 2, 150),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByRole("button", { name: "Value" }));

    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        squadSort: "value",
        squadDir: "desc",
      });
      expect(getLastSquadPlayersArgs()).toMatchObject({
        offset: 0,
        limit: 50,
        sortBy: "value",
        sortDir: "desc",
        requestedFields: ["height"],
      });
    });
    expect(
      within(screen.getByRole("table", { name: "Squad overview" })).getByRole(
        "columnheader",
        { name: "Value" },
      ),
    ).toHaveAttribute("aria-sort", "descending");
    expect(screen.getByText("Alex Scout")).toBeInTheDocument();
  });

  it("retains A until a stale cached Squad sort refetch succeeds", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara", 1, 200),
      squadPlayerNamed("Alice", 2, 100),
    ]);
    const { queryClient, router } = renderMyClubRoute({
      initialEntry: "/my-club",
    });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
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
      .findAll({ queryKey: squadKeys.playerPages() })
      .find((query) => {
        const descriptor = query.queryKey.at(-1);
        return (
          typeof descriptor === "object" &&
          descriptor !== null &&
          (descriptor as { sortBy?: unknown }).sortBy === "value"
        );
      });
    if (!cachedValueSort) {
      throw new Error("expected a cached Squad value sort");
    }
    await queryClient.invalidateQueries({ queryKey: cachedValueSort.queryKey });
    setSquadPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    await screen.findByRole("status");
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "2 players · sorted by CA (descending)",
      ),
    ).toBeInTheDocument();
    const retainedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!retainedRow) {
      throw new Error("expected a retained Squad row");
    }
    fireEvent.click(retainedRow);
    fireEvent.keyDown(retainedRow, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/my-club");

    rejectPendingSquadPlayersPageIpcMock("Could not refresh sorted squad.");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort squad. Could not refresh sorted squad.",
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

  it("retains committed Squad rows, blocks stale activation, and promotes only the latest sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara", 1, 200),
      squadPlayerNamed("Alice", 2, 100),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    const callsBeforeSort = getSquadPlayersCallCount();
    setSquadPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    await waitFor(() =>
      expect(getSquadPlayersCallCount()).toBe(callsBeforeSort + 1),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Sorting…");
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    const row = within(table)
      .getAllByRole("row")
      .find((candidate) => candidate.hasAttribute("data-index"));
    if (!row) {
      throw new Error("expected a retained Squad row");
    }
    expect(row).not.toHaveAttribute("tabindex");
    fireEvent.click(row);
    fireEvent.keyDown(row, { key: "ArrowDown" });
    fireEvent.keyDown(row, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/my-club");

    await user.click(within(table).getByRole("button", { name: "CA" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "CA" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    resolvePendingSquadPlayersPageIpcMock();
    await Promise.resolve();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "ascending");
    expect(within(table).getByText("Alice")).toBeInTheDocument();
  });

  it("falls back when removing a deferred requested dynamic Squad sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    usePlayerTableStore.getState().addColumns("squad", ["attr.Acceleration"]);
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Fast Squad", 1),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    setSquadPlayersPageIpcMockMode("pendingDynamicReplacement");
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
        squadSort: "ca",
        squadDir: "desc",
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
    expect(screen.getByText("Fast Squad")).toBeInTheDocument();

    resolvePendingSquadPlayersPageIpcMock();
    await Promise.resolve();
    expect(screen.getByRole("columnheader", { name: "CA" })).toHaveAttribute(
      "aria-sort",
      "descending",
    );
  });

  it("keeps committed Squad rows after a failed sort and retries the replacement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara", 1, 200),
      squadPlayerNamed("Alice", 2, 100),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    setSquadPlayersPageIpcMockMode("rejectReplacementOnce");
    await user.click(within(table).getByRole("button", { name: "Value" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort squad.",
    );
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    const retainedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!retainedRow) {
      throw new Error("expected a retained Squad row after a failed sort");
    }
    expect(retainedRow).not.toHaveAttribute("tabindex");
    fireEvent.click(retainedRow);
    retainedRow.focus();
    await user.keyboard("{Enter}");
    expect(router.state.location.pathname).toBe("/my-club");

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Value" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
  });

  it("moves focus between Squad rows with the arrow keys", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Alex Scout", 42, 160),
      squadPlayerNamed("Zara Scout", 43, 150),
    ]);
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("pendingSecondPage");
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("pendingSecondPage");
    renderMyClubRoute({ initialEntry: "/my-club" });

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

    const plannerLink = screen.getByRole("link", { name: "Planner" });
    plannerLink.focus();
    expect(plannerLink).toHaveFocus();

    resolvePendingSquadPlayersPageIpcMock();

    expect(await screen.findByText("Squad player 051")).toBeInTheDocument();
    expect(plannerLink).toHaveFocus();
  });

  it("offers a retry when a visible virtual Squad page fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    setSquadPlayersPageIpcMockMode("rejectSecondPageOnce");
    renderMyClubRoute({ initialEntry: "/my-club" });

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
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(manySquadPlayers(101));
    const { queryClient } = renderMyClubRoute({ initialEntry: "/my-club" });

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

  it("stacks fixed-height Squad identity without duplicate columns", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      {
        ...squadPlayerNamed("Identity Player", 501, 250),
        club: "Metro FC",
        division: "Premier Division",
      },
      {
        ...squadPlayerNamed("No context", 502, 249),
        club: null,
        division: null,
      },
      {
        ...squadPlayerNamed("Club only", 503, 248),
        club: "Metro FC",
        division: null,
      },
      {
        ...squadPlayerNamed("Division only", 504, 247),
        club: null,
        division: "Premier Division",
      },
      ...manySquadPlayers(99),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    expect(
      within(table).queryByRole("columnheader", { name: "Club" }),
    ).toBeNull();
    expect(
      within(table).queryByRole("columnheader", { name: "Division" }),
    ).toBeNull();
    const squadIdentityHeader = within(table).getAllByRole("columnheader")[0];
    expect(squadIdentityHeader).toHaveTextContent("Player");
    expect(squadIdentityHeader).toHaveAttribute("rowspan", "2");
    expect(squadIdentityHeader.className).toContain("sticky");
    expect(squadIdentityHeader.className).toContain("left-0");
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
      throw new Error("Expected stacked Squad identity rows.");
    }
    expect(identityRow).toHaveStyle({ height: "40px" });
    expect(identityRow).toHaveTextContent("Metro FC · Premier Division");
    expect(within(identityRow).getAllByRole("cell")[0].className).toContain(
      "sticky",
    );
    const missingIdentityCell =
      within(missingContextRow).getAllByRole("cell")[0];
    expect(missingIdentityCell).toHaveTextContent("No context");
    expect(missingIdentityCell).not.toHaveTextContent("—");
    expect(missingIdentityCell).not.toHaveTextContent(" · ");
    expect(within(clubOnlyRow).getAllByRole("cell")[0]).toHaveTextContent(
      "Metro FC",
    );
    expect(within(clubOnlyRow).getAllByRole("cell")[0]).not.toHaveTextContent(
      " · ",
    );
    expect(within(divisionOnlyRow).getAllByRole("cell")[0]).toHaveTextContent(
      "Premier Division",
    );
    expect(
      within(divisionOnlyRow).getAllByRole("cell")[0],
    ).not.toHaveTextContent(" · ");
    identityRow.focus();
    expect(identityRow).toHaveFocus();

    await user.click(
      within(identityRow).getByText("Metro FC · Premier Division"),
    );

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/501");
      expect(router.state.location.search).toEqual({});
    });
  });

  it("opens a Squad player from a metric cell", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByText("160"));

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/42");
      expect(router.state.location.search).toEqual({});
    });
  });

  it("opens a focused Squad row with Enter and restores its sort on back", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara Scout", 42, 160),
      squadPlayerNamed("Alex Scout", 43, 150),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByRole("button", { name: "CA" }));
    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        squadSort: "ca",
        squadDir: "asc",
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
      expect(router.state.location.pathname).toBe("/my-club");
      expect(router.state.location.search).toEqual({
        squadSort: "ca",
        squadDir: "asc",
      });
    });
    await screen.findByRole("table", {
      name: "Squad overview",
    });
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "2 players · sorted by CA (ascending)",
      ),
    ).toBeInTheDocument();
  });

  it("switches Club workspaces from navigation with history support", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Barcelona", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByText("Managed club: Barcelona"),
    ).toBeInTheDocument();
    const navigation = screen.getByRole("navigation", { name: "Primary" });
    const workspaceLink = (name: string) =>
      within(navigation).getByRole("link", { name });
    expect(workspaceLink("Squad")).toHaveAttribute("aria-current", "page");

    await user.click(workspaceLink("Tactic"));
    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({ view: "tactic" }),
    );
    expect(workspaceLink("Tactic")).toHaveAttribute("aria-current", "page");
    expect(
      await screen.findByRole("region", { name: "Tactic controls" }),
    ).toBeVisible();

    router.history.back();
    await waitFor(() =>
      expect(workspaceLink("Squad")).toHaveAttribute("aria-current", "page"),
    );
    expect(
      await screen.findByRole("table", { name: "Squad overview" }),
    ).toBeVisible();
  });

  it("lets an explicit Planner workspace override the default", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Barcelona", clubUid: 1 },
      sources: [],
    });
    renderMyClubRoute({ initialEntry: "/my-club?view=planner" });

    const navigation = await screen.findByRole("navigation", {
      name: "Primary",
    });
    expect(
      within(navigation).getByRole("link", { name: "Planner" }),
    ).toHaveAttribute("aria-current", "page");
    const matrix = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(matrix).toBeVisible();
    expect(
      within(matrix)
        .getAllByRole("row")
        .slice(2)
        .map(
          (row) => row.getAttribute("aria-label")?.match(/^IP: ([^ ]+)/)?.[1],
        ),
    ).toEqual([
      "GK",
      "DR",
      "DCR",
      "DCL",
      "DL",
      "DM",
      "MCR",
      "MCL",
      "AMR",
      "AML",
      "STC",
    ]);
  });

  it("uses the Squad default for the retired Club Setup workspace", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=clubs" });

    const navigation = await screen.findByRole("navigation", {
      name: "Primary",
    });
    expect(
      within(navigation).getByRole("link", { name: "Squad" }),
    ).not.toHaveAttribute("aria-current");
    expect(
      within(navigation).getByRole("link", { name: "Planner" }),
    ).not.toHaveAttribute("aria-current");
    expect(
      within(navigation).getByRole("link", { name: "Tactic" }),
    ).not.toHaveAttribute("aria-current");
    expect(
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toBeVisible();
  });

  it("edits linked IP and OOP lanes with filtered roles and weight control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes[1] = {
      ...tactic.lanes[1],
      ipPosition: "DCR",
      ipRoleId: "centre_back_ip",
      oopPosition: "DCR",
      oopRoleId: "covering_centre_back_oop",
    };
    tactic.lanes[2] = {
      ...tactic.lanes[2],
      ipPosition: "DC",
      oopPosition: "DC",
    };
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });

    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });
    const bothView = within(viewGroup).getByRole("button", { name: "Both" });
    expect(bothView).toHaveAttribute("aria-pressed", "true");
    const inspectors = screen.getAllByRole("region", {
      name: "Selected Slot",
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
    expect(
      within(ipPosition)
        .getAllByRole("option")
        .map((option) => option.getAttribute("value")),
    ).toEqual([
      "GK",
      "DR",
      "DCR",
      "DC",
      "DCL",
      "DL",
      "WBR",
      "DMCR",
      "DM",
      "DMCL",
      "WBL",
      "MR",
      "MCR",
      "MC",
      "MCL",
      "ML",
      "AMR",
      "AMCR",
      "AMC",
      "AMCL",
      "AML",
      "STCR",
      "STC",
      "STCL",
    ]);
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

  it("orders the tactic command bar, pitch/XI area, and beside-pitch inspector", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    const commandBar = await screen.findByRole("region", {
      name: "Tactic controls",
    });
    const pitches = screen.getAllByRole("group", { name: /pitch$/ });
    const settings = screen.getByRole("region", {
      name: "Selected Slot",
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
    expect(pitches).toHaveLength(1);
    expect(
      commandBar.compareDocumentPosition(pitches[0]) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      pitches[0].compareDocumentPosition(settings) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    // The inspector sits beside the pitch/XI area in a shared responsive
    // grid (stacking below lg), not on a full-width bottom shelf.
    const workspaceGrid = settings.closest("div.grid");
    expect(workspaceGrid?.className).toContain("lg:grid-cols-");
    expect(
      within(workspaceGrid as HTMLElement).getByRole("region", {
        name: "Tactical XI",
      }),
    ).toBeInTheDocument();
    expect(
      within(workspaceGrid as HTMLElement).getByRole("group", {
        name: /pitch$/,
      }),
    ).toBe(pitches[0]);
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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });

    for (const [view, markerCount] of [
      ["Both", 22],
      ["IP", 11],
      ["OOP", 11],
    ] as const) {
      await user.click(within(viewGroup).getByRole("button", { name: view }));

      const pitches = screen.getAllByRole("group", { name: /pitch$/ });
      expect(pitches).toHaveLength(1);

      const positionButtons = within(pitches[0]).getAllByRole("button");
      expect(positionButtons).toHaveLength(markerCount);
      expect(positionButtons[0]).toHaveAccessibleName(/: STC · /);
      expect(positionButtons[positionButtons.length - 1]).toHaveAccessibleName(
        /: GK · /,
      );
    }
  });

  it("places unique lanes on normalized canvas coordinates", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    const pitches = await screen.findAllByRole("group", { name: /pitch$/ });
    expect(pitches).toHaveLength(1);

    const pitch = pitches[0];
    const markers = pitch.querySelectorAll("[data-pitch-marker]");
    // Both renders two phase-distinguished markers per lane on one canvas.
    expect(markers).toHaveLength(22);
    // Nine lanes share their IP/OOP placement, so both phase markers sit on
    // one normalized point; the two winger lanes place IP and OOP apart.
    const markerPositions = Array.from(markers).map(
      (marker) =>
        `${(marker as HTMLElement).style.left}/${(marker as HTMLElement).style.top}`,
    );
    expect(new Set(markerPositions).size).toBe(13);
    expect(
      pitch.querySelectorAll(
        '[data-pitch-marker="centre_forward"][data-phase="ip"]',
      ),
    ).toHaveLength(1);
    expect(
      pitch.querySelectorAll(
        '[data-pitch-marker="centre_forward"][data-phase="oop"]',
      ),
    ).toHaveLength(1);
    expect(markers[0]).toHaveAttribute("data-placement", "STC");
    expect(markers[markers.length - 1]).toHaveAttribute("data-placement", "GK");

    // Colliding same-placement markers stay distinguishable: each phase
    // keeps its own accessible name, visible phase treatment, normalized
    // placement, and DOM order (IP before OOP).
    const strikerPair = Array.from(markers).filter(
      (marker) =>
        (marker as HTMLElement).style.left === "50%" &&
        (marker as HTMLElement).style.top === "8%",
    );
    expect(strikerPair).toHaveLength(2);
    expect(strikerPair[0]).toHaveAttribute("data-phase", "ip");
    expect(strikerPair[1]).toHaveAttribute("data-phase", "oop");
    expect(
      within(strikerPair[0] as HTMLElement).getByRole("button", {
        name: "IP: STC · Centre Forward",
      }),
    ).toBeInTheDocument();
    expect(
      within(strikerPair[1] as HTMLElement).getByRole("button", {
        name: "OOP: STC · Central Outlet Centre Forward",
      }),
    ).toBeInTheDocument();
    // Every Both-mode marker carries a visible phase badge.
    expect(
      within(pitch as HTMLElement).getAllByText("IP", { selector: "span" }),
    ).toHaveLength(11);
    expect(
      within(pitch as HTMLElement).getAllByText("OOP", { selector: "span" }),
    ).toHaveLength(11);
    const striker = pitch.querySelector(
      '[data-pitch-marker="centre_forward"][data-phase="ip"]',
    );
    expect(striker).toHaveAttribute("data-placement", "STC");
    expect(striker).toHaveStyle({ left: "50%", top: "8%" });
    const goalkeeper = pitch.querySelector(
      '[data-pitch-marker="goalkeeper"][data-phase="ip"]',
    );
    expect(goalkeeper).toHaveAttribute("data-placement", "GK");
    expect(goalkeeper).toHaveStyle({ left: "50%", top: "93%" });
    const rightMidfielder = pitch.querySelector(
      '[data-pitch-marker="left_central_midfielder"][data-phase="ip"]',
    );
    expect(rightMidfielder).toHaveAttribute("data-placement", "MCR");
    expect(rightMidfielder).toHaveStyle({ left: "65%", top: "46%" });
    const markerButton = within(rightMidfielder as HTMLElement).getByRole(
      "button",
      { name: "IP: MCR · Central Midfielder" },
    );
    expect(markerButton.className).toContain("min-h-11");
    const attackNote = within(pitch as HTMLElement).getByText(/attack/i);
    expect(attackNote).toBeVisible();
    expect(pitch).toHaveAttribute("aria-describedby", attackNote.id);
    const markings = pitch.querySelector('svg[aria-hidden="true"]');
    expect(markings).not.toBeNull();
    expect(markings?.getAttribute("class")).toContain("pointer-events-none");
  });

  it("moves a marker when its qualified position is edited", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await user.click(
      await screen.findByRole("button", {
        name: "IP: MCR · Central Midfielder",
      }),
    );
    const pitch = (await screen.findAllByRole("group", { name: /pitch$/ }))[0];
    expect(
      pitch.querySelector(
        '[data-pitch-marker="left_central_midfielder"][data-phase="ip"]',
      ),
    ).toHaveAttribute("data-placement", "MCR");

    await user.selectOptions(
      screen.getByRole("combobox", { name: "IP MCR position" }),
      "MC",
    );

    const movedMarker = pitch.querySelector(
      '[data-pitch-marker="left_central_midfielder"][data-phase="ip"]',
    );
    expect(movedMarker).toHaveAttribute("data-placement", "MC");
    expect(movedMarker).toHaveStyle({ left: "50%", top: "46%" });
    expect(
      screen.getByRole("button", { name: "IP: MC · Central Midfielder" }),
    ).toBeInTheDocument();
    // The linked OOP marker keeps its own normalized placement.
    expect(
      pitch.querySelector(
        '[data-pitch-marker="left_central_midfielder"][data-phase="oop"]',
      ),
    ).toHaveAttribute("data-placement", "MCR");
  });

  it("presents current linked positions without lane terminology", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    const pitch = (await screen.findAllByRole("group", { name: /pitch$/ }))[0];
    const ipButton = await within(pitch).findByRole("button", {
      name: /IP: AMC · Winger/,
    });
    const oopButton = within(pitch).getByRole("button", {
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

    await openMyClubWorkspace(user, "planner");
    const matrix = screen.getByRole("region", {
      name: "Squad depth board",
    });
    expect(within(matrix).getByText("IP: AMC · Winger")).toBeInTheDocument();
    expect(within(matrix).queryByText("Left winger")).not.toBeInTheDocument();
  });

  it("retains the edited tactic draft when save fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerTacticSaveError("Tactic save failed");
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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

  it("exposes both phase controls in the Selected Slot inspector for each tactic view", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });

    for (const view of ["IP", "OOP", "Both"] as const) {
      await user.click(within(viewGroup).getByRole("button", { name: view }));
      const inspector = screen.getByRole("region", {
        name: "Selected Slot",
      });
      expect(
        within(inspector).getByRole("heading", { name: "Selected Slot" }),
      ).toBeInTheDocument();
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
      expect(
        within(inspector).getByRole("slider", {
          name: "IP/OOP score weight",
        }),
      ).toBeInTheDocument();
    }
  });

  it("saves only the selected lane score weight", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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

  it("saves an explicit midfield side without clearing its compatible role", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await user.click(
      await screen.findByRole("button", {
        name: "IP: MCR · Central Midfielder",
      }),
    );
    const position = screen.getByRole("combobox", {
      name: "IP MCR position",
    });
    const role = screen.getByRole("combobox", {
      name: "IP MCR role",
    });

    await user.selectOptions(position, "MC");
    await user.selectOptions(position, "MCR");

    expect(role).toHaveValue("central_midfielder_ip");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));
    expect(resolvePlannerTacticIpcMock().lanes[6]).toMatchObject({
      ipPosition: "MCR",
      ipRoleId: "central_midfielder_ip",
    });
  });

  it("swaps two lanes when picking an occupied placement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await user.click(
      await screen.findByRole("button", {
        name: "IP: MCR · Central Midfielder",
      }),
    );
    await user.selectOptions(
      screen.getByRole("combobox", { name: "IP MCR position" }),
      "MCL",
    );

    expect(
      screen.getByRole("button", { name: "IP: MCL · Central Midfielder" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "IP: MCR · Central Midfielder" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("combobox", { name: "OOP MCR position" }),
    ).toHaveValue("MCR");
    // After the cross-lane IP swap each shared point pairs one IP and one
    // OOP marker; DOM order keeps IP before OOP so tab order matches the
    // IP-left/OOP-right split.
    const pitch = (await screen.findAllByRole("group", { name: /pitch$/ }))[0];
    const markers = Array.from(pitch.querySelectorAll("[data-pitch-marker]"));
    for (const placement of ["MCL", "MCR"]) {
      const phases = markers
        .filter((marker) => marker.getAttribute("data-placement") === placement)
        .map((marker) => marker.getAttribute("data-phase"));
      expect(phases).toEqual(["ip", "oop"]);
    }

    await user.selectOptions(
      screen.getByRole("combobox", { name: "IP MCL position" }),
      "ML",
    );

    expect(screen.getByRole("combobox", { name: "IP ML role" })).toHaveValue(
      "",
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Choose a compatible IP role for ML.",
    );
    expect(screen.getByRole("button", { name: "Save tactic" })).toBeDisabled();
  });

  it("connects Both-mode markers only when canonical placement changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes[2] = {
      ...tactic.lanes[2],
      ipPosition: "DCR",
      ipRoleId: "centre_back_ip",
      oopPosition: "DCL",
      oopRoleId: "covering_centre_back_oop",
    };
    tactic.lanes[3] = {
      ...tactic.lanes[3],
      ipPosition: "DCL",
      ipRoleId: "centre_back_ip",
      oopPosition: "DC",
      oopRoleId: "covering_centre_back_oop",
    };
    tactic.lanes[10] = {
      ...tactic.lanes[10],
      ipPosition: "ST",
      ipRoleId: "centre_forward_ip",
      oopPosition: "STC",
      oopRoleId: "central_outlet_centre_forward_oop",
    };
    setPlannerTacticIpcMock(tactic);
    const depth = resolvePlannerDepthIpcMock();
    depth.tactic = tactic;
    setPlannerDepthIpcMock(depth);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const pitch = (await screen.findAllByRole("group", { name: /pitch$/ }))[0];

    // A canonical placement change connects; legacy-equivalent ST/STC and
    // unchanged placements render no connector.
    expect(pitch.querySelectorAll("[data-tactic-connector]")).toHaveLength(4);
    expect(
      pitch.querySelector('[data-tactic-connector="left_centre_back"]'),
    ).not.toBeNull();
    expect(
      pitch.querySelector('[data-tactic-connector="right_centre_back"]'),
    ).not.toBeNull();
    expect(
      pitch.querySelector('[data-tactic-connector="centre_forward"]'),
    ).toBeNull();
    expect(
      pitch.querySelector('[data-tactic-connector="goalkeeper"]'),
    ).toBeNull();

    // Cross-lane split ends stop at the displayed marker inner edge, not
    // the shared gap anchor: DCL is shared by left_centre_back OOP and
    // right_centre_back IP, so those ends shift one viewBox unit into
    // their own button while unshared ends stay on the anchor.
    expect(
      pitch.querySelector('[data-tactic-connector="left_centre_back"]'),
    ).toHaveAttribute("x1", "65");
    expect(
      pitch.querySelector('[data-tactic-connector="left_centre_back"]'),
    ).toHaveAttribute("x2", "36");
    expect(
      pitch.querySelector('[data-tactic-connector="right_centre_back"]'),
    ).toHaveAttribute("x1", "34");
    expect(
      pitch.querySelector('[data-tactic-connector="right_centre_back"]'),
    ).toHaveAttribute("x2", "50");

    // The connector overlay carries no inert title: the transition text
    // below is the readable surface for sighted and AT users alike.
    expect(pitch.querySelector("[data-tactic-connector] title")).toBeNull();

    // Every slot keeps its readable IP role to OOP role transition for AT.
    expect(
      pitch.querySelector('[data-slot-transition="left_centre_back"]'),
    ).toHaveTextContent(
      "IP: DCR · Centre-Back / OOP: DCL · Covering Centre-Back",
    );
    expect(
      pitch.querySelector('[data-slot-transition="centre_forward"]'),
    ).toHaveTextContent(
      "IP: STC · Centre Forward / OOP: STC · Central Outlet Centre Forward",
    );

    // The selected slot transition is also visible: the default selection
    // is the goalkeeper and it follows selection via click and keyboard.
    const visibleTransition = pitch.querySelector(
      "[data-selected-slot-transition]",
    );
    expect(visibleTransition).toHaveAttribute(
      "data-selected-slot-transition",
      "goalkeeper",
    );
    expect(visibleTransition).toHaveTextContent(
      "IP: GK · Goalkeeper OOP: GK · Line-Holding Keeper",
    );
    expect(visibleTransition?.classList.contains("sr-only")).toBe(false);
    await user.click(
      screen.getByRole("button", { name: "IP: DCR · Centre-Back" }),
    );
    expect(
      pitch.querySelector("[data-selected-slot-transition]"),
    ).toHaveAttribute("data-selected-slot-transition", "left_centre_back");
    expect(
      pitch.querySelector("[data-selected-slot-transition]"),
    ).toHaveTextContent(
      "IP: DCR · Centre-Back OOP: DCL · Covering Centre-Back",
    );
    screen.getByRole("button", { name: "IP: DCL · Centre-Back" }).focus();
    await user.keyboard("{Enter}");
    expect(
      pitch.querySelector("[data-selected-slot-transition]"),
    ).toHaveAttribute("data-selected-slot-transition", "right_centre_back");
    expect(
      pitch.querySelector("[data-selected-slot-transition]"),
    ).toHaveTextContent("IP: DCL · Centre-Back OOP: DC · Covering Centre-Back");

    // Single-phase modes render no connectors.
    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });
    await user.click(within(viewGroup).getByRole("button", { name: "IP" }));
    const ipPitch = (
      await screen.findAllByRole("group", { name: /pitch$/ })
    )[0];
    expect(ipPitch.querySelectorAll("[data-tactic-connector]")).toHaveLength(0);
  });

  it("syncs Tactical XI panel selection with the pitch and inspector", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes = [...tactic.lanes.slice(1), tactic.lanes[0]];
    setPlannerTacticIpcMock(tactic);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

    await screen.findByRole("region", { name: "Tactic controls" });
    const panel = await screen.findByRole("region", { name: "Tactical XI" });
    expect(within(panel).queryByRole("listbox")).toBeNull();

    // All 11 lanes render in lane order even though the draft array was
    // rotated: the goalkeeper still leads with a readable transition.
    const rows = within(panel).getAllByRole("button");
    expect(rows).toHaveLength(11);
    expect(rows[0].textContent).toContain("Goalkeeper\nOOP:");
    expect(rows[0]).toHaveClass("whitespace-pre-line");
    expect(rows[0]).toHaveTextContent(
      "IP: GK · Goalkeeper OOP: GK · Line-Holding Keeper",
    );
    expect(rows[10]).toHaveTextContent(
      "IP: STC · Centre Forward OOP: STC · Central Outlet Centre Forward",
    );
    for (const row of rows) {
      expect(row).toHaveAttribute("aria-pressed");
      expect(row).not.toHaveAttribute("aria-selected");
      expect(row).not.toHaveAttribute("aria-current");
    }

    // The rotated draft selects the second lane, so exactly one non-leading
    // row starts pressed.
    const initiallyPressed = rows.filter(
      (row) => row.getAttribute("aria-pressed") === "true",
    );
    expect(initiallyPressed).toHaveLength(1);
    expect(initiallyPressed[0]).toHaveTextContent("IP: DL · Full-Back");

    // Panel to pitch and inspector: activating the goalkeeper row selects
    // it everywhere.
    await user.click(rows[0]);
    const afterPanelSelect = within(panel).getAllByRole("button");
    expect(
      afterPanelSelect.filter(
        (row) => row.getAttribute("aria-pressed") === "true",
      ),
    ).toHaveLength(1);
    expect(afterPanelSelect[0]).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.getByRole("button", { name: "IP: GK · Goalkeeper" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      within(screen.getByRole("region", { name: "Selected Slot" })).getByText(
        "IP: GK · Goalkeeper OOP: GK · Line-Holding Keeper",
      ),
    ).toBeInTheDocument();

    // Pitch to panel: selecting a marker updates the pressed panel row.
    await user.click(
      screen.getByRole("button", { name: "IP: DL · Full-Back" }),
    );
    const afterMarkerSelect = within(panel).getAllByRole("button");
    const markerPressed = afterMarkerSelect.filter(
      (row) => row.getAttribute("aria-pressed") === "true",
    );
    expect(markerPressed).toHaveLength(1);
    expect(markerPressed[0]).toHaveTextContent("IP: DL · Full-Back");

    // Ordinary-button rows stay keyboard operable: Enter on the leading
    // row moves selection back and the inspector follows.
    afterMarkerSelect[0].focus();
    await user.keyboard("{Enter}");
    expect(afterMarkerSelect[0]).toHaveAttribute("aria-pressed", "true");
    expect(
      within(screen.getByRole("region", { name: "Selected Slot" })).getByText(
        "IP: GK · Goalkeeper OOP: GK · Line-Holding Keeper",
      ),
    ).toBeInTheDocument();
  });

  it("treats legacy ST and canonical STC as the same placement", () => {
    const tactic = resolvePlannerTacticIpcMock();
    tactic.lanes[0] = {
      ...tactic.lanes[0],
      ipPosition: "ST",
      ipRoleId: "centre_forward_ip",
    };

    expect(
      validateTacticDraft(tactic, resolvePlannerTacticOptionsIpcMock()),
    ).toBe("STC is already used in the In-Possession phase.");
  });

  it("saves and reloads the selected lane importance rank", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerTacticSaveError("Tactic save failed");
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerTacticSaveError("Tactic save failed");
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute({ staleTime: 60_000 });

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
    await openMyClubWorkspace(user, "tactic");

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

    await openMyClubWorkspace(user, "planner");
    await user.click(cell);
    expect(
      await screen.findByRole("option", { name: /Updated tactic fit/ }),
    ).toBeInTheDocument();
  });

  it("refreshes the role reference after saving a tactic", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReference({
      lanes: [
        {
          laneId: "goalkeeper",
          players: [
            {
              playerUid: 1,
              name: "Before tactic save",
              currentScore: 80,
              potentialScore: 85,
            },
          ],
        },
      ],
      noEligible: [],
    });
    renderMyClubRoute({ staleTime: 60_000 });

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(
      await within(dialog).findByText("Before tactic save", { exact: true }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await openMyClubWorkspace(user, "tactic");
    const weight = screen.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    weight.focus();
    await user.keyboard("{ArrowRight}");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));
    await waitFor(() =>
      expect(screen.getByText("Tactic saved.")).toBeInTheDocument(),
    );

    setPlannerRoleReference({
      lanes: [
        {
          laneId: "goalkeeper",
          players: [
            {
              playerUid: 1,
              name: "After tactic save",
              currentScore: 90,
              potentialScore: 95,
            },
          ],
        },
      ],
      noEligible: [],
    });
    await openMyClubWorkspace(user, "planner");
    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const refreshedDialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(
      await within(refreshedDialog).findByText("After tactic save", {
        exact: true,
      }),
    ).toBeInTheDocument();
    expect(getPlannerRoleReferenceCalls()).toHaveLength(2);
  });

  it("resets a dirty tactic draft when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const { queryClient } = renderMyClubRoute({
      initialEntry: "/my-club?view=tactic",
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

    const secondSaveTactic = resolvePlannerTacticIpcMock();
    secondSaveTactic.lanes[0].ipWeight = 0.2;
    setPlannerTacticForContext(
      {
        saveId: SECOND_SAVE.id,
        contextToken: SECOND_SAVE.contextToken,
      },
      secondSaveTactic,
    );
    switchToSecondSave(queryClient);

    await waitFor(() =>
      expect(
        screen.getByRole("slider", { name: "IP/OOP score weight" }),
      ).toHaveValue("20"),
    );
  });

  it("blocks tactic saves while active-save data refreshes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const { queryClient } = renderMyClubRoute({
      initialEntry: "/my-club?view=tactic",
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
      queryKey: plannerKeys.tactic({ saveId: 1, contextToken: "save-token-1" }),
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

  it.each([
    ["get_planner_tactic", plannerKeys.tactic(CLUB_DNA_CONTEXT)],
    ["get_planner_tactic_options", plannerKeys.tacticOptions(CLUB_DNA_CONTEXT)],
  ])(
    "shows one tactic load retry and refetches both queries when %s fails",
    async (_failedCommand, failedQueryKey) => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
      const queryClient = new QueryClient({
        defaultOptions: {
          queries: {
            retry: false,
            retryOnMount: false,
            refetchOnMount: false,
            staleTime: 0,
          },
        },
      });
      const failedQuery = queryClient.fetchQuery({
        queryKey: failedQueryKey,
        queryFn: () => Promise.reject(new Error("Initial tactic load failed")),
      });
      await expect(failedQuery).rejects.toThrow("Initial tactic load failed");
      renderMyClubRoute({
        initialEntry: "/my-club?view=tactic",
        queryClient,
      });

      const navigation = await screen.findByRole("navigation", {
        name: "Primary",
      });
      expect(
        within(navigation).getByRole("link", { name: "Tactic" }),
      ).toHaveAttribute("aria-current", "page");
      const retryButtons = await screen.findAllByRole("button", {
        name: "Retry",
      });
      const visibleRetries = retryButtons.filter(
        (button) => button.closest("[hidden]") === null,
      );
      expect(visibleRetries).toHaveLength(1);
      const loadErrors = await screen.findAllByText("Could not load tactic");
      const visibleErrors = loadErrors.filter(
        (element) => element.closest("[hidden]") === null,
      );
      expect(visibleErrors).toHaveLength(1);
      const tacticCalls = getPlannerTacticIpcMockCalls().length;
      const optionsCalls = getPlannerTacticOptionsIpcMockCalls().length;

      await user.click(visibleRetries[0]);

      expect(
        await screen.findByRole("region", {
          name: "Tactic controls",
        }),
      ).toBeInTheDocument();
      expect(getPlannerTacticIpcMockCalls()).toHaveLength(tacticCalls + 1);
      expect(getPlannerTacticOptionsIpcMockCalls()).toHaveLength(
        optionsCalls + 1,
      );
    },
  );

  it("keeps cached tactic data read-only until a successful retry", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const { queryClient } = renderMyClubRoute({
      initialEntry: "/my-club?view=tactic",
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
      queryKey: plannerKeys.tactic(CLUB_DNA_CONTEXT),
      queryFn: () => Promise.reject(new Error("Tactic refresh failed")),
    });
    await expect(refreshRequest).rejects.toThrow("Tactic refresh failed");

    const saveButton = screen.getByRole("button", { name: "Save tactic" });
    await waitFor(() => expect(weight).toBeDisabled());
    expect(saveButton).toBeDisabled();
    expect(screen.getByRole("combobox", { name: "IP GK role" })).toBeDisabled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Could not refresh tactic",
    );

    await openMyClubWorkspace(user, "planner");
    expect(screen.getByRole("button", { name: "Manage teams" })).toBeEnabled();
    await openMyClubWorkspace(user, "tactic");

    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => expect(weight).toBeEnabled());
    expect(saveButton).toBeEnabled();
    expect(weight).toHaveValue("55");
    expect(
      screen.queryByText("Could not refresh tactic"),
    ).not.toBeInTheDocument();
  });

  it("renders every squad simultaneously with slot context, named strings, and truthful cards", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = resolvePlannerDepthIpcMock();
    setPlannerDepthIpcMock(withDepthAssignments(depth));
    renderMyClubRoute();

    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(board).toHaveClass("overflow-x-auto");
    expect(board).toHaveClass("max-h-[min(70vh,720px)]");
    const boardTable = within(board).getByRole("table", {
      name: "Squad depth board",
    });
    expect(boardTable).toHaveClass("w-max");
    expect(boardTable).not.toHaveClass("w-full");
    expect(
      screen.queryByRole("tablist", { name: "Squad planner teams" }),
    ).toBeNull();
    expect(screen.queryByRole("tab", { name: "Senior" })).toBeNull();
    for (const squad of ["Senior", "Reserves", "Youth"]) {
      expect(
        within(board).getByRole("columnheader", { name: squad }),
      ).toBeInTheDocument();
    }
    expect(
      within(board).queryByRole("columnheader", { name: "Senior squad" }),
    ).toBeNull();
    expect(
      within(board).getAllByRole("columnheader", { name: "1st string" }),
    ).toHaveLength(3);
    expect(
      within(board).getAllByRole("columnheader", { name: "2nd string" }),
    ).toHaveLength(1);
    expect(within(board).getAllByRole("rowheader")).toHaveLength(11);
    expect(
      within(board).getByRole("row", { name: /Goalkeeper/ }),
    ).toBeInTheDocument();
    expect(within(board).getByText("IP: GK · Goalkeeper")).toBeInTheDocument();
    expect(
      within(board).getByText("OOP: GK · Line-Holding Keeper"),
    ).toBeInTheDocument();
    expect(
      within(board).getByRole("img", {
        name: /Current combined role score: 82/,
      }),
    ).toBeInTheDocument();
    expect(
      within(board).getByRole("img", {
        name: /Potential combined role score: 91/,
      }),
    ).toBeInTheDocument();
    expect(within(board).getByText("Outside pool")).toBeInTheDocument();
    expect(within(board).getByText("Unresolved")).toBeInTheDocument();
    expect(
      within(board).getByRole("button", {
        name: /Missing Centre-Back/,
      }),
    ).toBeInTheDocument();
    const unavailableCell = within(board).getByRole("button", {
      name: /No Score Player, Resolved, current score —, potential score —/,
    });
    expect(unavailableCell).not.toBeDisabled();
    unavailableCell.focus();
    expect(document.activeElement).toBe(unavailableCell);
    expect(within(board).getAllByText("—").length).toBeGreaterThan(0);

    const assignActions = within(board).getAllByText("Assign");
    expect(assignActions.length).toBeGreaterThan(0);
    const emptyCell = within(board).getByRole("button", {
      name: /Reserves, 1st string, IP: GK .* Empty/,
    });
    expect(emptyCell).not.toBeDisabled();
    expect(within(emptyCell).getByText("Assign")).toBeInTheDocument();
    emptyCell.focus();
    expect(document.activeElement).toBe(emptyCell);
    await user.click(
      within(board).getByRole("button", {
        name: /Youth, 1st string, IP: GK .* Empty/,
      }),
    );
    expect(
      screen.getByRole("dialog", {
        name: `Find a player for ${KEEPER_POSITION}`,
      }),
    ).toBeInTheDocument();
  });

  it("groups squad actions above a bounded compact matrix", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(withDepthAssignments(resolvePlannerDepthIpcMock()));
    renderMyClubRoute();

    const toolbar = await screen.findByRole("group", {
      name: "Squad controls",
    });
    expect(
      within(toolbar).queryByRole("tablist", { name: "Squad planner teams" }),
    ).toBeNull();
    expect(
      within(toolbar).getByRole("button", { name: "Optimize squads" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Clear all" }),
    ).toBeInTheDocument();

    const board = screen.getByRole("region", {
      name: "Squad depth board",
    });
    expect(board).toHaveClass("max-h-[min(70vh,720px)]");
    expect(board).toHaveClass("overflow-x-auto");
    expect(within(board).getByRole("row", { name: /Goalkeeper/ })).toHaveClass(
      "h-table-row-height-two-line",
    );
  });

  it("opens the best role fit reference from the Planner toolbar", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(dialog).toBeInTheDocument();
    expect(
      within(dialog).getByRole("radio", { name: "In Possession" }),
    ).toBeChecked();
    expect(
      within(dialog).getByRole("radio", { name: "Current" }),
    ).toBeChecked();
    expect(
      within(dialog).getByText(
        "Focus or select this position to show its players in the reference table.",
        { exact: true },
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).queryByText(/linked counterpart/),
    ).not.toBeInTheDocument();
    expect(getPlannerRoleReferenceCalls()).toEqual([
      { phase: "in_possession", scoreBasis: "current" },
    ]);
  });

  it("switches the role reference phase and score basis without re-sorting in the client", async () => {
    const user = userEvent.setup();
    const reference: PlannerRoleReference = {
      lanes: [
        {
          laneId: "goalkeeper",
          players: [
            {
              playerUid: 2,
              name: "Bravo Keeper",
              currentScore: 90,
              potentialScore: 85,
            },
            {
              playerUid: 1,
              name: "Alpha Keeper",
              currentScore: 80,
              potentialScore: 95,
            },
          ],
        },
      ],
      noEligible: [
        {
          playerUid: 99,
          name: "Unavailable Player",
          currentScore: null,
          potentialScore: null,
        },
      ],
    };
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReference(reference);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    const table = await within(dialog).findByRole("table", {
      name: "Players best suited to GK Goalkeeper",
    });
    expect(within(table).getAllByRole("row")[1]).toHaveTextContent(
      "Bravo Keeper",
    );
    expect(
      within(dialog).getByRole("heading", {
        name: "No eligible role",
      }),
    ).toBeInTheDocument();

    await user.click(
      within(dialog).getByRole("radio", { name: "Out of Possession" }),
    );
    await user.click(within(dialog).getByRole("radio", { name: "Potential" }));
    expect(
      within(dialog).getByRole("radio", { name: "Out of Possession" }),
    ).toBeChecked();
    expect(
      within(dialog).getByRole("radio", { name: "Potential" }),
    ).toBeChecked();
    await waitFor(() => {
      expect(getPlannerRoleReferenceCalls()).toEqual([
        { phase: "in_possession", scoreBasis: "current" },
        { phase: "out_of_possession", scoreBasis: "current" },
        { phase: "out_of_possession", scoreBasis: "potential" },
      ]);
    });
    expect(
      within(
        within(dialog).getByRole("table", {
          name: "Players best suited to GK Line-Holding Keeper",
        }),
      ).getByRole("columnheader", { name: "Potential" }),
    ).toHaveAttribute("aria-sort", "descending");
  });

  it("selects a tactic lane and sorts its current and potential scores", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReference({
      lanes: [
        {
          laneId: "goalkeeper",
          players: [
            {
              playerUid: 1,
              name: "Alpha Keeper",
              currentScore: 80,
              potentialScore: 95,
            },
            {
              playerUid: 2,
              name: "Bravo Keeper",
              currentScore: 90,
              potentialScore: 85,
            },
          ],
        },
        {
          laneId: "left_back",
          players: [
            {
              playerUid: 3,
              name: "Charlie Full-Back",
              currentScore: 70,
              potentialScore: 88,
            },
          ],
        },
      ],
      noEligible: [],
    });
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "IP: DL · Full-Back" }),
    );
    expect(
      within(dialog).getByRole("heading", { name: "DL · Full-Back" }),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("table", {
        name: "Players best suited to DL Full-Back",
      }),
    ).toHaveTextContent("Charlie Full-Back");

    await user.click(
      within(dialog).getByRole("button", { name: "IP: GK · Goalkeeper" }),
    );
    const table = within(dialog).getByRole("table", {
      name: "Players best suited to GK Goalkeeper",
    });
    const currentHeader = within(table).getByRole("columnheader", {
      name: "Current",
    });
    expect(currentHeader).toHaveAttribute("aria-sort", "descending");
    const callsBeforeSort = getPlannerRoleReferenceCalls().length;
    await user.click(within(currentHeader).getByRole("button"));
    expect(within(table).getAllByRole("row")[1]).toHaveTextContent(
      "Alpha Keeper",
    );
    expect(
      within(table).getByRole("columnheader", { name: "Current" }),
    ).toHaveAttribute("aria-sort", "ascending");
    expect(getPlannerRoleReferenceCalls()).toHaveLength(callsBeforeSort);

    const nameHeader = within(table).getByRole("columnheader", {
      name: "Name",
    });
    await user.click(within(nameHeader).getByRole("button"));
    expect(nameHeader).toHaveAttribute("aria-sort", "ascending");
    expect(within(table).getAllByRole("row")[1]).toHaveTextContent(
      "Alpha Keeper",
    );
    await user.click(within(nameHeader).getByRole("button"));
    expect(nameHeader).toHaveAttribute("aria-sort", "descending");
    expect(within(table).getAllByRole("row")[1]).toHaveTextContent(
      "Bravo Keeper",
    );
    expect(getPlannerRoleReferenceCalls()).toHaveLength(callsBeforeSort);
  });

  it("keeps an empty selected role explicit inside the reference Modal", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReference({
      lanes: [
        { laneId: "goalkeeper", players: [] },
        {
          laneId: "left_back",
          players: [
            {
              playerUid: 7,
              name: "Full-Back Player",
              currentScore: 70,
              potentialScore: 80,
            },
          ],
        },
      ],
      noEligible: [],
    });
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(
      await within(dialog).findByText("No eligible players", { exact: true }),
    ).toBeInTheDocument();
    expect(
      within(dialog).queryByRole("heading", { name: "No eligible role" }),
    ).not.toBeInTheDocument();
  });

  it("distinguishes an empty managed-club cohort from an empty role", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReference({
      lanes: resolvePlannerTacticIpcMock().lanes.map((lane) => ({
        laneId: lane.laneId,
        players: [],
      })),
      noEligible: [],
    });
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Best role fit" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(
      await within(dialog).findByText("No players at your managed club", {
        exact: true,
      }),
    ).toBeInTheDocument();
    expect(
      within(dialog).queryByText("No eligible players", { exact: true }),
    ).not.toBeInTheDocument();
  });

  it("shows a role reference error and restores focus after Escape", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerRoleReferenceError("Role reference failed");
    renderMyClubRoute();

    const trigger = await screen.findByRole("button", {
      name: "Best role fit",
    });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Best role fit reference",
    });
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Role reference failed",
    );

    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(trigger).toHaveFocus();
  });

  it("renders every enabled squad at once with no width-dependent fallback", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(
      withSecondStringForEveryTeam(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(
      within(board).getByRole("columnheader", { name: "Senior" }),
    ).toBeInTheDocument();
    expect(
      within(board).getByRole("columnheader", { name: "Reserves" }),
    ).toBeInTheDocument();
    expect(
      within(board).getByRole("columnheader", { name: "Youth" }),
    ).toBeInTheDocument();
    expect(
      within(board).getAllByRole("columnheader", { name: "1st string" }),
    ).toHaveLength(3);
    expect(
      within(board).getAllByRole("columnheader", { name: "2nd string" }),
    ).toHaveLength(3);
    expect(
      within(board).getByRole("button", {
        name: /Youth, 2nd string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    expect(
      within(board)
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
      within(board).queryByRole("button", { name: /Clear .* squad/ }),
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

  it("keeps string columns at bounded fixed widths that never stretch", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(
      withSecondStringForEveryTeam(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(board).toHaveClass("overflow-x-auto");
    const boardTable = within(board).getByRole("table", {
      name: "Squad depth board",
    });
    expect(boardTable).toHaveClass("w-max");
    expect(boardTable).not.toHaveClass("w-full");
    const stringCell = within(board)
      .getByRole("button", { name: /Senior, 1st string, IP: GK .* Empty/ })
      .closest("td");
    expect(stringCell).toHaveClass("min-w-52");
    expect(stringCell).toHaveClass("max-w-52");
    const slotHeader = within(board).getAllByRole("rowheader")[0];
    expect(slotHeader).toHaveClass("sticky");
    expect(slotHeader).toHaveClass("min-w-52");
    expect(slotHeader).toHaveClass("max-w-52");
  });

  it("renders board cards on the container-high surface without primary text", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(withDepthAssignments(resolvePlannerDepthIpcMock()));
    renderMyClubRoute();

    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    const assignedCard = within(board).getByRole("button", {
      name: /Alex Keeper/,
    });
    const emptyCard = within(board).getByRole("button", {
      name: /Senior, 2nd string, IP: GK .* Empty/,
    });
    for (const card of [assignedCard, emptyCard]) {
      expect(card).toHaveClass("bg-surface-container-high");
      expect(card).toHaveClass("border-outline-variant");
      expect(card).toHaveClass("rounded-md");
    }
    expect(board.querySelector(".text-primary")).toBeNull();
  });

  it("announces only the latest successful squad action", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerOptimizeDepth(
      withReserveGoalkeeper(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute();

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
      setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
      renderMyClubRoute();

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute({ staleTime: 60_000 });

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

  it("requires confirmation before clearing an occupied slot", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute({ staleTime: 60_000 });

    const occupiedCell = await screen.findByRole("button", {
      name: /Reserves, 1st string, IP: GK .* Reserve Keeper, Resolved/,
    });
    const emptyCell = await screen.findByRole("button", {
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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute();

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
    expect(
      screen.getByRole("button", {
        name: /Reserves, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
  });

  it("cancels and fails without changing assignments, then restores the origin focus", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
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
    renderMyClubRoute();

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
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
  });

  it("confirms clearing every squad and reconciles all candidates", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = withAllTeamDepthAssignments(resolvePlannerDepthIpcMock());
    setPlannerDepthIpcMock(depth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Senior Keeper" }),
      slotCandidate({ playerUid: 79, name: "Reserve Keeper" }),
      slotCandidate({ playerUid: 80, name: "Youth Keeper" }),
    ]);
    renderMyClubRoute({ staleTime: 60_000 });

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
    expect(
      screen.getByRole("button", {
        name: /Reserves, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: /Youth, 1st string, IP: GK .* Empty/,
      }),
    ).toBeInTheDocument();
    await user.click(secondSeniorCell);
    expect(
      await screen.findByRole("option", { name: /Senior Keeper/ }),
    ).toHaveTextContent("Unassigned");
    expect(getPlannerSlotCandidateFetchCount()).toBe(
      candidateFetchesBeforeClear + 1,
    );
    await user.keyboard("{Escape}");
  });

  it("renders only configured teams with their persisted display names", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const configuredDepth = resolvePlannerDepthIpcMock();
    configuredDepth.teams = configuredDepth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(configuredDepth);
    renderMyClubRoute();

    expect(
      await screen.findByRole("columnheader", { name: "First Team" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "U19" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: "Reserves" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "First Team" })).toBeNull();

    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(
      within(board).getByRole("columnheader", { name: "First Team" }),
    ).toBeInTheDocument();
    expect(
      within(board).getByRole("columnheader", { name: "U19" }),
    ).toBeInTheDocument();
    expect(
      within(board).queryByRole("columnheader", { name: "Reserves" }),
    ).toBeNull();

    await user.click(screen.getByRole("button", { name: "Clear all" }));
    expect(
      screen.getByRole("dialog", { name: "Clear all squads?" }),
    ).toHaveTextContent(
      "This clears every assignment from First Team and U19.",
    );
  });

  it("opens squad team management with the current configuration", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    expect(dialog).toHaveTextContent("Senior");
    expect(dialog).toHaveTextContent("Reserves");
    expect(dialog).toHaveTextContent("Youth");
    expect(within(dialog).getAllByRole("checkbox")).toHaveLength(3);
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeInTheDocument();
    const manageButton = screen.getByRole("button", { name: "Manage teams" });
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(manageButton).toHaveFocus();

    await user.click(manageButton);
    await screen.findByRole("dialog", { name: "Manage squad teams" });
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(manageButton).toHaveFocus();
  });

  it("renames teams and confirms populated team removal", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = withReserveGoalkeeper(resolvePlannerDepthIpcMock());
    setPlannerDepthIpcMock({
      ...depth,
      teams: depth.teams.map((team) => ({
        ...team,
        displayName:
          team.team === "senior"
            ? "First Team"
            : team.team === "reserves"
              ? "B Team"
              : "U19",
      })),
    });
    setPlannerTeamRemovalImpacts([
      {
        team: "reserves",
        displayName: "B Team",
        assignmentCount: 1,
        staffingTargets: [
          { jobId: "manager", jobLabel: "Manager", slotCount: 2 },
        ],
        strings: [],
      },
    ]);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    const confirmation = await screen.findByRole("dialog", {
      name: "Remove planner teams?",
    });
    expect(confirmation).toHaveTextContent(
      "B Team: 1 assignment; Manager: 2 slots",
    );
    expect(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    ).toHaveFocus();
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);
    await user.click(
      within(confirmation).getByRole("button", { name: "Cancel" }),
    );
    const managementDialog = screen.getByRole("dialog", {
      name: "Manage squad teams",
    });
    expect(managementDialog).toBeInTheDocument();
    expect(
      within(managementDialog).getByRole("button", { name: "Save teams" }),
    ).toHaveFocus();

    await user.click(
      within(managementDialog).getByRole("button", { name: "Save teams" }),
    );
    await screen.findByRole("dialog", { name: "Remove planner teams?" });
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(
        screen.getByRole("dialog", { name: "Manage squad teams" }),
      ).toBeInTheDocument(),
    );
    expect(
      within(
        screen.getByRole("dialog", { name: "Manage squad teams" }),
      ).getByRole("button", { name: "Save teams" }),
    ).toHaveFocus();

    await user.click(
      within(
        screen.getByRole("dialog", { name: "Manage squad teams" }),
      ).getByRole("button", { name: "Save teams" }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Remove planner teams?" }),
      ).getByRole("button", { name: "Remove teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(getPlannerTeamSaveIpcMockCalls()).toEqual([
      {
        teams: [
          {
            team: "senior",
            displayName: "First Team",
            strings: [{ id: 1, displayName: "1st string" }],
          },
          {
            team: "youth",
            displayName: "U19",
            strings: [{ id: 3, displayName: "1st string" }],
          },
        ],
        confirmPopulatedRemoval: true,
      },
    ]);
    expect(
      screen.getByRole("columnheader", { name: "First Team" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "U19" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: "B Team" })).toBeNull();
    expect(await screen.findByText("Team settings saved.")).toBeInTheDocument();
  });

  it("restores a removed team with a custom name and an empty string", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = resolvePlannerDepthIpcMock();
    depth.teams = depth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(depth);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    );
    await user.clear(
      within(dialog).getByRole("textbox", { name: "Reserves display name" }),
    );
    await user.type(
      within(dialog).getByRole("textbox", { name: "Reserves display name" }),
      "B Team",
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(getPlannerTeamSaveIpcMockCalls()).toEqual([
      {
        teams: [
          {
            team: "senior",
            displayName: "First Team",
            strings: [{ id: 1, displayName: "1st string" }],
          },
          {
            team: "reserves",
            displayName: "B Team",
            strings: [{ id: null, displayName: "1st string" }],
          },
          {
            team: "youth",
            displayName: "U19",
            strings: [{ id: 3, displayName: "1st string" }],
          },
        ],
        confirmPopulatedRemoval: false,
      },
    ]);
    expect(
      screen.getByRole("columnheader", { name: "B Team" }),
    ).toBeInTheDocument();
    const restoredDepth = resolvePlannerDepthIpcMock();
    const restoredReserves = restoredDepth.teams.find(
      (team) => team.team === "reserves",
    );
    expect(restoredReserves).toMatchObject({ displayName: "B Team" });
    expect(restoredReserves?.strings).toHaveLength(1);
    expect(restoredReserves?.strings[0]?.assignments).toEqual([]);
  });

  it("restores both missing teams with distinct string ids", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = resolvePlannerDepthIpcMock();
    depth.teams = depth.teams.filter((team) => team.team === "senior");
    setPlannerDepthIpcMock(depth);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    );
    await user.click(within(dialog).getByRole("checkbox", { name: "Youth" }));
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(
      screen.getByRole("columnheader", { name: "Reserves" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "Youth" }),
    ).toBeInTheDocument();
    const restoredDepth = resolvePlannerDepthIpcMock();
    const restoredIds = restoredDepth.teams
      .filter((team) => team.team !== "senior")
      .flatMap((team) => team.strings.map((plannerString) => plannerString.id));
    expect(restoredIds).toHaveLength(2);
    expect(new Set(restoredIds).size).toBe(2);
  });

  it("submits retained string ids, names, and orders unchanged", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = resolvePlannerDepthIpcMock();
    setPlannerDepthIpcMock({
      ...depth,
      teams: depth.teams.map((team) =>
        team.team === "senior"
          ? {
              ...team,
              strings: [
                {
                  id: 7,
                  stringOrder: 1,
                  displayName: "Second",
                  assignments: [],
                },
                {
                  id: 5,
                  stringOrder: 0,
                  displayName: "First",
                  assignments: [],
                },
              ],
            }
          : team,
      ),
    });
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(getPlannerTeamSaveIpcMockCalls()).toEqual([
      {
        teams: [
          {
            team: "senior",
            displayName: "Senior",
            strings: [
              { id: 5, displayName: "First" },
              { id: 7, displayName: "Second" },
            ],
          },
          {
            team: "reserves",
            displayName: "Reserves",
            strings: [{ id: 2, displayName: "1st string" }],
          },
          {
            team: "youth",
            displayName: "Youth",
            strings: [{ id: 3, displayName: "1st string" }],
          },
        ],
        confirmPopulatedRemoval: false,
      },
    ]);
  });

  it("renames a planner string while keeping its assignments", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(withDepthAssignments(resolvePlannerDepthIpcMock()));
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const renameField = within(dialog).getByRole("textbox", {
      name: "Senior string 1 name",
    });
    expect(renameField).toHaveValue("1st string");
    await user.clear(renameField);
    await user.type(renameField, "First Choice");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    const seniorSave = getPlannerTeamSaveIpcMockCalls().find((call) =>
      call.teams.some((team) => team.team === "senior"),
    );
    expect(
      seniorSave?.teams.find((team) => team.team === "senior")?.strings,
    ).toEqual([
      { id: 1, displayName: "First Choice" },
      { id: 4, displayName: "2nd string" },
    ]);
    expect(await screen.findByText("Team settings saved.")).toBeInTheDocument();
    const matrix = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(
      within(matrix).getByRole("columnheader", { name: "First Choice" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("button", { name: /Alex Keeper/ }),
    ).toBeInTheDocument();
  });

  it("reorders planner strings while keeping stable ids", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(
      withSecondSeniorString(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Move Senior string 2 up" }),
    );
    expect(
      within(dialog).getByRole("textbox", { name: "Senior string 1 name" }),
    ).toHaveValue("2nd string");
    expect(
      within(dialog).getByRole("textbox", { name: "Senior string 2 name" }),
    ).toHaveValue("1st string");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    const seniorSave = getPlannerTeamSaveIpcMockCalls().find((call) =>
      call.teams.some((team) => team.team === "senior"),
    );
    expect(
      seniorSave?.teams.find((team) => team.team === "senior")?.strings,
    ).toEqual([
      { id: 4, displayName: "2nd string" },
      { id: 1, displayName: "1st string" },
    ]);
  });

  it("confirms populated string removal by name and deletes only that string", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(withDepthAssignments(resolvePlannerDepthIpcMock()));
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove Senior string 1" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    const confirmation = await screen.findByRole("dialog", {
      name: "Remove planner strings?",
    });
    expect(confirmation).toHaveTextContent("1st string: 4 assignments");
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);
    await user.click(
      within(confirmation).getByRole("button", { name: "Remove strings" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(getPlannerTeamSaveIpcMockCalls()).toEqual([
      {
        teams: [
          {
            team: "senior",
            displayName: "Senior",
            strings: [{ id: 4, displayName: "2nd string" }],
          },
          {
            team: "reserves",
            displayName: "Reserves",
            strings: [{ id: 2, displayName: "1st string" }],
          },
          {
            team: "youth",
            displayName: "Youth",
            strings: [{ id: 3, displayName: "1st string" }],
          },
        ],
        confirmPopulatedRemoval: true,
      },
    ]);
  });

  it("confirms mixed team and string removal with combined wording", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = withSecondSeniorString(
      withReserveGoalkeeper(resolvePlannerDepthIpcMock()),
    );
    setPlannerDepthIpcMock({
      ...depth,
      teams: depth.teams.map((team) => ({
        ...team,
        displayName:
          team.team === "senior"
            ? "First Team"
            : team.team === "reserves"
              ? "B Team"
              : "U19",
      })),
    });
    setPlannerTeamRemovalImpacts([
      {
        team: "senior",
        displayName: "First Team",
        assignmentCount: 1,
        staffingTargets: [],
        strings: [
          { stringId: 1, displayName: "1st string", assignmentCount: 1 },
        ],
      },
      {
        team: "reserves",
        displayName: "B Team",
        assignmentCount: 1,
        staffingTargets: [
          { jobId: "manager", jobLabel: "Manager", slotCount: 2 },
        ],
        strings: [],
      },
    ]);
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove Senior string 1" }),
    );
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    const confirmation = await screen.findByRole("dialog", {
      name: "Remove planner teams and strings?",
    });
    expect(confirmation).toHaveTextContent(
      "Removing these teams and strings permanently deletes their assignments and staffing targets.",
    );
    expect(confirmation).toHaveTextContent(
      "B Team: 1 assignment; Manager: 2 slots",
    );
    expect(confirmation).toHaveTextContent("1st string: 1 assignment");
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);
    await user.click(
      within(confirmation).getByRole("button", {
        name: "Remove teams and strings",
      }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(getPlannerTeamSaveIpcMockCalls()).toEqual([
      {
        teams: [
          {
            team: "senior",
            displayName: "First Team",
            strings: [{ id: 4, displayName: "2nd string" }],
          },
          {
            team: "youth",
            displayName: "U19",
            strings: [{ id: 3, displayName: "1st string" }],
          },
        ],
        confirmPopulatedRemoval: true,
      },
    ]);
  });

  it("prefills the next ordinal default for a new planner string", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(
      withSecondSeniorString(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Add string to Senior" }),
    );
    expect(
      within(dialog).getByRole("textbox", { name: "Senior string 3 name" }),
    ).toHaveValue("3rd string");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    const seniorSave = getPlannerTeamSaveIpcMockCalls().find((call) =>
      call.teams.some((team) => team.team === "senior"),
    );
    expect(
      seniorSave?.teams.find((team) => team.team === "senior")?.strings,
    ).toEqual([
      { id: 1, displayName: "1st string" },
      { id: 4, displayName: "2nd string" },
      { id: null, displayName: "3rd string" },
    ]);
  });

  it("blocks invalid planner string names with field errors", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerDepthIpcMock(
      withSecondSeniorString(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const firstName = within(dialog).getByRole("textbox", {
      name: "Senior string 1 name",
    });
    await user.clear(firstName);
    expect(within(dialog).getByText("Enter a string name")).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();
    await user.type(firstName, "2ND STRING");
    expect(
      within(dialog).getAllByText("String names must be unique"),
    ).toHaveLength(2);
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();
    fireEvent.change(firstName, { target: { value: "x".repeat(41) } });
    expect(
      within(dialog).getByText("Use 40 characters or fewer"),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);
  });

  it("keeps team-management drafts on validation and backend failure", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerTeamSaveError("Team settings failed");
    renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const seniorName = within(dialog).getByRole("textbox", {
      name: "Senior display name",
    });
    await user.clear(seniorName);
    expect(within(dialog).getByText("Enter a team name")).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);

    await user.type(seniorName, "First Team");
    await user.clear(
      within(dialog).getByRole("textbox", { name: "Reserves display name" }),
    );
    await user.type(
      within(dialog).getByRole("textbox", { name: "Reserves display name" }),
      "First Team",
    );
    expect(
      within(dialog).getAllByText("Team names must be unique"),
    ).toHaveLength(2);
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);

    const reservesName = within(dialog).getByRole("textbox", {
      name: "Reserves display name",
    });
    fireEvent.change(reservesName, { target: { value: "x".repeat(41) } });
    expect(
      within(dialog).getByText("Use 40 characters or fewer"),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save teams" }),
    ).toBeDisabled();

    await user.clear(reservesName);
    await user.type(reservesName, "Reserves Team");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Team settings failed",
    );
    expect(
      within(
        screen.getByRole("dialog", { name: "Manage squad teams" }),
      ).getByRole("textbox", { name: "Senior display name" }),
    ).toHaveValue("First Team");
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(1);
  });

  it("prevents removing the final team and duplicate management saves", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const depth = resolvePlannerDepthIpcMock();
    depth.teams = depth.teams.filter((team) => team.team === "senior");
    setPlannerDepthIpcMock(depth);
    setPlannerTeamSavePending(true);
    const { queryClient } = renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    expect(
      within(dialog).getByRole("checkbox", { name: "Senior" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    ).not.toBeChecked();

    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );
    await waitFor(() =>
      expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(1),
    );
    expect(
      queryClient.isMutating({
        mutationKey: playerResultContextMutationKey,
      }),
    ).toBeGreaterThan(0);
    expect(
      within(dialog).getByRole("button", { name: "Saving…" }),
    ).toBeDisabled();
    await user.click(within(dialog).getByRole("button", { name: "Saving…" }));
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(1);
  });

  it("cancels a pending removal preview when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerTeamRemovalImpacts([]);
    setPlannerTeamRemovalImpactPending(true);
    const { queryClient } = renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );
    expect(
      within(dialog).getByRole("button", { name: "Checking…" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("textbox", { name: "Senior display name" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("checkbox", { name: "Senior" }),
    ).toBeDisabled();

    switchToSecondSave(queryClient);

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    await act(async () => {
      resolvePendingPlannerTeamRemovalImpact();
    });
    expect(getPlannerTeamSaveIpcMockCalls()).toHaveLength(0);
  });

  it("discards an open management draft when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const { queryClient } = renderMyClubRoute();

    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const seniorName = within(dialog).getByRole("textbox", {
      name: "Senior display name",
    });
    await user.clear(seniorName);
    await user.type(seniorName, "Draft Only");

    const nextDepth = resolvePlannerDepthIpcMock();
    nextDepth.teams = nextDepth.teams
      .filter((team) => team.team === "youth")
      .map((team) => ({ ...team, displayName: "Fresh Save Team" }));
    setPlannerDepthIpcMock(nextDepth);
    queryClient.setQueryData(plannerKeys.depth(), nextDepth);
    switchToSecondSave(queryClient);

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(
      await within(board).findByRole("columnheader", {
        name: "Fresh Save Team",
      }),
    ).toBeInTheDocument();
    expect(
      within(board).queryByRole("columnheader", { name: "Draft Only" }),
    ).toBeNull();
  });

  it("refetches picker candidates after team settings change", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Alex Keeper" }),
    ]);
    renderMyClubRoute({ staleTime: 60_000 });

    const seniorCell = await screen.findByRole("button", {
      name: /Senior, 1st string, IP: GK .* Empty/,
    });
    await user.click(seniorCell);
    expect(
      await screen.findByRole("option", { name: /Alex Keeper/ }),
    ).toBeInTheDocument();
    const fetchesBeforeSave = getPlannerSlotCandidateFetchCount();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(screen.getByRole("button", { name: "Manage teams" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const seniorName = within(dialog).getByRole("textbox", {
      name: "Senior display name",
    });
    await user.clear(seniorName);
    await user.type(seniorName, "First Team");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(
      await screen.findByRole("button", {
        name: /First Team, 1st string, IP: GK .* Empty/,
      }),
    );
    expect(
      await screen.findByRole("option", { name: /Alex Keeper/ }),
    ).toBeInTheDocument();
    expect(getPlannerSlotCandidateFetchCount()).toBeGreaterThan(
      fetchesBeforeSave,
    );
  });

  it("moves focus to management after removing a squad", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    renderMyClubRoute();

    await screen.findByRole("columnheader", { name: "Reserves" });
    await user.click(screen.getByRole("button", { name: "Manage teams" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    await user.click(
      within(dialog).getByRole("checkbox", { name: "Reserves" }),
    );
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );

    const manageButton = await screen.findByRole("button", {
      name: "Manage teams",
    });
    await waitFor(() => expect(manageButton).toHaveFocus());
    expect(screen.queryByRole("columnheader", { name: "Reserves" })).toBeNull();
    expect(
      screen.getByRole("columnheader", { name: "Senior" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "Youth" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "Reserves" })).toBeNull();
  });

  it("resets board state when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const previousDepth = resolvePlannerDepthIpcMock();
    previousDepth.teams = previousDepth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(previousDepth);
    const { queryClient } = renderMyClubRoute();

    const previousCell = await screen.findByRole("button", {
      name: /U19, 1st string, IP: GK .* Empty/,
    });
    await user.click(previousCell);
    expect(await screen.findByRole("dialog")).toBeInTheDocument();

    const nextDepth = resolvePlannerDepthIpcMock();
    nextDepth.teams = nextDepth.teams
      .filter((team) => team.team === "senior")
      .map((team) => ({ ...team, displayName: "Fresh Save Team" }));
    setPlannerDepthIpcMock(nextDepth);
    queryClient.setQueryData(plannerKeys.depth(), nextDepth);
    switchToSecondSave(queryClient);

    expect(
      await screen.findByRole("columnheader", {
        name: "Fresh Save Team",
      }),
    ).toBeInTheDocument();
    const board = await screen.findByRole("region", {
      name: "Squad depth board",
    });
    expect(
      within(board).queryByRole("columnheader", { name: "U19" }),
    ).toBeNull();
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(document.activeElement).not.toBe(previousCell);
  });

  it("uses configured display names for picker assignment locations", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const configuredDepth = withDepthAssignments(resolvePlannerDepthIpcMock());
    configuredDepth.teams = configuredDepth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(configuredDepth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Alex Keeper" }),
    ]);
    renderMyClubRoute();

    const target = await screen.findByRole("button", {
      name: /U19, 1st string, IP: GK .* Empty/,
    });
    await user.click(target);
    const candidate = await screen.findByRole("option", {
      name: /Alex Keeper/,
    });
    expect(candidate).toHaveTextContent(
      `Assigned: First Team · 1st string · ${KEEPER_POSITION}`,
    );
    await user.click(candidate);
    expect(
      screen.getByRole("dialog", { name: "Move Alex Keeper?" }),
    ).toHaveTextContent(
      `Move Alex Keeper from First Team · 1st string · ${KEEPER_POSITION} to U19 · 1st string · ${KEEPER_POSITION}?`,
    );
  });

  it("prevents duplicate clear-all requests while confirmation is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerClearAllPending(true);
    renderMyClubRoute();

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    const optimizedDepth = withReserveGoalkeeper(resolvePlannerDepthIpcMock());
    setPlannerOptimizeDepth(optimizedDepth);
    setPlannerSlotCandidates([
      slotCandidate({ playerUid: 77, name: "Reserve Keeper" }),
    ]);
    renderMyClubRoute({ staleTime: 60_000 });

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
    expect(
      screen.getByRole("button", { name: /Reserve Keeper, Resolved/ }),
    ).toBeInTheDocument();
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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerOptimizeError("Optimize failed");
    renderMyClubRoute();

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerOptimizePending(true);
    renderMyClubRoute();

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
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    setPlannerOptimizePending(true);
    renderMyClubRoute();

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

describe("Suggested Training column", () => {
  async function renderConfiguredSquad(players: SquadPlayer[]) {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(players);
    renderMyClubRoute({ initialEntry: "/my-club" });
    return screen.findByRole("table", { name: "Squad overview" });
  }

  it("shows Suggested Training as the default far-right Squad column", async () => {
    const table = await renderConfiguredSquad([
      {
        ...squadPlayerNamed("Alex Scout", 42),
        dynamicValues: { height: 188 },
      },
    ]);

    const headerLabels = within(table)
      .getAllByRole("columnheader")
      .map((header) => header.getAttribute("aria-label"));
    expect(headerLabels).toContain("Height");
    expect(headerLabels).toContain("Suggested Training");
    expect(headerLabels[headerLabels.length - 1]).toBe("Suggested Training");
    expect(within(table).getByText("188 cm")).toBeInTheDocument();
  });

  it("renders the focus name, and a dash with an accessible name for null", async () => {
    usePlayerTableStore.getState().addColumns("squad", ["attr.Acceleration"]);
    const table = await renderConfiguredSquad([
      {
        ...squadPlayerNamed("Focused Scout", 42),
        dynamicValues: { "attr.Acceleration": 16 },
        suggestedTraining: "Quickness",
      },
      {
        ...squadPlayerNamed("Unassigned Scout", 43),
        dynamicValues: { "attr.Acceleration": 14 },
        suggestedTraining: null,
      },
    ]);

    const focusedRow = within(table).getByText("Focused Scout").closest("tr");
    if (!focusedRow) {
      throw new Error("Expected the focused player row.");
    }
    const focusCell = within(focusedRow).getByText("Quickness").closest("td");
    expect(focusCell).toHaveAccessibleName("Quickness");

    const unassignedRow = within(table)
      .getByText("Unassigned Scout")
      .closest("tr");
    if (!unassignedRow) {
      throw new Error("Expected the unassigned player row.");
    }
    const dashCell = within(unassignedRow).getByRole("cell", {
      name: "No suggested training",
    });
    expect(dashCell).toHaveAccessibleName("No suggested training");

    await waitFor(() => {
      expect(getLastSquadPlayersArgs()).toMatchObject({
        requestedFields: ["attr.Acceleration", "height"],
      });
    });
    expect(
      ((getLastSquadPlayersArgs()?.requestedFields as string[]) ?? []).includes(
        "suggested_training",
      ),
    ).toBe(false);
  });

  it("never changes the sort when the Suggested Training header is clicked", async () => {
    const user = userEvent.setup();
    const table = await renderConfiguredSquad([
      squadPlayerNamed("Alex Scout", 42),
    ]);

    const header = within(table).getByRole("columnheader", {
      name: "Suggested Training",
    });
    expect(header).not.toHaveAttribute("aria-sort");
    const button = within(header).getByRole("button", {
      name: "Suggested Training",
    });
    expect(button).toHaveAttribute("title", "Suggested Training");
    await user.click(button);

    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
  });

  it("removes and re-adds Suggested Training through the header menu", async () => {
    const user = userEvent.setup();
    const table = await renderConfiguredSquad([
      squadPlayerNamed("Alex Scout", 42),
    ]);

    const header = within(table).getByRole("columnheader", {
      name: "Suggested Training",
    });
    fireEvent.contextMenu(header);
    await user.click(
      screen.getByRole("menuitem", { name: "Remove Suggested Training" }),
    );
    expect(
      within(table).queryByRole("columnheader", {
        name: "Suggested Training",
      }),
    ).toBeNull();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");

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
    await user.click(
      screen.getByRole("option", { name: "Suggested Training" }),
    );

    expect(
      await within(table).findByRole("columnheader", {
        name: "Suggested Training",
      }),
    ).toBeInTheDocument();
    expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
      "suggested_training",
    );
  });
});

describe("squad table toolbar", () => {
  async function renderToolbarSquad(players: SquadPlayer[]) {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
      sources: [],
    });
    setSquadPlayersOverride(players);
    renderMyClubRoute({ initialEntry: "/my-club" });
    return screen.findByRole("table", { name: "Squad overview" });
  }

  it("associates the squad summary and grouped Columns in one table toolbar", async () => {
    await renderToolbarSquad([squadPlayerNamed("Alex Scout", 42)]);

    const toolbar = screen.getByRole("toolbar", {
      name: "Squad results toolbar",
    });
    expect(
      within(toolbar).getByText(/players? · sorted by/),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Columns" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).queryByRole("button", { name: "Edit filters" }),
    ).toBeNull();
    expect(
      within(toolbar).queryByRole("button", { name: "Clear all" }),
    ).toBeNull();
    const table = screen.getByRole("table", { name: "Squad overview" });
    expect(
      toolbar.compareDocumentPosition(table) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  it("keeps squad boosts outside the generic toolbar", async () => {
    await renderToolbarSquad([squadPlayerNamed("Alex Scout", 42)]);

    const toolbar = screen.getByRole("toolbar", {
      name: "Squad results toolbar",
    });
    for (const name of ["Boost all CA", "Make all Wonderkids"]) {
      expect(screen.getByRole("button", { name })).toBeInTheDocument();
      expect(within(toolbar).queryByRole("button", { name })).toBeNull();
    }
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
              {
                id: 4,
                stringOrder: 1,
                displayName: "2nd string",
                assignments: [],
              },
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
              {
                id: 4,
                stringOrder: 1,
                displayName: "2nd string",
                assignments: [],
              },
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
              {
                id: 4,
                stringOrder: 1,
                displayName: "2nd string",
                assignments: [],
              },
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
                {
                  id: 4,
                  stringOrder: 1,
                  displayName: "2nd string",
                  assignments: [],
                },
              ]
            : team.strings.map((plannerString) => ({
                ...plannerString,
                assignments: [assignment],
              })),
      };
    }),
  };
}
