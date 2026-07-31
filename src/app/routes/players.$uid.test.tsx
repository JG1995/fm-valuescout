import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
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
});
