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
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import {
  defaultPlayerTableLayouts,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import {
  resolvePendingPlannerTeamSaveIpcMock,
  setPlannerDepthError,
  setPlannerTeamRemovalImpacts,
  setPlannerTeamSavePending,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaff,
  fixtureStaffAssignmentOptimization,
  fixtureStaffAssignmentTargets,
  getLastStaffArgs,
  getLastStaffAssignmentOptimizerIpcArgs,
  getLastStaffAssignmentTargetsIpcArgs,
  getMyStaffBoostIpcMockCalls,
  rejectPendingMyStaffBoostIpcMock,
  resolvePendingMyStaffBoostIpcMock,
  sendPendingMyStaffBoostProgressIpcMock,
  setMyStaffBoostIpcMockMode,
  setStaffAssignmentOptimizationIpcMock,
  setStaffAssignmentTargetsIpcMock,
  setStaffFamilyConfigured,
  setStaffListIpcMockMode,
  setStaffOverride,
  setStaffShortlistOverride,
} from "@/testing/staff-ipc-mock";
import {
  getLastStaffShortlistImportIpcArgs,
  setStaffShortlistImportIpcMockResult,
} from "@/testing/staff-shortlist-import-ipc-mock";

const { openCsvDialog } = vi.hoisted(() => ({ openCsvDialog: vi.fn() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openCsvDialog }));

function renderStaffRoute(initialEntry = "/staff") {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [initialEntry] }),
  });
  return {
    router,
    queryClient,
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    ),
  };
}

describe("staff route", () => {
  beforeEach(() => {
    openCsvDialog.mockReset();
    usePlayerTableStore.setState({ layouts: defaultPlayerTableLayouts() });
  });

  it("adds Staff Search to navigation and opens Search-only with all default role columns", async () => {
    await resolveLoadDataIpcMock();
    renderStaffRoute();

    const staffLink = await screen.findByRole("link", { name: "Staff Search" });
    expect(staffLink).toHaveAttribute("aria-current", "page");
    expect(
      await screen.findByRole("heading", { name: "Staff Search" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("tablist")).toBeNull();

    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(within(table).getAllByRole("columnheader")).toHaveLength(26);
    expect(
      within(table).getByRole("columnheader", { name: "Coach — Goalkeeping" }),
    ).toBeInTheDocument();
    expect(within(table).getByText("Alex Coach")).toBeInTheDocument();
  });

  it("keeps projected shortlist columns sortable without showing job metadata", async () => {
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([
      fixtureStaff({
        shortlist: {
          preferredJob: "Technical Director",
          clubJob: "Technical Director",
          coachingQualifications: "Continental Pro",
        },
      }),
    ]);
    const user = userEvent.setup();
    const { router } = renderStaffRoute("/staff?shortlistOnly=true");

    const allJobsTable = await screen.findByRole("table", {
      name: "Staff Shortlist",
    });
    await user.click(
      within(allJobsTable).getByRole("button", { name: "Name" }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistSort: "name",
        shortlistDir: "asc",
      });
    });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Preferred Job" }),
      "Technical Director",
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        preferredJob: "Technical Director",
        shortlistSort: "name",
        shortlistDir: "asc",
        shortlistContextSort: "role.technical_director",
        shortlistContextDir: "desc",
      });
    });
    const table = await screen.findByRole("table", { name: "Staff Shortlist" });
    expect(
      within(table).queryByRole("columnheader", { name: "Preferred Job" }),
    ).toBeNull();
    expect(
      within(table).queryByRole("columnheader", { name: "Club Job" }),
    ).toBeNull();
    await user.click(
      within(table).getByRole("button", { name: "Coaching Qualifications" }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistSort: "name",
        shortlistDir: "asc",
        shortlistContextSort: "coaching_qualifications",
        shortlistContextDir: "asc",
      });
    });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Preferred Job" }),
      "",
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistSort: "name",
        shortlistDir: "asc",
      });
      expect(router.state.location.search).not.toHaveProperty(
        "shortlistContextSort",
      );
      expect(router.state.location.search).not.toHaveProperty(
        "shortlistContextDir",
      );
    });
    expect(
      within(
        await screen.findByRole("table", { name: "Staff Shortlist" }),
      ).getByRole("columnheader", { name: "Name" }),
    ).toHaveAttribute("aria-sort", "ascending");
  });

  it("does not truncate a staff member's Age / DOB cell", async () => {
    await resolveLoadDataIpcMock();
    renderStaffRoute();

    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    const cell = within(table).getAllByTitle(/\(44\)$/)[0];
    expect(cell).toHaveClass("whitespace-nowrap");
    expect(cell).not.toHaveClass("truncate");
  });

  it("normalizes invalid view and sort state to Search and CA", async () => {
    await resolveLoadDataIpcMock();
    const { router } = renderStaffRoute(
      "/staff?view=wrong&sort=role.not_real&dir=sideways",
    );
    await screen.findByRole("table", { name: "Staff search results" });
    expect(router.state.location.search).toMatchObject({
      view: "search",
      sort: "ca",
      dir: "desc",
    });
  });

  it("keeps standalone Staff Search sorting in sync with its URL state", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute("/staff");
    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });

    await user.click(within(table).getByRole("button", { name: "Name" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
        searchSort: "name",
        searchDir: "asc",
      });
    });
    const sortedTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(sortedTable).getByRole("columnheader", { name: "Name" }),
    ).toHaveAttribute("aria-sort", "ascending");
  });

  it("retains managed-club Staff sort state in My Club", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute("/my-club?view=staff");
    const myStaffTable = await screen.findByRole("table", {
      name: "Staff overview",
    });
    expect(router.state.location.search).toMatchObject({
      view: "staff",
    });
    await user.click(within(myStaffTable).getByRole("button", { name: "PA" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "staff",
        staffSort: "pa",
        staffDir: "desc",
      });
    });
  });

  it("opens a staff profile from a Search row", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute(
      "/staff?sort=name&dir=asc&filters=%5B%5D&combine=and",
    );
    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    const row = within(table)
      .getAllByRole("row")
      .find((item) => item.hasAttribute("data-index"));
    expect(row).toBeDefined();
    await user.click(row as HTMLElement);
    expect(
      await screen.findByRole("heading", { name: "Alex Coach" }),
    ).toBeInTheDocument();
    expect(router.history.location.pathname).toBe("/staff/101");
    expect(router.history.location.search).toBe("");
  });

  it("does not load the staff table while opening a profile", async () => {
    await resolveLoadDataIpcMock();
    setStaffListIpcMockMode("error");
    renderStaffRoute("/staff/101");
    expect(
      await screen.findByRole("heading", { name: "Alex Coach" }),
    ).toBeInTheDocument();
  });

  it("opens a staff profile from a managed-club Staff row with Enter", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/my-club?view=staff");
    const table = await screen.findByRole("table", {
      name: "Staff overview",
    });
    const row = within(table)
      .getAllByRole("row")
      .find((item) => item.hasAttribute("data-index"));
    expect(row).toBeDefined();
    (row as HTMLElement).focus();
    await user.keyboard("{Enter}");
    expect(
      await screen.findByRole("heading", { name: "Alex Coach" }),
    ).toBeInTheDocument();
  });

  it("supports keyboard My Club workspace tabs without changing the table contract", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/my-club?view=staff");
    await screen.findByRole("table", { name: "Staff overview" });
    const squadTab = screen.getByRole("tab", { name: "Squad" });
    squadTab.focus();
    fireEvent.keyDown(squadTab.parentElement as HTMLElement, {
      key: "ArrowRight",
    });
    await user.click(screen.getByRole("tab", { name: "Staff" }));
    const myStaffTable = await screen.findByRole("table", {
      name: "Staff overview",
    });
    expect(within(myStaffTable).getAllByRole("columnheader")).toHaveLength(26);
    expect(within(myStaffTable).getByText("Alex Coach")).toBeInTheDocument();
  });

  it("renders missing scores as em dashes instead of zero", async () => {
    await resolveLoadDataIpcMock();
    setStaffOverride([
      {
        ...fixtureStaff(),
        dynamicValues: {},
      },
    ]);
    renderStaffRoute();
    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(within(table).getAllByText("—").length).toBeGreaterThan(0);
    expect(
      within(table).queryByRole("img", { name: /role score:/ }),
    ).toBeNull();
  });

  it("uses the shared score ramp for role scores in managed-club Staff", async () => {
    await resolveLoadDataIpcMock();
    const staff = fixtureStaff();
    setStaffOverride([
      {
        ...staff,
        dynamicValues: {
          ...staff.dynamicValues,
          "role.assistant_manager": 20,
          "role.coach_attacking_technical": 50,
          "role.coach_attacking_tactical": 70,
          "role.coach_defending_technical": 90,
        },
      },
    ]);
    renderStaffRoute("/my-club?view=staff");

    const staffTable = await screen.findByRole("table", {
      name: "Staff overview",
    });
    expect(
      within(staffTable).getByRole("img", {
        name: "Assistant Manager role score: 20, Weak",
      }),
    ).toHaveClass("text-score-1");
    expect(
      within(staffTable).getByRole("img", {
        name: "Coach — Attacking Technical role score: 50, Average",
      }),
    ).toHaveClass("text-score-2");
    expect(
      within(staffTable).getByRole("img", {
        name: "Coach — Attacking Tactical role score: 70, Good",
      }),
    ).toHaveClass("text-score-3");
    expect(
      within(staffTable).getByRole("img", {
        name: "Coach — Defending Technical role score: 90, Excellent",
      }),
    ).toHaveClass("text-score-4");
  });

  it("can remove and re-add a staff role column through the shared picker", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute();
    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });

    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Scout" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Remove Scout" }));
    expect(
      within(table).queryByRole("columnheader", { name: "Scout" }),
    ).toBeNull();

    const updatedTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    fireEvent.contextMenu(
      within(updatedTable).getByRole("columnheader", { name: "Name" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "scout",
    );
    await user.click(screen.getByRole("option", { name: "Scout" }));
    const finalTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(finalTable).getByRole("columnheader", { name: "Scout" }),
    ).toBeInTheDocument();
  });

  it("does not claim role scores are unavailable when no role column is requested", async () => {
    await resolveLoadDataIpcMock();
    setStaffOverride([{ ...fixtureStaff(), dynamicValues: {} }]);
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        "staff-search": { columnIds: ["name", "ca"], widths: {} },
      },
    });
    renderStaffRoute();
    await screen.findByRole("table", { name: "Staff search results" });
    expect(screen.queryByText(/Staff role scores are unavailable/)).toBeNull();
  });

  it("keeps managed-club Staff layout changes on its stable table ID", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const staffSearchColumns = [
      ...usePlayerTableStore.getState().layouts["staff-search"].columnIds,
    ];
    renderStaffRoute("/my-club?view=staff");
    const myStaffTable = await screen.findByRole("table", {
      name: "Staff overview",
    });
    fireEvent.contextMenu(
      within(myStaffTable).getByRole("columnheader", { name: "Scout" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Remove Scout" }));
    expect(
      within(myStaffTable).queryByRole("columnheader", { name: "Scout" }),
    ).toBeNull();
    expect(
      usePlayerTableStore.getState().layouts["staff-search"].columnIds,
    ).toEqual(staffSearchColumns);
  });

  it("distinguishes an unconfigured managed club from an empty Staff overview", async () => {
    await resolveLoadDataIpcMock();
    setStaffFamilyConfigured(false);
    renderStaffRoute("/my-club?view=staff");
    expect(
      await screen.findByText("Choose your managed club", { exact: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toHaveAttribute("href", "/my-club#managed-club");
  });

  it("describes an empty managed-club Staff overview as one managed club", async () => {
    await resolveLoadDataIpcMock();
    setStaffOverride([]);
    renderStaffRoute("/my-club?view=staff");

    expect(
      await screen.findByText(
        "No current-snapshot staff match your managed club.",
      ),
    ).toBeInTheDocument();
  });

  it("offers one bulk CA boost on managed-club Staff and no row actions", async () => {
    await resolveLoadDataIpcMock();
    renderStaffRoute("/my-club?view=staff");

    const myStaffTable = await screen.findByRole("table", {
      name: "Staff overview",
    });
    expect(
      within(myStaffTable).queryByRole("columnheader", { name: "Actions" }),
    ).toBeNull();
    expect(
      within(myStaffTable).queryByRole("button", { name: "Boost CA" }),
    ).toBeNull();
    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeEnabled();
  });

  it("confirms and reports a managed-club bulk CA boost", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { queryClient } = renderStaffRoute("/my-club?view=staff");
    queryClient.setQueryData(["staff", "probe"], []);
    queryClient.setQueryData(["snapshot", "probe"], []);
    await screen.findByRole("table", {
      name: "Staff overview",
    });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", { name: "Boost all CA?" });
    expect(dialog).toHaveTextContent(
      "every eligible staff member at your managed club",
    );
    expect(dialog).toHaveTextContent("Each boost stops at PA or 200.");
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );
    expect(
      within(dialog).getByRole("button", { name: "Boosting…" }),
    ).toBeDisabled();
    expect(dialog).toHaveTextContent("0 of 2 staff processed.");
    sendPendingMyStaffBoostProgressIpcMock();
    await waitFor(() =>
      expect(dialog).toHaveTextContent("1 of 2 staff processed."),
    );
    expect(getMyStaffBoostIpcMockCalls()).toHaveLength(1);
    expect(getMyStaffBoostIpcMockCalls()[0]).toHaveProperty("onProgress");
    resolvePendingMyStaffBoostIpcMock();

    expect(await screen.findByRole("status")).toHaveTextContent(
      "2 processed — 2 updated, 0 skipped, 0 failed.",
    );
    expect(queryClient.getQueryState(["staff", "probe"])?.isInvalidated).toBe(
      true,
    );
    expect(
      queryClient.getQueryState(["snapshot", "probe"])?.isInvalidated,
    ).toBe(true);
  });

  it("clears bulk recovery after Load Data establishes a new snapshot", async () => {
    await resolveLoadDataIpcMock();
    setMyStaffBoostIpcMockMode("recoveryRequired");
    const user = userEvent.setup();
    const { queryClient } = renderStaffRoute("/my-club?view=staff");

    await screen.findByRole("table", { name: "Staff overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    await user.click(
      within(
        await screen.findByRole("dialog", { name: "Boost all CA?" }),
      ).getByRole("button", { name: "Boost all CA" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Load Data is required before another boost.",
    );
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeDisabled();

    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
    );
    if (!snapshot) throw new Error("Expected a current Staff snapshot");
    queryClient.setQueryData<SnapshotSummary>(
      currentSnapshotQueryOptions.queryKey,
      { ...snapshot, id: snapshot.id + 1 },
    );

    await waitFor(() => {
      expect(screen.queryByRole("status")).toBeNull();
      expect(
        screen.getByRole("button", { name: "Boost all CA" }),
      ).toBeEnabled();
    });
  });

  it("keeps a global bulk bridge error in the confirmation", async () => {
    await resolveLoadDataIpcMock();
    setMyStaffBoostIpcMockMode("pending");
    const user = userEvent.setup();
    renderStaffRoute("/my-club?view=staff");

    await screen.findByRole("table", { name: "Staff overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", { name: "Boost all CA?" });
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );
    sendPendingMyStaffBoostProgressIpcMock();
    await waitFor(() =>
      expect(dialog).toHaveTextContent("1 of 2 staff processed."),
    );
    rejectPendingMyStaffBoostIpcMock(new Error("Bridge is unavailable."));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Could not boost Staff. Bridge is unavailable.",
    );
    expect(within(dialog).getByRole("alert")).toHaveTextContent(
      "1 processed — 1 updated, 0 skipped, 0 failed.",
    );
    expect(dialog).toBeInTheDocument();
  });

  it("keeps shortlist upload, Configure Club Staff, and Optimize visible with conditional metadata filters", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute("/staff");

    expect(
      await screen.findByRole("button", { name: "Upload CSV" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Configure Club Staff" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("combobox", { name: "Preferred Job" }),
    ).toBeNull();
    expect(
      screen.queryByRole("checkbox", { name: "Only unemployed" }),
    ).toBeNull();

    await user.click(
      await screen.findByRole("switch", { name: "Shortlist: Off" }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistOnly: true,
      });
    });
    expect(
      await screen.findByRole("combobox", { name: "Preferred Job" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("checkbox", { name: "Only unemployed" }),
    ).toBeInTheDocument();
  });

  it("round-trips the shortlist toggle through the URL and refetches flagged search", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute("/staff");

    await user.click(
      await screen.findByRole("switch", { name: "Shortlist: Off" }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistOnly: true,
      });
    });
    expect(
      await screen.findByRole("switch", { name: "Shortlist: On" }),
    ).toHaveAttribute("aria-checked", "true");
    await waitFor(() => {
      expect(getLastStaffArgs()).toMatchObject({ shortlistOnly: true });
    });

    await user.click(screen.getByRole("switch", { name: "Shortlist: On" }));
    await waitFor(() => {
      expect(router.state.location.search.shortlistOnly).toBeUndefined();
    });
    expect(
      await screen.findByRole("switch", { name: "Shortlist: Off" }),
    ).toHaveAttribute("aria-checked", "false");
  });

  it("treats an invalid shortlist toggle value as off", async () => {
    await resolveLoadDataIpcMock();
    renderStaffRoute("/staff?shortlistOnly=maybe");

    expect(
      await screen.findByRole("switch", { name: "Shortlist: Off" }),
    ).toHaveAttribute("aria-checked", "false");
    expect(
      await screen.findByRole("table", { name: "Staff search results" }),
    ).toBeInTheDocument();
  });

  it("turns shortlist filtering on after a successful upload and resets metadata filters", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    openCsvDialog.mockResolvedValue("C:\\exports\\staff.csv");
    setStaffShortlistImportIpcMockResult({
      totalStaff: 3,
      storedStaff: 2,
      skippedStaff: 1,
    });
    const { router } = renderStaffRoute(
      "/staff?preferredJob=Coach&unemployedOnly=true",
    );

    await user.click(await screen.findByRole("button", { name: "Upload CSV" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Upload Staff Shortlist CSV",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Choose CSV" }),
    );

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        shortlistOnly: true,
      });
    });
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty("preferredJob");
    });
    expect(router.state.location.search.unemployedOnly).toBe(false);
    await waitFor(() => {
      expect(getLastStaffArgs()).toMatchObject({ shortlistOnly: true });
    });
    expect(getLastStaffShortlistImportIpcArgs()).toEqual({
      path: "C:\\exports\\staff.csv",
    });
    expect(
      await screen.findByText("Stored 2 of 3 staff IDs; 1 skipped."),
    ).toBeInTheDocument();
  });

  it("keeps Staff Search usable when Planner context fails", async () => {
    await resolveLoadDataIpcMock();
    setPlannerDepthError("Planner unavailable");
    renderStaffRoute();

    expect(
      await screen.findByRole("table", { name: "Staff search results" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Upload CSV" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Optimize assignments" }),
    ).toBeDisabled();
  });

  it("adds applied shortlist filter metrics to the visible layout", async () => {
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([fixtureStaff()]);
    const user = userEvent.setup();
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        "staff-search": { columnIds: ["name"], widths: {} },
        "staff-shortlist": { columnIds: ["name"], widths: {} },
      },
    });
    const filters = encodeURIComponent(
      JSON.stringify([{ id: "ca", field: "ca", op: "gt", value: 0 }]),
    );
    renderStaffRoute(`/staff?shortlistOnly=true&filters=${filters}`);

    await screen.findByRole("table", { name: "Staff Shortlist" });
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    await user.click(screen.getByRole("button", { name: "Done" }));

    await waitFor(() => {
      expect(
        usePlayerTableStore.getState().layouts["staff-shortlist"].columnIds,
      ).toContain("ca");
    });
    expect(
      usePlayerTableStore.getState().layouts["staff-search"].columnIds,
    ).toEqual(["name"]);
  });

  it("shows setup feedback with Upload available when filtering on with no list", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setStaffAssignmentOptimizationIpcMock(
      fixtureStaffAssignmentOptimization({ state: "no_shortlist" }),
    );
    renderStaffRoute("/staff?shortlistOnly=true");

    expect(
      await screen.findByText("No Staff Shortlist uploaded"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Upload CSV" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("switch", { name: "Shortlist: On" }),
    ).toHaveAttribute("aria-checked", "true");

    await user.click(
      screen.getByRole("button", { name: "Optimize assignments" }),
    );
    expect(
      await screen.findByText(
        "Upload a Staff Shortlist before optimizing assignments.",
      ),
    ).toBeInTheDocument();
  });

  it("shows shortlist-filter guidance when the preferred job excludes every row and escapes via the switch", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([
      fixtureStaff({
        shortlist: {
          preferredJob: "Coach",
          clubJob: "-",
          coachingQualifications: "Continental A",
        },
      }),
    ]);
    const { router } = renderStaffRoute(
      "/staff?shortlistOnly=true&preferredJob=Manager",
    );

    expect(
      await screen.findByText("No shortlist staff match these filters"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Adjust or clear filters to widen the results."),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("switch", { name: "Shortlist: On" }),
    ).toHaveAttribute("aria-checked", "true");

    await user.click(screen.getByRole("switch", { name: "Shortlist: On" }));
    await waitFor(() => {
      expect(router.state.location.search.shortlistOnly).toBeUndefined();
    });
    expect(
      await screen.findByRole("table", { name: "Staff search results" }),
    ).toBeInTheDocument();
  });

  it("describes a core-filtered empty shortlist as a filter mismatch", async () => {
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([fixtureStaff()]);
    const filters = encodeURIComponent(
      JSON.stringify([{ id: "ca", field: "ca", op: "gt", value: 200 }]),
    );
    renderStaffRoute(`/staff?shortlistOnly=true&filters=${filters}`);

    expect(
      await screen.findByText("No shortlist staff match these filters"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Adjust or clear filters to widen the results."),
    ).toBeInTheDocument();
  });

  it("opens a staff profile from a shortlisted row", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setStaffShortlistOverride([fixtureStaff()]);
    const { router } = renderStaffRoute("/staff?shortlistOnly=true");

    const table = await screen.findByRole("table", {
      name: "Staff Shortlist",
    });
    const row = within(table)
      .getAllByRole("row")
      .find((item) => item.hasAttribute("data-index"));
    expect(row).toBeDefined();
    await user.click(row as HTMLElement);
    expect(
      await screen.findByRole("heading", { name: "Alex Coach" }),
    ).toBeInTheDocument();
    expect(router.history.location.pathname).toBe("/staff/101");
  });

  it("optimizes without shortlist presentation filters", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    renderStaffRoute(
      "/staff?shortlistOnly=true&preferredJob=Coach&unemployedOnly=true",
    );

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

  it("uses the route context for complete slot saves and token replacement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams[1] = { ...targets.teams[1], displayName: "B Squad" };
    setStaffAssignmentTargetsIpcMock(targets);
    const { queryClient } = renderStaffRoute("/staff?shortlistOnly=true");

    await user.click(
      await screen.findByRole("button", { name: "Configure Club Staff" }),
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
      await screen.findByRole("button", { name: "Configure Club Staff" }),
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
    const configureClubStaff = await screen.findByRole("button", {
      name: "Configure Club Staff",
    });
    expect(configureClubStaff).toHaveFocus();
    await user.click(configureClubStaff);
    expect(
      screen.getAllByRole("spinbutton", { name: "Assistant Manager slots" })[0],
    ).toHaveValue(0);
  });

  it("renders standalone Club sections through Configure Club Staff without Senior", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams = targets.teams.filter(({ team }) => team !== "senior");
    targets.targets = targets.targets.filter(({ scope }) => scope !== "senior");
    setStaffAssignmentTargetsIpcMock(targets);
    renderStaffRoute("/staff?shortlistOnly=true");

    await user.click(
      await screen.findByRole("button", { name: "Configure Club Staff" }),
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

  it("suppresses recommendations during a pending Planner team save and recovers after resolve", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerTeamRemovalImpacts([]);
    setPlannerTeamSavePending(true);
    const { queryClient, router } = renderStaffRoute(
      "/staff?shortlistOnly=true",
    );

    await user.click(
      await screen.findByRole("button", { name: "Optimize assignments" }),
    );
    expect(
      await screen.findByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).toBeInTheDocument();

    router.history.push("/my-club?view=planner");
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
        queryClient.isMutating({
          mutationKey: playerResultContextMutationKey,
        }),
      ).toBeGreaterThan(0),
    );

    router.history.push("/staff?shortlistOnly=true");
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Optimize assignments" }),
      ).toBeDisabled(),
    );
    expect(
      screen.queryByRole("table", {
        name: "Staff assignment recommendations and vacancies",
      }),
    ).not.toBeInTheDocument();

    resolvePendingPlannerTeamSaveIpcMock();
    router.history.push("/my-club?view=planner");
    expect(
      await screen.findByRole("tab", { name: "First Team" }),
    ).toBeInTheDocument();
    router.history.push("/staff?shortlistOnly=true");
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

  it("formats Staff wage and labels the internal job value explicitly", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        "staff-search": {
          columnIds: ["name", "wage", "job_id"],
          widths: {},
        },
      },
    });
    renderStaffRoute();

    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(table).getByRole("columnheader", { name: "Job ID" }),
    ).toBeInTheDocument();
    expect(within(table).getAllByText("€15k")).toHaveLength(2);
  });

  it("carries a legacy generic sort into the shortlist sort and result order", async () => {
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([fixtureStaff()]);
    const { router } = renderStaffRoute(
      "/staff?view=shortlist&sort=name&dir=asc",
    );

    // Legacy links replace-normalize to Staff Search with filtering on while
    // the generic legacy sort becomes the shortlist sort.
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "search",
        shortlistOnly: true,
        shortlistSort: "name",
        shortlistDir: "asc",
      });
    });
    expect(router.state.location.href).not.toContain("view=shortlist");
    await screen.findByRole("table", { name: "Staff Shortlist" });
    await waitFor(() => {
      expect(getLastStaffArgs()).toMatchObject({
        shortlistOnly: true,
        sortBy: "name",
        sortDir: "asc",
      });
    });
  });

  it("keeps explicit shortlist sort keys ahead of a legacy generic sort", async () => {
    await resolveLoadDataIpcMock();
    setStaffShortlistOverride([fixtureStaff()]);
    const { router } = renderStaffRoute(
      "/staff?view=shortlist&sort=ca&dir=desc&shortlistSort=name&shortlistDir=asc",
    );

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "search",
        shortlistOnly: true,
        shortlistSort: "name",
        shortlistDir: "asc",
      });
    });
    await screen.findByRole("table", { name: "Staff Shortlist" });
    await waitFor(() => {
      expect(getLastStaffArgs()).toMatchObject({
        shortlistOnly: true,
        sortBy: "name",
        sortDir: "asc",
      });
    });
  });
});
