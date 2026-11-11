import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type {
  DashboardLineupSlot,
  DashboardRecommendation,
  DashboardSignings,
  DashboardSigningsRequest,
} from "@/features/dashboard/types/dashboard";
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
    laneId,
    score,
    improvement,
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
    shortlistedPlayers: 6,
    totalRecommendations: recommendations.length,
    recommendations,
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
  if (request.laneId === "goalkeeper" || request.shortlistId === 7) {
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
    { id: 7, name: "Targets", fileName: null },
    { id: 8, name: "Loans", fileName: "loans.fmf" },
  ]);
  setDashboardSigningsIpcMock(answerSignings);
}

/** Memory history, so a mode written to the URL cannot reach the next test. */
function renderDashboard(entry = "/") {
  return renderWithProviders({ initialEntries: [entry] });
}

function signingRows() {
  const table = screen.getByRole("table", {
    name: /Shortlisted players ranked by improvement/,
  });
  return within(table)
    .getAllByRole("row")
    .slice(1)
    .map((row) => within(row).getByRole("rowheader").textContent);
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
      expect(signingRows()).toEqual([
        "Better Left BackOther FC · 24 years",
        "Better KeeperOther FC · 24 years",
      ]),
    );
    expect(
      screen.getByText("2 of 6 shortlisted players would improve your XI"),
    ).toBeInTheDocument();
    const best = screen.getByRole("row", { name: /Better Left Back/ });
    expect(within(best).getByText("Home Left Back")).toBeInTheDocument();
    expect(
      within(best).getByRole("img", {
        name: "Better Left Back at DL: 80, Good",
      }),
    ).toBeInTheDocument();
    expect(within(best).getByText("+20")).toBeInTheDocument();
    expect(within(best).getByText("€12.5M")).toBeInTheDocument();
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

  it("switches every score to Moneyball mode and keeps the mode in the URL", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    const { router } = renderDashboard();
    await waitFor(() => expect(signingRows()).toHaveLength(2));

    await user.click(screen.getByRole("button", { name: "Moneyball" }));

    await waitFor(() =>
      expect(signingRows()).toEqual([
        "Better KeeperOther FC · 24 years · 1,240 min",
      ]),
    );
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

    await waitFor(() => expect(signingRows()).toHaveLength(1));
    expect(getDashboardSigningsIpcRequests()).toEqual([
      { mode: "moneyball", shortlistId: null, laneId: null },
    ]);
  });

  it("narrows the signings to a selected slot and to one shortlist", async () => {
    const user = userEvent.setup();
    await seedManagedClub();
    renderDashboard();
    await waitFor(() => expect(signingRows()).toHaveLength(2));

    await user.click(screen.getByRole("button", { name: /Home Keeper/ }));

    await waitFor(() => expect(signingRows()).toHaveLength(1));
    expect(
      screen.getByText("1 shortlisted player would improve GK"),
    ).toBeInTheDocument();
    expect(getDashboardSigningsIpcRequests().at(-1)).toEqual({
      mode: "general",
      shortlistId: null,
      laneId: "goalkeeper",
    });

    await user.click(screen.getByRole("button", { name: "Show all slots" }));
    await waitFor(() => expect(signingRows()).toHaveLength(2));

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Shortlist" }),
      "Targets",
    );
    await waitFor(() => expect(signingRows()).toHaveLength(1));
    expect(getDashboardSigningsIpcRequests().at(-1)).toEqual({
      mode: "general",
      shortlistId: 7,
      laneId: null,
    });
  });

  it("says what is missing when the save has no linked file or shortlist", async () => {
    await seedManagedClub();
    setSavePlayerShortlistsIpcMock(1, []);
    setDashboardSigningsIpcMock(() =>
      signings([], { shortlistedPlayers: 0, tacticIsDefault: true }),
    );
    renderDashboard();

    expect(await screen.findByText("No shortlist loaded")).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Player Search" }),
    ).toHaveAttribute("href", "/search");
    expect(
      within(screen.getByRole("region", { name: "Setup" })).getByRole("link", {
        name: "Open Settings",
      }),
    ).toHaveAttribute("href", "/settings");
    expect(
      screen.getByRole("link", { name: "Set up your tactic" }),
    ).toHaveAttribute("href", "/my-club?view=tactic");
    expect(screen.queryByRole("combobox", { name: "Shortlist" })).toBeNull();
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

    await waitFor(() => expect(signingRows()).toHaveLength(2));
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
        },
      ],
    });
    setDashboardProspectsIpcMock([
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
    ]);
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
    expect(contracts).toHaveTextContent("€900k");
    expect(screen.getByText("2 more end later in this period.")).toBeVisible();
    const prospect = within(
      screen.getByRole("table", {
        name: "Your young players by potential score in their best tactic slot",
      }),
    ).getByRole("row", { name: /Young Left Back/ });
    expect(prospect).toHaveTextContent("+30");
    expect(
      within(prospect).getByRole("img", {
        name: "Young Left Back potential at DL: 80, Good",
      }),
    ).toBeInTheDocument();

    setDashboardSigningsIpcMockError(null);
    await user.click(screen.getByRole("button", { name: "Retry" }));

    await waitFor(() => expect(signingRows()).toHaveLength(2));
  });
});
