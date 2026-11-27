import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import type {
  DashboardLineupSlot,
  DashboardRecommendation,
  DashboardSignings,
  DashboardSigningsRequest,
} from "@/features/dashboard/types/dashboard";
import { routeTree } from "@/routeTree.gen";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  getDashboardSigningsIpcRequests,
  setDashboardExpiringContractsIpcMock,
  setDashboardProspectsIpcMock,
  setDashboardSigningsIpcMock,
  setDashboardSigningsIpcMockError,
} from "@/testing/dashboard-ipc-mock";
import {
  getLastManagedClubSaveArgs,
  setManagedClubIpcMock,
  setPlannerAvailableClubs,
} from "@/testing/planner-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveLoadDataIpcMock,
  setSavePlayerShortlistsIpcMock,
} from "@/testing/snapshot-ipc-mock";

const KEEPER_SLOT: DashboardLineupSlot = {
  laneId: "goalkeeper",
  ipPosition: "GK",
  ipRoleId: "goalkeeper_ip",
  oopPosition: "GK",
  oopRoleId: "line_holding_keeper_oop",
  scoreBasis: "attributes",
  incumbent: { uid: 1, name: "Home Keeper", score: 70, source: "planner" },
  cover: { uid: 3, name: "Reserve Keeper", score: 55 },
  unscoredPlannerPlayer: null,
  bestSigning: null,
  recruitmentFilters: [],
};
const LEFT_BACK_SLOT: DashboardLineupSlot = {
  laneId: "left_back",
  ipPosition: "DL",
  ipRoleId: "full_back_ip",
  oopPosition: "DL",
  oopRoleId: "holding_full_back_oop",
  scoreBasis: "attributes",
  incumbent: { uid: 2, name: "Home Left Back", score: 60, source: "best_fit" },
  cover: null,
  unscoredPlannerPlayer: null,
  bestSigning: null,
  recruitmentFilters: [],
};
const STRIKER_SLOT: DashboardLineupSlot = {
  laneId: "centre_forward",
  ipPosition: "STC",
  ipRoleId: "centre_forward_ip",
  oopPosition: "STC",
  oopRoleId: "central_outlet_centre_forward_oop",
  scoreBasis: "attributes",
  incumbent: null,
  cover: null,
  unscoredPlannerPlayer: null,
  bestSigning: null,
  recruitmentFilters: [],
};

function recommendation(
  uid: number,
  name: string,
  laneId: string,
  score: number,
  improvement: number,
): DashboardRecommendation {
  return {
    uid,
    name,
    age: 24,
    club: "Other FC",
    clubUid: 2000,
    marketValueGbp: 12_500_000,
    minutes: null,
    borrowedStatsDate: null,
    laneId,
    score,
    improvement,
    agePenalty: 0,
    otherLanes: [],
  };
}

const LEFT_BACK_TARGET = recommendation(
  11,
  "Better Left Back",
  "left_back",
  80,
  20,
);
const KEEPER_TARGET = recommendation(10, "Better Keeper", "goalkeeper", 75, 5);

function signings(
  recommendations: DashboardRecommendation[],
  overrides: Partial<DashboardSignings> = {},
): DashboardSignings {
  return {
    tacticIsDefault: false,
    scoresAvailable: true,
    slots: [KEEPER_SLOT, LEFT_BACK_SLOT, STRIKER_SLOT],
    candidates: null,
    totalRecommendations: recommendations.length,
    recommendations,
    filteredOut: 0,
    underMinimumMinutes: 0,
    minimumMinutes: 450,
    seasonStatsAvailable: true,
    seasonStatsApplied: true,
    ...overrides,
  };
}

/** General mode ranks two targets; a slot, a shortlist, or Moneyball narrows them. */
function answerSignings(request: DashboardSigningsRequest) {
  if (request.mode === "moneyball") {
    return signings([
      { ...KEEPER_TARGET, score: 88, improvement: 30, minutes: 1240 },
    ]);
  }
  if (request.shortlistId === 7) {
    return signings([KEEPER_TARGET], { candidates: 6 });
  }
  if (request.laneId === "goalkeeper") {
    return signings([KEEPER_TARGET]);
  }
  return signings([LEFT_BACK_TARGET, KEEPER_TARGET]);
}

async function seedManagedClub() {
  await resolveLoadDataIpcMock();
  setManagedClubIpcMock({
    clubName: "Home FC",
    clubUid: 1000,
    status: "available",
    unclassifiedPlayerCount: 0,
  });
  setSavePlayerShortlistsIpcMock(1, [
    { id: 7, name: "Targets" },
    { id: 8, name: "Loans" },
  ]);
  setDashboardSigningsIpcMock(answerSignings);
}

/** Memory history, so a mode written to the URL cannot reach the next test. */
function renderDashboard(entry = "/") {
  return renderWithProviders({ initialEntries: [entry] });
}

/**
 * A router whose loader can run before anything renders, as a hover on the
 * Dashboard link runs it. It records every IPC request and leaves the
 * commands in `held` pending; the test setup restores the shared IPC mock.
 */
function dashboardRouter(entry: string, held: readonly string[] = []) {
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          cmd: string,
          args?: Record<string, unknown>,
          options?: unknown,
        ) => Promise<unknown>;
      };
    }
  ).__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  const requests: string[] = [];
  internals.invoke = (cmd, args, options) => {
    requests.push(cmd);
    return held.includes(cmd)
      ? new Promise(() => {})
      : invoke(cmd, args, options);
  };
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [entry] }),
  });
  return {
    router,
    requestCount: (cmd: string) =>
      requests.filter((request) => request === cmd).length,
    renderRouter: () =>
      render(
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>,
      ),
  };
}

function signingCards() {
  return within(
    screen.getByRole("list", { name: "Recommended signings, best first" }),
  ).getAllByRole("listitem");
}

function signingNames() {
  return signingCards().map(
    (card) => within(card).getByRole("link").textContent,
  );
}

describe("Dashboard", () => {
  beforeEach(() => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
  });

  it("names Load Data as the next step when the save has no snapshot", async () => {
    renderDashboard();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(screen.queryByText("Choose your club")).toBeNull();
  });

  it("asks for the managed club on the dashboard and opens once it is saved", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Home FC", clubUid: 1000 }]);
    setDashboardSigningsIpcMock(answerSignings);
    renderDashboard();

    expect(
      await screen.findByRole("heading", { name: "Choose your club" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Recommended signings")).toBeNull();

    await user.type(
      await screen.findByRole("combobox", { name: "Managed club" }),
      "Home",
    );
    await user.click(await screen.findByRole("option", { name: "Home FC" }));
    await user.click(screen.getByRole("button", { name: "Save managed club" }));

    expect(
      await screen.findByRole("heading", { name: "Recommended signings" }),
    ).toBeInTheDocument();
    expect(getLastManagedClubSaveArgs()).toEqual({
      clubName: "Home FC",
      clubUid: 1000,
    });
    expect(screen.getByText("Managed club: Home FC")).toBeInTheDocument();
  });

  it("ranks recommended signings against the starting XI", async () => {
    await seedManagedClub();
    renderDashboard();

    await waitFor(() =>
      expect(signingNames()).toEqual(["Better Left Back", "Better Keeper"]),
    );
    expect(
      screen.getByText(/^2 realistic signings would improve your XI/),
    ).toBeInTheDocument();
    const [best] = signingCards();
    expect(best).toHaveTextContent("Other FC · 24 years");
    expect(best).toHaveTextContent("€12.5M");
    expect(best).toHaveTextContent("Replaces Home Left Back");
    expect(
      within(best).getByRole("img", { name: /^Home Left Back at DL: 60,/ }),
    ).toBeInTheDocument();
    expect(
      within(best).getByRole("img", {
        name: "Better Left Back at DL: 80, Good",
      }),
    ).toBeInTheDocument();
    expect(within(best).getByText("+20")).toBeInTheDocument();
    expect(
      within(best).getByRole("link", { name: "Better Left Back" }),
    ).toHaveAttribute("href", "/players/11?view=general");

    const lineup = screen.getByRole("list", {
      name: "Starting XI by tactic slot",
    });
    const keeper = within(lineup).getByRole("button", { name: /Home Keeper/ });
    expect(keeper).toHaveTextContent("Planner pick:");
    expect(keeper).toHaveTextContent("Cover: Reserve Keeper 55");
    expect(
      within(lineup).getByRole("button", { name: /No player/ }),
    ).toHaveTextContent("Gap");
    expect(
      within(lineup).getByRole("button", { name: /Home Left Back/ }),
    ).toHaveTextContent("Weakest");
  });

  it("starts every panel read from the loader with the panels' own request", async () => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    await seedManagedClub();
    const { router, requestCount, renderRouter } =
      dashboardRouter("/?stats=false");

    await router.load();
    expect(getDashboardSigningsIpcRequests()).toEqual([
      {
        mode: "moneyball",
        shortlistId: null,
        laneId: null,
        seasonStats: false,
      },
    ]);
    expect(requestCount("get_dashboard_prospects")).toBe(1);
    expect(requestCount("get_dashboard_expiring_contracts")).toBe(1);
    expect(requestCount("list_managed_club_options")).toBe(0);

    renderRouter();
    await waitFor(() => expect(signingNames()).toEqual(["Better Keeper"]));
    // Every visit reads again, but always with the request the loader made.
    expect(
      new Set(getDashboardSigningsIpcRequests().map((r) => JSON.stringify(r))),
    ).toEqual(
      new Set([
        JSON.stringify({
          mode: "moneyball",
          shortlistId: null,
          laneId: null,
          seasonStats: false,
        }),
      ]),
    );
  });

  it("shows the dashboard while the panel reads are still running", async () => {
    await seedManagedClub();
    const { router, renderRouter } = dashboardRouter("/", [
      "get_dashboard_signings",
      "get_dashboard_prospects",
      "get_dashboard_expiring_contracts",
    ]);

    await router.load();
    renderRouter();

    for (const label of [
      "Loading recommended signings…",
      "Loading starting XI…",
      "Loading development prospects…",
      "Loading expiring contracts…",
    ]) {
      expect(
        screen.getByText(label).closest('[role="status"]'),
      ).toHaveAttribute("aria-busy", "true");
    }
    expect(screen.queryByText("Loading dashboard…")).toBeNull();
    expect(
      screen.getByRole("group", { name: "Score mode" }),
    ).toBeInTheDocument();
  });

  it("switches every score to Moneyball mode and keeps the mode in the URL", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    const { router } = renderDashboard();
    await waitFor(() => expect(signingCards()).toHaveLength(2));

    await user.click(screen.getByRole("button", { name: "Moneyball" }));

    await waitFor(() => expect(signingNames()).toEqual(["Better Keeper"]));
    expect(signingCards()[0]).toHaveTextContent("€12.5M · 1,240 min");
    expect(screen.getByRole("button", { name: "Moneyball" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(router.state.location.search).toEqual({ mode: "moneyball" });
    expect(screen.getByRole("link", { name: "Better Keeper" })).toHaveAttribute(
      "href",
      "/players/10?view=moneyball",
    );
  });

  it("starts in the configured default analysis view", async () => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    await seedManagedClub();
    renderDashboard();

    await waitFor(() => expect(signingCards()).toHaveLength(1));
    // The loader and the panel both read; each read asks for Moneyball.
    for (const request of getDashboardSigningsIpcRequests()) {
      expect(request).toEqual({
        mode: "moneyball",
        shortlistId: null,
        laneId: null,
        seasonStats: true,
      });
    }
  });

  it("narrows the signings to a selected slot and to one shortlist", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    renderDashboard();
    await waitFor(() => expect(signingCards()).toHaveLength(2));

    await user.click(screen.getByRole("button", { name: /Home Keeper/ }));

    await waitFor(() => expect(signingCards()).toHaveLength(1));
    expect(
      screen.getByText(/^1 realistic signing would improve GK/),
    ).toBeInTheDocument();
    expect(getDashboardSigningsIpcRequests().at(-1)).toEqual({
      mode: "general",
      shortlistId: null,
      laneId: "goalkeeper",
      seasonStats: true,
    });

    await user.click(screen.getByRole("button", { name: "Show all slots" }));
    await waitFor(() => expect(signingCards()).toHaveLength(2));

    const candidates = screen.getByRole("combobox", { name: "Candidates" });
    expect(candidates).toHaveDisplayValue("Realistic signings");
    await user.selectOptions(candidates, "Targets");
    await waitFor(() => expect(signingCards()).toHaveLength(1));
    expect(getDashboardSigningsIpcRequests().at(-1)).toEqual({
      mode: "general",
      shortlistId: 7,
      laneId: null,
      seasonStats: true,
    });
    expect(
      screen.getByText(/^1 of 6 shortlisted players would improve your XI/),
    ).toBeInTheDocument();
  });

  it("shows each slot's recruitment filters and links to edit them", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    const filteredSlots = [
      {
        ...KEEPER_SLOT,
        recruitmentFilters: [{ field: "height", op: "gte", value: 190 }],
      },
      {
        ...LEFT_BACK_SLOT,
        recruitmentFilters: [
          { field: "moneyball.progressive_passes", op: "pct_gte", value: 25 },
        ],
      },
      STRIKER_SLOT,
    ];
    setDashboardSigningsIpcMock((request) =>
      request.laneId === "goalkeeper"
        ? signings([KEEPER_TARGET], { slots: filteredSlots, filteredOut: 3 })
        : request.laneId === "left_back"
          ? signings([], { slots: filteredSlots, filteredOut: 2 })
          : signings([LEFT_BACK_TARGET, KEEPER_TARGET], {
              slots: filteredSlots,
              filteredOut: 1,
            }),
    );
    renderDashboard();
    await waitFor(() => expect(signingCards()).toHaveLength(2));

    expect(
      screen.getByText(
        /^Recruitment filters on 2 slots · 1 realistic signing filtered out ·/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Edit filters" })).toHaveAttribute(
      "href",
      "/my-club?view=tactic",
    );

    await user.click(
      screen.getByRole("button", {
        name: /has recruitment filters.*Home Keeper/,
      }),
    );
    await waitFor(() => expect(signingCards()).toHaveLength(1));
    expect(
      screen.getByText(
        /^Filters: Height ≥ 190 · 3 realistic signings filtered out ·/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Edit filters" })).toHaveAttribute(
      "href",
      "/my-club?view=tactic&slot=goalkeeper",
    );

    await user.click(screen.getByRole("button", { name: /Home Left Back/ }));
    expect(
      await screen.findByText("No realistic signing passes DL's filters"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /2 realistic signings would improve it but fail its recruitment filters/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Edit filters" })).toHaveAttribute(
      "href",
      "/my-club?view=tactic&slot=left_back",
    );
  });

  it("turns season stats off for General mode and keeps them for Moneyball", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    setDashboardSigningsIpcMock((request) =>
      request.mode === "general" && request.seasonStats
        ? signings([KEEPER_TARGET], {
            filteredOut: 3,
            underMinimumMinutes: 2,
          })
        : signings([LEFT_BACK_TARGET, KEEPER_TARGET], {
            seasonStatsApplied: request.mode === "moneyball",
          }),
    );
    const { router } = renderDashboard();
    await waitFor(() => expect(signingCards()).toHaveLength(1));

    const toggle = screen.getByRole("switch", { name: "Season stats" });
    expect(toggle).toBeChecked();
    expect(toggle).toHaveAccessibleDescription(
      /Candidates need 450\+ competitive minutes this season/,
    );
    expect(
      screen.getByText(
        "3 realistic signings filtered out, 2 of them under 450 minutes",
      ),
    ).toBeInTheDocument();

    await user.click(toggle);
    await waitFor(() => expect(signingCards()).toHaveLength(2));
    expect(getDashboardSigningsIpcRequests().at(-1)).toMatchObject({
      mode: "general",
      seasonStats: false,
    });
    expect(router.state.location.search).toEqual({ stats: false });

    await user.click(screen.getByRole("button", { name: "Moneyball" }));
    const locked = screen.getByRole("switch", { name: "Season stats" });
    expect(locked).toBeChecked();
    expect(locked).toBeDisabled();
    expect(router.state.location.search).toEqual({
      stats: false,
      mode: "moneyball",
    });
  });

  it("offers to turn season stats off when too few minutes empty a slot", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    setDashboardSigningsIpcMock((request) =>
      request.laneId === "left_back" && request.seasonStats
        ? signings([], { filteredOut: 2, underMinimumMinutes: 2 })
        : signings([LEFT_BACK_TARGET], { seasonStatsApplied: false }),
    );
    renderDashboard();
    await waitFor(() => expect(signingCards()).toHaveLength(1));

    await user.click(screen.getByRole("button", { name: /Home Left Back/ }));

    expect(
      await screen.findByText("No realistic signing for DL has enough minutes"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /2 realistic signings would improve it but have under 450 minutes this season/,
      ),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: "Turn off season stats" }),
    );
    await waitFor(() => expect(signingCards()).toHaveLength(1));
    expect(getDashboardSigningsIpcRequests().at(-1)).toMatchObject({
      laneId: "left_back",
      seasonStats: false,
    });
  });

  it("says what is missing when the save has no linked file and no signing to compare", async () => {
    await seedManagedClub();
    setSavePlayerShortlistsIpcMock(1, []);
    setDashboardSigningsIpcMock(() =>
      signings([], { candidates: 0, tacticIsDefault: true }),
    );
    renderDashboard();

    expect(
      await screen.findByText("No realistic signings to compare"),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole("region", { name: "Setup" })).getByRole("link", {
        name: "Open Settings",
      }),
    ).toHaveAttribute("href", "/settings");
    expect(
      screen.getByRole("link", { name: "Set up your tactic" }),
    ).toHaveAttribute("href", "/my-club?view=tactic");
    // Without a shortlist there is nothing to choose between.
    expect(screen.queryByRole("combobox", { name: "Candidates" })).toBeNull();
    // With no signing and no young player there is no fuller view to open.
    expect(screen.queryByRole("button", { name: "Best per slot" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Youth XI" })).toBeNull();
  });

  it("offers General scores when Moneyball mode has no season stats", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    setDashboardSigningsIpcMock((request) =>
      request.mode === "moneyball"
        ? signings([], { scoresAvailable: false })
        : answerSignings(request),
    );
    renderDashboard("/?mode=moneyball");

    expect(
      await screen.findByText("No season stats in this snapshot"),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Use General scores" }),
    );

    await waitFor(() => expect(signingCards()).toHaveLength(2));
  });

  it("keeps a failed signings read retryable without blanking the other panels", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    setDashboardSigningsIpcMockError("database is locked");
    setDashboardExpiringContractsIpcMock({
      windowMonths: 12,
      total: 3,
      players: [
        {
          uid: 2,
          name: "Home Left Back",
          age: 29,
          ca: 132,
          marketValueGbp: 900_000,
          weeklyWageGbp: null,
          expiryDate: "2027-06-30",
          daysRemaining: 320,
          loanedOut: true,
        },
      ],
    });
    setDashboardProspectsIpcMock({
      players: [
        {
          uid: 4,
          name: "Young Left Back",
          age: 19,
          laneId: "left_back",
          ipPosition: "DL",
          ipRoleId: "full_back_ip",
          oopPosition: "DL",
          oopRoleId: "holding_full_back_oop",
          currentScore: 50,
          potentialScore: 80,
        },
      ],
      youthXi: [],
      maxAge: 23,
    });
    renderDashboard();

    expect(
      await screen.findByText("Could not load recommended signings"),
    ).toBeInTheDocument();
    expect(screen.getByText("database is locked")).toBeInTheDocument();

    const contracts = within(
      await screen.findByRole("table", {
        name: "Your players by contract end date, soonest first",
      }),
    ).getByRole("row", { name: /Home Left Back/ });
    expect(contracts).toHaveTextContent("30 Jun 2027");
    expect(contracts).toHaveTextContent("in 10 months");
    expect(contracts).toHaveTextContent("€900k · On loan");
    expect(contracts).toHaveClass("bg-info/10");
    expect(screen.getByText("2 more end later in this period.")).toBeVisible();
    const prospect = within(
      screen.getByRole("list", {
        name: "Development prospects, highest potential first",
      }),
    ).getByRole("listitem");
    expect(prospect).toHaveTextContent("19 years");
    expect(prospect).toHaveTextContent("+30");
    // The save's development age limit bounds the prospects.
    expect(screen.getByText("Aged 23 or younger")).toBeVisible();
    expect(
      within(prospect).getByRole("img", {
        name: "Young Left Back potential at DL: 80, Good",
      }),
    ).toBeInTheDocument();

    setDashboardSigningsIpcMockError(null);
    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => expect(signingCards()).toHaveLength(2));
  });

  it("says how many points an older signing's score lost for their age", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    const veteran = { ...LEFT_BACK_TARGET, age: 32, agePenalty: 5 };
    setDashboardSigningsIpcMock(() =>
      signings([veteran, KEEPER_TARGET], {
        slots: [
          KEEPER_SLOT,
          { ...LEFT_BACK_SLOT, bestSigning: veteran },
          STRIKER_SLOT,
        ],
      }),
    );
    renderDashboard();

    await waitFor(() => expect(signingCards()).toHaveLength(2));
    const [older, younger] = signingCards();
    expect(older).toHaveTextContent("+20Age −5");
    expect(younger).not.toHaveTextContent("Age −");

    await user.click(screen.getByRole("button", { name: "Best per slot" }));
    expect(
      within(
        await screen.findByRole("dialog", {
          name: "Best signing for each slot",
        }),
      ).getByText("Other FC · 32 years · −5 for age · €12.5M"),
    ).toBeInTheDocument();
  });

  it("opens the best signing for each slot and rescores it in place", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    setDashboardSigningsIpcMock((request) =>
      signings([LEFT_BACK_TARGET, KEEPER_TARGET], {
        slots: [
          {
            ...KEEPER_SLOT,
            bestSigning:
              request.mode === "moneyball"
                ? { ...KEEPER_TARGET, score: 88, improvement: 18 }
                : KEEPER_TARGET,
          },
          { ...LEFT_BACK_SLOT, bestSigning: LEFT_BACK_TARGET },
          STRIKER_SLOT,
        ],
      }),
    );
    renderDashboard();

    await user.click(
      await screen.findByRole("button", { name: "Best per slot" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Best signing for each slot",
    });
    expect(dialog).toHaveTextContent("Realistic signings with 450+ minutes");
    // Goalkeeper to striker, whatever order the slots arrive in.
    const [keeper, leftBack, striker] = within(dialog)
      .getAllByRole("row")
      .slice(1);
    expect(keeper).toHaveTextContent("Home Keeper");
    expect(keeper).toHaveTextContent("Better Keeper");
    expect(keeper).toHaveTextContent("+5");
    expect(leftBack).toHaveTextContent("Home Left Back");
    expect(leftBack).toHaveTextContent("+20");
    expect(
      within(leftBack).getByRole("link", { name: "Better Left Back" }),
    ).toHaveAttribute("href", "/players/11?view=general");
    expect(striker).toHaveTextContent("Empty slot");
    expect(striker).toHaveTextContent(
      "No realistic signing improves this slot",
    );

    await user.click(within(dialog).getByRole("button", { name: "Moneyball" }));

    await waitFor(() =>
      expect(within(dialog).getAllByRole("row")[1]).toHaveTextContent("+18"),
    );

    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("opens the youth XI in the tactic's slots", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    const { bestSigning: _signing, ...keeperSlot } = KEEPER_SLOT;
    setDashboardProspectsIpcMock({
      players: [],
      youthXi: [
        {
          laneId: "left_back",
          ipPosition: "DL",
          ipRoleId: "full_back_ip",
          oopPosition: "DL",
          oopRoleId: "holding_full_back_oop",
          player: {
            uid: 4,
            name: "Young Left Back",
            age: 19,
            currentScore: 50,
            potentialScore: 80,
          },
        },
        {
          laneId: keeperSlot.laneId,
          ipPosition: keeperSlot.ipPosition,
          ipRoleId: keeperSlot.ipRoleId,
          oopPosition: keeperSlot.oopPosition,
          oopRoleId: keeperSlot.oopRoleId,
          player: null,
        },
      ],
      maxAge: 23,
    });
    renderDashboard();

    // No prospect has room to grow, yet the young players still form an XI.
    expect(
      await screen.findByText("No prospects with room to grow"),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Youth XI" }));

    const dialog = await screen.findByRole("dialog", { name: "Youth XI" });
    const [keeper, leftBack] = within(
      within(dialog).getByRole("list", { name: "Youth XI by tactic slot" }),
    ).getAllByRole("listitem");
    expect(keeper).toHaveTextContent("No player");
    expect(keeper).toHaveTextContent("GK");
    expect(
      within(leftBack).getByRole("link", { name: "Young Left Back" }),
    ).toHaveAttribute("href", "/players/4");
    expect(
      within(leftBack).getByRole("img", {
        name: "Young Left Back potential at DL: 80, Good",
      }),
    ).toBeInTheDocument();
  });
});
