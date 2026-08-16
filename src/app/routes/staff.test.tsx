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
  resolvePendingMyStaffBoostIpcMock,
  sendPendingMyStaffBoostProgressIpcMock,
  setMyStaffBoostIpcMockMode,
  setStaffFamilyConfigured,
  setStaffListIpcMockMode,
  setStaffOverride,
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

  it("adds Staff to navigation and opens Search with all default role columns", async () => {
    await resolveLoadDataIpcMock();
    renderStaffRoute();

    const staffLink = await screen.findByRole("link", { name: "Staff" });
    expect(staffLink).toHaveAttribute("aria-current", "page");
    expect(
      await screen.findByRole("heading", { name: "Staff" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Search" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(within(table).getAllByRole("columnheader")).toHaveLength(25);
    expect(
      within(table).getByRole("columnheader", { name: "Coach — Goalkeeping" }),
    ).toBeInTheDocument();
    expect(within(table).getByText("Alex Coach")).toBeInTheDocument();
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

  it("retains independent Search and My Staff sort state", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { router } = renderStaffRoute();
    const searchTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    await user.click(within(searchTable).getByRole("button", { name: "Name" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
        searchSort: "name",
        searchDir: "asc",
      });
    });

    await user.click(screen.getByRole("tab", { name: "My Staff" }));
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
      myStaffSort: "ca",
      myStaffDir: "desc",
    });
    await user.click(within(myStaffTable).getByRole("button", { name: "PA" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "pa",
        dir: "desc",
        searchSort: "name",
        searchDir: "asc",
        myStaffSort: "pa",
        myStaffDir: "desc",
      });
    });

    await user.click(screen.getByRole("tab", { name: "Search" }));
    await screen.findByRole("table", { name: "Staff search results" });
    expect(router.state.location.search).toMatchObject({
      sort: "name",
      dir: "asc",
      searchSort: "name",
      searchDir: "asc",
      myStaffSort: "pa",
      myStaffDir: "desc",
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

  it("opens a staff profile from a My Staff row with Enter", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");
    const table = await screen.findByRole("table", {
      name: "My Staff overview",
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

  it("supports keyboard workspace tabs without changing the table contract", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute();
    await screen.findByRole("table", { name: "Staff search results" });
    const searchTab = screen.getByRole("tab", { name: "Search" });
    searchTab.focus();
    fireEvent.keyDown(searchTab.parentElement as HTMLElement, {
      key: "ArrowRight",
    });
    await user.click(screen.getByRole("tab", { name: "My Staff" }));
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    expect(within(myStaffTable).getAllByRole("columnheader")).toHaveLength(25);
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

  it("uses the shared score ramp for role scores in Search and My Staff", async () => {
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
    const user = userEvent.setup();
    renderStaffRoute();

    const searchTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(searchTable).getByRole("img", {
        name: "Assistant Manager role score: 20, Weak",
      }),
    ).toHaveClass("text-score-1");
    expect(
      within(searchTable).getByRole("img", {
        name: "Coach — Attacking Technical role score: 50, Average",
      }),
    ).toHaveClass("text-score-2");
    expect(
      within(searchTable).getByRole("img", {
        name: "Coach — Attacking Tactical role score: 70, Good",
      }),
    ).toHaveClass("text-score-3");
    expect(
      within(searchTable).getByRole("img", {
        name: "Coach — Defending Technical role score: 90, Excellent",
      }),
    ).toHaveClass("text-score-4");

    await user.click(screen.getByRole("tab", { name: "My Staff" }));
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    expect(
      within(myStaffTable).getByRole("img", {
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

  it("keeps My Staff layout changes separate from Search", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    fireEvent.contextMenu(
      within(myStaffTable).getByRole("columnheader", { name: "Scout" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Remove Scout" }));
    expect(
      within(myStaffTable).queryByRole("columnheader", { name: "Scout" }),
    ).toBeNull();

    await user.click(screen.getByRole("tab", { name: "Search" }));
    const searchTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(searchTable).getByRole("columnheader", { name: "Scout" }),
    ).toBeInTheDocument();
  });

  it("distinguishes an unconfigured club family from an empty overview", async () => {
    await resolveLoadDataIpcMock();
    setStaffFamilyConfigured(false);
    renderStaffRoute("/staff?view=my-staff");
    expect(
      await screen.findByText("Set up your club family", { exact: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Club Setup" }),
    ).toHaveAttribute("href", "/#club-setup");
  });

  it("offers one bulk CA boost on My Staff and no row actions", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");

    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    expect(
      within(myStaffTable).queryByRole("columnheader", { name: "Actions" }),
    ).toBeNull();
    expect(
      within(myStaffTable).queryByRole("button", { name: "Boost CA" }),
    ).toBeNull();
    expect(screen.getByRole("button", { name: "Boost all CA" })).toBeEnabled();

    await user.click(screen.getByRole("tab", { name: "Search" }));
    const searchTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(searchTable).queryByRole("columnheader", { name: "Actions" }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: "Boost all CA" })).toBeNull();
  });

  it("confirms and reports a configured-family bulk CA boost", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { queryClient } = renderStaffRoute("/staff?view=my-staff");
    queryClient.setQueryData(["staff", "probe"], []);
    queryClient.setQueryData(["snapshot", "probe"], []);
    await screen.findByRole("table", {
      name: "My Staff overview",
    });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", { name: "Boost all CA?" });
    expect(dialog).toHaveTextContent(
      "every eligible staff member in your configured club family",
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
    const { queryClient } = renderStaffRoute("/staff?view=my-staff");

    await screen.findByRole("table", { name: "My Staff overview" });
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
    setMyStaffBoostIpcMockMode("error");
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");

    await screen.findByRole("table", { name: "My Staff overview" });
    await user.click(screen.getByRole("button", { name: "Boost all CA" }));
    const dialog = await screen.findByRole("dialog", { name: "Boost all CA?" });
    await user.click(
      within(dialog).getByRole("button", { name: "Boost all CA" }),
    );

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Could not boost My Staff. Bridge is unavailable.",
    );
    expect(dialog).toBeInTheDocument();
  });
});
