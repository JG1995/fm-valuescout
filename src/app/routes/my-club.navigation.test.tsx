import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { routeTree } from "@/routeTree.gen";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  resolveSavePlannerClubFamilyIpcMock,
  setPlannerAvailableClubs,
  setPlannerUnassignedPlayersIpcMock,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import { setSquadPlayersOverride } from "@/testing/squad-ipc-mock";

type Invoke = (
  cmd: string,
  args?: Record<string, unknown>,
  options?: unknown,
) => Promise<unknown>;

/**
 * Records every IPC request and leaves the commands in `held` pending. The
 * test setup registers the shared IPC mock again after each test.
 */
function recordIpc(held: readonly string[] = []) {
  const internals = (
    window as unknown as { __TAURI_INTERNALS__: { invoke: Invoke } }
  ).__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  const requests: { cmd: string; args: Record<string, unknown> }[] = [];
  internals.invoke = (cmd, args, options) => {
    requests.push({ cmd, args: args ?? {} });
    return held.includes(cmd)
      ? new Promise(() => {})
      : invoke(cmd, args, options);
  };
  return {
    of: (cmd: string) =>
      requests.filter((request) => request.cmd === cmd).map(({ args }) => args),
  };
}

/** Production's freshness window, so a mounted observer reuses loader data. */
function myClubRouter(entry: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [entry] }),
  });
  const renderRouter = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
  return { router, queryClient, renderRouter };
}

async function seedSquad() {
  await resolveLoadDataIpcMock();
  resolveSavePlannerClubFamilyIpcMock({
    primaryClub: { clubName: "Metro FC", clubUid: 1 },
    sources: [],
  });
  setSquadPlayersOverride([
    {
      uid: 42,
      name: "Alex Scout",
      age: 25,
      birthYear: 2001,
      birthDayOfYear: 80,
      nationalities: ["ENG"],
      club: "Metro FC",
      currentClubUid: null,
      division: "Premier Division",
      ca: 160,
      pa: 165,
      marketValueGbp: 16_000_000,
      suggestedTraining: null,
      loanedOut: false,
    },
  ]);
}

describe("My Club navigation", () => {
  it("warms Squad's first page with the table's sort and columns", async () => {
    await seedSquad();
    usePlayerTableStore.getState().addColumns("squad", ["club_dna"]);
    const ipc = recordIpc();
    const { router, renderRouter } = myClubRouter(
      "/my-club?squadSort=age&squadDir=asc",
    );

    await router.load();
    await waitFor(() => expect(ipc.of("list_squad_players")).toHaveLength(1));
    expect(ipc.of("list_squad_players")[0]).toMatchObject({
      offset: 0,
      sortBy: "age",
      sortDir: "asc",
      requestedFields: expect.arrayContaining(["club_dna"]),
    });

    renderRouter();
    expect(await screen.findByText("Alex Scout")).toBeInTheDocument();
    // The table observes the read the loader started instead of asking again.
    expect(ipc.of("list_squad_players")).toHaveLength(1);
  });

  it("holds the table's box with a labelled placeholder while the first page loads", async () => {
    await seedSquad();
    recordIpc(["list_squad_players"]);
    const { renderRouter } = myClubRouter("/my-club");
    renderRouter();

    const label = await screen.findByText("Loading squad overview…");
    expect(label.closest('[role="status"]')).toHaveAttribute(
      "aria-busy",
      "true",
    );
    expect(
      screen.getByRole("toolbar", { name: "Squad results toolbar" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Loading squad")).toBeNull();
  });

  it("paints Club DNA ready and leaves the club list for the picker", async () => {
    await seedSquad();
    setPlannerAvailableClubs([
      { clubName: "Metro FC", clubUid: 1 },
      { clubName: "Barcelona", clubUid: 2 },
    ]);
    const ipc = recordIpc();
    const { router, renderRouter } = myClubRouter("/my-club");

    await router.load();
    renderRouter();

    expect(screen.getByRole("button", { name: "Define DNA" })).toBeEnabled();
    expect(ipc.of("list_managed_club_options")).toHaveLength(0);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Edit managed club" }));
    await user.type(
      screen.getByRole("combobox", { name: "Managed club" }),
      "{Control>}a{/Control}Bar",
    );
    expect(
      await screen.findByRole("option", { name: "Barcelona" }),
    ).toBeInTheDocument();
    expect(ipc.of("list_managed_club_options")).toHaveLength(1);
  });

  it("paints the Planner with its tactic and squad counts, and warms the youth XI", async () => {
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs([{ clubName: "Barcelona", clubUid: 1 }]);
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Barcelona", clubUid: 1 },
      sources: [],
    });
    setPlannerUnassignedPlayersIpcMock([
      {
        playerUid: 91,
        name: "Spare Keeper",
        age: 29,
        laneId: "goalkeeper",
        currentScore: 71,
        expectedScore: 71,
      },
    ]);
    const ipc = recordIpc();
    const { router, renderRouter } = myClubRouter("/my-club?view=planner");

    await router.load();
    expect(ipc.of("get_dashboard_prospects")).toHaveLength(1);
    renderRouter();

    expect(
      screen.getByRole("button", { name: "Unassigned (1)" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Loading tactic…")).toBeNull();
  });
});
