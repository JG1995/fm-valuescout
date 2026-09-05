import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Suspense } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { renderWithProviders as renderApp } from "@/testing/render-with-providers";
import {
  getLastSnapshotManagementIpcArgs,
  observeSnapshotIpcCall,
  resolveBusyLoadDataRequest,
  resolveBusySnapshotDateEditRequest,
  resolveBusySnapshotDeleteRequest,
  type SnapshotMetadata,
  setLoadDataIpcMockMode,
  setSnapshotDateEditIpcMockMode,
  setSnapshotDeleteIpcMockMode,
  setSnapshotHistoryIpcMock,
  setSnapshotRenameIpcMockMode,
} from "@/testing/snapshot-ipc-mock";
import { SnapshotPanelsWithErrorBoundary } from "./snapshot-panels-with-error-boundary";

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
    customName: "Transfer window",
    gameDate: "2026-08-01",
    gameDateSource: "inGame",
    playerCount: 24,
    loadedAtUtc: "2026-07-28T15:00:00.000Z",
    isCurrent: true,
  },
  {
    id: 13,
    contextToken: "snapshot-token-13",
    saveId: 1,
    customName: null,
    gameDate: null,
    gameDateSource: "unknown",
    playerCount: 19,
    loadedAtUtc: "2026-07-28T16:00:00.000Z",
    isCurrent: false,
  },
];

function seedHistory() {
  setSnapshotHistoryIpcMock(HISTORY);
}

function renderWithProviders() {
  return renderApp({ initialEntries: ["/settings"] });
}

function renderPanels(
  onBeforeContextChange: () => Promise<void>,
  onCurrentContextChanged?: () => void,
) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  render(
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={null}>
        <SnapshotPanelsWithErrorBoundary
          onBeforeContextChange={onBeforeContextChange}
          onCurrentContextChanged={onCurrentContextChanged}
        />
      </Suspense>
    </QueryClientProvider>,
  );
  return queryClient;
}

describe("snapshot panels", () => {
  beforeEach(() => {
    setLoadDataIpcMockMode("success");
  });

  afterEach(() => {
    resolveBusyLoadDataRequest();
    resolveBusySnapshotDeleteRequest();
    resolveBusySnapshotDateEditRequest();
  });

  it("shows empty snapshot guidance on open", async () => {
    renderWithProviders();

    expect(await screen.findByText(/^Snapshot$/i)).toBeInTheDocument();
    expect(
      screen.getByText(/No snapshot loaded for the active save/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/^Saves$/i)).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Active save" })).toHaveValue(
      "1",
    );
  });

  it("shows snapshot metadata without a sanity table after Load Data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded for the active save/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/In database:/i)).toBeInTheDocument();
    expect(
      screen.queryByRole("table", { name: "Player sanity list" }),
    ).toBeNull();
    expect(screen.queryByText("Alex Morgan")).toBeNull();
  });

  it("shows truncated banner after capped Load Data", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded for the active save/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(
        /Incomplete snapshot: scan was capped at 500 players/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Loaded 500 players into the database/i),
    ).toBeInTheDocument();
  });

  it("creates and switches saves", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByRole("combobox", { name: "Active save" });
    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    expect(select).toHaveValue("1");
    expect(
      screen.getByRole("option", { name: "Youth intake" }),
    ).toBeInTheDocument();
    await user.selectOptions(select, "2");
    expect(select).toHaveValue("2");
  });

  it("clears and restores snapshot overview when switching saves", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded for the active save/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(/In database:/i)).toBeInTheDocument();

    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    await user.selectOptions(select, "2");

    expect(
      await screen.findByText(/No snapshot loaded for the active save/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(/In database:/i)).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Loaded 3 players into the database/i),
    ).not.toBeInTheDocument();

    await user.selectOptions(select, "1");
    expect(await screen.findByText(/In database:/i)).toBeInTheDocument();
  });

  it("retargets the rename field when the top bar switches save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    await user.selectOptions(select, "2");

    // A draft left over from the previous save would rename the new one to the
    // old name on the next submit.
    await waitFor(() => {
      expect(screen.getByLabelText("Rename active save")).toHaveValue(
        "Youth intake",
      );
    });
  });

  it("renames the active save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const renameInput = await screen.findByLabelText("Rename active save");
    await user.clear(renameInput);
    await user.type(renameInput, "Main career");
    await user.click(screen.getByRole("button", { name: "Rename save" }));

    expect(
      await screen.findByRole("combobox", { name: "Active save" }),
    ).toHaveDisplayValue("Main career");
  });

  it("lists snapshots by in-game date, with undated snapshots last", async () => {
    seedHistory();
    renderWithProviders();

    const table = await screen.findByRole("table", {
      name: "Snapshot history",
    });
    const rows = within(table).getAllByRole("row");

    expect(rows).toHaveLength(4);
    expect(rows[1]).toHaveTextContent("Transfer window");
    expect(rows[1]).toHaveTextContent("2026-08-01");
    expect(rows[2]).toHaveTextContent("2026-06-01");
    expect(rows[3]).toHaveTextContent("Unknown in-game date");
  });

  it("renames a snapshot without changing its date order", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: "Rename snapshot 2026-08-01",
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Rename snapshot/ });
    const name = within(dialog).getByLabelText("Snapshot name");
    await user.clear(name);
    await user.type(name, "Pre-season review");
    await user.click(within(dialog).getByRole("button", { name: "Save name" }));

    await screen.findByText("Pre-season review");
    const rows = within(
      screen.getByRole("table", { name: "Snapshot history" }),
    ).getAllByRole("row");
    expect(rows[1]).toHaveTextContent("Pre-season review");
    expect(rows[1]).toHaveTextContent("2026-08-01");
    expect(rows[2]).toHaveTextContent("2026-06-01");
  });

  it("confirms the cascade before deleting a historical snapshot", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Delete snapshot/ });
    expect(dialog).toHaveTextContent("Moneyball import data");
    expect(dialog).toHaveTextContent("Planner, Academy, and Youth data stay");
    await user.click(
      within(dialog).getByRole("button", { name: "Delete snapshot" }),
    );

    await waitFor(() => {
      expect(
        screen.queryByRole("button", {
          name: /^Delete snapshot 2026-06-01/,
        }),
      ).not.toBeInTheDocument();
    });
    expect(screen.getByText("Transfer window")).toBeInTheDocument();
  });

  it("distinguishes duplicate dated snapshots before deletion", async () => {
    setSnapshotHistoryIpcMock([
      {
        ...HISTORY[0],
        id: 21,
        contextToken: "snapshot-token-21",
        loadedAtUtc: "2026-07-28T13:00:00.000Z",
        isCurrent: false,
      },
      {
        ...HISTORY[0],
        id: 22,
        contextToken: "snapshot-token-22",
        loadedAtUtc: "2026-07-28T15:00:00.000Z",
        isCurrent: true,
      },
    ]);
    const user = userEvent.setup();
    renderWithProviders();

    expect(
      await screen.findByRole("button", {
        name: "Delete snapshot 2026-06-01 (loaded 2026-07-28 15:00 UTC; snapshot #22)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Delete snapshot 2026-06-01 (loaded 2026-07-28 13:00 UTC; snapshot #21)",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: "Delete snapshot 2026-06-01 (loaded 2026-07-28 15:00 UTC; snapshot #22)",
      }),
    );
    expect(
      screen.getByRole("dialog", {
        name: "Delete snapshot 2026-06-01 (loaded 2026-07-28 15:00 UTC; snapshot #22)?",
      }),
    ).toBeInTheDocument();
  });

  it("waits for the injected callback before deleting the current snapshot", async () => {
    seedHistory();
    const user = userEvent.setup();
    let releaseContextChange!: () => void;
    const onBeforeContextChange = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          releaseContextChange = resolve;
        }),
    );
    const queryClient = renderPanels(onBeforeContextChange);
    let tauriWasCalled = false;
    observeSnapshotIpcCall("deleteSnapshot", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot Transfer window/,
      }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Delete snapshot" }),
    );

    await waitFor(() => expect(onBeforeContextChange).toHaveBeenCalledOnce());
    expect(tauriWasCalled).toBe(false);
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }),
    ).toBe(1);

    releaseContextChange();
    await waitFor(() => expect(tauriWasCalled).toBe(true));
  });

  it("deletes a non-current snapshot without the callback or shared key", async () => {
    seedHistory();
    const user = userEvent.setup();
    const onBeforeContextChange = vi.fn(async () => undefined);
    const queryClient = renderPanels(onBeforeContextChange);
    let tauriWasCalled = false;
    observeSnapshotIpcCall("deleteSnapshot", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot 2026-06-01/,
      }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Delete snapshot" }),
    );

    await waitFor(() => expect(tauriWasCalled).toBe(true));
    expect(onBeforeContextChange).not.toHaveBeenCalled();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }),
    ).toBe(0);
  });

  it("promotes the next in-game-date snapshot when deleting the current one", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot Transfer window/,
      }),
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Delete snapshot" }),
    );

    const rows = await waitFor(() => {
      const nextRows = within(
        screen.getByRole("table", { name: "Snapshot history" }),
      ).getAllByRole("row");
      expect(nextRows).toHaveLength(3);
      return nextRows;
    });
    expect(rows[1]).toHaveTextContent("2026-06-01");
    expect(rows[1]).toHaveTextContent("Current");
  });

  it("keeps the active save when deleting an inactive save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.click(
      await screen.findByRole("button", { name: /^Delete save Archive/ }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Delete save/ });
    expect(dialog).toHaveTextContent("The active save stays unchanged");
    await user.click(
      within(dialog).getByRole("button", { name: "Delete save" }),
    );

    await expect(
      screen.getByRole("combobox", { name: "Active save" }),
    ).toHaveValue("1");
    expect(
      screen.queryByRole("button", { name: /^Delete save Archive/ }),
    ).not.toBeInTheDocument();
  });

  it("distinguishes duplicate save names before deletion", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await screen.findByRole("button", { name: "Delete save Archive (save 2)" });
    const createSaveInput = screen.getByLabelText("New save");
    await waitFor(() => expect(createSaveInput).toHaveValue(""));
    await user.clear(createSaveInput);
    await user.type(createSaveInput, "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    expect(
      await screen.findByRole("button", {
        name: "Delete save Archive (save 2)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Delete save Archive (save 3)" }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Delete save Archive (save 2)" }),
    );
    expect(
      screen.getByRole("dialog", {
        name: "Delete save Archive (save 2)?",
      }),
    ).toBeInTheDocument();
  });

  it("waits for the injected callback before deleting the active save", async () => {
    const user = userEvent.setup();
    let releaseContextChange!: () => void;
    const onBeforeContextChange = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          releaseContextChange = resolve;
        }),
    );
    const queryClient = renderPanels(onBeforeContextChange);
    let tauriWasCalled = false;
    observeSnapshotIpcCall("deleteSave", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", { name: /^Delete save Default save/ }),
    );
    await user.click(
      within(screen.getByRole("dialog", { name: /^Delete save/ })).getByRole(
        "button",
        { name: "Delete save" },
      ),
    );

    await waitFor(() => expect(onBeforeContextChange).toHaveBeenCalledOnce());
    expect(tauriWasCalled).toBe(false);
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }),
    ).toBe(1);

    releaseContextChange();
    await waitFor(() => expect(tauriWasCalled).toBe(true));
  });

  it("deletes an inactive save without the callback or shared key", async () => {
    const user = userEvent.setup();
    const onBeforeContextChange = vi.fn(async () => undefined);
    const queryClient = renderPanels(onBeforeContextChange);
    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    let tauriWasCalled = false;
    observeSnapshotIpcCall("deleteSave", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", { name: /^Delete save Archive/ }),
    );
    await user.click(
      within(screen.getByRole("dialog", { name: /^Delete save/ })).getByRole(
        "button",
        { name: "Delete save" },
      ),
    );

    await waitFor(() => expect(tauriWasCalled).toBe(true));
    expect(onBeforeContextChange).not.toHaveBeenCalled();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }),
    ).toBe(0);
  });

  it("switches to another save after deleting the active save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.click(
      await screen.findByRole("button", { name: /^Delete save Default save/ }),
    );
    await user.click(
      within(screen.getByRole("dialog", { name: /^Delete save/ })).getByRole(
        "button",
        { name: "Delete save" },
      ),
    );

    await expect(
      screen.getByRole("combobox", { name: "Active save" }),
    ).toHaveDisplayValue("Archive");
  });

  it("replaces the final deleted save with a blank Default save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", { name: /^Delete save Default save/ }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Delete save/ });
    expect(dialog).toHaveTextContent("A blank Default save will replace it");
    await user.click(
      within(dialog).getByRole("button", { name: "Delete save" }),
    );

    await expect(
      screen.getByRole("combobox", { name: "Active save" }),
    ).toHaveDisplayValue("Default save");
  });

  it("clears the Load Data outcome when a final save is recreated", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /^Delete save Default save/ }),
    );
    await user.click(
      within(screen.getByRole("dialog", { name: /^Delete save/ })).getByRole(
        "button",
        { name: "Delete save" },
      ),
    );

    expect(
      await screen.findByText(/No snapshot loaded for the active save/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Loaded 3 players into the database/i),
    ).not.toBeInTheDocument();
  });

  it("restores focus to a cancelled snapshot deletion trigger", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    const trigger = await screen.findByRole("button", {
      name: /^Delete snapshot 2026-06-01/,
    });
    trigger.focus();
    await user.click(trigger);
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Cancel" }),
    );

    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("keeps destructive controls pending until the deletion settles", async () => {
    seedHistory();
    setSnapshotDeleteIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Delete snapshot/ });
    await user.click(
      within(dialog).getByRole("button", { name: "Delete snapshot" }),
    );

    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Deleting…" }),
    ).toBeDisabled();
    await user.keyboard("{Escape}");
    expect(
      screen.getByRole("dialog", { name: /^Delete snapshot/ }),
    ).toBeVisible();
  });

  it("uses the originally confirmed snapshot identity after the active save changes", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.click(
      await screen.findByRole("button", {
        name: /^Delete snapshot 2026-06-01/,
      }),
    );
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );
    await user.click(
      within(
        screen.getByRole("dialog", { name: /^Delete snapshot/ }),
      ).getByRole("button", { name: "Delete snapshot" }),
    );

    await waitFor(() => {
      expect(getLastSnapshotManagementIpcArgs()).toEqual({
        snapshotId: 11,
        contextToken: "snapshot-token-11",
      });
    });
  });

  it("renders a disambiguated Edit date action per snapshot row", async () => {
    setSnapshotHistoryIpcMock([
      {
        ...HISTORY[0],
        id: 21,
        contextToken: "snapshot-token-21",
        loadedAtUtc: "2026-07-28T13:00:00.000Z",
        isCurrent: false,
      },
      {
        ...HISTORY[0],
        id: 22,
        contextToken: "snapshot-token-22",
        loadedAtUtc: "2026-07-28T15:00:00.000Z",
        isCurrent: true,
      },
    ]);
    renderWithProviders();

    expect(
      await screen.findByRole("button", {
        name: "Edit date for snapshot 2026-06-01 (loaded 2026-07-28 15:00 UTC; snapshot #22)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Edit date for snapshot 2026-06-01 (loaded 2026-07-28 13:00 UTC; snapshot #21)",
      }),
    ).toBeInTheDocument();
  });

  it("rejects empty, malformed, and impossible dates without losing state", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();
    let tauriWasCalled = false;
    observeSnapshotIpcCall("updateSnapshotDate", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    const input = within(dialog).getByLabelText("In-game date");
    const save = within(dialog).getByRole("button", { name: "Save date" });

    await user.clear(input);
    await user.click(save);
    expect(
      within(dialog).getByText("Enter a valid date in YYYY-MM-DD format."),
    ).toBeVisible();
    expect(tauriWasCalled).toBe(false);

    await user.type(input, "not-a-date");
    await user.click(save);
    expect(
      within(dialog).getByText("Enter a valid date in YYYY-MM-DD format."),
    ).toBeVisible();
    expect(tauriWasCalled).toBe(false);

    await user.clear(input);
    await user.type(input, "2026-02-30");
    await user.click(save);
    expect(await within(dialog).findByText(/valid date/i)).toBeVisible();
    expect(input).toHaveValue("2026-02-30");
    expect(
      screen.getByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    ).toBeInTheDocument();
  });

  it("promotes an older snapshot and notifies the route when the winner changes", async () => {
    seedHistory();
    const user = userEvent.setup();
    const onBeforeContextChange = vi.fn(async () => undefined);
    const onCurrentContextChanged = vi.fn();
    renderPanels(onBeforeContextChange, onCurrentContextChanged);

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-09-01",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    const rows = await waitFor(() => {
      const nextRows = within(
        screen.getByRole("table", { name: "Snapshot history" }),
      ).getAllByRole("row");
      expect(nextRows[1]).toHaveTextContent("2026-09-01");
      return nextRows;
    });
    expect(rows[1]).toHaveTextContent("Current");
    expect(rows[2]).toHaveTextContent("Transfer window");
    expect(onCurrentContextChanged).toHaveBeenCalledOnce();
    expect(getLastSnapshotManagementIpcArgs()).toEqual({
      snapshotId: 11,
      contextToken: "snapshot-token-11",
      gameDate: "2026-09-01",
    });
  });

  it("refreshes only history when a non-current edit does not promote", async () => {
    seedHistory();
    const user = userEvent.setup();
    const onBeforeContextChange = vi.fn(async () => undefined);
    const onCurrentContextChanged = vi.fn();
    renderPanels(onBeforeContextChange, onCurrentContextChanged);

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-07-01",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    await waitFor(() => {
      const rows = within(
        screen.getByRole("table", { name: "Snapshot history" }),
      ).getAllByRole("row");
      expect(rows[2]).toHaveTextContent("2026-07-01");
    });
    const rows = within(
      screen.getByRole("table", { name: "Snapshot history" }),
    ).getAllByRole("row");
    expect(rows[1]).toHaveTextContent("Transfer window");
    expect(rows[1]).toHaveTextContent("Current");
    expect(onCurrentContextChanged).not.toHaveBeenCalled();
  });

  it("patches the cached current summary when the edited current snapshot stays current", async () => {
    seedHistory();
    const user = userEvent.setup();
    const onBeforeContextChange = vi.fn(async () => undefined);
    const onCurrentContextChanged = vi.fn();
    renderPanels(onBeforeContextChange, onCurrentContextChanged);

    expect(await screen.findByText(/24 players/)).toHaveTextContent(
      "2026-08-01",
    );
    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot Transfer window/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-07-15",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    await waitFor(() => {
      expect(screen.getByText(/24 players/)).toHaveTextContent("2026-07-15");
    });
    expect(screen.getByText(/24 players/)).toHaveTextContent("24 players");
    expect(onCurrentContextChanged).not.toHaveBeenCalled();
  });

  it("keeps the switched save current summary unchanged when a retained date edit stays current", async () => {
    setSnapshotHistoryIpcMock([
      ...HISTORY,
      {
        id: 21,
        contextToken: "snapshot-token-21",
        saveId: 2,
        customName: null,
        gameDate: "2026-05-01",
        gameDateSource: "inGame",
        playerCount: 42,
        loadedAtUtc: "2026-07-29T12:00:00.000Z",
        isCurrent: true,
      },
    ]);
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Archive");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await screen.findByRole("button", {
      name: "Delete save Archive (save 2)",
    });

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot Transfer window/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );
    await screen.findByText(/42 players/);
    expect(screen.getByText(/42 players/)).toHaveTextContent("2026-05-01");

    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-07-15",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: /^Edit date/ }),
      ).not.toBeInTheDocument();
    });
    expect(getLastSnapshotManagementIpcArgs()).toEqual({
      snapshotId: 12,
      contextToken: "snapshot-token-12",
      gameDate: "2026-07-15",
    });
    expect(screen.getByText(/42 players/)).toHaveTextContent("2026-05-01");
    expect(screen.getByText(/42 players/)).not.toHaveTextContent("2026-07-15");
  });

  it("waits for the injected callback before every date-edit IPC call", async () => {
    seedHistory();
    const user = userEvent.setup();
    let releaseContextChange!: () => void;
    const onBeforeContextChange = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          releaseContextChange = resolve;
        }),
    );
    renderPanels(onBeforeContextChange);
    let tauriWasCalled = false;
    observeSnapshotIpcCall("updateSnapshotDate", () => {
      tauriWasCalled = true;
    });

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-07-01",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    await waitFor(() => expect(onBeforeContextChange).toHaveBeenCalledOnce());
    expect(tauriWasCalled).toBe(false);

    releaseContextChange();
    await waitFor(() => expect(tauriWasCalled).toBe(true));
  });

  it("keeps stale-identity backend errors in the dialog with the target retained", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    setSnapshotHistoryIpcMock(
      HISTORY.map((snapshot) =>
        snapshot.id === 11
          ? { ...snapshot, contextToken: "snapshot-token-11-rotated" }
          : snapshot,
      ),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    const input = within(dialog).getByLabelText("In-game date");
    await user.clear(input);
    await user.type(input, "2026-07-01");
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    expect(
      await within(dialog).findByText("Snapshot changed or no longer exists"),
    ).toBeVisible();
    expect(input).toHaveValue("2026-07-01");
    expect(
      screen.getByRole("dialog", { name: /^Edit date/ }),
    ).toBeInTheDocument();
  });

  it("keeps date-edit controls pending until the submission settles", async () => {
    seedHistory();
    setSnapshotDateEditIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: /^Edit date for snapshot 2026-06-01/,
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Edit date/ });
    await user.clear(within(dialog).getByLabelText("In-game date"));
    await user.type(
      within(dialog).getByLabelText("In-game date"),
      "2026-07-01",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save date" }));

    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Saving…" }),
    ).toBeDisabled();
    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog", { name: /^Edit date/ })).toBeVisible();

    resolveBusySnapshotDateEditRequest();
    await waitFor(() => {
      expect(
        screen.queryByRole("dialog", { name: /^Edit date/ }),
      ).not.toBeInTheDocument();
    });
  });

  it("restores focus to a cancelled date-edit trigger", async () => {
    seedHistory();
    const user = userEvent.setup();
    renderWithProviders();

    const trigger = await screen.findByRole("button", {
      name: /^Edit date for snapshot 2026-06-01/,
    });
    trigger.focus();
    await user.click(trigger);
    await user.click(
      within(screen.getByRole("dialog", { name: /^Edit date/ })).getByRole(
        "button",
        { name: "Cancel" },
      ),
    );

    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("keeps snapshot management errors in the dialog", async () => {
    seedHistory();
    setSnapshotRenameIpcMockMode("failure");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("button", {
        name: "Rename snapshot 2026-06-01",
      }),
    );
    const dialog = screen.getByRole("dialog", { name: /^Rename snapshot/ });
    await user.click(within(dialog).getByRole("button", { name: "Save name" }));

    expect(
      await within(dialog).findByText("Snapshot rename failed"),
    ).toBeVisible();
  });
});
