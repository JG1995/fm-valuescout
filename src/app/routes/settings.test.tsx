import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import {
  DEFAULT_GRAPHICS_STATUS,
  setGraphicsChooseIpcMockMode,
  setGraphicsMutationIpcMockMode,
  setGraphicsStatusIpcMock,
  setGraphicsStatusIpcMockMode,
} from "@/features/graphics/api/graphics-ipc-mock";
import { graphicsKeys } from "@/features/graphics/api/graphics-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SaveSummary } from "@/features/snapshot/types/save";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  getManagedClubBoostIpcMockCalls,
  setManagedClubBoostError,
  setManagedClubBoostRecoveryRequired,
} from "@/testing/managed-club-boost-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  type SnapshotMetadata,
  setCurrentSnapshotIpcMockFailure,
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
  beforeEach(() => {
    useMoneyballPreferences.setState({ defaultAnalysisView: "general" });
  });

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

  it("confirms and applies every managed-club boost in one action", async () => {
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    const section = await screen.findByRole("region", {
      name: "All boosts",
    });
    await user.click(
      within(section).getByRole("button", { name: "Apply all boosts" }),
    );

    const dialog = screen.getByRole("dialog", { name: "Apply all boosts?" });
    expect(dialog).toHaveTextContent("Make all Wonderkids");
    expect(dialog).toHaveTextContent("age-restricted player CA boost");
    expect(dialog).toHaveTextContent("staff CA boost");
    await user.click(
      within(dialog).getByRole("button", { name: "Apply all boosts" }),
    );

    expect(
      await within(section).findByText(
        "Completed — 7 updated, 2 skipped, 0 failed.",
      ),
    ).toBeInTheDocument();
    expect(getManagedClubBoostIpcMockCalls()).toHaveLength(1);
  });

  it("blocks a recovery retry until Load Data replaces the snapshot", async () => {
    setManagedClubBoostRecoveryRequired(true);
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const section = await screen.findByRole("region", { name: "All boosts" });
    const action = within(section).getByRole("button", {
      name: "Apply all boosts",
    });

    await user.click(action);
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Apply all boosts?" }),
      ).getByRole("button", { name: "Apply all boosts" }),
    );

    expect(await within(section).findByText(/Stopped —/)).toBeInTheDocument();
    expect(action).toBeDisabled();

    setSnapshotHistoryIpcMock([
      ...HISTORY.map((snapshot) => ({ ...snapshot, isCurrent: false })),
      {
        ...HISTORY[1],
        id: 13,
        contextToken: "snapshot-token-13",
        gameDate: "2026-09-01",
        isCurrent: true,
      },
    ]);
    await queryClient.invalidateQueries({ queryKey: snapshotKeys.all });

    await waitFor(() => {
      const currentSection = screen.getByRole("region", { name: "All boosts" });
      expect(
        within(currentSection).getByRole("button", {
          name: "Apply all boosts",
        }),
      ).toBeEnabled();
      expect(within(currentSection).queryByText(/Stopped —/)).toBeNull();
    });
  });

  it("clears a snapshot-less save error when the active save changes", async () => {
    setManagedClubBoostError("Load Data before using boosts");
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const section = await screen.findByRole("region", { name: "All boosts" });

    await user.click(
      within(section).getByRole("button", { name: "Apply all boosts" }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: "Apply all boosts?" }),
      ).getByRole("button", { name: "Apply all boosts" }),
    );
    expect(
      await screen.findByText(/Load Data before using boosts/),
    ).toBeInTheDocument();

    const saves = queryClient.getQueryData<SaveSummary[]>(
      savesQueryOptions.queryKey,
    );
    expect(saves).toHaveLength(1);
    if (!saves) throw new Error("Expected the default save");
    queryClient.setQueryData<SaveSummary[]>(savesQueryOptions.queryKey, [
      { ...saves[0], isActive: false },
      {
        id: 2,
        contextToken: "save-token-2",
        name: "Other save",
        isActive: true,
        createdAtUtc: "2026-07-29T00:00:00.000Z",
        updatedAtUtc: "2026-07-29T00:00:00.000Z",
      },
    ]);

    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: "Apply all boosts?" }),
      ).toBeNull();
      expect(screen.queryByText(/Load Data before using boosts/)).toBeNull();
    });
  });

  it("renders safe graphics status and pathless controls", async () => {
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    const graphics = await screen.findByRole("region", { name: "Graphics" });
    expect(graphics).toHaveTextContent("No root selected");
    expect(graphics).toHaveTextContent("never displays its path");
    expect(
      within(graphics).getByRole("button", { name: "Choose graphics folder" }),
    ).toBeInTheDocument();
    expect(
      within(graphics).getByRole("button", { name: "Rescan graphics" }),
    ).toBeDisabled();
    expect(
      within(graphics).getByRole("button", { name: "Clear graphics folder" }),
    ).toBeDisabled();

    await user.click(
      within(graphics).getByRole("button", { name: "Choose graphics folder" }),
    );
    expect(await screen.findByText(/index is rebuilding/)).toBeInTheDocument();
  });

  it("shows bounded nonzero scan diagnostics, including configured limits", async () => {
    setGraphicsStatusIpcMock({
      generation: 2,
      selected: true,
      candidate: { available: false, source: "absent" },
      summary: {
        configs: 4,
        mappings: 7,
        truncated: true,
        diagnostics: {
          configLimit: 1,
          entryLimit: 2,
          depthLimit: 3,
          mappingLimit: 4,
          configTooLarge: 5,
          configUnreadable: 6,
          malformedConfig: 7,
          invalidMapping: 8,
          sourceUnreadable: 9,
        },
      },
    });
    renderWithProviders({ initialEntries: ["/settings"] });

    const graphics = await screen.findByRole("region", { name: "Graphics" });
    expect(graphics).toHaveTextContent(
      "1 config-limit hits, 2 entry-limit hits, 3 depth-limit hits, 4 mapping-limit hits, 5 oversized configs, 6 unreadable configs, 7 malformed configs, 8 invalid mappings, 9 unreadable sources.",
    );
  });

  it("clears an earlier action error when a later action succeeds", async () => {
    const user = userEvent.setup();
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      selected: true,
    });
    setGraphicsMutationIpcMockMode("failed");
    renderWithProviders({ initialEntries: ["/settings"] });
    const graphics = await screen.findByRole("region", { name: "Graphics" });

    await user.click(
      within(graphics).getByRole("button", { name: "Choose graphics folder" }),
    );
    expect(
      await within(graphics).findByText(/Graphics update failed/),
    ).toBeInTheDocument();

    setGraphicsMutationIpcMockMode("ready");
    await user.click(
      within(graphics).getByRole("button", { name: "Rescan graphics" }),
    );
    await within(graphics).findByText(/index is rebuilding/);
    expect(within(graphics).queryByText(/Graphics update failed/)).toBeNull();
  });

  it("keeps graphics status unchanged when choosing is cancelled", async () => {
    const user = userEvent.setup();
    setGraphicsStatusIpcMock({
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 7,
      selected: true,
    });
    setGraphicsChooseIpcMockMode("cancel");
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });

    const graphics = await screen.findByRole("region", { name: "Graphics" });
    const statusBefore = queryClient.getQueryData(graphicsKeys.status());
    await user.click(
      within(graphics).getByRole("button", { name: "Choose graphics folder" }),
    );

    expect(
      await within(graphics).findByText(
        "Folder choice cancelled. Graphics settings are unchanged.",
      ),
    ).toBeInTheDocument();
    expect(graphics).toHaveTextContent("Generation 7");
    expect(graphics).not.toHaveTextContent("index is rebuilding");
    expect(queryClient.getQueryData(graphicsKeys.status())).toEqual(
      statusBefore,
    );
    expect(
      queryClient.getQueryState(graphicsKeys.status())?.isInvalidated,
    ).toBe(false);
  });

  it("keeps other Settings sections rendered when boost context fails", async () => {
    setCurrentSnapshotIpcMockFailure(true);
    renderWithProviders({ initialEntries: ["/settings"] });

    expect(
      await screen.findByText("Could not load boost context"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Graphics" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Save data" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Bridge" })).toBeInTheDocument();
  });

  it("keeps the other Settings sections rendered when graphics status fails", async () => {
    setGraphicsStatusIpcMockMode("failed");
    renderWithProviders({ initialEntries: ["/settings"] });

    expect(
      await screen.findByText("Could not load graphics data"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Save data" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Bridge" })).toBeInTheDocument();
  });

  it("sets the shared default player analysis view", async () => {
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    const control = await screen.findByRole("combobox", {
      name: "Default player analysis view",
    });
    expect(control).toHaveValue("general");
    expect(
      screen.getByText(
        "Used by Player Search and Player Profile when their URL does not specify a view.",
      ),
    ).toBeInTheDocument();

    await user.selectOptions(control, "moneyball");

    expect(control).toHaveValue("moneyball");
    expect(useMoneyballPreferences.getState().defaultAnalysisView).toBe(
      "moneyball",
    );
  });

  it("invalidates current-only products when deleting the current snapshot", async () => {
    setSnapshotHistoryIpcMock(HISTORY);
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    queryClient.setQueryData(searchKeys.all, []);
    queryClient.setQueryData(playerKeys.all, []);
    queryClient.setQueryData(clubDnaKeys.all, []);
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
      expect(queryClient.getQueryState(clubDnaKeys.all)?.isInvalidated).toBe(
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
