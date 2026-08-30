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
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { searchKeys } from "@/features/search/api/search-keys";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastSearchPlayersArgs,
  getSearchPlayersCallCount,
  rejectPendingSearchPlayersPageIpcMock,
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
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
  });

  it("lists Player Search in the nav rail and opens the no-snapshot empty state", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const searchLink = await screen.findByRole("link", {
      name: "Player Search",
    });
    await user.click(searchLink);

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(searchLink).toHaveAttribute("aria-current", "page");
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
  });

  it("opens the opt-in Moneyball workspace with its own query view and pool", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Moneyball Scout", 160),
        dynamicValues: { "moneyball.average_rating": 7.25 },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    renderSearchRoute("/search?view=moneyball");

    expect(
      await screen.findByRole("tab", { name: "Moneyball" }),
    ).toHaveAttribute("aria-selected", "true");
    expect(
      screen.getByRole("button", { name: "Upload Moneyball CSV" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        comparisonPool: "filtered",
      });
    });
  });

  it("selects, requests, sorts, and renders Moneyball role columns without changing raw percentiles", async () => {
    await resolveLoadDataIpcMock();
    const wingBackIp = "moneyball_role.wbl_wbr_wing_back_ip";
    const fullBackIp = "moneyball_role.dl_dr_wing_back_ip";
    usePlayerTableStore
      .getState()
      .addColumns("moneyball-search", [wingBackIp, fullBackIp]);
    setSearchPlayersOverride([
      {
        ...playerNamed("Role-fit Scout", 160),
        dynamicValues: {
          "moneyball.average_rating": 7.2,
          [wingBackIp]: 0,
          [fullBackIp]: null,
        },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    const { router } = renderSearchRoute("/search?view=moneyball");

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(
      within(table).getByRole("columnheader", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", {
        name: "Wing-Back (IP · DR/DL)",
      }),
    ).toBeInTheDocument();
    expect(
      within(table).getByRole("img", {
        name: "Moneyball role · Wing-Back (IP · WBR/WBL): 0, Weak",
      }),
    ).toBeInTheDocument();
    expect(within(table).getAllByText("—").length).toBeGreaterThan(0);
    expect(
      within(table).getByRole("img", {
        name: "Average Rating: 83, Excellent",
      }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        requestedFields: expect.arrayContaining([fullBackIp, wingBackIp]),
      });
    });

    await userEvent.setup().click(
      within(table).getByRole("button", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(router.state.location.search).toMatchObject({
      sort: wingBackIp,
      dir: "desc",
    });
  });

  it("applies a Moneyball role filter to the URL and comparison-pool request", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const roleField = "moneyball_role.wbl_wbr_wing_back_ip";
    setSearchPlayersOverride([
      {
        ...playerNamed("Role-fit Scout", 160),
        dynamicValues: {
          "moneyball.average_rating": 7.2,
          [roleField]: 72,
        },
        moneyballPercentiles: { "moneyball.average_rating": 83 },
      },
    ]);
    const { router } = renderSearchRoute("/search?view=moneyball");

    await screen.findByText("Role-fit Scout");
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    await user.click(
      within(dialog).getByRole("button", {
        name: "Field: Average Rating",
      }),
    );
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "wing-back",
    );
    await user.click(
      within(dialog).getByRole("option", {
        name: "Wing-Back (IP · WBR/WBL)",
      }),
    );
    expect(
      within(dialog).getByText(
        /role filters apply after the comparison cohort is calculated/i,
      ),
    ).toBeInTheDocument();
    fireEvent.change(within(dialog).getByLabelText("Value"), {
      target: { value: "70" },
    });
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        view: "moneyball",
        filters: [
          expect.objectContaining({ field: roleField, op: "gt", value: 70 }),
        ],
      });
      expect(getLastSearchPlayersArgs()).toMatchObject({
        filters: [{ field: roleField, op: "gt", value: 70 }],
        requestedFields: expect.arrayContaining([roleField]),
      });
    });

    await user.click(screen.getByRole("button", { name: "Full CSV" }));
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        comparisonPool: "fullCsv",
        filters: [{ field: roleField, op: "gt", value: 70 }],
      });
    });
  });

  it("uses the saved default only when Search has no explicit view", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    setSearchPlayersOverride([playerNamed("Moneyball Scout", 160)]);

    renderSearchRoute();

    expect(
      await screen.findByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
        searchView: "moneyball",
        sortBy: "moneyball.average_rating",
        comparisonPool: "filtered",
      });
    });
  });

  it("keeps an explicit General Search view above the saved default", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });

    renderSearchRoute("/search?view=general");

    expect(
      await screen.findByRole("tab", { name: "General", selected: true }),
    ).toBeInTheDocument();
  });

  it("makes General explicit when selected from an implicit Moneyball default", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    const { router } = renderSearchRoute();

    await user.click(
      await screen.findByRole("tab", { name: "General", selected: false }),
    );

    await waitFor(() => {
      expect(router.state.location.search.view).toBe("general");
      expect(
        screen.getByRole("tab", { name: "General", selected: true }),
      ).toBeInTheDocument();
    });
  });

  it("keeps the Moneyball upload label when a cohort exists", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Existing cohort", 160),
        dynamicValues: { "moneyball.average_rating": 7.25 },
      },
    ]);
    renderSearchRoute("/search?view=moneyball");

    expect(
      await screen.findByRole("button", { name: "Upload Moneyball CSV" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Replace Moneyball CSV" }),
    ).toBeNull();
  });

  it("moves focus to the selected Search tab during keyboard navigation", async () => {
    await resolveLoadDataIpcMock();
    renderSearchRoute();

    const general = await screen.findByRole("tab", { name: "General" });
    general.focus();
    fireEvent.keyDown(general, { key: "ArrowRight" });

    await waitFor(() => {
      expect(screen.getByRole("tab", { name: "Moneyball" })).toHaveFocus();
    });
  });

  it("renders a virtualized page of basic columns via search_players", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride(manyPlayers(80));
    renderSearchRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
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

  it("renders every nationality flag in stored order", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      {
        ...playerNamed("Flagged Scout", 160),
        nationalities: ["England", "Wales", "South Korea"],
      },
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const flags = await within(table).findAllByRole("img");

    expect(flags.map((flag) => flag.getAttribute("aria-label"))).toEqual([
      "England",
      "Wales",
      "South Korea",
    ]);
  });

  it("renders current, potential, and Club DNA scores while leaving other dynamic metrics neutral", async () => {
    await resolveLoadDataIpcMock();
    usePlayerTableStore
      .getState()
      .addColumns("search", [
        "role.goalkeeper_ip",
        "potential_role.goalkeeper_ip",
        "club_dna",
        "attr.Acceleration",
      ]);
    setSearchPlayersOverride([
      ...[
        ["Weak fit", 164, 20],
        ["Average fit", 163, 50],
        ["Good fit", 162, 70],
        ["Excellent fit", 161, 90],
      ].map(([name, ca, score]) => ({
        ...playerNamed(String(name), Number(ca)),
        dynamicValues: {
          "role.goalkeeper_ip": Number(score),
          "potential_role.goalkeeper_ip": Number(score),
          club_dna: Number(score),
          "attr.Acceleration": 16,
        },
      })),
      {
        ...playerNamed("Missing fit", 160),
        dynamicValues: { "attr.Acceleration": 16, club_dna: null },
      },
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    for (const [score, tier, scoreClass] of [
      [20, "Weak", "text-score-1"],
      [50, "Average", "text-score-2"],
      [70, "Good", "text-score-3"],
      [90, "Excellent", "text-score-4"],
    ] as const) {
      expect(
        within(table).getByRole("img", {
          name: `Role · Goalkeeper (IP): ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
      expect(
        within(table).getByRole("img", {
          name: `Potential role · Goalkeeper (IP): ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
      expect(
        within(table).getByRole("img", {
          name: `Club DNA: ${score}, ${tier}`,
        }),
      ).toHaveClass(scoreClass);
    }

    const missingRow = within(table).getByText("Missing fit").closest("tr");
    if (!missingRow) {
      throw new Error("Expected the missing-score player row.");
    }
    expect(within(missingRow).queryAllByRole("img")).toHaveLength(0);
    expect(within(missingRow).getAllByText("—")).toHaveLength(3);
    expect(within(missingRow).getByText("16")).not.toHaveAttribute(
      "role",
      "img",
    );
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
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });
    const caSort = within(caHeader).getByRole("button", { name: "CA" });
    expect(
      within(table).queryByRole("button", { name: "Manage CA column" }),
    ).toBeNull();
    caSort.focus();
    fireEvent.keyDown(caSort, { key: "F10", shiftKey: true });
    await user.keyboard("{Escape}");
    expect(caSort).toHaveFocus();
    fireEvent.contextMenu(caHeader);
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

  it("reorders Search columns from the menu without changing its query, virtual row, or widths", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    const store = usePlayerTableStore.getState();
    store.addColumns("search", ["attr.Acceleration", "attr.Agility"]);
    store.setColumnWidth("search", "attr.Acceleration", 216);
    setSearchPlayersOverride(
      manyPlayers(101).map((player) => ({
        ...player,
        dynamicValues: { "attr.Acceleration": 16, "attr.Agility": 15 },
      })),
    );
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const scroller = screen.getByTestId("search-results-scroller");
    mockScrollerScrollTo(scroller);
    fireEvent.scroll(scroller, { target: { scrollTop: 1_950 } });
    await waitFor(() => {
      expect(getLastSearchPlayersArgs()).toMatchObject({
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
    const callCountBeforeReorder = getSearchPlayersCallCount();
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
    expect(getSearchPlayersCallCount()).toBe(callCountBeforeReorder);
    expect(
      screen.getByRole("separator", { name: "Resize Acceleration column" }),
    ).toHaveAttribute("aria-valuenow", "216");
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });
  });

  it("moves Search columns from the header menu with edge guards and focus restoration", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Movable menu", 160)]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });
    fireEvent.contextMenu(caHeader);
    await user.click(screen.getByRole("menuitem", { name: "Move left" }));

    expect(screen.getByRole("button", { name: "CA" })).toHaveFocus();
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "name",
      "age",
      "ca",
      "nationality",
      "pa",
      "value",
    ]);
    expect(router.state.location.search).toMatchObject({
      sort: "ca",
      dir: "desc",
    });

    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Name" }),
    );
    expect(screen.getByRole("menuitem", { name: "Move left" })).toBeDisabled();
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Value" }),
    );
    expect(screen.getByRole("menuitem", { name: "Move right" })).toBeDisabled();
  });

  it("closes a header menu after either pointer button is pressed outside", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Dismissible menu", 160)]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const caHeader = within(table).getByRole("columnheader", { name: "CA" });

    fireEvent.contextMenu(caHeader);
    expect(
      screen.getByRole("menu", { name: "CA column actions" }),
    ).toBeInTheDocument();
    fireEvent.pointerDown(document.body, { button: 0 });
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();

    fireEvent.contextMenu(caHeader);
    expect(
      screen.getByRole("menu", { name: "CA column actions" }),
    ).toBeInTheDocument();
    fireEvent.pointerDown(document.body, { button: 2 });
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();

    fireEvent.contextMenu(caHeader);
    fireEvent.pointerDown(
      within(table).getByRole("separator", { name: "Resize CA column" }),
      { button: 0, pointerId: 1, clientX: 0 },
    );
    expect(
      screen.queryByRole("menu", { name: "CA column actions" }),
    ).toBeNull();
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
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Name" }),
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

  it.each([
    { view: "general" as const, entry: "/search?view=general" },
    { view: "moneyball" as const, entry: "/search?view=moneyball" },
  ])(
    "stacks fixed-height player identity without duplicate columns in $view Search",
    async ({ view, entry }) => {
      const user = userEvent.setup();
      await resolveLoadDataIpcMock();
      setSearchPlayersOverride([
        {
          ...playerNamed("Identity Player", 250),
          club: "Test FC",
          division: "Premier Division",
          dynamicValues: { "moneyball.average_rating": 9.9 },
        },
        {
          ...playerNamed("No context", 249),
          club: null,
          division: null,
          dynamicValues: { "moneyball.average_rating": 9.8 },
        },
        {
          ...playerNamed("Club only", 248),
          club: "Test FC",
          division: null,
          dynamicValues: { "moneyball.average_rating": 9.7 },
        },
        {
          ...playerNamed("Division only", 247),
          club: null,
          division: "Premier Division",
          dynamicValues: { "moneyball.average_rating": 9.6 },
        },
        ...manyPlayers(99),
      ]);
      const { router } = renderSearchRoute(entry);

      const table = await screen.findByRole("table", {
        name: "Player search results",
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
        throw new Error("Expected stacked player identity rows.");
      }
      const identityCell = within(identityRow).getAllByRole("cell")[0];
      const missingContextCell =
        within(missingContextRow).getAllByRole("cell")[0];
      const clubOnlyCell = within(clubOnlyRow).getAllByRole("cell")[0];
      const divisionOnlyCell = within(divisionOnlyRow).getAllByRole("cell")[0];
      expect(identityRow).toHaveStyle({ height: "40px" });
      expect(identityCell).toHaveTextContent("Test FC · Premier Division");
      expect(missingContextCell).toHaveTextContent("No context");
      expect(missingContextCell).not.toHaveTextContent("—");
      expect(missingContextCell).not.toHaveTextContent(" · ");
      expect(clubOnlyCell).toHaveTextContent("Test FC");
      expect(clubOnlyCell).not.toHaveTextContent(" · ");
      expect(divisionOnlyCell).toHaveTextContent("Premier Division");
      expect(divisionOnlyCell).not.toHaveTextContent(" · ");
      identityRow.focus();
      expect(identityRow).toHaveFocus();

      await user.click(
        within(identityRow).getByText("Test FC · Premier Division"),
      );

      await waitFor(() => {
        expect(router.state.location.pathname).toBe("/players/250");
        expect(router.state.location.search).toEqual({ view });
      });
    },
  );

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
      expect(router.state.location.search).toEqual({ view: "general" });
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

  it("deduplicates the initial Search page-zero IPC request", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Only once", 160)]);
    renderSearchRoute();

    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBe(1);
  });

  it("shows an initial Search failure and retries page zero", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Recovered Search", 160)]);
    setSearchPlayersPageIpcMockMode("rejectInitial");
    const user = userEvent.setup();
    renderSearchRoute();

    expect(
      await screen.findByText("Could not load players"),
    ).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBeGreaterThanOrEqual(1);

    setSearchPlayersPageIpcMockMode("success");
    await user.click(screen.getByRole("button", { name: "Retry" }));
    expect(
      await screen.findByRole("table", { name: "Player search results" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Recovered Search")).toBeInTheDocument();
    expect(getSearchPlayersCallCount()).toBe(2);
  });

  it("blocks Search results after an owner refresh fails", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Committed Search", 160)]);
    const { queryClient } = renderSearchRoute();

    expect(await screen.findByText("Committed Search")).toBeInTheDocument();
    const refresh = queryClient.fetchQuery({
      ...currentSnapshotQueryOptions,
      staleTime: 0,
      queryFn: () => Promise.reject(new Error("Snapshot refresh failed")),
    });
    await expect(refresh).rejects.toThrow("Snapshot refresh failed");

    expect(
      await screen.findByText("Loading player results…"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("table", { name: "Player search results" }),
    ).toBeNull();
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
    await waitFor(() =>
      expect(nameHeader).toHaveAttribute("aria-sort", "ascending"),
    );

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

  it("clears Search rows while a visible-field projection loads", async () => {
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([playerNamed("Projected Search", 160)]);
    renderSearchRoute();

    expect(await screen.findByText("Projected Search")).toBeInTheDocument();
    const callsBeforeProjection = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingProjection");
    act(() => {
      usePlayerTableStore
        .getState()
        .addColumns("search", ["attr.Acceleration"]);
    });

    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeProjection + 1),
    );
    expect(screen.getByText("Loading player results…")).toBeInTheDocument();
    expect(
      screen.queryByRole("table", { name: "Player search results" }),
    ).toBeNull();
    expect(screen.queryByText("Projected Search")).toBeNull();

    resolvePendingSearchPlayersPageIpcMock();
    expect(await screen.findByText("Projected Search")).toBeInTheDocument();
    act(() => {
      usePlayerTableStore
        .getState()
        .removeColumn("search", "attr.Acceleration");
    });
  });

  it("retains A until a stale cached Search sort refetch succeeds", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { queryClient, router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
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
      .findAll({ queryKey: searchKeys.playerPages() })
      .find((query) => {
        const descriptor = query.queryKey.at(-1);
        return (
          typeof descriptor === "object" &&
          descriptor !== null &&
          (descriptor as { sortBy?: unknown }).sortBy === "name"
        );
      });
    if (!cachedNameSort) {
      throw new Error("expected a cached Search name sort");
    }
    await queryClient.invalidateQueries({ queryKey: cachedNameSort.queryKey });
    setSearchPlayersPageIpcMockMode("pendingReplacement");
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
      throw new Error("expected a retained Search row");
    }
    fireEvent.click(retainedRow);
    fireEvent.keyDown(retainedRow, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/search");

    rejectPendingSearchPlayersPageIpcMock("Could not refresh sorted players.");
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort players. Could not refresh sorted players.",
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

  it("retains committed Search rows and blocks activation while a replacement sort loads", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    const callsBeforeSort = getSearchPlayersCallCount();
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Name" }));

    await waitFor(() =>
      expect(getSearchPlayersCallCount()).toBe(callsBeforeSort + 1),
    );
    expect(screen.getByRole("status")).toHaveTextContent("Sorting…");
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    expect(within(table).getByText("Zara")).toBeInTheDocument();

    const row = within(table)
      .getAllByRole("row")
      .find((candidate) => candidate.hasAttribute("data-index"));
    if (!row) {
      throw new Error("expected a retained Search row");
    }
    expect(row).not.toHaveAttribute("tabindex");
    fireEvent.click(row);
    fireEvent.keyDown(row, { key: "ArrowDown" });
    fireEvent.keyDown(row, { key: "Enter" });
    expect(router.state.location.pathname).toBe("/search");

    resolvePendingSearchPlayersPageIpcMock();
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Name" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    expect(within(table).getByText("Alice")).toBeInTheDocument();
  });

  it("falls back when removing a deferred requested dynamic Search sort", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    usePlayerTableStore.getState().addColumns("search", ["attr.Acceleration"]);
    setSearchPlayersOverride([
      {
        ...playerNamed("Fast Search", 160),
        dynamicValues: { "attr.Acceleration": 16 },
      },
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("pendingDynamicReplacement");
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
        sort: "ca",
        dir: "desc",
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
    expect(screen.getByText("Fast Search")).toBeInTheDocument();

    resolvePendingSearchPlayersPageIpcMock();
    await Promise.resolve();
    expect(screen.getByRole("columnheader", { name: "CA" })).toHaveAttribute(
      "aria-sort",
      "descending",
    );
  });

  it("keeps committed Search rows after a failed sort and retries the replacement", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    const { router } = renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("rejectReplacementOnce");
    await user.click(within(table).getByRole("button", { name: "Name" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not sort players.",
    );
    expect(within(table).getByText("Zara")).toBeInTheDocument();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "descending");
    const retainedRow = within(table)
      .getAllByRole("row")
      .find((row) => row.hasAttribute("data-index"));
    if (!retainedRow) {
      throw new Error("expected a retained Search row after a failed sort");
    }
    expect(retainedRow).not.toHaveAttribute("tabindex");
    fireEvent.click(retainedRow);
    retainedRow.focus();
    await user.keyboard("{Enter}");
    expect(router.state.location.pathname).toBe("/search");

    await user.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "Name" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("ignores a superseded Search sort when its deferred result resolves", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setSearchPlayersOverride([
      playerNamed("Zara", 200),
      playerNamed("Alice", 100),
    ]);
    renderSearchRoute();

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    setSearchPlayersPageIpcMockMode("pendingReplacement");
    await user.click(within(table).getByRole("button", { name: "Name" }));
    await screen.findByRole("status");
    await user.click(within(table).getByRole("button", { name: "CA" }));

    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: "CA" }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );
    resolvePendingSearchPlayersPageIpcMock();
    await Promise.resolve();
    expect(
      within(table).getByRole("columnheader", { name: "CA" }),
    ).toHaveAttribute("aria-sort", "ascending");
    expect(within(table).getByText("Alice")).toBeInTheDocument();
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
    await waitFor(() =>
      expect(
        within(table).getByRole("columnheader", { name: /CA/i }),
      ).toHaveAttribute("aria-sort", "ascending"),
    );

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
