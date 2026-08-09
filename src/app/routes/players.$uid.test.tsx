import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import {
  fixturePlayerDetail,
  setGetPlayerOverride,
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
    expect(screen.getByText("Test FC")).toBeInTheDocument();
    expect(screen.getByText("21/03/2001 (25)")).toBeInTheDocument();
    expect(screen.getByText("Premier Division")).toBeInTheDocument();
    expect(screen.getByText("140")).toBeInTheDocument();
    expect(screen.getByText("160")).toBeInTheDocument();
    expect(screen.getByText("182 cm")).toBeInTheDocument();
    expect(screen.getByText("Right")).toBeInTheDocument();
    expect(screen.getByText("ENG, WAL")).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Overview", selected: true }),
    ).toBeInTheDocument();
  });

  it("shows not-found empty state for an unknown uid", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(null);
    renderProfileRoute("/players/999");

    expect(
      await screen.findByText("Player not in this snapshot"),
    ).toBeInTheDocument();
  });

  it("selects Attributes and Roles chrome from the tab search param", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42?tab=attributes");

    expect(
      await screen.findByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Attributes" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { level: 2, name: "Overview" }),
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Roles" }));

    expect(
      await screen.findByRole("tab", { name: "Roles", selected: true }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Roles" }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "roles" });
  });

  it("moves between tabs with arrow keys", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    const user = userEvent.setup();
    const { router } = renderProfileRoute("/players/42");

    const overview = await screen.findByRole("tab", {
      name: "Overview",
      selected: true,
    });
    overview.focus();
    await user.keyboard("{ArrowRight}");

    expect(
      await screen.findByRole("tab", { name: "Attributes", selected: true }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "attributes" });

    await user.keyboard("{ArrowRight}");
    expect(
      await screen.findByRole("tab", { name: "Roles", selected: true }),
    ).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ tab: "roles" });
  });

  it("shows Load Data empty state when no snapshot is loaded", async () => {
    setGetPlayerOverride(null);
    renderProfileRoute("/players/42");

    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
  });

  it("shows attribute groups with tabular values and em dash for nulls", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        attributes: {
          Acceleration: 14,
          Crossing: null,
          Handling: 11,
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
    renderProfileRoute("/players/42?tab=attributes");

    expect(
      await screen.findByRole("heading", { level: 3, name: "Technical" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Mental" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Physical" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Goalkeeping" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Hidden" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Personality" }),
    ).toBeInTheDocument();

    const technical = screen.getByRole("region", { name: "Technical" });
    const crossingTerm = within(technical).getByText("Crossing");
    expect(crossingTerm.parentElement).toHaveTextContent(/^Crossing—$/);

    const physical = screen.getByRole("region", { name: "Physical" });
    const accelerationTerm = within(physical).getByText("Acceleration");
    expect(accelerationTerm.parentElement).toHaveTextContent(
      /^Acceleration14$/,
    );
    expect(accelerationTerm.parentElement?.querySelector("dd")).toHaveClass(
      "tabular-nums",
    );

    const hidden = screen.getByRole("region", { name: "Hidden" });
    expect(
      within(hidden).getByText("Consistency").parentElement,
    ).toHaveTextContent(/^Consistency—$/);
    expect(
      within(hidden).getByText("Dirtiness").parentElement,
    ).toHaveTextContent(/^Dirtiness8$/);

    const personality = screen.getByRole("region", { name: "Personality" });
    expect(
      within(personality).getByText("Ambition").parentElement,
    ).toHaveTextContent(/^Ambition15$/);
    expect(
      within(personality).getByText("Loyalty").parentElement,
    ).toHaveTextContent(/^Loyalty—$/);
  });

  it("shows a hero ScoreBadge for the best non-null role on Overview", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42");

    expect(
      await screen.findByLabelText("Deep-Lying Playmaker: 82, Starter"),
    ).toBeInTheDocument();
    expect(screen.getByText("Best role")).toBeInTheDocument();
    expect(screen.getByText("Deep-Lying Playmaker")).toBeInTheDocument();
  });

  it("groups Roles by position family with labelled current and potential badges", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderProfileRoute("/players/42?tab=roles");

    expect(
      await screen.findByRole("heading", { level: 3, name: "Goalkeeper" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Centre-back" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Defensive midfield" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Central midfield" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Striker" }),
    ).toBeInTheDocument();

    const centreBack = screen.getByRole("region", { name: "Centre-back" });
    expect(within(centreBack).getByText("Centre-Back")).toBeInTheDocument();
    expect(
      within(centreBack).getByLabelText("Centre-Back (Potential): unavailable"),
    ).toHaveTextContent("—");

    expect(
      screen.getByLabelText("Deep-Lying Playmaker (Current): 82, Starter"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Deep-Lying Playmaker (Potential): 94, Elite"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Central Midfielder (Current): 72, Starter"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Central Midfielder (Potential): 84, Starter"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Goalkeeper (Current): 40, Fringe"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Goalkeeper (Potential): 47, Fringe"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Advanced Forward (Current): 55, Rotation"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Advanced Forward (Potential): 67, Rotation"),
    ).toBeInTheDocument();

    const goalkeeper = screen.getByRole("region", { name: "Goalkeeper" });
    expect(within(goalkeeper).getByText("IP")).toBeInTheDocument();
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
    renderProfileRoute("/players/42?tab=roles");

    expect(
      await screen.findByLabelText("Catalog Role 1 (Current): 60, Rotation"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Catalog Role 68 (Potential): 70, Starter"),
    ).toBeInTheDocument();
    expect(
      screen.getAllByLabelText(/Catalog Role \d+ \(Current\):/),
    ).toHaveLength(68);
    expect(
      screen.getAllByLabelText(/Catalog Role \d+ \(Potential\):/),
    ).toHaveLength(68);
  });
});
