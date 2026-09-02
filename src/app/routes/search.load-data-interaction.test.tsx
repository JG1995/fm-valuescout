import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import { renderWithProviders } from "@/testing/render-with-providers";
import { setSearchPlayersOverride } from "@/testing/search-ipc-mock";
import {
  resolveBusyLoadDataRequest,
  resolveLoadDataIpcMock,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

function playerNamed(name: string, uid: number, ca = 160): PlayerSummary {
  return {
    uid,
    name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: "Test FC",
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
  };
}

describe("search route retains interaction during delayed Load Data", () => {
  beforeEach(async () => {
    setLoadDataIpcMockMode("success");
    await resolveLoadDataIpcMock();
  });

  it("keeps established rows mounted, sortable and activatable during busy Load Data through AppTopBar", async () => {
    const user = userEvent.setup();
    setSearchPlayersOverride([
      playerNamed("Alice Scout", 101, 200),
      playerNamed("Zara Scout", 102, 100),
    ]);

    const { queryClient, router } = renderWithProviders({
      initialEntries: ["/search"],
    });

    const table = await screen.findByRole("table", {
      name: "Player search results",
    });
    expect(within(table).getByText("Alice Scout")).toBeInTheDocument();
    expect(within(table).getByText("Zara Scout")).toBeInTheDocument();
    expect(
      screen.queryByText("Loading player results…"),
    ).not.toBeInTheDocument();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
        0,
    ).toBe(false);

    setLoadDataIpcMockMode("busy");

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Scanning…" }),
      ).toBeInTheDocument(),
    );

    // Rows remain mounted and readable while Load Data is pending
    expect(within(table).getByText("Alice Scout")).toBeInTheDocument();
    expect(within(table).getByText("Zara Scout")).toBeInTheDocument();
    expect(
      screen.queryByText("Loading player results…"),
    ).not.toBeInTheDocument();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
        0,
    ).toBe(false);

    // Exercise existing sort while Load Data is pending
    const nameHeader = within(table).getByRole("button", { name: "Name" });
    await user.click(nameHeader);

    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({
        sort: "name",
        dir: "asc",
      }),
    );
    expect(within(table).getByText("Alice Scout")).toBeInTheDocument();
    expect(
      screen.queryByText("Loading player results…"),
    ).not.toBeInTheDocument();

    // Exercise row activation while Load Data is pending (navigates to player profile)
    const aliceRow = within(table).getByText("Alice Scout").closest("tr");
    if (!aliceRow) throw new Error("Expected Alice row");
    fireEvent.click(aliceRow);

    await waitFor(() =>
      expect(router.state.location.pathname).toBe("/players/101"),
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
