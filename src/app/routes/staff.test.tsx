import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { fireEvent, render, screen, within } from "@testing-library/react";
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
import { fixtureStaff, setStaffOverride } from "@/testing/staff-ipc-mock";

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
    expect(screen.getByRole("tab", { name: "My Staff" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByText("My Staff overview is coming next"),
    ).toBeInTheDocument();
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
});
