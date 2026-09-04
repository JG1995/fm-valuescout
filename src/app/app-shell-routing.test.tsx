import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { routeTree } from "@/routeTree.gen";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  fixturePlayerDetail,
  setGetPlayerOverride,
} from "@/testing/player-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaffDetail,
  setStaffDetailOverride,
} from "@/testing/staff-ipc-mock";

describe("app shell routing", () => {
  beforeEach(() => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
    setGetPlayerOverride(undefined);
    setStaffDetailOverride(undefined);
  });

  it("renders the layout shell on the index route", async () => {
    renderWithProviders();

    expect(await screen.findByTestId("app-header")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
  });

  it("renders the utility bar before the navigation bar with no rail remnant", async () => {
    renderWithProviders();

    const header = await screen.findByTestId("app-header");
    const nav = await screen.findByRole("navigation", { name: "Primary" });

    expect(header.compareDocumentPosition(nav)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(screen.queryByTestId("app-nav-rail")).toBeNull();
    expect(
      screen.queryByRole("button", { name: "Toggle navigation" }),
    ).toBeNull();
  });

  it("lists all ten grouped destinations with working targets", async () => {
    renderWithProviders();

    const nav = await screen.findByRole("navigation", { name: "Primary" });
    const links = within(nav).getAllByRole("link");

    expect(
      links.map((link) => [link.textContent, link.getAttribute("href")]),
    ).toEqual([
      ["Dashboard", "/"],
      ["Search", "/search?view=general"],
      ["Moneyball", "/search?view=moneyball"],
      ["Staff Search", "/staff?view=search"],
      ["My Staff", "/staff?view=my-staff"],
      ["Squad", "/my-club?view=squad"],
      ["Planner", "/my-club?view=planner"],
      ["Tactic", "/my-club?view=tactic"],
      ["Youth", "/academy"],
      ["Settings", "/settings"],
    ]);
    expect(
      [...nav.querySelectorAll("[data-nav-caption]")].map(
        (caption) => caption.textContent,
      ),
    ).toEqual(["Home", "Players", "Staff", "Club", "Settings"]);
    expect(nav.querySelectorAll("[data-nav-separator]")).toHaveLength(4);
  });

  it.each([
    ["/", "Dashboard"],
    ["/search?view=general", "Search"],
    ["/search?view=moneyball", "Moneyball"],
    ["/search", "Search"],
    ["/staff?view=search", "Staff Search"],
    ["/staff", "Staff Search"],
    ["/my-club?view=squad", "Squad"],
    ["/my-club", "Squad"],
    ["/my-club?view=planner", "Planner"],
    ["/my-club?view=tactic", "Tactic"],
    ["/academy", "Youth"],
    ["/academy?view=graduates", "Youth"],
    ["/settings", "Settings"],
  ])("marks only %s as current at %s", async (entry, name) => {
    renderWithProviders({ initialEntries: [entry] });

    const nav = await screen.findByRole("navigation", { name: "Primary" });
    const current = within(nav).getAllByRole("link", { current: "page" });

    expect(current).toHaveLength(1);
    expect(current[0]).toHaveAccessibleName(name);
  });

  it("marks Moneyball current when the saved default selects it", async () => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "moneyball" });
    renderWithProviders({ initialEntries: ["/search"] });

    const nav = await screen.findByRole("navigation", { name: "Primary" });
    const current = within(nav).getAllByRole("link", { current: "page" });

    expect(current).toHaveLength(1);
    expect(current[0]).toHaveAccessibleName("Moneyball");
  });

  it("marks only the Players group on a player profile", async () => {
    await resolveLoadDataIpcMock();
    setGetPlayerOverride(fixturePlayerDetail());
    renderWithProviders({ initialEntries: ["/players/42"] });

    const nav = await screen.findByRole("navigation", { name: "Primary" });

    expect(
      await screen.findByRole("heading", { level: 1, name: "Alex Scout" }),
    ).toBeInTheDocument();
    expect(within(nav).queryByRole("link", { current: "page" })).toBeNull();
    expect(nav.querySelectorAll("[aria-current]")).toHaveLength(1);
    expect(within(nav).getByText("Players")).toHaveAttribute(
      "aria-current",
      "location",
    );
  });

  it("marks only the Staff group on a staff profile", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(fixtureStaffDetail());
    renderWithProviders({ initialEntries: ["/staff/101"] });

    const nav = await screen.findByRole("navigation", { name: "Primary" });

    expect(
      await screen.findByRole("heading", { level: 1, name: "Alex Coach" }),
    ).toBeInTheDocument();
    expect(within(nav).queryByRole("link", { current: "page" })).toBeNull();
    expect(nav.querySelectorAll("[aria-current]")).toHaveLength(1);
    expect(within(nav).getByText("Staff")).toHaveAttribute(
      "aria-current",
      "location",
    );
  });

  it("marks My Staff current on its canonical Staff destination", async () => {
    const user = userEvent.setup();
    const { router } = renderWithProviders();

    await user.click(await screen.findByRole("link", { name: "My Staff" }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/staff");
      expect(router.state.location.search).toMatchObject({
        view: "my-staff",
      });
    });

    const nav = await screen.findByRole("navigation", { name: "Primary" });
    const current = within(nav).getAllByRole("link", { current: "page" });
    expect(current).toHaveLength(1);
    expect(current[0]).toHaveAccessibleName("My Staff");
  });

  it.each([["/players/42/extra"], ["/staff/101/extra"]])(
    "marks nothing current on profile-shaped unknown route %s",
    async (entry) => {
      renderWithProviders({ initialEntries: [entry] });

      expect(
        await screen.findByRole("heading", { name: "Page not found" }),
      ).toBeInTheDocument();
      const nav = await screen.findByRole("navigation", { name: "Primary" });
      expect(nav.querySelectorAll("[aria-current]")).toHaveLength(0);
    },
  );

  it("offers a skip link to the main region", async () => {
    renderWithProviders();

    const skipLink = await screen.findByRole("link", {
      name: "Skip to content",
    });

    expect(skipLink).toHaveAttribute("href", "#main-content");
    expect(screen.getByRole("main")).toHaveAttribute("id", "main-content");
  });

  it("renders the not-found page for unknown routes with no destination current", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    const router = createRouter({
      routeTree,
      context: { queryClient } satisfies RouterContext,
      defaultPreloadStaleTime: 0,
      history: createMemoryHistory({ initialEntries: ["/does-not-exist"] }),
    });

    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(
      await screen.findByRole("heading", { name: "Page not found" }),
    ).toBeInTheDocument();
    const nav = await screen.findByRole("navigation", { name: "Primary" });
    expect(within(nav).queryByRole("link", { current: "page" })).toBeNull();
    expect(nav.querySelectorAll("[aria-current]")).toHaveLength(0);
  });

  it("navigates to Settings and preserves the route through browser history", async () => {
    const user = userEvent.setup();
    const { router } = renderWithProviders();

    await user.click(await screen.findByRole("link", { name: "Settings" }));
    expect(
      await screen.findByRole("heading", { level: 1, name: "Settings" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "Dashboard" }));
    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();

    await act(async () => router.history.back());
    expect(
      await screen.findByRole("heading", { level: 1, name: "Settings" }),
    ).toBeInTheDocument();

    await act(async () => router.history.forward());
    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
  });

  it("restores pathname, search, and hash through top-bar history controls", async () => {
    const user = userEvent.setup();
    const { router } = renderWithProviders();

    await act(async () => {
      await router.navigate({
        to: "/search",
        search: {
          sort: "moneyball.average_rating",
          dir: "desc",
          filters: [],
          combine: "and",
          view: "moneyball",
          comparisonPool: "filtered",
        },
        hash: "filters",
      });
    });
    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    const searchHref = router.history.location.href;

    await act(async () => {
      await router.navigate({ to: "/settings", hash: "bridge" });
    });
    expect(
      await screen.findByRole("heading", { level: 1, name: "Settings" }),
    ).toBeInTheDocument();
    const settingsHref = router.history.location.href;

    await user.click(screen.getByRole("button", { name: "Back" }));
    expect(
      await screen.findByRole("heading", { level: 1, name: "Player Search" }),
    ).toBeInTheDocument();
    expect(searchHref).toContain("view=moneyball");
    expect(searchHref).toMatch(/#filters$/);
    expect(router.history.location.href).toBe(searchHref);

    await user.click(screen.getByRole("button", { name: "Forward" }));
    expect(
      await screen.findByRole("heading", { level: 1, name: "Settings" }),
    ).toBeInTheDocument();
    expect(router.history.location.href).toBe(settingsHref);
  });
});
