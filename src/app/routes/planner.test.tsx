import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { routeTree } from "@/routeTree.gen";
import {
  resolvePlannerClubFamilyIpcMock,
  setPlannerAvailableClubs,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";

function renderPlannerRoute() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: ["/planner"] }),
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("planner route", () => {
  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderPlannerRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Squad Planner" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("persists a separate Reserves club in the club family", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona", "Barca Athletic", "Barcelona U19"]);
    renderPlannerRoute();

    const primaryClub = await screen.findByLabelText("Primary club");
    await user.selectOptions(primaryClub, "Barcelona");
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    const addReserves = await screen.findByRole("button", {
      name: "Add Reserves source",
    });
    await user.click(addReserves);
    await user.selectOptions(
      screen.getByLabelText("Reserves club 1"),
      "Barca Athletic",
    );
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    expect(
      (await screen.findAllByRole("option", { name: "Barca Athletic" })).length,
    ).toBeGreaterThan(0);
    await user.click(screen.getByRole("button", { name: "Add Youth source" }));
    expect(screen.getByLabelText("Youth club 1")).toBeInTheDocument();
    expect(screen.getByLabelText("Youth player level 1")).toBeInTheDocument();
    expect(
      resolvePlannerClubFamilyIpcMock().sources.some(
        (source) =>
          source.clubName === "Barca Athletic" &&
          source.team === "reserves" &&
          source.teamLevel === null,
      ),
    ).toBe(true);
    expect(
      within(screen.getByRole("main")).getByText(/Associated clubs/),
    ).toBeInTheDocument();
  });
});
