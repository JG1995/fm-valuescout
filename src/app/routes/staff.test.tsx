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
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import {
  defaultPlayerTableLayouts,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaff,
  getMyStaffBoostIpcMockCalls,
  rejectPendingMyStaffBoostIpcMock,
  resolvePendingMyStaffBoostIpcMock,
  sendPendingMyStaffBoostProgressIpcMock,
  setMyStaffBoostIpcMockMode,
  setStaffFamilyConfigured,
  setStaffListIpcMockMode,
  setStaffOverride,
  setStaffShortlistOverride,
} from "@/testing/staff-ipc-mock";

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
    useLayoutStore.setState({ railExpanded: true });
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
    const { router } = renderStaffRoute("/my-club?view=staff-shortlist");

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
});
