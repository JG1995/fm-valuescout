import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import type { SquadPlayer } from "@/features/squad/types/squad-player";
import { resolveSavePlannerClubFamilyIpcMock } from "@/testing/planner-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveBusyLoadDataRequest,
  resolveLoadDataIpcMock,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";
import { setSquadPlayersOverride } from "@/testing/squad-ipc-mock";

function squadPlayerNamed(name: string, uid: number, ca = 160): SquadPlayer {
  return {
    uid,
    name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: "Metro FC",
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
    suggestedTraining: null,
  };
}

describe("my club squad retains interaction during delayed Load Data", () => {
  beforeEach(async () => {
    setLoadDataIpcMockMode("success");
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
  });

  it("keeps squad rows mounted, sortable and activatable during busy Load Data through AppTopBar", async () => {
    const user = userEvent.setup();
    setSquadPlayersOverride([
      squadPlayerNamed("Alice Squad", 201, 180),
      squadPlayerNamed("Bob Squad", 202, 160),
    ]);

    const { queryClient, router } = renderWithProviders({
      initialEntries: ["/my-club"],
    });

    const table = await screen.findByRole("table", { name: "Squad overview" });
    expect(within(table).getByText("Alice Squad")).toBeInTheDocument();
    expect(within(table).getByText("Bob Squad")).toBeInTheDocument();
    expect(
      screen.queryByText("Loading squad overview…"),
    ).not.toBeInTheDocument();

    setLoadDataIpcMockMode("busy");

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Scanning…" }),
      ).toBeInTheDocument(),
    );

    expect(within(table).getByText("Alice Squad")).toBeInTheDocument();
    expect(within(table).getByText("Bob Squad")).toBeInTheDocument();
    expect(
      screen.queryByText("Loading squad overview…"),
    ).not.toBeInTheDocument();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
        0,
    ).toBe(false);

    // Exercise sort while pending: click Name header
    const nameHeader = within(table).getByRole("button", { name: "Name" });
    await user.click(nameHeader);
    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({
        squadSort: "name",
      }),
    );
    expect(within(table).getByText("Alice Squad")).toBeInTheDocument();

    // Exercise row activation while pending: click Squad player link/row
    const aliceLink = within(table).getByRole("link", { name: /Alice Squad/ });
    await user.click(aliceLink);
    await waitFor(() =>
      expect(router.state.location.pathname).toBe("/players/201"),
    );
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
        0,
    ).toBe(false);

    resolveBusyLoadDataRequest();
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Scanning…" }),
      ).not.toBeInTheDocument(),
    );
  });
});
