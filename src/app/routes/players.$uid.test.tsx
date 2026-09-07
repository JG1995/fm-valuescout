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
import { Route } from "@/app/routes/players.$uid";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  fixturePlayerMoneyball,
  fixturePlayerMoneyballWithoutNaturalPosition,
  resolvePendingPlayerMoneyball,
  setPlayerMoneyballOverride,
  setPlayerMoneyballPending,
} from "@/testing/moneyball-ipc-mock";
import {
  fixturePlayerDetail,
  getCurrentAbilityBoostIpcMockCalls,
  getSetPlayerHiddenInformationRevealedIpcMockCalls,
  getWonderkidMentalityBoostIpcMockCalls,
  resolvePendingCurrentAbilityBoostIpcMock,
  resolvePendingWonderkidMentalityBoostIpcMock,
  setCurrentAbilityBoostIpcMockMode,
  setGetPlayerOverride,
  setPlayerHiddenInformationRevealedIpcMockMode,
  setWonderkidMentalityBoostIpcMockMode,
} from "@/testing/player-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";

function renderProfileRoute(initialEntry: string) {
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

describe("player profile route", () => {
  beforeEach(() => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
    setGetPlayerOverride(undefined);
  });

  it("shows rail identity with overview analysis for a known player", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const rail = await screen.findByRole("complementary", {
      name: "Player identity",
    });
    expect(
      within(rail).getByRole("heading", {
        level: 1,
        name: "Alex Scout",
      }),
    ).toBeInTheDocument();
    expect(
      within(rail).getByText("Test FC · Premier Division"),
    ).toBeInTheDocument();
    expect(within(rail).getByText("21/03/2001 (25)")).toBeInTheDocument();
    expect(within(rail).getByText("182 cm")).toBeInTheDocument();
    expect(within(rail).getByText("Right")).toBeInTheDocument();
    expect(within(rail).getByRole("img", { name: "England" })).toHaveAttribute(
      "title",
      "England",
    );
    expect(within(rail).getByRole("img", { name: "Wales" })).toHaveAttribute(
      "title",
      "Wales",
    );

    const summary = screen.getByRole("region", {
      name: "Alex Scout summary",
    });
    expect(
      within(summary).queryByRole("heading", {
        level: 1,
        name: "Alex Scout",
      }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("140")).toBeInTheDocument();
    expect(screen.getByText("160")).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
  });

  it("uses the saved Moneyball default when the profile URL omits a view", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());

    renderProfileRoute("/players/42");

    expect(
      await screen.findByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeInTheDocument();
  });

  it("maps legacy General with an attribute tab to Attributes above the saved Moneyball default", async () => {
    await resolveLoadDataIpcMock();
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    setGetPlayerOverride(fixturePlayerDetail());

    renderProfileRoute("/players/42?view=general&tab=hidden");

    expect(
      await screen.findByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Hidden", selected: true }),
    ).toBeInTheDocument();
  });

  it("uses the explicit Moneyball view without losing the General attribute tab", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(
      fixturePlayerMoneyball({
        statistics: { goals: 10, goals_per_90: 0.6 },
        percentiles: { goals: 83, goals_per_90: 75 },
      }),
    );
    const user = userEvent.setup();
    const { router } = renderProfileRoute(
      "/players/42?view=moneyball&tab=hidden",
    );

    expect(
      await screen.findByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getByText("Starts")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "Goals: 83, Excellent" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Boost CA" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Overview" }));

    expect(
      await screen.findByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Hidden", selected: true }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({
      section: "overview",
      tab: "hidden",
    });
  });

  it("shows Moneyball summaries and the ready role workspace", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { MC: 20 },
        roleScores: [
          {
            roleId: "attribute-best",
            displayName: "Attribute Best Role",
            phase: "in_possession",
            positionTags: ["MC"],
            score: 99,
            potentialScore: 100,
          },
        ],
      }),
    );
    setPlayerMoneyballOverride(
      fixturePlayerMoneyball({
        roleScores: [
          {
            roleId: "mc_moneyball_ip",
            displayName: "Moneyball IP Specialist",
            phase: "in_possession",
            positionFamily: "central_midfielder",
            positionTags: ["MC"],
            score: 81,
            contributions: [],
          },
          {
            roleId: "mc_moneyball_oop",
            displayName: "Moneyball OOP Specialist",
            phase: "out_of_possession",
            positionFamily: "central_midfielder",
            positionTags: ["MC"],
            score: 74,
            contributions: [],
          },
        ],
      }),
    );

    renderProfileRoute("/players/42?view=moneyball");

    expect(
      screen.queryByRole("region", { name: "Alex Scout summary" }),
    ).not.toBeInTheDocument();

    const tacticalSummary = await screen.findByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(tacticalSummary).getByLabelText("Moneyball IP: 81, Excellent"),
    ).toBeInTheDocument();
    expect(
      within(tacticalSummary).getByLabelText("Moneyball OOP: 74, Good"),
    ).toBeInTheDocument();
    expect(
      within(tacticalSummary).getByText("Moneyball IP Specialist"),
    ).toBeInTheDocument();
    expect(
      within(tacticalSummary).queryByText("Current IP"),
    ).not.toBeInTheDocument();
    expect(
      within(tacticalSummary).queryByText("Attribute Best Role"),
    ).not.toBeInTheDocument();

    expect(screen.getByText("Starts")).toBeInTheDocument();
    const roleFit = screen.getByRole("region", {
      name: "Moneyball role fit for MC",
    });
    expect(
      within(roleFit).getByLabelText(
        "Moneyball IP Specialist Moneyball score: 81, Excellent",
      ),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByRole("columnheader", { name: "Moneyball score" }),
    ).toBeInTheDocument();

    const workspace = screen.getByRole("tabpanel", { name: "Moneyball" });
    const primaryColumn = screen.getByTestId("moneyball-primary-column");
    expect(workspace.children).toHaveLength(2);
    expect(workspace.firstElementChild).toBe(primaryColumn);
    expect(workspace.lastElementChild).toContainElement(roleFit);
    expect(primaryColumn.children).toHaveLength(2);
    expect(primaryColumn.firstElementChild).toBe(tacticalSummary);
    expect(primaryColumn.lastElementChild).toContainElement(
      screen.getByText("Starts"),
    );
    expect(primaryColumn.lastElementChild).toHaveClass(
      "min-h-0",
      "overflow-hidden",
    );
    expect(primaryColumn).toHaveClass(
      "min-h-0",
      "min-w-0",
      "lg:grid-rows-[auto_minmax(0,1fr)]",
    );
    expect(workspace.lastElementChild).toHaveClass(
      "min-h-0",
      "overflow-hidden",
    );
    expect(roleFit).toHaveClass("min-h-0");
    expect(
      within(roleFit).getByTestId("moneyball-role-position-picker-scroller"),
    ).toHaveClass("min-h-0", "overflow-y-auto");
  });

  it("aligns Moneyball panels with bounded workspace hierarchy and score language", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ positions: { MC: 20 } }));
    setPlayerMoneyballOverride(
      fixturePlayerMoneyball({
        statistics: { goals: 10, goals_per_90: 0.6 },
        percentiles: { goals: 83, goals_per_90: 75 },
        comparisonBasis: {
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        },
        roleScores: [
          {
            roleId: "mc-moneyball-ip",
            displayName: "Moneyball IP Specialist",
            phase: "in_possession",
            positionFamily: "central_midfielder",
            positionTags: ["MC"],
            score: 81,
            contributions: [],
          },
        ],
      }),
    );

    renderProfileRoute("/players/42?section=moneyball");

    const workspace = await screen.findByRole("tabpanel", {
      name: "Moneyball",
    });
    const primaryColumn = screen.getByTestId("moneyball-primary-column");
    const profilePanel = within(primaryColumn)
      .getByRole("heading", { name: "Moneyball" })
      .closest("section");
    if (!profilePanel) throw new Error("Expected the Moneyball profile panel");
    expect(profilePanel).toHaveClass("min-h-0", "w-full", "flex", "flex-col");
    expect(profilePanel.querySelectorAll(".overflow-y-auto")).toHaveLength(1);
    expect(profilePanel.querySelector(".overflow-y-auto")).toHaveClass(
      "min-h-0",
      "flex-1",
    );
    expect(within(profilePanel).getByText("10")).toBeInTheDocument();
    expect(
      within(profilePanel).getByRole("img", { name: "Goals: 83, Excellent" }),
    ).toBeInTheDocument();
    expect(
      within(profilePanel).getByText(
        "Natural positions: MC · 24 comparison players",
      ),
    ).toBeInTheDocument();
    expect(
      within(profilePanel).queryByText(/Potential/),
    ).not.toBeInTheDocument();

    const roleFit = screen.getByRole("region", {
      name: "Moneyball role fit for MC",
    });
    const roleFitPanel = roleFit.parentElement?.closest("section");
    if (!roleFitPanel) throw new Error("Expected the Moneyball role-fit panel");
    expect(roleFitPanel).toHaveClass("min-h-0", "w-full", "flex", "flex-col");
    expect(roleFitPanel.querySelectorAll(".overflow-y-auto")).toHaveLength(2);
    expect(
      within(roleFit).getByLabelText(
        "Moneyball IP Specialist Moneyball score: 81, Excellent",
      ),
    ).toHaveClass("text-right", "tabular-nums");
    expect(within(workspace).queryByText("Potential")).not.toBeInTheDocument();
  });

  it("keeps market value general-only inside the summary analysis details", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const first = renderProfileRoute("/players/42");

    const generalSummary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const generalAnalysis = within(generalSummary).getByTestId(
      "player-profile-summary-analysis-details",
    );
    expect(
      within(generalAnalysis).getByText("Market Value"),
    ).toBeInTheDocument();
    first.unmount();

    renderProfileRoute("/players/42?view=moneyball");

    const tacticalSummary = await screen.findByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(tacticalSummary).queryByText("Value"),
    ).not.toBeInTheDocument();
  });

  it("keeps Moneyball no-data states actionable without rendering a role panel", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride({ state: "noData" });
    const first = renderProfileRoute("/players/42?view=moneyball");

    expect(
      await screen.findByText(/not included in the current Moneyball import/i),
    ).toBeInTheDocument();
    const noDataSummary = screen.getByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(noDataSummary).getByLabelText("Moneyball IP: unavailable"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Moneyball role fit for MC" }),
    ).not.toBeInTheDocument();

    first.unmount();
    setPlayerMoneyballOverride({ state: "needsReimport" });
    renderProfileRoute("/players/42?view=moneyball");

    expect(
      await screen.findByText(/before percentile scores were available/i),
    ).toBeInTheDocument();
    const needsReimportSummary = screen.getByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(needsReimportSummary).getByLabelText("Moneyball OOP: unavailable"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Moneyball role fit for MC" }),
    ).not.toBeInTheDocument();
  });

  it("keeps no-natural-position raw metrics distinct from unavailable scores", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ positions: { MC: 20 } }));
    setPlayerMoneyballOverride(
      fixturePlayerMoneyballWithoutNaturalPosition({
        statistics: { goals: 10 },
        percentiles: { goals: 50 },
        roleCatalogVersion: 1,
        roleScores: [
          {
            roleId: "stale-score",
            displayName: "Stale score",
            phase: "in_possession",
            positionFamily: "central_midfielder",
            positionTags: ["MC"],
            score: 50,
            contributions: [],
          },
        ],
      }),
    );

    renderProfileRoute("/players/42?view=moneyball");

    expect(await screen.findByText("10")).toBeInTheDocument();
    const tacticalSummary = screen.getByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(tacticalSummary).getByLabelText("Moneyball IP: unavailable"),
    ).toBeInTheDocument();
    expect(
      within(tacticalSummary).queryByLabelText("Moneyball IP: 50, Average"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Percentile scores unavailable: this player has no natural position.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Role scores unavailable: this player has no natural position.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("img", { name: "Goals: 50, Average" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByLabelText("Stale score Moneyball score: 50, Average"),
    ).not.toBeInTheDocument();
  });

  it("keeps the standard header and relocates Moneyball summaries to its section", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        name: "Alexandra Maximilian Scout",
        nationalities: ["England", "Wales"],
        positions: { MC: 20, AMC: 20, AMR: 20 },
      }),
    );
    setPlayerMoneyballOverride(
      fixturePlayerMoneyball({
        comparisonBasis: {
          kind: "available",
          naturalPositions: ["MC", "AMC", "AMR"],
          comparisonPlayerCount: 24,
        },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const generalHeader = await screen.findByTestId("player-profile-header");
    const generalSummary = within(generalHeader).getByRole("region", {
      name: "Alexandra Maximilian Scout summary",
    });
    const generalTabs = within(generalHeader).getByRole("tablist", {
      name: "Player analysis view",
    });
    const generalDetails = within(generalSummary).getByTestId(
      "player-profile-summary-details",
    );

    expect(generalHeader.firstElementChild).toBe(generalTabs);
    expect(generalHeader.lastElementChild).toBe(generalSummary);
    const generalRail = screen.getByRole("complementary", {
      name: "Player identity",
    });
    expect(
      within(generalRail).getByRole("heading", {
        level: 1,
        name: "Alexandra Maximilian Scout",
      }),
    ).toHaveClass("break-words");
    expect(
      within(generalRail).getByRole("heading", {
        level: 1,
        name: "Alexandra Maximilian Scout",
      }),
    ).not.toHaveClass("truncate");
    expect(generalDetails).toHaveClass(
      "lg:grid-cols-2",
      "xl:grid-cols-3",
      "2xl:grid-cols-5",
    );
    expect(within(generalDetails).getByText("Current Ability")).toBeVisible();
    expect(within(generalDetails).getByText("Potential Ability")).toBeVisible();
    expect(within(generalDetails).getByText("Market Value")).toBeVisible();
    expect(
      within(generalDetails).getByText("Best In-Possession Role"),
    ).toBeVisible();
    expect(
      within(generalDetails).getByText("Best Out-of-Possession Role"),
    ).toBeVisible();
    const generalTactical = within(generalDetails).getByTestId(
      "overview-tactical-fit",
    );
    expect(generalTactical).toHaveClass("contents");
    expect(
      within(generalTactical).getByTestId("overview-tactical-fit-ip"),
    ).toBeInTheDocument();
    expect(
      within(generalTactical).getByTestId("overview-tactical-fit-oop"),
    ).toBeInTheDocument();
    const generalActionSlot = within(generalSummary).getByTestId(
      "player-profile-action-slot",
    );
    expect(generalActionSlot).toHaveClass("min-h-10", "overflow-visible");
    expect(generalActionSlot).not.toHaveClass("overflow-y-auto");
    expect(
      within(generalDetails).getByTestId(
        "player-profile-summary-analysis-details",
      ),
    ).toHaveClass("contents");
    expect(
      within(generalRail).getByRole("img", { name: "England" }),
    ).toHaveAttribute("title", "England");
    expect(
      within(generalRail).getByRole("img", { name: "Wales" }),
    ).toHaveAttribute("title", "Wales");
    expect(within(generalRail).queryByText("England, Wales")).toBeNull();
    expect(
      within(generalSummary).queryByRole("heading", { level: 1 }),
    ).not.toBeInTheDocument();
    expect(
      within(generalTactical).getAllByText(/Current → Potential/),
    ).toHaveLength(2);
    const generalRoleFit = screen.getByRole("region", {
      name: /^Role fit for /,
    });
    expect(generalRoleFit).toHaveClass(
      "lg:grid-cols-[minmax(240px,360px)_minmax(0,1fr)]",
    );
    expect(
      within(generalRoleFit).getByTestId(
        "player-role-position-picker-scroller",
      ),
    ).toHaveClass("min-h-0", "overflow-y-auto");

    const overview = within(generalTabs).getByRole("tab", {
      name: "Overview",
      selected: true,
    });
    overview.focus();
    await user.keyboard("{ArrowRight}");

    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(
      within(generalSummary).queryByTestId("player-profile-summary-details"),
    ).not.toBeInTheDocument();
    expect(
      within(generalSummary).queryByTestId(
        "player-profile-summary-analysis-details",
      ),
    ).not.toBeInTheDocument();
    await user.keyboard("{ArrowRight}");

    expect(
      await screen.findByRole("tab", {
        name: "Role Fit",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(
      within(generalSummary).queryByTestId("player-profile-summary-details"),
    ).not.toBeInTheDocument();
    await user.keyboard("{ArrowRight}");

    const moneyball = await screen.findByRole("tab", {
      name: "Moneyball",
      selected: true,
    });
    await waitFor(() => expect(moneyball).toHaveFocus());

    const moneyballHeader = screen.getByTestId("player-profile-header");
    const moneyballTabs = within(moneyballHeader).getByRole("tablist", {
      name: "Player analysis view",
    });

    expect(moneyballHeader.firstElementChild).toBe(moneyballTabs);
    expect(moneyballHeader.lastElementChild).toBe(moneyballTabs);
    expect(
      within(moneyballHeader).queryByRole("region", {
        name: "Alexandra Maximilian Scout summary",
      }),
    ).not.toBeInTheDocument();
    const moneyballSummary = screen.getByRole("region", {
      name: "Moneyball tactical summaries",
    });
    expect(
      within(moneyballSummary).getByText("Moneyball IP"),
    ).toBeInTheDocument();
    expect(
      within(moneyballSummary).getByText("Moneyball OOP"),
    ).toBeInTheDocument();
    const moneyballRail = screen.getByRole("complementary", {
      name: "Player identity",
    });
    expect(
      within(moneyballRail).getByRole("heading", {
        level: 1,
        name: "Alexandra Maximilian Scout",
      }),
    ).toBeInTheDocument();
    expect(within(moneyballRail).getByText("Age / DOB")).toBeInTheDocument();
    expect(within(moneyballRail).getByText("Nationality")).toBeInTheDocument();
    expect(
      within(moneyballSummary).getByRole("heading", {
        level: 2,
        name: "Moneyball tactical summaries",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Natural positions: MC, AMR, AMC · 24 comparison players",
      ),
    ).toBeInTheDocument();
    const moneyballRoleFit = screen.getByRole("region", {
      name: /^Moneyball role fit for /,
    });
    expect(moneyballRoleFit).toHaveClass(
      "lg:grid-cols-[minmax(240px,360px)_minmax(0,1fr)]",
    );
    expect(
      within(moneyballRoleFit).getByTestId(
        "moneyball-role-position-picker-scroller",
      ),
    ).toHaveClass("min-h-0", "overflow-y-auto");
  });

  it("moves between profile sections with arrow keys", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    const overview = await screen.findByRole("tab", {
      name: "Overview",
      selected: true,
    });
    overview.focus();
    await user.keyboard("{ArrowRight}");

    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({
      section: "attributes",
    });

    await user.keyboard("{ArrowRight}");
    expect(
      await screen.findByRole("tab", {
        name: "Role Fit",
        selected: true,
      }),
    ).toBeInTheDocument();

    await user.keyboard("{ArrowLeft}");
    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();

    await user.keyboard("{End}");
    const moneyball = await screen.findByRole("tab", {
      name: "Moneyball",
      selected: true,
    });
    await waitFor(() => expect(moneyball).toHaveFocus());

    await user.keyboard("{Home}");
    expect(
      await screen.findByRole("tab", {
        name: "Overview",
        selected: true,
      }),
    ).toBeInTheDocument();
  });

  it("restores focus after directly clicking Moneyball", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("tab", { name: "Moneyball", selected: false }),
    );

    const moneyball = await screen.findByRole("tab", {
      name: "Moneyball",
      selected: true,
    });
    await waitFor(() => expect(moneyball).toHaveFocus());
  });

  it("restores analysis-tab focus after delayed Moneyball navigation", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballPending();
    const user = userEvent.setup();
    renderProfileRoute("/players/42?section=role-fit");

    const roleFit = await screen.findByRole("tab", {
      name: "Role Fit",
      selected: true,
    });
    roleFit.focus();
    await user.keyboard("{ArrowRight}");
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => resolve()),
    );

    resolvePendingPlayerMoneyball(fixturePlayerMoneyball());

    const moneyball = await screen.findByRole("tab", {
      name: "Moneyball",
      selected: true,
    });
    await waitFor(() => expect(moneyball).toHaveFocus());
  });

  it("exposes four sections with roving selection", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const tablist = await screen.findByRole("tablist", {
      name: "Player analysis view",
    });
    const tabs = within(tablist).getAllByRole("tab");
    expect(tabs.map((tab) => tab.textContent)).toEqual([
      "Overview",
      "Attributes",
      "Role Fit",
      "Moneyball",
    ]);
    expect(tabs[0]).toHaveAttribute("aria-selected", "true");
    expect(tabs.slice(1)).toSatisfy((rest: Element[]) =>
      rest.every((tab) => tab.getAttribute("aria-selected") === "false"),
    );
    expect(tabs.map((tab) => tab.getAttribute("tabindex"))).toEqual([
      "0",
      "-1",
      "-1",
      "-1",
    ]);
  });

  it("lets a canonical section win over a conflicting legacy view", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42?section=role-fit&view=moneyball");

    expect(
      await screen.findByRole("tab", { name: "Role Fit", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Role fit for MC" }),
    ).toBeInTheDocument();
  });

  it("renders canonical section URLs with the matching panel", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const first = renderProfileRoute("/players/42?section=attributes");

    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Role fit for MC" }),
    ).not.toBeInTheDocument();
    first.unmount();

    renderProfileRoute("/players/42?section=moneyball");
    expect(
      await screen.findByRole("tab", {
        name: "Moneyball",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("Starts")).toBeInTheDocument();
  });

  it("retains the player across section switches", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    expect(
      await screen.findByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();
    for (const name of ["Attributes", "Role Fit", "Moneyball", "Overview"]) {
      await user.click(screen.getByRole("tab", { name }));
      expect(
        await screen.findByRole("tab", { name, selected: true }),
      ).toBeInTheDocument();
      expect(
        within(screen.getByTestId("player-identity-rail")).getByRole(
          "heading",
          { level: 1, name: "Alex Scout" },
        ),
      ).toBeInTheDocument();
    }
    expect(router.state.location.search).toMatchObject({
      section: "overview",
    });
  });

  it("keeps section URLs direct-linkable across Back and Forward", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const { router } = renderProfileRoute("/players/42?section=attributes");

    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();

    await router.navigate({
      to: "/players/$uid",
      params: { uid: "42" },
      search: { section: "role-fit" },
    });
    expect(
      await screen.findByRole("tab", {
        name: "Role Fit",
        selected: true,
      }),
    ).toBeInTheDocument();

    router.history.back();
    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();

    router.history.forward();
    expect(
      await screen.findByRole("tab", {
        name: "Role Fit",
        selected: true,
      }),
    ).toBeInTheDocument();
  });

  it("keeps tab section switches on one history entry behind Search Back/Forward", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, staleTime: 60_000 },
      },
    });
    const router = createRouter({
      routeTree,
      context: { queryClient } satisfies RouterContext,
      defaultPreloadStaleTime: 0,
      history: createMemoryHistory({
        initialEntries: ["/search", "/players/42?view=general"],
        initialIndex: 1,
      }),
    });
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();

    expect(
      await screen.findByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Attributes" }));
    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({
      section: "attributes",
    });

    router.history.back();
    expect(
      await screen.findByRole("heading", { name: "Player Search" }),
    ).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/search");

    router.history.forward();
    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(
      within(screen.getByTestId("player-identity-rail")).getByRole("heading", {
        level: 1,
        name: "Alex Scout",
      }),
    ).toBeInTheDocument();
  });

  it("renders standard sections without waiting for Moneyball data", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballPending();
    renderProfileRoute("/players/42?section=attributes");

    const rail = await screen.findByTestId("player-identity-rail");
    expect(
      within(rail).getByRole("heading", { level: 1, name: "Alex Scout" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();

    resolvePendingPlayerMoneyball(fixturePlayerMoneyball());
  });

  it("prefetches Moneyball data in the loader for resolved Moneyball sections", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const loaderOption = Route.options.loader;
    expect(loaderOption).toBeDefined();
    const loader =
      typeof loaderOption === "function" ? loaderOption : loaderOption?.handler;
    expect(loader).toBeDefined();
    if (!loader) throw new Error("Expected the profile route loader");

    const moneyballClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    await loader({
      context: { queryClient: moneyballClient },
      params: { uid: "42" },
      deps: { section: "moneyball" },
    } as never);
    expect(moneyballClient.getQueryData(moneyballKeys.profile(42))).toEqual(
      expect.objectContaining({ state: "ready" }),
    );

    const standardClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    await loader({
      context: { queryClient: standardClient },
      params: { uid: "42" },
      deps: { section: "attributes" },
    } as never);
    expect(
      standardClient.getQueryData(moneyballKeys.profile(42)),
    ).toBeUndefined();
  });

  it("falls through an unknown profile view to the saved default", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const first = renderProfileRoute("/players/42?view=not-a-view");

    expect(
      await screen.findByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();
    first.unmount();

    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    renderProfileRoute("/players/42?view=not-a-view");

    expect(
      await screen.findByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeInTheDocument();
  });

  it("starts goalkeeper profiles with goalkeeper mental and physical attributes", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { GK: 20 },
        attributes: {
          AerialReach: 13,
          CommandOfArea: 12,
          Communication: 11,
          Eccentricity: 10,
          FirstTouch: 9,
          Handling: 14,
          Kicking: 15,
          OneOnOnes: 16,
          Passing: 8,
          Punching: 7,
          Reflexes: 17,
          RushingOut: 13,
          Technique: 6,
          Throwing: 12,
        },
      }),
    );
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    const attributeTablist = await screen.findByRole("tablist", {
      name: "Attribute groups",
    });
    const tabs = within(attributeTablist).getAllByRole("tab");
    expect(tabs.map((item) => item.textContent)).toEqual([
      "Goalkeeping",
      "Outfield",
      "Hidden",
      "Personality",
    ]);
    expect(tabs[0]).toHaveAttribute("aria-selected", "true");

    const goalkeeping = screen.getByRole("tabpanel", { name: "Goalkeeping" });
    expect(
      within(goalkeeping)
        .getAllByRole("heading", { level: 3 })
        .map((item) => item.textContent),
    ).toEqual(["Goalkeeping", "Mental", "Physical"]);
    expect(
      within(goalkeeping)
        .getAllByRole("term")
        .map((item) => item.textContent),
    ).toEqual([
      "Aerial Reach",
      "Command Of Area",
      "Communication",
      "Eccentricity",
      "First Touch",
      "Handling",
      "Kicking",
      "One On Ones",
      "Passing",
      "Punching",
      "Reflexes",
      "Rushing Out",
      "Technique",
      "Throwing",
      "Aggression",
      "Anticipation",
      "Bravery",
      "Composure",
      "Concentration",
      "Decisions",
      "Determination",
      "Flair",
      "Leadership",
      "Off The Ball",
      "Positioning",
      "Teamwork",
      "Vision",
      "Work Rate",
      "Acceleration",
      "Agility",
      "Balance",
      "Jumping Reach",
      "Natural Fitness",
      "Pace",
      "Stamina",
      "Strength",
    ]);

    await user.click(screen.getByRole("tab", { name: "Outfield" }));

    const outfield = screen.getByRole("tabpanel", { name: "Outfield" });
    expect(
      within(outfield)
        .getAllByRole("heading", { level: 3 })
        .map((item) => item.textContent),
    ).toEqual(["Technical"]);
    const technical = within(outfield).getByRole("region", {
      name: "Technical",
    });
    expect(
      within(technical).queryByText("First Touch"),
    ).not.toBeInTheDocument();
    expect(within(technical).queryByText("Passing")).not.toBeInTheDocument();
    expect(within(technical).queryByText("Technique")).not.toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "outfield" });
  });

  it("honors an explicit outfield tab on goalkeeper profiles", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ positions: { GK: 20 } }));
    renderProfileRoute("/players/42?tab=outfield");

    const attributeTablist = await screen.findByRole("tablist", {
      name: "Attribute groups",
    });
    const tabs = within(attributeTablist).getAllByRole("tab");
    expect(tabs.map((item) => item.textContent)).toEqual([
      "Goalkeeping",
      "Outfield",
      "Hidden",
      "Personality",
    ]);
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
  });

  it("keeps the hidden-information control separate from modification actions", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const displayControl = within(summary).getByTestId(
      "player-profile-display-control",
    );
    const actionSlot = within(summary).getByTestId(
      "player-profile-action-slot",
    );
    const hiddenInformation = within(displayControl).getByRole("button", {
      name: "Reveal hidden information",
    });

    expect(
      within(actionSlot).getByRole("button", { name: "Boost CA" }),
    ).toBeInTheDocument();
    expect(
      within(actionSlot).getByRole("button", {
        name: "Wonderkid Mentality",
      }),
    ).toBeInTheDocument();
    expect(
      within(actionSlot).queryByRole("button", {
        name: "Reveal hidden information",
      }),
    ).not.toBeInTheDocument();
    expect(
      within(displayControl).queryByRole("button", { name: "Boost CA" }),
    ).not.toBeInTheDocument();
    expect(actionSlot.contains(hiddenInformation)).toBe(false);
  });

  it("hides modification actions behind Modify Player until opened", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const actionSlot = within(summary).getByTestId(
      "player-profile-action-slot",
    );

    const disclosure = within(actionSlot).getByRole("button", {
      name: "Modify Player",
    });
    expect(disclosure).toHaveAttribute("aria-expanded", "false");
    const controlledPanelId = disclosure.getAttribute("aria-controls");
    expect(controlledPanelId).toBeTruthy();
    const controlledPanel = document.getElementById(controlledPanelId ?? "");
    expect(controlledPanel).toBeInTheDocument();
    expect(controlledPanel).toHaveAttribute("hidden");
    expect(controlledPanel).toHaveAttribute("aria-hidden", "true");
    expect(controlledPanel).not.toBeVisible();
    expect(controlledPanel?.querySelector("button")).toBeInTheDocument();
    expect(
      within(actionSlot).queryByRole("button", { name: "Boost CA" }),
    ).not.toBeInTheDocument();
    expect(
      within(actionSlot).queryByRole("button", {
        name: "Wonderkid Mentality",
      }),
    ).not.toBeInTheDocument();
  });

  it("opens and closes modification actions from the keyboard", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const actionSlot = within(summary).getByTestId(
      "player-profile-action-slot",
    );
    const disclosure = within(actionSlot).getByRole("button", {
      name: "Modify Player",
    });

    disclosure.focus();
    expect(disclosure).toHaveFocus();
    await user.keyboard("{Enter}");

    expect(disclosure).toHaveAttribute("aria-expanded", "true");
    expect(
      within(actionSlot).getByRole("button", { name: "Boost CA" }),
    ).toBeInTheDocument();
    expect(
      within(actionSlot).getByRole("button", {
        name: "Wonderkid Mentality",
      }),
    ).toBeInTheDocument();

    await user.keyboard("{Enter}");

    expect(disclosure).toHaveAttribute("aria-expanded", "false");
    expect(
      within(actionSlot).queryByRole("button", { name: "Boost CA" }),
    ).not.toBeInTheDocument();
  });

  it("keeps the hidden-information toggle outside the Modify Player disclosure", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const displayControl = within(summary).getByTestId(
      "player-profile-display-control",
    );
    const actionSlot = within(summary).getByTestId(
      "player-profile-action-slot",
    );
    const disclosure = within(actionSlot).getByRole("button", {
      name: "Modify Player",
    });

    await user.click(disclosure);

    expect(
      within(displayControl).getByRole("button", {
        name: "Reveal hidden information",
      }),
    ).toBeInTheDocument();
    expect(
      within(actionSlot).queryByRole("button", {
        name: "Reveal hidden information",
      }),
    ).not.toBeInTheDocument();
    expect(
      within(displayControl).queryByRole("button", { name: "Boost CA" }),
    ).not.toBeInTheDocument();
    expect(
      within(displayControl).queryByRole("button", {
        name: "Modify Player",
      }),
    ).not.toBeInTheDocument();
  });

  it("conceals hidden information without leaving direct or indirect values in the profile", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        hiddenInformationRevealed: true,
        attributes: { Acceleration: 14 },
        potentialAttributes: { Acceleration: 16 },
        hiddenAttributes: { Consistency: 12 },
        personality: { Ambition: 10 },
      }),
    );
    const user = userEvent.setup();
    const { queryClient } = renderProfileRoute("/players/42");
    queryClient.setQueryData(["staff", "probe"], []);

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const toggle = within(summary).getByRole("button", {
      name: "Reveal hidden information",
    });
    expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(within(summary).getByText("160")).toBeInTheDocument();

    await user.click(toggle);

    const reveal = await within(summary).findByRole("button", {
      name: "Reveal hidden information",
    });
    expect(reveal).toHaveAttribute("aria-pressed", "false");
    expect(within(summary).queryByText("PA")).not.toBeInTheDocument();
    expect(within(summary).queryByText("160")).not.toBeInTheDocument();
    expect(within(summary).queryByText("Boost CA")).not.toBeInTheDocument();
    expect(
      within(summary).queryByText("Wonderkid Mentality"),
    ).not.toBeInTheDocument();
    const tactical = within(summary).getByTestId("overview-tactical-fit");
    const ip = within(tactical).getByTestId("overview-tactical-fit-ip");
    const oop = within(tactical).getByTestId("overview-tactical-fit-oop");
    expect(within(ip).queryByRole("img")).not.toBeInTheDocument();
    expect(within(oop).queryByRole("img")).not.toBeInTheDocument();
    const concealedIpDash = within(ip).getByText("—");
    expect(concealedIpDash).not.toHaveAttribute("title");
    expect(within(oop).getAllByText("—")).toHaveLength(3);
    expect(
      within(ip).getByText(
        "Deep-Lying Playmaker, In possession: Current 82, Potential concealed",
      ),
    ).toBeInTheDocument();
    expect(
      within(tactical).getByText("Deep-Lying Playmaker"),
    ).toBeInTheDocument();
    expect(
      within(oop).getByText(
        "Out of possession: Current unavailable, Potential concealed",
      ),
    ).toBeInTheDocument();
    expect(within(tactical).queryByText("Concealed")).not.toBeInTheDocument();

    const technical = screen.getByRole("region", { name: "Technical" });
    expect(
      within(technical).queryByText("Current 14, Potential 16"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Consistency")).not.toBeInTheDocument();
    expect(screen.queryByText("Ambition")).not.toBeInTheDocument();

    const roleFit = screen.getByRole("region", { name: "Role fit for MC" });
    expect(
      within(roleFit).queryByRole("columnheader", { name: "Potential" }),
    ).not.toBeInTheDocument();
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .every((row) => within(row).getAllByRole("cell").length === 3),
    ).toBe(true);
    expect(getSetPlayerHiddenInformationRevealedIpcMockCalls()).toEqual([
      { revealed: false },
    ]);
    expect(queryClient.getQueryState(["staff", "probe"])?.isInvalidated).toBe(
      true,
    );

    await user.click(reveal);
    expect(
      await within(summary).findByRole("button", {
        name: "Reveal hidden information",
      }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(within(summary).getByText("160")).toBeInTheDocument();
    expect(screen.getByText("Current 14, Potential 16")).toBeInTheDocument();
  });

  it("keeps the server-backed visibility state and reports setter failures", async () => {
    await resolveLoadDataIpcMock();
    setPlayerHiddenInformationRevealedIpcMockMode("error");
    setGetPlayerOverride(
      fixturePlayerDetail({ hiddenInformationRevealed: true }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    await user.click(
      within(summary).getByRole("button", {
        name: "Reveal hidden information",
      }),
    );

    expect(await within(summary).findByRole("alert")).toHaveTextContent(
      /^Could not update hidden information\.$/,
    );
    expect(
      within(summary).getByRole("button", {
        name: "Reveal hidden information",
      }),
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("drops visibility mutation feedback when the active save changes", async () => {
    await resolveLoadDataIpcMock();
    setPlayerHiddenInformationRevealedIpcMockMode("error");
    setGetPlayerOverride(
      fixturePlayerDetail({ hiddenInformationRevealed: true }),
    );
    const user = userEvent.setup();
    const { queryClient } = renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    await user.click(
      within(summary).getByRole("button", {
        name: "Reveal hidden information",
      }),
    );
    expect(await within(summary).findByRole("alert")).toBeInTheDocument();

    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      snapshotKeys.current(),
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot in the profile query");
    }
    queryClient.setQueryData(snapshotKeys.current(), {
      ...snapshot,
      saveId: snapshot.saveId + 1,
    });

    await waitFor(() =>
      expect(within(summary).queryByRole("alert")).not.toBeInTheDocument(),
    );
    expect(
      within(summary).getByRole("button", {
        name: "Reveal hidden information",
      }),
    ).toBeEnabled();
  });

  it("keeps player context beside tabbed attributes and position-filtered roles", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { MC: 20, ST: 15 },
        attributes: { Passing: 14, Determination: 12 },
        potentialAttributes: { Passing: 17, Determination: 14 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42?tab=technical");

    expect(
      await screen.findByRole("region", { name: "Alex Scout summary" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "MC, familiarity 20", pressed: true }),
    ).toBeInTheDocument();

    const roleFit = screen.getByRole("region", { name: "Role fit for MC" });
    expect(
      within(roleFit).getByText("Deep-Lying Playmaker"),
    ).toBeInTheDocument();
    expect(within(roleFit).getByText("Central Midfielder")).toBeInTheDocument();
    expect(
      within(roleFit).queryByText("Advanced Forward"),
    ).not.toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "ST, familiarity 15" }),
    );

    const strikerFit = screen.getByRole("region", { name: "Role fit for ST" });
    expect(
      within(strikerFit).getByText("Advanced Forward"),
    ).toBeInTheDocument();
    expect(
      within(strikerFit).queryByText("Deep-Lying Playmaker"),
    ).not.toBeInTheDocument();
  });

  it("gives Attributes full-width geometry and one local scroll surface", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42?section=attributes");

    const analysisPanel = await screen.findByRole("tabpanel", {
      name: "Attributes",
    });
    expect(analysisPanel).toHaveClass(
      "min-h-0",
      "min-w-0",
      "w-full",
      "flex-1",
      "lg:h-0",
    );

    const attributesHeading = within(analysisPanel).getByRole("heading", {
      name: "Attributes",
    });
    const attributes = attributesHeading.closest("section");
    if (!attributes) {
      throw new Error("Expected the Attributes panel section");
    }
    expect(attributes).toHaveClass("w-full");
    expect(
      within(analysisPanel).queryByRole("heading", { name: "Role fit" }),
    ).not.toBeInTheDocument();

    const scrollContainers = attributes.querySelectorAll(".overflow-y-auto");
    expect(scrollContainers).toHaveLength(1);
    expect(scrollContainers[0]).toHaveClass(
      "min-h-0",
      "flex-1",
      "overflow-y-auto",
    );

    const localTabs = within(attributes).getByRole("tablist", {
      name: "Attribute groups",
    });
    expect(
      within(analysisPanel).queryByRole("tablist", {
        name: "Player analysis view",
      }),
    ).not.toBeInTheDocument();

    const outfield = within(localTabs).getByRole("tab", {
      name: "Outfield",
      selected: true,
    });
    outfield.focus();
    await user.keyboard("{ArrowRight}");

    expect(
      within(localTabs).getByRole("tab", {
        name: "Goalkeeping",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({
      section: "attributes",
      tab: "goalkeeping",
    });
    expect(
      screen.getByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
  });

  it("mutes unfamiliar positions without lowering playable thresholds", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: {
          AMR: 20,
          MR: 15,
          AMC: 14,
          DL: 5,
          DC: 6,
          DR: null,
          SW: 18,
          GK: 14,
          ST: 0,
          WBL: null,
        },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    expect(
      await screen.findByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "AMR, familiarity 20" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "MR, familiarity 15" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "AMC, familiarity 14" }),
    ).toHaveClass(
      "border-outline-variant",
      "bg-surface-container/85",
      "text-on-surface-variant",
    );
    expect(
      within(
        screen.getByRole("button", { name: "AMC, familiarity 14" }),
      ).getByText("14"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "DL, familiarity 5" }),
    ).toHaveClass(
      "border-outline-variant",
      "bg-surface-container/85",
      "text-on-surface-variant",
    );
    expect(
      screen.getByRole("button", { name: "DC, familiarity 6" }),
    ).toHaveClass("bg-surface-container/85");
    const unrecordedPosition = screen.getByRole("button", {
      name: "DR, no recorded familiarity",
    });
    expect(unrecordedPosition).toHaveClass(
      "border-outline-variant",
      "bg-surface-container/85",
      "text-on-surface-variant",
    );
    expect(within(unrecordedPosition).getByText("—")).toBeInTheDocument();
    const zeroFamiliarityPosition = screen.getByRole("button", {
      name: "ST, no recorded familiarity",
    });
    expect(zeroFamiliarityPosition).toHaveClass(
      "border-outline-variant",
      "bg-surface-container/85",
      "text-on-surface-variant",
    );
    expect(within(zeroFamiliarityPosition).getByText("—")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "MR, familiarity 15" }),
    ).not.toHaveClass("bg-surface-container/85", "text-on-surface-variant");
    expect(
      screen.queryByRole("button", { name: "SW, familiarity 18" }),
    ).toBeNull();

    const lowFamiliarityPosition = screen.getByRole("button", {
      name: "AMC, familiarity 14",
    });
    expect(lowFamiliarityPosition).toHaveAttribute("aria-pressed", "false");
    expect(lowFamiliarityPosition).not.toHaveClass("opacity-45");

    await user.click(lowFamiliarityPosition);

    expect(lowFamiliarityPosition).toHaveAttribute("aria-pressed", "true");
    expect(lowFamiliarityPosition).toHaveClass(
      "border-primary",
      "bg-primary-container",
      "text-on-primary-container",
    );
    expect(lowFamiliarityPosition).not.toHaveClass(
      "border-outline-variant",
      "bg-surface-container/85",
      "text-on-surface-variant",
    );
  });

  it("marks selected positions without relying on colour and keeps keyboard selection", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { MC: 20, ST: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const selectedPosition = await screen.findByRole("button", {
      name: "MC, familiarity 20",
      pressed: true,
    });
    expect(selectedPosition).toHaveClass("border-2");

    const keyboardPosition = screen.getByRole("button", {
      name: "ST, familiarity 15",
      pressed: false,
    });
    keyboardPosition.focus();
    await user.keyboard("{Enter}");

    expect(keyboardPosition).toHaveAttribute("aria-pressed", "true");
    expect(keyboardPosition).toHaveClass("border-2");
    expect(selectedPosition).toHaveAttribute("aria-pressed", "false");
  });

  it("shows not-found empty state for an unknown uid", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(null);
    renderProfileRoute("/players/999");

    expect(
      await screen.findByText("Player not in this snapshot"),
    ).toBeInTheDocument();
  });

  it("normalizes legacy visible tabs to the outfield panel", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42?tab=mental");

    expect(
      await screen.findByRole("tab", { name: "Outfield", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Mental" })).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Technical" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Physical" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Goalkeeping" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Hidden" }));

    expect(
      await screen.findByRole("tab", { name: "Hidden", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getByText("Current only")).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "hidden" });
  });

  it("moves between tabs with arrow keys", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    const outfield = await screen.findByRole("tab", {
      name: "Outfield",
      selected: true,
    });
    outfield.focus();
    await user.keyboard("{ArrowRight}");

    expect(
      await screen.findByRole("tab", { name: "Goalkeeping", selected: true }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "goalkeeping" });

    await user.keyboard("{ArrowRight}");
    expect(
      await screen.findByRole("tab", { name: "Hidden", selected: true }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "hidden" });
  });

  it("shows Load Data empty state when no snapshot is loaded", async () => {
    setGetPlayerOverride(null);
    renderProfileRoute("/players/42");

    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
  });

  it("shows visible attributes as current to potential values and keeps other groups current-only", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: {
          Acceleration: 14,
          Crossing: null,
          Handling: 11,
        },
        potentialAttributes: {
          Acceleration: 16,
          Crossing: null,
          Handling: 12,
        },
        hiddenAttributes: {
          Consistency: null,
          Dirtiness: 8,
        },
        personality: {
          Ambition: 15,
          Loyalty: null,
        },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42?tab=technical");

    const technical = await screen.findByRole("region", { name: "Technical" });
    expect(screen.getByRole("region", { name: "Mental" })).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Physical" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Set Pieces" }),
    ).toBeInTheDocument();
    const crossingTerm = within(technical).getByText("Crossing");
    expect(
      crossingTerm.parentElement?.querySelector('[aria-hidden="true"]'),
    ).toHaveTextContent("—→—");
    expect(
      crossingTerm.parentElement?.querySelector(".sr-only"),
    ).toHaveTextContent("Current —, Potential —");

    const physical = screen.getByRole("region", { name: "Physical" });
    const accelerationTerm = within(physical).getByText("Acceleration");
    expect(
      accelerationTerm.parentElement?.querySelector('[aria-hidden="true"]'),
    ).toHaveTextContent("14→16");
    expect(
      within(physical).getByText("Current 14, Potential 16").parentElement,
    ).toHaveClass("tabular-nums");
    expect(
      accelerationTerm.parentElement?.querySelector('[data-tier="3"]'),
    ).toHaveAttribute("title", "Good");
    expect(
      accelerationTerm.parentElement?.querySelector('[data-tier="4"]'),
    ).toHaveAttribute("title", "Excellent");

    await user.click(screen.getByRole("tab", { name: "Hidden" }));
    const hidden = screen.getByRole("region", { name: "Hidden" });
    expect(
      within(hidden).getByText("Consistency").parentElement,
    ).toHaveTextContent(/^Consistency—$/);
    expect(
      within(hidden).getByText("Dirtiness").parentElement,
    ).toHaveTextContent(/^Dirtiness8$/);

    await user.click(screen.getByRole("tab", { name: "Personality" }));
    const personality = screen.getByRole("region", { name: "Personality" });
    expect(
      within(personality).getByText("Ambition").parentElement,
    ).toHaveTextContent(/^Ambition15$/);
    expect(
      within(personality).getByText("Loyalty").parentElement,
    ).toHaveTextContent(/^Loyalty—$/);
  });

  it("shows paired same-role tactical-fit summaries only on Overview", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { MC: 15, AMC: 16, ST: 14 },
        roleScores: [
          {
            roleId: "current-ip-specialist",
            displayName: "Current IP Specialist",
            phase: "in_possession",
            positionTags: ["MC"],
            score: 82,
            potentialScore: 88,
          },
          {
            roleId: "current-ip-tie",
            displayName: "Current IP Tie",
            phase: "in_possession",
            positionTags: ["AMC"],
            score: 82,
            potentialScore: 80,
          },
          {
            roleId: "potential-ip-specialist",
            displayName: "Potential IP Specialist",
            phase: "in_possession",
            positionTags: ["AMC"],
            score: 70,
            potentialScore: 94,
          },
          {
            roleId: "current-oop-specialist",
            displayName: "Current OOP Specialist",
            phase: "out_of_possession",
            positionTags: ["MC"],
            score: 79,
            potentialScore: 90,
          },
          {
            roleId: "potential-oop-specialist",
            displayName: "Potential OOP Specialist",
            phase: "out_of_possession",
            positionTags: ["AMC"],
            score: 74,
            potentialScore: 93,
          },
          {
            roleId: "unplayable-specialist",
            displayName: "Unplayable Specialist",
            phase: "in_possession",
            positionTags: ["ST"],
            score: 99,
            potentialScore: 100,
          },
        ],
      }),
    );
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const tactical = within(summary).getByRole("region", {
      name: "Tactical fit",
    });
    expect(tactical).toHaveAttribute("data-testid", "overview-tactical-fit");
    const ip = within(tactical).getByTestId("overview-tactical-fit-ip");
    const oop = within(tactical).getByTestId("overview-tactical-fit-oop");

    expect(within(ip).getByText("Best In-Possession Role")).toBeInTheDocument();
    expect(
      within(oop).getByText("Best Out-of-Possession Role"),
    ).toBeInTheDocument();
    expect(ip).toHaveClass("tabular-nums");
    expect(oop).toHaveClass("tabular-nums");
    expect(within(ip).queryByRole("img")).not.toBeInTheDocument();
    expect(within(oop).queryByRole("img")).not.toBeInTheDocument();
    expect(
      within(ip).getByText(
        "Current IP Specialist, In possession: Current 82, Potential 88",
      ),
    ).toBeInTheDocument();
    expect(
      within(oop).getByText(
        "Current OOP Specialist, Out of possession: Current 79, Potential 90",
      ),
    ).toBeInTheDocument();
    expect(within(ip).getByText("Current IP Specialist")).toBeInTheDocument();
    expect(within(oop).getByText("Current OOP Specialist")).toBeInTheDocument();
    expect(
      within(tactical).queryByText("Potential IP Specialist"),
    ).not.toBeInTheDocument();
    expect(
      within(tactical).queryByText("Potential OOP Specialist"),
    ).not.toBeInTheDocument();
    expect(
      within(tactical).queryByText("Current IP Tie"),
    ).not.toBeInTheDocument();
    expect(
      within(tactical).queryByText("Unplayable Specialist"),
    ).not.toBeInTheDocument();
    expect(within(tactical).queryByText("Current IP")).not.toBeInTheDocument();
    expect(
      within(tactical).queryByText("Potential IP"),
    ).not.toBeInTheDocument();
    expect(within(tactical).queryByText("Current OOP")).not.toBeInTheDocument();
    expect(
      within(tactical).queryByText("Potential OOP"),
    ).not.toBeInTheDocument();
    expect(within(summary).getByTestId("overview-ability")).toBeInTheDocument();
  });

  it("keeps Moneyball role summaries out of standard profile sections", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    expect(
      within(summary).getByTestId("overview-tactical-fit"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Moneyball tactical summaries" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Attributes" }));
    expect(
      await screen.findByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit"),
    ).not.toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit-ip"),
    ).not.toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit-oop"),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Moneyball tactical summaries" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Role Fit" }));
    expect(
      await screen.findByRole("tab", { name: "Role Fit", selected: true }),
    ).toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit"),
    ).not.toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit-ip"),
    ).not.toBeInTheDocument();
    expect(
      within(summary).queryByTestId("overview-tactical-fit-oop"),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("region", { name: "Moneyball tactical summaries" }),
    ).not.toBeInTheDocument();
  });

  it("renders unavailable paired potentials as a neutral dash without a badge", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        roleScores: [
          {
            roleId: "current-only",
            displayName: "Current Only",
            phase: "in_possession",
            positionTags: ["MC"],
            score: 82,
            potentialScore: null,
          },
        ],
      }),
    );
    renderProfileRoute("/players/42");

    const tactical = await screen.findByTestId("overview-tactical-fit");
    const ip = within(tactical).getByTestId("overview-tactical-fit-ip");
    expect(within(ip).queryByRole("img")).not.toBeInTheDocument();
    const potentialIp = within(ip).getByText("—");
    expect(potentialIp).not.toHaveAttribute("title");
    expect(
      within(ip).getByText(
        "Current Only, In possession: Current 82, Potential unavailable",
      ),
    ).toBeInTheDocument();

    const oop = within(tactical).getByTestId("overview-tactical-fit-oop");
    expect(within(oop).queryByRole("img")).not.toBeInTheDocument();
    expect(within(oop).getAllByText("—")).toHaveLength(3);
    expect(
      within(oop).getByText(
        "Out of possession: Current unavailable, Potential unavailable",
      ),
    ).toBeInTheDocument();
  });

  it("filters roles by pitch position with labelled current and potential badges", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    renderProfileRoute("/players/42?tab=technical");

    const midfield = await screen.findByRole("region", {
      name: "Role fit for MC",
    });

    expect(
      within(midfield).getByLabelText(
        "Deep-Lying Playmaker (Current): 82, Excellent",
      ),
    ).toBeInTheDocument();
    expect(
      within(midfield).getByLabelText(
        "Deep-Lying Playmaker (Potential): 94, Excellent",
      ),
    ).toBeInTheDocument();
    expect(
      within(midfield).getByLabelText("Central Midfielder (Current): 72, Good"),
    ).toBeInTheDocument();
    expect(
      within(midfield).getByLabelText(
        "Central Midfielder (Potential): 84, Excellent",
      ),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "GK, no recorded familiarity" }),
    );
    const goalkeeper = screen.getByRole("region", { name: "Role fit for GK" });
    expect(
      within(goalkeeper).getByLabelText("Goalkeeper (Current): 40, Weak"),
    ).toBeInTheDocument();
    expect(
      within(goalkeeper).getByLabelText("Goalkeeper (Potential): 47, Average"),
    ).toBeInTheDocument();
    expect(within(goalkeeper).getByText("IP")).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "ST, no recorded familiarity" }),
    );
    const striker = screen.getByRole("region", { name: "Role fit for ST" });
    expect(
      within(striker).getByLabelText("Advanced Forward (Current): 55, Average"),
    ).toBeInTheDocument();
    expect(
      within(striker).getByLabelText("Advanced Forward (Potential): 67, Good"),
    ).toBeInTheDocument();
  });

  it("aligns Role Fit role, phase, current, and potential columns", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        roleScores: Array.from({ length: 79 }, (_, index) => ({
          roleId: `catalog-role-${index}`,
          displayName: `Catalog Role ${index + 1}`,
          phase: index % 2 === 0 ? "in_possession" : "out_of_possession",
          positionTags: ["MC"],
          score: 60,
          potentialScore: 70,
        })),
      }),
    );
    renderProfileRoute("/players/42?tab=technical");

    const roleFit = await screen.findByRole("region", {
      name: "Role fit for MC",
    });
    expect(
      within(roleFit).getByRole("columnheader", { name: /Role/ }),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByRole("columnheader", { name: "Phase" }),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByRole("columnheader", { name: "Current" }),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByRole("columnheader", { name: "Potential" }),
    ).toBeInTheDocument();

    const roleRows = within(roleFit).getAllByRole("row").slice(1);
    expect(roleRows).toHaveLength(79);
    roleRows.forEach((row, index) => {
      const [roleCell, phaseCell, currentCell, potentialCell] =
        within(row).getAllByRole("cell");
      const expectedPhase = index % 2 === 0 ? "IP" : "OOP";
      const expectedFullPhase =
        expectedPhase === "IP" ? "In possession" : "Out of possession";
      const phaseAbbreviation = within(phaseCell).getByText(expectedPhase, {
        exact: true,
      });
      const phaseChip = phaseAbbreviation.parentElement;
      expect(within(roleCell).queryByRole("img")).not.toBeInTheDocument();
      expect(phaseChip).toBeInTheDocument();
      expect(phaseChip).not.toHaveAttribute("role");
      expect(phaseAbbreviation).toHaveAttribute("aria-hidden", "true");
      expect(
        within(phaseChip as HTMLElement).getByText(expectedFullPhase, {
          exact: true,
        }),
      ).toHaveClass("sr-only");
      expect(phaseChip).toHaveTextContent(expectedPhase);
      expect(phaseChip).toHaveTextContent(expectedFullPhase);
      expect(currentCell).toHaveClass("text-right", "tabular-nums");
      expect(potentialCell).toHaveClass("text-right", "tabular-nums");
    });
    expect(
      within(roleFit).getByLabelText("Catalog Role 1 (Current): 60, Average"),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByLabelText("Catalog Role 79 (Current): 60, Average"),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getByLabelText("Catalog Role 79 (Potential): 70, Good"),
    ).toBeInTheDocument();
    expect(
      within(roleFit).getAllByLabelText(/Catalog Role \d+ \(Current\):/),
    ).toHaveLength(79);
    expect(
      within(roleFit).getAllByLabelText(/Catalog Role \d+ \(Potential\):/),
    ).toHaveLength(79);
  });

  it("sorts roles from the Current and Potential column headers", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        positions: { MC: 20 },
        roleScores: [
          {
            roleId: "current-leader",
            displayName: "Current Leader",
            phase: "in_possession",
            positionTags: ["MC"],
            score: 90,
            potentialScore: 72,
          },
          {
            roleId: "potential-leader",
            displayName: "Potential Leader",
            phase: "out_of_possession",
            positionTags: ["MC"],
            score: 60,
            potentialScore: 95,
          },
        ],
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const roleFit = await screen.findByRole("region", {
      name: "Role fit for MC",
    });
    const currentHeader = within(roleFit).getByRole("columnheader", {
      name: "Current",
    });
    const potentialHeader = within(roleFit).getByRole("columnheader", {
      name: "Potential",
    });

    expect(currentHeader).toHaveAttribute("aria-sort", "descending");
    expect(potentialHeader).not.toHaveAttribute("aria-sort");

    await user.click(
      within(currentHeader).getByRole("button", { name: "Current" }),
    );
    expect(currentHeader).toHaveAttribute("aria-sort", "ascending");
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .map((row) => within(row).getAllByRole("cell")[0].textContent),
    ).toEqual(["Potential Leader", "Current Leader"]);

    await user.click(
      within(potentialHeader).getByRole("button", { name: "Potential" }),
    );
    expect(potentialHeader).toHaveAttribute("aria-sort", "descending");
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .map((row) => within(row).getAllByRole("cell")[0].textContent),
    ).toEqual(["Potential Leader", "Current Leader"]);

    await user.click(
      within(potentialHeader).getByRole("button", { name: "Potential" }),
    );
    expect(potentialHeader).toHaveAttribute("aria-sort", "ascending");
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .map((row) => within(row).getAllByRole("cell")[0].textContent),
    ).toEqual(["Current Leader", "Potential Leader"]);
  });

  it("previews and confirms the age-21 CA boost from the current snapshot", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    const action = await screen.findByRole("button", { name: "Boost CA" });
    expect(screen.getByText("CA 140 → 150 (+10)")).toBeInTheDocument();

    await user.click(action);

    const dialog = screen.getByRole("dialog");
    expect(
      within(dialog).getByRole("heading", { level: 2, name: "Boost CA?" }),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText("This raises current ability from 140 to 150."),
    ).toBeInTheDocument();
    expect(
      within(dialog).getAllByText(
        "FM may redistribute attributes over the following in-game days, sometimes up to one month.",
      ),
    ).toHaveLength(1);

    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));

    expect(
      await screen.findByText("CA boosted from 140 to 150."),
    ).toBeInTheDocument();
    expect(screen.getByText("CA 150 → 160 (+10)")).toBeInTheDocument();
    expect(getCurrentAbilityBoostIpcMockCalls()).toEqual([{ uid: 42 }]);
  });

  it("uses the age-28 increment while capping the preview at PA", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 28, ca: 192, pa: 195 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText("CA 192 → 195 (+3) · capped by PA"),
    ).toBeInTheDocument();
  });

  it("disables CA boost when age is unknown", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: null }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText(
        "Age is unavailable. Load Data again to refresh this player.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();
  });

  it("disables CA boost at age 29 without invoking the bridge", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 29, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText(
        "Current ability boosts are unavailable for players aged 29 or older.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();
    expect(getCurrentAbilityBoostIpcMockCalls()).toEqual([]);
  });

  it("disables CA boost when PA is unavailable", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ pa: null }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText(
        "Potential ability is unavailable. Load Data again to refresh this player.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();
  });

  it("disables CA boost when CA already equals PA", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ ca: 160, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText(
        "Current ability is already at this player’s potential ability.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();
  });

  it("disables CA boost at the 200 ceiling", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ ca: 200, pa: 200 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText(
        "Current ability is already at the maximum of 200.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();
  });

  it("reports the verified CA result and refreshes the age-20 profile", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 20, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    const dialog = screen.getByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));

    expect(
      await screen.findByText("CA boosted from 140 to 145."),
    ).toBeInTheDocument();
    expect(screen.getByText("CA 145 → 150 (+5)")).toBeInTheDocument();
    expect(getCurrentAbilityBoostIpcMockCalls()).toEqual([{ uid: 42 }]);
  });

  it("does not carry a settled boost outcome to another player", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 42, name: "Alex Scout", age: 21 }),
    );
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Boost CA",
      }),
    );
    expect(
      await screen.findByText("CA boosted from 140 to 150."),
    ).toBeInTheDocument();

    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 99, name: "Jamie Scout", age: 22 }),
    );
    await router.navigate({
      to: "/players/$uid",
      params: { uid: "99" },
      search: { tab: "outfield" },
    });

    await waitFor(() =>
      expect(
        within(screen.getByTestId("player-identity-rail")).getByRole(
          "heading",
          { level: 1, name: "Jamie Scout" },
        ),
      ).toBeInTheDocument(),
    );
    expect(
      screen.queryByText("CA boosted from 140 to 150."),
    ).not.toBeInTheDocument();
  });

  it("keeps a settled boost outcome across standard sections but clears it on a Moneyball round-trip", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 42, name: "Alex Scout", age: 21 }),
    );
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Boost CA",
      }),
    );
    expect(
      await screen.findByText("CA boosted from 140 to 150."),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Attributes" }));
    expect(
      await screen.findByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getByText("CA boosted from 140 to 150.")).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Moneyball" }));
    expect(
      await screen.findByRole("tab", { name: "Moneyball", selected: true }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Overview" }));
    expect(
      await screen.findByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(
        screen.queryByText("CA boosted from 140 to 150."),
      ).not.toBeInTheDocument(),
    );
  });

  it("does not carry an in-flight boost outcome to another player", async () => {
    await resolveLoadDataIpcMock();
    setCurrentAbilityBoostIpcMockMode("pending");
    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 42, name: "Alex Scout", age: 21 }),
    );
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Boost CA",
      }),
    );

    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 99, name: "Jamie Scout", age: 22 }),
    );
    await router.navigate({
      to: "/players/$uid",
      params: { uid: "99" },
      search: { tab: "outfield" },
    });
    await waitFor(() =>
      expect(
        within(screen.getByTestId("player-identity-rail")).getByRole(
          "heading",
          { level: 1, name: "Jamie Scout" },
        ),
      ).toBeInTheDocument(),
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    resolvePendingCurrentAbilityBoostIpcMock({
      snapshotId: 1,
      operation: "boost-current-ability",
      previousCurrentAbility: 140,
      currentAbility: 150,
      potentialAbility: 160,
      previousAmbition: null,
      ambition: null,
      previousProfessionalism: null,
      professionalism: null,
      previousDetermination: null,
      determination: null,
    });

    await waitFor(() => {
      expect(
        screen.queryByText("CA boosted from 140 to 150."),
      ).not.toBeInTheDocument();
    });
  });

  it("prevents a duplicate CA boost while the first request is pending", async () => {
    await resolveLoadDataIpcMock();
    setCurrentAbilityBoostIpcMockMode("pending");
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    const confirm = within(screen.getByRole("dialog")).getByRole("button", {
      name: "Boost CA",
    });
    await user.click(confirm);
    await user.click(confirm);

    expect(getCurrentAbilityBoostIpcMockCalls()).toHaveLength(1);
    expect(confirm).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Wonderkid Mentality" }),
    ).toBeDisabled();

    resolvePendingCurrentAbilityBoostIpcMock();
    expect(
      await screen.findByText("CA boosted from 140 to 150."),
    ).toBeInTheDocument();
  });

  it("keeps phase-specific bridge errors in the confirmation", async () => {
    await resolveLoadDataIpcMock();
    setCurrentAbilityBoostIpcMockMode("snapshotSyncError");
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    const dialog = screen.getByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "Boost CA" }));

    expect(
      await within(dialog).findByRole("alert", { name: "" }),
    ).toHaveTextContent(
      "Load Data required. FM may have changed. Load Data again.",
    );
  });

  it("restores focus to the CA action after cancelling confirmation", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    const action = await screen.findByRole("button", { name: "Boost CA" });
    action.focus();
    await user.click(action);
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Cancel",
      }),
    );

    await waitFor(() => expect(action).toHaveFocus());
  });

  it("keeps CA confirmation content during its exit transition", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Cancel",
      }),
    );

    expect(
      screen.getByRole("heading", { level: 2, name: "Boost CA?" }),
    ).toBeInTheDocument();
  });

  it("previews only eligible Wonderkid Mentality values", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: null },
        personality: { Ambition: 10, Professionalism: 11 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    ).toBeEnabled();
    expect(screen.getByText("Ambition 10 → random 11–20")).toBeInTheDocument();
    expect(
      screen.getByText("Professionalism 11 → unchanged"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Determination unavailable → unchanged"),
    ).toBeInTheDocument();
  });

  it("disables Wonderkid Mentality when no known value is 10 or lower", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 15 },
        personality: { Ambition: 11, Professionalism: null },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );

    expect(
      await screen.findByText("No known mentality attribute is 10 or lower."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Wonderkid Mentality" }),
    ).toBeDisabled();
  });

  it("confirms Wonderkid Mentality without previewing a random result", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    );

    const dialog = screen.getByRole("dialog");
    expect(
      within(dialog).getByRole("heading", {
        level: 2,
        name: "Apply Wonderkid Mentality?",
      }),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "FM assigns each eligible value a random number from 11 to 20.",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText("Ambition 10 → random 11–20"),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText("Professionalism 15 → unchanged"),
    ).toBeInTheDocument();
  });

  it("reports exact verified Wonderkid Mentality values and refreshes the profile", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    );
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Apply Wonderkid Mentality",
      }),
    );

    expect(
      await screen.findByText(
        "Wonderkid Mentality updated Ambition from 10 to 20, Determination from 8 to 18.",
      ),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(screen.getByText("Ambition 20 → unchanged")).toBeInTheDocument();
    expect(
      screen.getByText("Determination 18 → unchanged"),
    ).toBeInTheDocument();
    expect(getWonderkidMentalityBoostIpcMockCalls()).toEqual([{ uid: 42 }]);
  });

  it("keeps a completed Wonderkid outcome reachable in the fixed action band", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const actionSlot = within(summary).getByTestId(
      "player-profile-action-slot",
    );
    expect(actionSlot).toHaveClass("min-h-10");
    expect(actionSlot).not.toHaveClass("overflow-y-auto");
    expect(
      within(actionSlot).getByTestId("player-development-outcome"),
    ).toHaveClass("max-h-16", "overflow-y-auto");

    await user.click(
      within(actionSlot).getByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      within(actionSlot).getByRole("button", { name: "Wonderkid Mentality" }),
    );
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Apply Wonderkid Mentality",
      }),
    );

    expect(
      await within(actionSlot).findByText(
        "Wonderkid Mentality updated Ambition from 10 to 20, Determination from 8 to 18.",
      ),
    ).toBeInTheDocument();
  });

  it("shares the pending lock across both development actions", async () => {
    await resolveLoadDataIpcMock();
    setWonderkidMentalityBoostIpcMockMode("pending");
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    );
    const confirm = within(screen.getByRole("dialog")).getByRole("button", {
      name: "Apply Wonderkid Mentality",
    });
    await user.click(confirm);
    await user.click(confirm);

    expect(getWonderkidMentalityBoostIpcMockCalls()).toHaveLength(1);
    expect(confirm).toBeDisabled();
    expect(screen.getByRole("button", { name: "Boost CA" })).toBeDisabled();

    resolvePendingWonderkidMentalityBoostIpcMock();
    expect(
      await screen.findByText(
        "Wonderkid Mentality updated Ambition from 10 to 20, Determination from 8 to 18.",
      ),
    ).toBeInTheDocument();
  });

  it("keeps Wonderkid Mentality bridge errors in the confirmation", async () => {
    await resolveLoadDataIpcMock();
    setWonderkidMentalityBoostIpcMockMode("liveValueError");
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    );
    const dialog = screen.getByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: "Apply Wonderkid Mentality" }),
    );

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "FM values changed. player values changed in FM; Load Data again",
    );
  });

  it("does not carry a CA error into a Wonderkid confirmation", async () => {
    await resolveLoadDataIpcMock();
    setCurrentAbilityBoostIpcMockMode("snapshotSyncError");
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(await screen.findByRole("button", { name: "Boost CA" }));
    const caDialog = screen.getByRole("dialog");
    await user.click(
      within(caDialog).getByRole("button", { name: "Boost CA" }),
    );
    await within(caDialog).findByRole("alert");
    await user.click(within(caDialog).getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );

    await user.click(
      screen.getByRole("button", { name: "Wonderkid Mentality" }),
    );

    expect(
      within(screen.getByRole("dialog")).queryByRole("alert"),
    ).not.toBeInTheDocument();
  });

  it("restores focus to Wonderkid Mentality after cancelling confirmation", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    const action = await screen.findByRole("button", {
      name: "Wonderkid Mentality",
    });
    action.focus();
    await user.click(action);
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Cancel",
      }),
    );

    await waitFor(() => expect(action).toHaveFocus());
  });

  it("moves focus to the verified outcome when Wonderkid becomes unavailable", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        ca: 160,
        pa: 160,
        attributes: { Determination: 8 },
        personality: { Ambition: 10, Professionalism: 15 },
      }),
    );
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    await user.click(
      await screen.findByRole("button", { name: "Modify Player" }),
    );
    await user.click(
      await screen.findByRole("button", { name: "Wonderkid Mentality" }),
    );
    await user.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Apply Wonderkid Mentality",
      }),
    );

    const outcome = await screen.findByText(
      "Wonderkid Mentality updated Ambition from 10 to 20, Determination from 8 to 18.",
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(outcome.parentElement).toHaveFocus();
  });

  it("shows the persistent identity rail with the same identity and value on every section", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ transferListed: true }));
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    for (const name of ["Overview", "Attributes", "Role Fit", "Moneyball"]) {
      if (name !== "Overview") {
        await user.click(screen.getByRole("tab", { name }));
      }
      expect(
        await screen.findByRole("tab", { name, selected: true }),
      ).toBeInTheDocument();
      const rail = screen.getByRole("complementary", {
        name: "Player identity",
      });
      expect(
        within(rail).getByRole("heading", { level: 1, name: "Alex Scout" }),
      ).toBeInTheDocument();
      expect(
        within(rail).getByText("Test FC · Premier Division"),
      ).toBeInTheDocument();
      expect(within(rail).getByText("21/03/2001 (25)")).toBeInTheDocument();
      expect(within(rail).getByText("Nationality")).toBeInTheDocument();
      expect(within(rail).getByText("182 cm")).toBeInTheDocument();
      expect(within(rail).getByText("Right")).toBeInTheDocument();
      expect(within(rail).getByText("Transfer listed")).toBeInTheDocument();
      expect(within(rail).getByText("€12.5M")).toBeInTheDocument();
    }
  });

  it("reserves neutral portrait and crest placeholders without loading images", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const rail = await screen.findByRole("complementary", {
      name: "Player identity",
    });
    expect(
      within(rail).getByRole("img", { name: "Player portrait placeholder" }),
    ).toBeInTheDocument();
    expect(
      within(rail).getByRole("img", { name: "Club crest placeholder" }),
    ).toBeInTheDocument();
    expect(rail.querySelector("img")).toBeNull();
  });

  it("keeps the identity rail and analysis workspace distinct with no analytical values in the rail", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    renderProfileRoute("/players/42?section=role-fit");

    const rail = await screen.findByRole("complementary", {
      name: "Player identity",
    });
    const workspace = screen.getByRole("region", { name: "Player analysis" });
    expect(workspace).toBeInTheDocument();
    expect(workspace).not.toContainElement(rail);
    expect(rail).not.toContainElement(workspace);
    expect(within(rail).queryByText("CA")).not.toBeInTheDocument();
    expect(within(rail).queryByText("PA")).not.toBeInTheDocument();
    expect(within(rail).queryByText("140")).not.toBeInTheDocument();
    expect(within(rail).queryByText("160")).not.toBeInTheDocument();
    expect(within(rail).queryByText("Current IP")).not.toBeInTheDocument();
    expect(within(rail).queryByText("Potential OOP")).not.toBeInTheDocument();
    expect(
      within(rail).queryByText("Deep-Lying Playmaker"),
    ).not.toBeInTheDocument();
  });

  it("owns a single player heading with no workspace identity copies on any section", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ transferListed: true }));
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    for (const name of ["Overview", "Attributes", "Role Fit", "Moneyball"]) {
      if (name !== "Overview") {
        await user.click(screen.getByRole("tab", { name }));
      }
      expect(
        await screen.findByRole("tab", { name, selected: true }),
      ).toBeInTheDocument();
      expect(
        screen.getAllByRole("heading", { level: 1, name: "Alex Scout" }),
      ).toHaveLength(1);
      const rail = screen.getByRole("complementary", {
        name: "Player identity",
      });
      expect(
        within(rail).getByRole("heading", {
          level: 1,
          name: "Alex Scout",
        }),
      ).toBeInTheDocument();
      expect(within(rail).getByText("€12.5M")).toBeInTheDocument();
      const workspace = screen.getByTestId("player-analysis-workspace");
      expect(
        within(workspace).queryByRole("heading", {
          level: 1,
          name: "Alex Scout",
        }),
      ).not.toBeInTheDocument();
      expect(
        within(workspace).queryByText("Test FC · Premier Division"),
      ).not.toBeInTheDocument();
      expect(
        within(workspace).queryByText("21/03/2001 (25)"),
      ).not.toBeInTheDocument();
      expect(
        within(workspace).queryByText("Age / DOB"),
      ).not.toBeInTheDocument();
      expect(
        within(workspace).queryByText("Nationality"),
      ).not.toBeInTheDocument();
      expect(within(workspace).queryByText("Height")).not.toBeInTheDocument();
      expect(within(workspace).queryByText("Foot")).not.toBeInTheDocument();
      expect(within(workspace).queryByText("182 cm")).not.toBeInTheDocument();
      expect(
        within(workspace).queryByText("Transfer listed"),
      ).not.toBeInTheDocument();
      if (name === "Overview") {
        expect(within(workspace).getByText("Market Value")).toBeInTheDocument();
        expect(within(workspace).getByText("€12.5M")).toBeInTheDocument();
      } else {
        expect(
          within(workspace).queryByText("Market Value"),
        ).not.toBeInTheDocument();
        expect(within(workspace).queryByText("€12.5M")).not.toBeInTheDocument();
      }
    }
  });

  it("exposes a distinct Overview ability region with CA, PA, and market value only on Overview", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    setPlayerMoneyballOverride(fixturePlayerMoneyball());
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const ability = within(summary).getByTestId("overview-ability");
    expect(within(ability).getByText("Current Ability")).toBeInTheDocument();
    expect(within(ability).getByText("140")).toBeInTheDocument();
    expect(within(ability).getByText("Potential Ability")).toBeInTheDocument();
    expect(within(ability).getByText("160")).toBeInTheDocument();
    expect(within(ability).getByText("Market Value")).toBeInTheDocument();
    expect(within(ability).getByText("€12.5M")).toBeInTheDocument();
    expect(within(ability).getByText("140")).toHaveClass("tabular-nums");
    expect(within(ability).getByText("€12.5M")).toHaveClass("tabular-nums");
    expect(within(ability).queryByText("Age / DOB")).not.toBeInTheDocument();
    expect(within(ability).queryByText("Nationality")).not.toBeInTheDocument();
    expect(within(ability).queryByText("Height")).not.toBeInTheDocument();
    expect(within(ability).queryByText("Foot")).not.toBeInTheDocument();
    const rail = screen.getByRole("complementary", {
      name: "Player identity",
    });
    expect(within(rail).getByText("€12.5M")).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Attributes" }));
    expect(
      await screen.findByRole("tab", {
        name: "Attributes",
        selected: true,
      }),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("overview-ability")).not.toBeInTheDocument();
    expect(
      within(screen.getByTestId("player-analysis-workspace")).queryByText(
        "Current Ability",
      ),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Role Fit" }));
    expect(
      await screen.findByRole("tab", { name: "Role Fit", selected: true }),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("overview-ability")).not.toBeInTheDocument();
    expect(
      within(screen.getByTestId("player-analysis-workspace")).queryByText(
        "Current Ability",
      ),
    ).not.toBeInTheDocument();
  });

  it("hides PA in the Overview ability region when hidden information is concealed", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({ hiddenInformationRevealed: false }),
    );
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const ability = within(summary).getByTestId("overview-ability");
    expect(within(ability).getByText("Current Ability")).toBeInTheDocument();
    expect(within(ability).getByText("140")).toBeInTheDocument();
    expect(
      within(ability).queryByText("Potential Ability"),
    ).not.toBeInTheDocument();
    expect(within(ability).queryByText("160")).not.toBeInTheDocument();
    expect(within(ability).getByText("Market Value")).toBeInTheDocument();
    expect(within(ability).getByText("€12.5M")).toBeInTheDocument();
  });

  it("renders missing market value as an em dash in the rail and the Overview ability region", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ marketValueGbp: null }));
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const ability = within(summary).getByTestId("overview-ability");
    expect(within(ability).getByText("Market Value")).toBeInTheDocument();
    expect(within(ability).getByText("—")).toBeInTheDocument();
    const rail = screen.getByRole("complementary", {
      name: "Player identity",
    });
    expect(within(rail).getByText("—")).toBeInTheDocument();
  });

  it("renders section navigation before the Overview ability content", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const header = await screen.findByTestId("player-profile-header");
    const tabs = within(header).getByRole("tablist", {
      name: "Player analysis view",
    });
    const ability = within(header).getByTestId("overview-ability");
    expect(
      tabs.compareDocumentPosition(ability) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(header.firstElementChild).toBe(tabs);
  });
});
