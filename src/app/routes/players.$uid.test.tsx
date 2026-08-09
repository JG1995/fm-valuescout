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
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import {
  fixturePlayerDetail,
  getCurrentAbilityBoostIpcMockCalls,
  getWonderkidMentalityBoostIpcMockCalls,
  resolvePendingCurrentAbilityBoostIpcMock,
  resolvePendingWonderkidMentalityBoostIpcMock,
  setCurrentAbilityBoostIpcMockMode,
  setGetPlayerOverride,
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

  it("shows independent current and potential best-role summaries on Overview", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(
      fixturePlayerDetail({
        roleScores: [
          {
            roleId: "current-specialist",
            displayName: "Current Specialist",
            phase: "in_possession",
            positionTags: ["MC"],
            score: 82,
            potentialScore: 88,
          },
          {
            roleId: "potential-specialist",
            displayName: "Potential Specialist",
            phase: "in_possession",
            positionTags: ["ST"],
            score: 70,
            potentialScore: 94,
          },
        ],
      }),
    );
    renderProfileRoute("/players/42");

    expect(
      await screen.findByLabelText("Best role (Current): 82, Starter"),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText("Best potential role (Potential): 94, Elite"),
    ).toBeInTheDocument();
    expect(screen.getByText("Best role (Current)")).toBeInTheDocument();
    expect(
      screen.getByText("Best potential role (Potential)"),
    ).toBeInTheDocument();
    expect(screen.getByText("Current Specialist")).toBeInTheDocument();
    expect(screen.getByText("Potential Specialist")).toBeInTheDocument();
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

    const potential = await screen.findByRole("img", {
      name: "Best potential role (Potential): unavailable",
    });
    expect(potential).toHaveTextContent("—");
    expect(potential).not.toHaveClass("inline-flex");
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

  it("previews and confirms the age-21 CA boost from the current snapshot", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
    const user = userEvent.setup();
    renderProfileRoute("/players/42");

    const action = await screen.findByRole("button", { name: "Boost CA" });
    expect(screen.getByText("CA 140 → 145 (+5)")).toBeInTheDocument();

    await user.click(action);

    expect(
      screen.getByRole("heading", { level: 2, name: "Boost CA?" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("This raises current ability from 140 to 145."),
    ).toBeInTheDocument();
  });

  it("uses the age-22 increment while capping the preview at PA", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 22, ca: 192, pa: 195 }));
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

  it("reports the verified CA result and refreshes the profile", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail({ age: 21, ca: 140, pa: 160 }));
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
      await screen.findByText("CA boosted from 140 to 145."),
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
