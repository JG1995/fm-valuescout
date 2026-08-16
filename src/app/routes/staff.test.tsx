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
});
