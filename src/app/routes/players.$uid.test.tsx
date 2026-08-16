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
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
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
    useLayoutStore.setState({ railExpanded: true });
    setGetPlayerOverride(undefined);
  });

  it("shows overview identity fields for a known player", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Alex Scout" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Test FC · Premier Division")).toBeInTheDocument();
    expect(screen.getByText("21/03/2001 (25)")).toBeInTheDocument();
    expect(screen.getByText("140")).toBeInTheDocument();
    expect(screen.getByText("160")).toBeInTheDocument();
    expect(screen.getByText("182 cm")).toBeInTheDocument();
    expect(screen.getByText("Right")).toBeInTheDocument();
    expect(screen.getByText("ENG, WAL")).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Outfield", selected: true }),
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

    const tabs = await screen.findAllByRole("tab");
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

    const tabs = await screen.findAllByRole("tab");
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

  it("keeps the hidden-information control last in the action row", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    const summary = await screen.findByRole("region", {
      name: "Alex Scout summary",
    });
    const boostCa = within(summary).getByRole("button", { name: "Boost CA" });
    const wonderkidMentality = within(summary).getByRole("button", {
      name: "Wonderkid Mentality",
    });
    const hiddenInformation = within(summary).getByRole("button", {
      name: "Reveal hidden information",
    });

    expect(within(summary).getAllByRole("button")).toEqual([
      boostCa,
      wonderkidMentality,
      hiddenInformation,
    ]);
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
    renderProfileRoute("/players/42");

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
    const concealedPotentialIp = within(summary).getByRole("img", {
      name: "Potential IP: concealed",
    });
    const concealedPotentialOop = within(summary).getByRole("img", {
      name: "Potential OOP: concealed",
    });
    expect(concealedPotentialIp).toHaveTextContent("—");
    expect(concealedPotentialOop).toHaveTextContent("—");
    expect(within(summary).getAllByText("Concealed")).toHaveLength(2);

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
    expect(getSetPlayerHiddenInformationRevealedIpcMockCalls()).toEqual([
      { revealed: false },
    ]);

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

  it("shows phase-specific current and potential best-role summaries", async () => {
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

    expect(
      within(summary).getByLabelText("Current IP: 82, Excellent"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByLabelText("Current OOP: 79, Good"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByLabelText("Potential IP: 94, Excellent"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByLabelText("Potential OOP: 93, Excellent"),
    ).toBeInTheDocument();
    expect(within(summary).getByText("Current IP")).toBeInTheDocument();
    expect(within(summary).getByText("Current OOP")).toBeInTheDocument();
    expect(within(summary).getByText("Potential IP")).toBeInTheDocument();
    expect(within(summary).getByText("Potential OOP")).toBeInTheDocument();
    expect(
      within(summary).getByText("Current IP Specialist"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByText("Current OOP Specialist"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByText("Potential IP Specialist"),
    ).toBeInTheDocument();
    expect(
      within(summary).getByText("Potential OOP Specialist"),
    ).toBeInTheDocument();
    expect(
      within(summary).queryByText("Current IP Tie"),
    ).not.toBeInTheDocument();
    expect(
      within(summary).queryByText("Unplayable Specialist"),
    ).not.toBeInTheDocument();
  });

  it("renders unavailable potential summary values without a score badge", async () => {
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

    const potentialIp = await screen.findByRole("img", {
      name: "Potential IP: unavailable",
    });
    const potentialOop = screen.getByRole("img", {
      name: "Potential OOP: unavailable",
    });
    expect(potentialIp).toHaveTextContent("—");
    expect(potentialOop).toHaveTextContent("—");
    expect(potentialIp).not.toHaveAttribute("title");
    expect(potentialOop).not.toHaveAttribute("title");
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

  it("shows labelled current and potential values for every supplied role", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        roleScores: Array.from({ length: 68 }, (_, index) => ({
          roleId: `catalog-role-${index}`,
          displayName: `Catalog Role ${index + 1}`,
          phase: "in_possession",
          positionTags: ["MC"],
          score: 60,
          potentialScore: 70,
        })),
      }),
    );
    renderProfileRoute("/players/42?tab=technical");

    expect(
      await screen.findByLabelText("Catalog Role 1 (Current): 60, Average"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Catalog Role 68 (Potential): 70, Good"),
    ).toBeInTheDocument();
    expect(
      screen.getAllByLabelText(/Catalog Role \d+ \(Current\):/),
    ).toHaveLength(68);
    expect(
      screen.getAllByLabelText(/Catalog Role \d+ \(Potential\):/),
    ).toHaveLength(68);
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
    ).toEqual(["Potential LeaderOOP", "Current LeaderIP"]);

    await user.click(
      within(potentialHeader).getByRole("button", { name: "Potential" }),
    );
    expect(potentialHeader).toHaveAttribute("aria-sort", "descending");
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .map((row) => within(row).getAllByRole("cell")[0].textContent),
    ).toEqual(["Potential LeaderOOP", "Current LeaderIP"]);

    await user.click(
      within(potentialHeader).getByRole("button", { name: "Potential" }),
    );
    expect(potentialHeader).toHaveAttribute("aria-sort", "ascending");
    expect(
      within(roleFit)
        .getAllByRole("row")
        .slice(1)
        .map((row) => within(row).getAllByRole("cell")[0].textContent),
    ).toEqual(["Current LeaderIP", "Potential LeaderOOP"]);
  });

  it("previews and confirms the age-21 CA boost from the current snapshot", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

    expect(
      await screen.findByText("CA 192 → 195 (+3) · capped by PA"),
    ).toBeInTheDocument();
  });

  it("disables CA boost when age is unknown", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: null }));
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

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

    expect(
      await screen.findByRole("heading", { level: 1, name: "Jamie Scout" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("CA boosted from 140 to 150."),
    ).not.toBeInTheDocument();
  });

  it("does not carry an in-flight boost outcome to another player", async () => {
    await resolveLoadDataIpcMock();
    setCurrentAbilityBoostIpcMockMode("pending");
    setGetPlayerOverride(
      fixturePlayerDetail({ uid: 42, name: "Alex Scout", age: 21 }),
    );
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

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
    expect(
      await screen.findByRole("heading", { level: 1, name: "Jamie Scout" }),
    ).toBeInTheDocument();
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
    renderProfileRoute("/players/42");

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
    renderProfileRoute("/players/42");

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
});
