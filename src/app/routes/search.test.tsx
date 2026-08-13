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
import type { PlayerSummary } from "@/features/search/types/player-summary";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastSearchPlayersArgs,
  getSearchPlayersCallCount,
  resolvePendingSearchPlayersPageIpcMock,
  setSearchPlayersOverride,
  setSearchPlayersPageIpcMockMode,
} from "@/testing/search-ipc-mock";
import {
  resolveCreateSaveIpcMock,
  resolveLoadDataIpcMock,
} from "@/testing/snapshot-ipc-mock";

function renderSearchRoute(initialEntry = "/search") {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 60_000 },
    },
  });

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
    division: index % 3 === 0 ? null : "Premier Division",
    ca: 200 - index,
    pa: 200 - (index % 40),
    marketValueGbp: index % 5 === 0 ? null : (200 - index) * 50_000,
  }));
}

function mockScrollerScrollTo(scroller: HTMLElement) {
  Object.defineProperty(scroller, "scrollTo", {
    configurable: true,
    value: (options: { top?: number }) => {
      scroller.scrollTop = options.top ?? scroller.scrollTop;
    },
  });
}

describe("search route", () => {
  beforeEach(() => {
    useLayoutStore.setState({ railExpanded: true });
  });

  it("lists Search in the nav rail and opens the no-snapshot empty state", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const searchLink = await screen.findByRole("link", { name: "Search" });
    await user.click(searchLink);

    expect(
      await screen.findByRole("heading", { level: 1, name: "Search" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
  });

  it("renders a virtualized page of basic columns via search_players", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride(manyPlayers(80));
    renderSearchRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Search" }),
    ).toBeInTheDocument();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", { name: "Name" }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toBeInTheDocument();
    expect(await within(table).findByText("Player 001")).toBeInTheDocument();

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    expect(bodyRows.length).toBeGreaterThan(0);
    expect(bodyRows.length).toBeLessThan(80);
    const scroller = screen.getByTestId("search-results-scroller");
    expect(scroller).toHaveClass("h-full", "min-h-0", "overflow-auto");
    expect(scroller.parentElement).toHaveClass("relative", "min-h-0", "flex-1");
    expect(getLastSearchPlayersArgs()).toMatchObject({
      requestedFields: [],
    });
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
    const manageCa = within(table).getByRole("button", {
      name: "Manage CA column",
    });
    await user.click(manageCa);
    await user.keyboard("{Escape}");
    expect(manageCa).toHaveFocus();
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "CA" }),
    );
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
        requestedFields: ["attr.Acceleration"],
      });
    });
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
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

    fireEvent.click(within(table).getByRole("button", { name: "Name" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
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
    const { router } = renderSearchRoute("/search?sort=name&dir=asc");

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    await user.click(
      within(table).getByRole("button", { name: "Manage Name column" }),
    );
    await user.click(screen.getByRole("menuitem", { name: "Remove Name" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
      });
      expect(screen.queryByRole("columnheader", { name: "Name" })).toBeNull();
    });
  });

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
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    await user.click(within(table).getByRole("button", { name: "Name" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
      });
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
    const restoredTable = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(restoredTable).getByRole("columnheader", { name: "Name" }),
    ).toHaveAttribute("aria-sort", "ascending");
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

  it("does not reclaim focus from a Search header while a virtual page is pending", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride(manyPlayers(101));
    setSearchPlayersPageIpcMockMode("pendingSecondPage");
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const scroller = screen.getByTestId("search-results-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({ offset: 50 });
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

    const nameHeader = within(table).getByRole("button", { name: "Name" });
    await user.click(nameHeader);
    expect(nameHeader).toHaveFocus();
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
      });
    });

    resolvePendingSearchPlayersPageIpcMock();

    expect(await screen.findByText("Player 051")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Name" })).toHaveFocus();
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
    expect(screen.getByText(/sorted by CA \(descending\)/)).toBeInTheDocument();
  });

  it("writes sort into URL search params when a header is clicked", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    // CA desc → Zara first; name asc → Alice first (orders must diverge).
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

    await user.click(within(table).getByRole("button", { name: /^Name$/i }));

    expect(router.state.location.search).toMatchObject({
      sort: "name",
      dir: "asc",
    });
    const nameHeader = within(table).getByRole("columnheader", {
      name: /Name/i,
    });
    expect(nameHeader).toHaveAttribute("aria-sort", "ascending");

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
    expect(
      within(table).getByRole("columnheader", { name: /Name/i }),
    ).toHaveAttribute("aria-sort", "ascending");

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstRow = bodyRows[0];
    if (!firstRow) {
      throw new Error("expected a virtualized body row");
    }
    expect(within(firstRow).getByText("Alice")).toBeInTheDocument();
  });

  it("keeps filter edits query-silent until Done, then applies the complete draft", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    const { router } = renderSearchRoute();

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    expect(screen.getByText("Low CA")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    expect(dialog).toBeInTheDocument();
    const callsBeforeEdit = getSearchPlayersCallCount();

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );

    await user.click(within(dialog).getByRole("button", { name: "Field: CA" }));
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "ca",
    );
    await user.click(within(dialog).getByRole("option", { name: "CA" }));

    const valueField = within(dialog).getByLabelText("Value");
    fireEvent.change(valueField, { target: { value: "150" } });
    await user.click(within(dialog).getByRole("button", { name: "or" }));

    expect(getSearchPlayersCallCount()).toBe(callsBeforeEdit);
    expect(router.state.location.search).toMatchObject({
      combine: "and",
      filters: [],
    });
    expect(screen.getByText("Low CA")).toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    await waitFor(() => {
      expect(getSearchPlayersCallCount()).toBeGreaterThan(callsBeforeEdit);
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gt", value: 150 },
      ]);
      expect(getLastSearchPlayersArgs()?.filterCombine).toBe("or");
    });
    expect(router.state.location.search).toMatchObject({
      combine: "or",
      filters: [expect.objectContaining({ field: "ca", op: "gt", value: 150 })],
    });
    expect(
      screen.getByRole("button", {
        name: /Remove filter CA > 150/i,
      }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
    expect(screen.getByText("High CA")).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /Remove filter CA > 150/i }),
    );
    expect(
      screen.queryByRole("button", { name: /Remove filter CA > 150/i }),
    ).toBeNull();
    expect(await screen.findByText("Low CA")).toBeInTheDocument();
  });

  it("keeps a drafted potential role filter query-silent until Done", async () => {
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
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    const callsBeforeEdit = getSearchPlayersCallCount();
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    await user.click(within(dialog).getByRole("button", { name: "Field: CA" }));
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

    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "potential_role.goalkeeper_ip", op: "gt", value: 50 },
      ]);
    });
    expect(
      await screen.findByRole("columnheader", {
        name: "Potential role · Goalkeeper (IP)",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: /Remove filter Potential role · Goalkeeper \(IP\) > 50/i,
      }),
    );

    expect(
      await screen.findByRole("columnheader", {
        name: "Potential role · Goalkeeper (IP)",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        requestedFields: ["potential_role.goalkeeper_ip"],
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
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "250" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(
      await screen.findByText("No players match these filters"),
    ).toBeInTheDocument();
    expect(screen.queryByText("High CA")).not.toBeInTheDocument();
  });

  it("keeps focus in the filter value field while editing", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    renderSearchRoute();

    await user.click(
      await screen.findByRole("button", { name: "Edit filters" }),
    );
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );

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
    const { router } = renderSearchRoute();

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    await user.click(within(dialog).getByRole("button", { name: "or" }));
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "150" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        sort: "ca",
        dir: "desc",
        combine: "or",
        filters: [
          expect.objectContaining({
            field: "ca",
            op: "gt",
            value: 150,
          }),
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
        name: /Remove filter CA > 150/i,
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gt", value: 150 },
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
      JSON.stringify([{ id: "seed", field: "ca", op: "gt", value: 150 }]),
    );
    renderSearchRoute(
      `/search?sort=ca&dir=desc&combine=and&filters=${encodedFilters}`,
    );

    expect(
      await screen.findByRole("button", {
        name: /Remove filter CA > 150/i,
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gt", value: 150 },
      ]);
    });
    expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
  });

  it("stops adding filter rules once the UI cap is reached", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("High CA", 180)]);
    const { router } = renderSearchRoute();

    await user.click(
      await screen.findByRole("button", { name: "Edit filters" }),
    );
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    const addButton = within(dialog).getByRole("button", {
      name: "Add filter",
    });

    for (let index = 0; index < 32; index += 1) {
      await user.click(addButton);
    }

    expect(addButton).toBeDisabled();
    expect(
      within(dialog).getAllByRole("button", { name: "Remove filter rule" }),
    ).toHaveLength(32);

    await user.click(addButton);
    expect(
      within(dialog).getAllByRole("button", { name: "Remove filter rule" }),
    ).toHaveLength(32);
    expect(router.state.location.search.filters).toHaveLength(0);
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
    expect(
      within(table).getByRole("columnheader", { name: /CA/i }),
    ).toHaveAttribute("aria-sort", "ascending");

    const bodyRows = within(table)
      .getAllByRole("row")
      .filter((row) => row.hasAttribute("data-index"));
    const firstRow = bodyRows[0];
    if (!firstRow) {
      throw new Error("expected a virtualized body row");
    }
    expect(within(firstRow).getByText("Low")).toBeInTheDocument();
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
        { field: "role.deep_lying_playmaker_ip", op: "gt", value: 70 },
        { field: "attr.Acceleration", op: "gt", value: 12 },
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
      requestedFields: [],
      filters: [
        { field: "role.deep_lying_playmaker_ip", op: "gt", value: 70 },
        { field: "attr.Acceleration", op: "gt", value: 12 },
      ],
    });
  });
});
