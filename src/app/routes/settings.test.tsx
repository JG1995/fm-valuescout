import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  type SnapshotMetadata,
  setSnapshotHistoryIpcMock,
} from "@/testing/snapshot-ipc-mock";

const HISTORY: SnapshotMetadata[] = [
  {
    id: 11,
    contextToken: "snapshot-token-11",
    saveId: 1,
    customName: null,
    gameDate: "2026-06-01",
    gameDateSource: "inGame",
    playerCount: 21,
    loadedAtUtc: "2026-07-28T13:00:00.000Z",
    isCurrent: false,
  },
  {
    id: 12,
    contextToken: "snapshot-token-12",
    saveId: 1,
    customName: null,
    gameDate: "2026-08-01",
    gameDateSource: "inGame",
    playerCount: 24,
    loadedAtUtc: "2026-07-28T15:00:00.000Z",
    isCurrent: true,
  },
];

describe("Settings", () => {
  it("renders Save data and Bridge without a managed-club section", async () => {
    renderWithProviders({ initialEntries: ["/settings"] });

    expect(
      await screen.findByRole("heading", { level: 1, name: "Settings" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Save data" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Managed club" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Managed club" })).toBeNull();
    expect(screen.getByRole("region", { name: "Bridge" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Load Data" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("combobox", { name: "Active save" }),
    ).toBeInTheDocument();
  });

  it("invalidates current-only products when deleting the current snapshot", async () => {
    setSnapshotHistoryIpcMock(HISTORY);
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    queryClient.setQueryData(searchKeys.all, []);
    queryClient.setQueryData(playerKeys.all, []);
    queryClient.setQueryData(plannerKeys.all, []);
    queryClient.setQueryData(staffKeys.all, []);
    queryClient.setQueryData(academyKeys.classes(), []);

    expect(await screen.findByText(/24 players/)).toBeInTheDocument();
    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot 2026-08-01/,
      }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Delete snapshot" }),
    );

    await waitFor(() => {
      expect(screen.getByText(/21 players/)).toBeInTheDocument();
      expect(queryClient.getQueryState(searchKeys.all)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryState(playerKeys.all)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryState(plannerKeys.all)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryState(staffKeys.all)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryData(academyKeys.classes())).toBeUndefined();
    });
  });
});
