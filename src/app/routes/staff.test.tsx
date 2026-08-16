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
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import {
  defaultPlayerTableLayouts,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaff,
  getStaffBoostIpcMockCalls,
  resolvePendingStaffBoostIpcMock,
  setStaffBoostIpcMockMode,
  setStaffFamilyConfigured,
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

  it("keeps Search rows non-interactive until the profile route is delivered", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute();
    const table = await screen.findByRole("table", {
      name: "Staff search results",
    });
    const row = within(table)
      .getAllByRole("row")
      .find((item) => item.hasAttribute("data-index"));
    expect(row).toBeDefined();
    expect(row).not.toHaveAttribute("tabindex");
    await user.click(row as HTMLElement);
    expect(screen.getByRole("heading", { name: "Staff" })).toBeInTheDocument();
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

  it("adds a fixed Boost CA action to My Staff only", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");

    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    expect(
      within(myStaffTable).getByRole("columnheader", { name: "Actions" }),
    ).toBeInTheDocument();
    const alexRow = within(myStaffTable)
      .getAllByRole("row")
      .find((row) => within(row).queryByText("Alex Coach"));
    expect(alexRow).toBeDefined();
    expect(
      within(alexRow as HTMLElement).getByRole("button", { name: "Boost CA" }),
    ).toBeEnabled();

    await user.click(screen.getByRole("tab", { name: "Search" }));
    const searchTable = await screen.findByRole("table", {
      name: "Staff search results",
    });
    expect(
      within(searchTable).queryByRole("columnheader", { name: "Actions" }),
    ).toBeNull();
  });

  it("confirms a UID-only fixed +10 boost and refreshes the row", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const { queryClient } = renderStaffRoute("/staff?view=my-staff");
    queryClient.setQueryData(["staff", "probe"], []);
    queryClient.setQueryData(["snapshot", "probe"], []);
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    const alexRow = within(myStaffTable)
      .getAllByRole("row")
      .find((row) => within(row).queryByText("Alex Coach"));
    expect(alexRow).toBeDefined();

    await user.click(
      within(alexRow as HTMLElement).getByRole("button", { name: "Boost CA" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Boost CA?" });
    expect(dialog).toHaveTextContent("CA 145 → 155 (+10)");
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));

    await waitFor(() => {
      expect(getStaffBoostIpcMockCalls()).toEqual([{ uid: 101 }]);
      expect(within(myStaffTable).getByText("155")).toBeInTheDocument();
    });
    expect(queryClient.getQueryState(["staff", "probe"])?.isInvalidated).toBe(
      true,
    );
    expect(
      queryClient.getQueryState(["snapshot", "probe"])?.isInvalidated,
    ).toBe(true);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Staff CA boosted from 145 to 155.",
    );
  });

  it("keeps boost feedback and focus when a sorted row leaves the virtual window", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    const staff = Array.from({ length: 35 }, (_, index) =>
      index === 20
        ? fixtureStaff({ ca: 120, pa: 200 })
        : fixtureStaff({
            uid: 1000 + index,
            name: `Staff ${index}`,
            ca: 100 + index,
            pa: 200,
          }),
    );
    setStaffOverride(staff);
    renderStaffRoute("/staff?view=my-staff&sort=ca&dir=asc");

    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    const alexRow = within(myStaffTable)
      .getAllByRole("row")
      .find((row) => within(row).queryByText("Alex Coach"));
    expect(alexRow).toBeDefined();
    await user.click(
      within(alexRow as HTMLElement).getByRole("button", { name: "Boost CA" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Boost CA?" });
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));

    await waitFor(() => {
      expect(
        screen.getByText("Staff CA boosted from 120 to 130."),
      ).toBeInTheDocument();
      expect(screen.getByTestId("staff-boost-outcome")).toHaveFocus();
    });
  });

  it("previews a PA cap and disables the action at the limit", async () => {
    await resolveLoadDataIpcMock();
    const user = userEvent.setup();
    setStaffOverride([
      fixtureStaff({ ca: 155, pa: 160 }),
      fixtureStaff({ uid: 102, name: "Jordan Analyst", ca: 160, pa: 160 }),
    ]);
    renderStaffRoute("/staff?view=my-staff");

    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    const rows = within(myStaffTable)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const cappedRow = rows.find((row) => within(row).queryByText("Alex Coach"));
    const limitRow = rows.find((row) =>
      within(row).queryByText("Jordan Analyst"),
    );
    expect(cappedRow).toBeDefined();
    expect(limitRow).toBeDefined();
    const cappedButton = within(cappedRow as HTMLElement).getByRole("button", {
      name: "Boost CA",
    });
    expect(cappedButton).toHaveAttribute(
      "title",
      "CA 155 → 160 (+5) · capped by PA",
    );
    await user.click(cappedButton);
    expect(
      await screen.findByRole("dialog", { name: "Boost CA?" }),
    ).toHaveTextContent("CA 155 → 160 (+5)");
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    const limitButton = within(limitRow as HTMLElement).getByRole("button", {
      name: "Boost CA",
    });
    expect(limitButton).toBeDisabled();
    expect(limitButton).toHaveAttribute(
      "title",
      "Current ability is already at this staff member’s potential ability.",
    );
  });

  it("keeps the confirmation open and explains recovery-required failures", async () => {
    await resolveLoadDataIpcMock();
    setStaffBoostIpcMockMode("snapshotSyncError");
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    const alexRow = within(myStaffTable)
      .getAllByRole("row")
      .find((row) => within(row).queryByText("Alex Coach"));
    await user.click(
      within(alexRow as HTMLElement).getByRole("button", { name: "Boost CA" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Boost CA?" });
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Load Data required",
    );
    expect(dialog).toBeInTheDocument();
  });

  it("locks the row while a staff boost is pending", async () => {
    await resolveLoadDataIpcMock();
    setStaffBoostIpcMockMode("pending");
    const user = userEvent.setup();
    renderStaffRoute("/staff?view=my-staff");
    const myStaffTable = await screen.findByRole("table", {
      name: "My Staff overview",
    });
    const alexRow = within(myStaffTable)
      .getAllByRole("row")
      .find((row) => within(row).queryByText("Alex Coach"));
    await user.click(
      within(alexRow as HTMLElement).getByRole("button", { name: "Boost CA" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Boost CA?" });
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));
    await waitFor(() => expect(getStaffBoostIpcMockCalls()).toHaveLength(1));
    expect(
      within(alexRow as HTMLElement).getByRole("button", {
        name: "Boosting…",
      }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Boosting…" }),
    ).toBeDisabled();
    resolvePendingStaffBoostIpcMock();
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });
});
