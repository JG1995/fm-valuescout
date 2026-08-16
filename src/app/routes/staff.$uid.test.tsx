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
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaffDetail,
  setStaffDetailOverride,
} from "@/testing/staff-ipc-mock";

function renderStaffProfileRoute(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [initialEntry] }),
  });
  return {
    router,
    queryClient,
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    ),
  };
}

describe("staff profile route", () => {
  beforeEach(() => {
    useLayoutStore.setState({ railExpanded: true });
    setStaffDetailOverride(undefined);
  });

  it("shows staff summary, grouped attributes, and current role fit", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(fixtureStaffDetail());
    renderStaffProfileRoute("/staff/101");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Alex Coach" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Metro FC · Premier Division")).toBeInTheDocument();
    expect(screen.getByText("30/04/1982 (44)")).toBeInTheDocument();
    expect(screen.getAllByText("15").length).toBeGreaterThan(0);
    expect(screen.getByText("160")).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: "Coaching", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Coach — Fitness").length).toBeGreaterThan(0);
    expect(screen.getByText("Scout")).toBeInTheDocument();
    expect(screen.queryByText("Wonderkid Mentality")).toBeNull();
    expect(screen.queryByText("Select a pitch position")).toBeNull();
  });

  it("uses canonical tabs and retains current staff attributes when hidden info is concealed", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(fixtureStaffDetail());
    const user = userEvent.setup();
    const { queryClient } = renderStaffProfileRoute("/staff/101?tab=mental");
    queryClient.setQueryData([...playerKeys.all, "probe"], []);
    queryClient.setQueryData([...staffKeys.all, "probe"], []);

    expect(
      await screen.findByRole("tab", { name: "Mental", selected: true }),
    ).toBeInTheDocument();
    expect(screen.getByText("Adaptability")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Hide hidden info" }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Reveal hidden info" }),
      ).toBeInTheDocument();
      expect(screen.queryByText("160")).toBeNull();
      expect(screen.getByText("145")).toBeInTheDocument();
      expect(screen.getByText("Adaptability")).toBeInTheDocument();
    });
    expect(
      queryClient.getQueryState([...playerKeys.all, "probe"])?.isInvalidated,
    ).toBe(true);
    expect(
      queryClient.getQueryState([...staffKeys.all, "probe"])?.isInvalidated,
    ).toBe(true);
    await user.click(screen.getByRole("tab", { name: "Mental" }));
    expect(
      within(screen.getByRole("tabpanel", { name: "Mental" })).getByText(
        "Authority",
      ),
    ).toBeInTheDocument();
  });

  it("summarizes the highest available role score", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(
      fixtureStaffDetail({
        roleScores: [
          {
            roleId: "coach_fitness",
            displayName: "Coach — Fitness",
            score: 60,
          },
          { roleId: "scout", displayName: "Scout", score: 90 },
          { roleId: "physio", displayName: "Physio", score: null },
        ],
      }),
    );
    renderStaffProfileRoute("/staff/101");

    const summary = await screen.findByRole("region", {
      name: "Alex Coach summary",
    });
    expect(within(summary).getByText("Scout")).toBeInTheDocument();
    expect(within(summary).getByText("90")).toBeInTheDocument();
  });

  it("renders an empty state for an unknown staff UID", async () => {
    await resolveLoadDataIpcMock();
    renderStaffProfileRoute("/staff/999");
    expect(
      await screen.findByText("Staff member not in this snapshot", {
        exact: true,
      }),
    ).toBeInTheDocument();
  });
});
