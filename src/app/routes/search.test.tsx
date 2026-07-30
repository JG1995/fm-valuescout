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
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastSearchPlayersArgs,
  setSearchPlayersOverride,
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

  it("renders filter tags, opens editor, and applies filters immediately", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("High CA", 180),
      playerNamed("Low CA", 100),
    ]);
    renderSearchRoute();

    expect(await screen.findByText("High CA")).toBeInTheDocument();
    expect(screen.getByText("Low CA")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    expect(dialog).toBeInTheDocument();

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );

    const valueField = within(dialog).getByLabelText("Value");
    fireEvent.change(valueField, { target: { value: "150" } });

    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gt", value: 150 },
      ]);
      expect(
        screen.getByRole("button", {
          name: /Remove filter CA > 150/i,
        }),
      ).toBeInTheDocument();
      expect(screen.queryByText("Low CA")).not.toBeInTheDocument();
    });
    expect(screen.getByText("High CA")).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /Remove filter CA > 150/i }),
    );
    expect(screen.queryByRole("button", { name: /Remove filter/i })).toBeNull();
    expect(await screen.findByText("Low CA")).toBeInTheDocument();
  });

  it("sends filterCombine or when OR mode is selected in the editor", async () => {
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

    await user.click(within(dialog).getByRole("button", { name: "or" }));
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );

    const valueField = within(dialog).getByLabelText("Value");
    fireEvent.change(valueField, { target: { value: "150" } });

    await waitFor(() => {
      expect(getLastSearchPlayersArgs()?.filterCombine).toBe("or");
      expect(getLastSearchPlayersArgs()?.filters).toEqual([
        { field: "ca", op: "gt", value: 150 },
      ]);
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
    expect(router.state.location.search.filters).toHaveLength(32);
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
});
