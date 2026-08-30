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
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import type { MyClubWorkspace } from "@/features/my-club/components/my-club-workspace-tabs";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
import type { PlannerRoleReference } from "@/features/planner/types/role-reference";
import type { PlannerTactic } from "@/features/planner/types/tactic";
import {
  phasePositionLabel,
  validateTacticDraft,
} from "@/features/planner/utils/tactic-editor";
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
  getPlannerAddStringIpcMockCalls,
  getPlannerClearAllIpcMockCalls,
  getPlannerDepthIpcMockCalls,
  getPlannerOptimizeIpcMockBases,
  getPlannerOptimizeIpcMockCalls,
  getPlannerRoleReferenceCalls,
  getPlannerSlotCandidateFetchCount,
  getPlannerTeamSaveIpcMockCalls,
  observeManagedClubSaveCall,
  resolvePendingManagedClubSave,
  resolvePendingPlannerTeamRemovalImpact,
  resolvePendingPlannerTeamSaveIpcMock,
  resolvePlannerDepthIpcMock,
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
  resolveSavePlannerClubFamilyIpcMock,
  setManagedClubIpcMock,
  setManagedClubOptionsError,
  setManagedClubSavePending,
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
  setPlannerRoleReference,
  setPlannerRoleReferenceError,
  setPlannerSlotCandidates,
  setPlannerTacticIpcMock,
  setPlannerTacticSaveError,
  setPlannerTeamRemovalImpactPending,
  setPlannerTeamRemovalImpacts,
  setPlannerTeamSaveError,
  setPlannerTeamSavePending,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
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
import {
  fixtureStaff,
  fixtureStaffAssignmentTargets,
  getLastStaffAssignmentOptimizerIpcArgs,
  getLastStaffAssignmentTargetsIpcArgs,
  setStaffAssignmentTargetsIpcMock,
  setStaffOverride,
  setStaffShortlistOverride,
} from "@/testing/staff-ipc-mock";

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
    staff: "Staff",
    "staff-shortlist": "Staff Shortlist",
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

function manyStaff(count: number) {
  return Array.from({ length: count }, (_, index) =>
    fixtureStaff({
      uid: index + 1,
      name: `Staff member ${String(index + 1).padStart(3, "0")}`,
    }),
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

describe("My Club route", () => {
  it("exposes the five My Club workspaces in order", async () => {
    await resolveLoadDataIpcMock();
    renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByRole("tab", { name: "Staff" }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "Squad",
      "Planner",
      "Tactic",
      "Staff",
      "Staff Shortlist",
    ]);
  });

  it("renders managed-club Staff inside My Club", async () => {
    await resolveLoadDataIpcMock();
    renderMyClubRoute({ initialEntry: "/my-club?view=staff" });

    expect(await screen.findByRole("tab", { name: "Staff" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      await screen.findByRole("table", { name: "Staff overview" }),
    ).toBeInTheDocument();
  });

  it("places Configure slots only in Staff Shortlist", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderMyClubRoute({ initialEntry: "/my-club?view=staff" });

    await screen.findByRole("table", { name: "Staff overview" });
    expect(
      screen.queryByRole("button", { name: "Configure slots" }),
    ).toBeNull();
    await openMyClubWorkspace(user, "staff-shortlist");
    expect(
      await screen.findByRole("button", { name: "Configure slots" }),
    ).toBeInTheDocument();
  });

  it("optimizes without shortlist presentation filters", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderMyClubRoute({
      initialEntry:
        "/my-club?view=staff-shortlist&preferredJob=Coach&unemployedOnly=true",
    });

    await user.click(
      await screen.findByRole("button", { name: "Optimize assignments" }),
    );

    await waitFor(() =>
      expect(getLastStaffAssignmentOptimizerIpcArgs()).toEqual({
        expectedSaveContextToken: "save-token-1",
        expectedSnapshotContextToken: "snapshot-token-1",
      }),
    );
  });

  it("suppresses recommendations during an actual pending Planner team save", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerTeamRemovalImpacts([]);
    setPlannerTeamSavePending(true);
    const { queryClient } = renderMyClubRoute({
      initialEntry: "/my-club?view=staff-shortlist",
    });

    await user.click(
      await screen.findByRole("button", { name: "Optimize assignments" }),
    );
    expect(
      await screen.findByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).toBeInTheDocument();

    await openMyClubWorkspace(user, "planner");
    await user.click(
      await screen.findByRole("button", { name: "Manage teams" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Manage squad teams",
    });
    const seniorDisplayName = within(dialog).getByRole("textbox", {
      name: "Senior display name",
    });
    await user.clear(seniorDisplayName);
    await user.type(seniorDisplayName, "First Team");
    await user.click(
      within(dialog).getByRole("button", { name: "Save teams" }),
    );
    await waitFor(() =>
      expect(
        queryClient.isMutating({ mutationKey: playerResultContextMutationKey }),
      ).toBeGreaterThan(0),
    );

    await openMyClubWorkspace(user, "staff-shortlist");
    expect(
      screen.queryByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeDisabled();

    resolvePendingPlannerTeamSaveIpcMock();
    await openMyClubWorkspace(user, "planner");
    expect(
      await screen.findByRole("tab", { name: "First Team" }),
    ).toBeInTheDocument();
    await openMyClubWorkspace(user, "staff-shortlist");
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

  it("uses the route context for complete slot saves and token replacement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams[1] = { ...targets.teams[1], displayName: "B Squad" };
    setStaffAssignmentTargetsIpcMock(targets);
    const { queryClient } = renderMyClubRoute({
      initialEntry: "/my-club?view=staff-shortlist",
    });

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Configure assignment slots",
    });
    expect(within(dialog).getByText("B Squad")).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );
    await waitFor(() =>
      expect(getLastStaffAssignmentTargetsIpcArgs()).toEqual(
        expect.objectContaining({ expectedSaveContextToken: "save-token-1" }),
      ),
    );
    expect(
      (
        getLastStaffAssignmentTargetsIpcArgs() as
          | { targets?: unknown[] }
          | undefined
      )?.targets,
    ).toHaveLength(28);

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const reopened = await screen.findByRole("dialog");
    const assistantManager = within(reopened).getAllByRole("spinbutton", {
      name: "Assistant Manager slots",
    })[0];
    await user.clear(assistantManager);
    await user.type(assistantManager, "12");
    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      snapshotKeys.current(),
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot");
    }
    queryClient.setQueryData<SnapshotSummary | null>(
      snapshotKeys.current(),
      () => ({ ...snapshot, contextToken: "snapshot-token-replacement" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    const configureSlots = await screen.findByRole("button", {
      name: "Configure slots",
    });
    expect(configureSlots).toHaveFocus();
    await user.click(configureSlots);
    expect(
      screen.getAllByRole("spinbutton", { name: "Assistant Manager slots" })[0],
    ).toHaveValue(0);
  });

  it("renders standalone Club sections through the Staff Shortlist route without Senior", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams = targets.teams.filter(({ team }) => team !== "senior");
    targets.targets = targets.targets.filter(({ scope }) => scope !== "senior");
    setStaffAssignmentTargetsIpcMock(targets);
    renderMyClubRoute({ initialEntry: "/my-club?view=staff-shortlist" });

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Configure assignment slots",
    });
    const club = within(dialog).getByRole("group", { name: "Club" });

    expect(within(dialog).queryByRole("group", { name: "Senior" })).toBeNull();
    expect(
      within(club).getByRole("group", { name: "Recruitment" }),
    ).toBeInTheDocument();
  });

  it.each([
    {
      workspace: "staff",
      caption: "Staff overview",
      scrollerTestId: "my-staff-results-scroller",
      setStaff: setStaffOverride,
    },
    {
      workspace: "staff-shortlist",
      caption: "Staff Shortlist",
      scrollerTestId: "staff-shortlist-results-scroller",
      setStaff: setStaffShortlistOverride,
    },
  ] as const)(
    "keeps $caption inside its virtualized workspace",
    async ({ workspace, caption, scrollerTestId, setStaff }) => {
      await resolveLoadDataIpcMock();
      setStaff(manyStaff(101));
      renderMyClubRoute({ initialEntry: `/my-club?view=${workspace}` });

      const table = await screen.findByRole("table", { name: caption });
      const panel = document.getElementById(
        `my-club-workspace-panel-${workspace}`,
      );
      expect(panel).toHaveClass("flex", "min-h-0", "flex-1", "flex-col");

      const scroller = screen.getByTestId(scrollerTestId);
      expect(scroller).toHaveClass("h-full", "min-h-0", "overflow-auto");
      expect(scroller.parentElement).toHaveClass(
        "relative",
        "min-h-0",
        "flex-1",
      );

      const virtualRows = within(table)
        .getAllByRole("row")
        .filter((row) => row.hasAttribute("data-index"));
      expect(virtualRows.length).toBeGreaterThan(0);
      expect(virtualRows.length).toBeLessThan(101);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
      status: "available",
      unclassifiedPlayerCount: 0,
    });
    setPlannerAvailableClubs(["Barcelona"]);
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
    await waitFor(() => expect(defineButton).toBeEnabled());

    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries");
    await user.click(defineButton);
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
    await user.click(defineButton);
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
    await user.click(defineButton);
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

    await user.click(defineButton);
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
      status: "missing",
      unclassifiedPlayerCount: 0,
    });
    setPlannerAvailableClubs(["Legacy FC", "Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
        status: "available",
        unclassifiedPlayerCount: 0,
      });
      setPlannerAvailableClubs(["Second FC"]);
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
        status: "available",
        unclassifiedPlayerCount: 0,
      });
    });
  });

  it("defaults to Squad and keeps Planner and Tactic mounted", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderMyClubRoute({ initialEntry: "/my-club" });

    await screen.findByRole("link", { name: "Open Managed Club" });
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
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toHaveAttribute("href", "/my-club#managed-club");
    const tacticPanel = document.getElementById(
      "my-club-workspace-panel-tactic",
    );
    const plannerPanel = document.getElementById(
      "my-club-workspace-panel-planner",
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
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([squadPlayerNamed("Alex Scout", 42)]);
    renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    for (const column of [
      "Name",
      "Age / DOB",
      "Nationality",
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
      within(table).getByRole("link", { name: /Alex Scout/ }),
    ).toHaveAttribute("href", "/players/42");
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
        requestedFields: ["attr.Acceleration"],
      });
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "attr.Acceleration",
    );
  });

  it("renders nullable Club DNA scores through ScoreBadge", async () => {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
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
    expect(within(unavailableRow).getByText("—")).toBeInTheDocument();
  });

  it("reorders Squad columns from the menu without changing its query, virtual row, or widths", async () => {
    const user = userEvent.setup();
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
    renderMyClubRoute({ initialEntry: "/my-club" });

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
    fireEvent.contextMenu(accelerationHeader);
    await user.click(screen.getByRole("menuitem", { name: "Move right" }));

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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
    await user.click(within(table).getByRole("button", { name: "Name" }));

    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        squadSort: "name",
        squadDir: "asc",
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

  it("retains A until a stale cached Squad sort refetch succeeds", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
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
    await user.click(within(table).getByRole("button", { name: "Name" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Name" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    await user.click(within(table).getByRole("button", { name: "CA" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "CA" }),
      ).toHaveAttribute("aria-sort", "descending"),
    );
    const cachedNameSort = queryClient
      .getQueryCache()
      .findAll({ queryKey: squadKeys.playerPages() })
      .find((query) => {
        const descriptor = query.queryKey.at(-1);
        return (
          typeof descriptor === "object" &&
          descriptor !== null &&
          (descriptor as { sortBy?: unknown }).sortBy === "name"
        );
      });
    if (!cachedNameSort) {
      throw new Error("expected a cached Squad name sort");
    }
    await queryClient.invalidateQueries({ queryKey: cachedNameSort.queryKey });
    setSquadPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Name" }));

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
        within(table).getByRole("columnheader", { name: "Name" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
  });

  it("retains committed Squad rows, blocks stale activation, and promotes only the latest sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
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
    await user.click(within(table).getByRole("button", { name: "Name" }));

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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
    await user.click(within(table).getByRole("button", { name: "Name" }));

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
    expect(
      within(retainedRow).queryByRole("link", { name: "Zara" }),
    ).toBeNull();
    fireEvent.click(retainedRow);
    retainedRow.focus();
    await user.keyboard("{Enter}");
    expect(router.state.location.pathname).toBe("/my-club");

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Name" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
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
    expect(missingContextRow).toHaveTextContent("No context");
    expect(missingContextRow).not.toHaveTextContent("—");
    expect(missingContextRow).not.toHaveTextContent(" · ");
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
      primaryClub: "Metro FC",
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
      primaryClub: "Metro FC",
      sources: [],
    });
    setSquadPlayersOverride([
      squadPlayerNamed("Zara Scout", 42),
      squadPlayerNamed("Alex Scout", 43),
    ]);
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    const table = await screen.findByRole("table", {
      name: "Squad overview",
    });
    await user.click(within(table).getByRole("button", { name: "Name" }));
    await waitFor(() => {
      expect(router.state.location.search).toEqual({
        squadSort: "name",
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
        squadSort: "name",
        squadDir: "asc",
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
    const { router } = renderMyClubRoute({ initialEntry: "/my-club" });

    expect(
      await screen.findByText("Managed club: Barcelona"),
    ).toBeInTheDocument();
    const squadTab = screen.getByRole("tab", { name: "Squad" });
    expect(squadTab).toHaveAttribute("aria-selected", "true");
    squadTab.focus();
    await user.keyboard("{End}");

    const shortlistTab = screen.getByRole("tab", { name: "Staff Shortlist" });
    expect(shortlistTab).toHaveAttribute("aria-selected", "true");
    expect(shortlistTab).toHaveFocus();
    expect(shortlistTab).toHaveAttribute("tabIndex", "0");
    expect(squadTab).toHaveAttribute("tabIndex", "-1");
    expect(router.state.location.search).toEqual({ view: "staff-shortlist" });
    await user.keyboard("{ArrowLeft}");
    const staffTab = screen.getByRole("tab", { name: "Staff" });
    expect(staffTab).toHaveAttribute("aria-selected", "true");
    expect(staffTab).toHaveFocus();
    expect(staffTab).toHaveAttribute("tabIndex", "0");
    expect(shortlistTab).toHaveAttribute("tabIndex", "-1");
    expect(router.state.location.search).toEqual({ view: "staff" });
    staffTab.focus();
    await user.keyboard("{Home}");
    expect(squadTab).toHaveAttribute("aria-selected", "true");
    expect(squadTab).toHaveFocus();
    expect(squadTab).toHaveAttribute("tabIndex", "0");
    expect(staffTab).toHaveAttribute("tabIndex", "-1");
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
    renderMyClubRoute({ initialEntry: "/my-club?view=planner" });

    expect(await screen.findByRole("tab", { name: "Planner" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    const matrix = screen.getByRole("region", {
      name: "Senior squad depth matrix",
    });
    expect(matrix).toBeVisible();
    expect(
      within(matrix)
        .getAllByRole("row")
        .slice(1)
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
    setPlannerAvailableClubs(["Barcelona"]);
    renderMyClubRoute({ initialEntry: "/my-club?view=clubs" });

    expect(await screen.findByRole("tab", { name: "Squad" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toBeVisible();
  });

  it("edits linked IP and OOP lanes with filtered roles and weight control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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

  it("orders the tactic command bar, pitches, and one settings shelf", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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

    await openMyClubWorkspace(user, "planner");
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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    expect(
      within(mcGroup as HTMLElement)
        .getAllByRole("button")
        .map((button) => button.getAttribute("aria-label")),
    ).toEqual([
      "IP: MCL · Box-to-Box Midfielder",
      "IP: MC · Advanced Playmaker",
      "IP: MCR · Central Midfielder",
    ]);
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
      gridColumn: "3 / span 2",
      gridRow: "1",
    });
    expect(
      defensiveMidfielder.closest('[data-position-group="DM"]'),
    ).toHaveStyle({ gridColumn: "3 / span 6" });
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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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

  it("keeps phase controls in one inspector for each tactic view", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderMyClubRoute({ initialEntry: "/my-club?view=tactic" });

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
    setPlannerAvailableClubs(["Barcelona"]);
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

  it("blocks two lanes from using the same sided placement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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

    expect(screen.getByRole("alert")).toHaveTextContent(
      "MCL is already used in the In-Possession phase.",
    );
    expect(screen.getByRole("button", { name: "Save tactic" })).toBeDisabled();
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    renderMyClubRoute();

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
    renderMyClubRoute();

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

  it("opens the best role fit reference from the Planner toolbar", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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

  it("groups all teams in one semantic table when the matrix fits", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerDepthIpcMock(
      withSecondStringForEveryTeam(resolvePlannerDepthIpcMock()),
    );
    renderMyClubRoute();
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
    renderMyClubRoute();
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
    renderMyClubRoute();
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    renderMyClubRoute({ staleTime: 60_000 });

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
    renderMyClubRoute({ staleTime: 60_000 });

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
    renderMyClubRoute();

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
    renderMyClubRoute();

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
    renderMyClubRoute();

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
    renderMyClubRoute();

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
    renderMyClubRoute();

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
    renderMyClubRoute();

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

  it("renders only configured teams with their persisted display names", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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
      await screen.findByRole("tab", { name: "First Team" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "U19" })).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "Reserves" })).toBeNull();

    await setPlannerMatrixWidth(2_000);
    const matrix = await screen.findByRole("region", {
      name: "All squads depth matrix",
    });
    expect(
      within(matrix).getByRole("columnheader", { name: "First Team squad" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).getByRole("columnheader", { name: "U19 squad" }),
    ).toBeInTheDocument();
    expect(
      within(matrix).queryByRole("columnheader", { name: "Reserves squad" }),
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
          { team: "senior", displayName: "First Team" },
          { team: "youth", displayName: "U19" },
        ],
        confirmPopulatedRemoval: true,
      },
    ]);
    expect(screen.getByRole("tab", { name: "First Team" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "U19" })).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "B Team" })).toBeNull();
    expect(await screen.findByText("Team settings saved.")).toBeInTheDocument();
  });

  it("restores a removed team with a custom name and an empty string", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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
          { team: "senior", displayName: "First Team" },
          { team: "reserves", displayName: "B Team" },
          { team: "youth", displayName: "U19" },
        ],
        confirmPopulatedRemoval: false,
      },
    ]);
    expect(screen.getByRole("tab", { name: "B Team" })).toBeInTheDocument();
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    expect(screen.getByRole("tab", { name: "Reserves" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Youth" })).toBeInTheDocument();
    const restoredDepth = resolvePlannerDepthIpcMock();
    const restoredIds = restoredDepth.teams
      .filter((team) => team.team !== "senior")
      .flatMap((team) => team.strings.map((plannerString) => plannerString.id));
    expect(restoredIds).toHaveLength(2);
    expect(new Set(restoredIds).size).toBe(2);
  });

  it("keeps team-management drafts on validation and backend failure", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(
      await screen.findByRole("tab", { name: "Fresh Save Team" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "Draft Only" })).toBeNull();
  });

  it("refetches picker candidates after team settings change", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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

  it("keeps one team selected and moves focus after removing the selected team", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderMyClubRoute();

    const reservesTab = await screen.findByRole("tab", { name: "Reserves" });
    await user.click(reservesTab);
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

    const youthTab = await screen.findByRole("tab", { name: "Youth" });
    await waitFor(() => expect(youthTab).toHaveFocus());
    expect(screen.queryByRole("tab", { name: "Reserves" })).toBeNull();
    expect(screen.getByRole("tab", { name: "Senior" })).toBeInTheDocument();
  });

  it("returns focus to management after removing a selected team in the combined layout", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderMyClubRoute();

    await user.click(await screen.findByRole("tab", { name: "Reserves" }));
    await setPlannerMatrixWidth(800);
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
    expect(
      screen.getByRole("columnheader", { name: "Senior squad" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "Youth squad" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: "Reserves squad" }),
    ).toBeNull();
  });

  it("cycles keyboard team selection through only the available teams", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const configuredDepth = resolvePlannerDepthIpcMock();
    configuredDepth.teams = configuredDepth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(configuredDepth);
    renderMyClubRoute();

    const firstTeamTab = await screen.findByRole("tab", { name: "First Team" });
    firstTeamTab.focus();
    await user.keyboard("{ArrowRight}");
    expect(screen.getByRole("tab", { name: "U19" })).toHaveFocus();
    await user.keyboard("{ArrowRight}");
    expect(firstTeamTab).toHaveFocus();
    await user.keyboard("{End}");
    expect(screen.getByRole("tab", { name: "U19" })).toHaveFocus();
    await user.keyboard("{Home}");
    expect(firstTeamTab).toHaveFocus();
  });

  it("keeps keyboard team selection on the only available team", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const configuredDepth = resolvePlannerDepthIpcMock();
    configuredDepth.teams = configuredDepth.teams
      .filter((team) => team.team === "senior")
      .map((team) => ({ ...team, displayName: "First Team" }));
    setPlannerDepthIpcMock(configuredDepth);
    renderMyClubRoute();

    const firstTeamTab = await screen.findByRole("tab", { name: "First Team" });
    firstTeamTab.focus();
    await user.keyboard("{ArrowRight}{ArrowLeft}{Home}{End}");
    expect(firstTeamTab).toHaveFocus();
    expect(
      within(
        screen.getByRole("tablist", { name: "Squad planner teams" }),
      ).getAllByRole("tab"),
    ).toHaveLength(1);
  });

  it("resets matrix state when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const previousDepth = resolvePlannerDepthIpcMock();
    previousDepth.teams = previousDepth.teams
      .filter((team) => team.team !== "reserves")
      .map((team) => ({
        ...team,
        displayName: team.team === "senior" ? "First Team" : "U19",
      }));
    setPlannerDepthIpcMock(previousDepth);
    const { queryClient } = renderMyClubRoute();

    const previousTeamTab = await screen.findByRole("tab", { name: "U19" });
    await user.click(previousTeamTab);
    await user.click(
      await screen.findByRole("button", {
        name: /U19, 1st string, IP: GK .* Empty/,
      }),
    );
    expect(await screen.findByRole("dialog")).toBeInTheDocument();

    const nextDepth = resolvePlannerDepthIpcMock();
    nextDepth.teams = nextDepth.teams
      .filter((team) => team.team === "senior")
      .map((team) => ({ ...team, displayName: "Fresh Save Team" }));
    setPlannerDepthIpcMock(nextDepth);
    queryClient.setQueryData(plannerKeys.depth(), nextDepth);
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

    expect(
      await screen.findByRole("tab", { name: "Fresh Save Team" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "U19" })).toBeNull();
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(document.activeElement).not.toBe(previousTeamTab);
  });

  it("uses configured display names for picker assignment locations", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
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

    await user.click(await screen.findByRole("tab", { name: "U19" }));
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
    setPlannerAvailableClubs(["Barcelona"]);
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
