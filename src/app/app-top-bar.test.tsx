import {
  QueryClient,
  QueryClientProvider,
  useQuery,
} from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterContextProvider,
  RouterProvider,
} from "@tanstack/react-router";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppTopBar } from "@/app/components/app-top-bar";
import type { RouterContext } from "@/app/router-context";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { academyClassesQueryOptions } from "@/features/academy/api/academy-classes-query-options";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { setBridgeStatusIpcMockMode } from "@/features/memory-read/api/bridge-status-ipc-mock";
import { searchKeys } from "@/features/search/api/search-keys";
import { squadKeys } from "@/features/squad/api/squad-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { routeTree } from "@/routeTree.gen";
import {
  deferAcademyClassesFetch,
  setAcademyClasses,
} from "@/testing/academy-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  emitLoadDataProgress,
  getLastLoadDataIpcArgs,
  observeSnapshotIpcCall,
  rejectBusyLoadDataRequest,
  resolveBusyLoadDataRequest,
  resolveCreateSaveIpcMock,
  resolvePendingSetActiveSaveIpcMock,
  setActiveSaveIpcMockMode,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

function AcademyCacheProbe() {
  const { data: classes } = useQuery(academyClassesQueryOptions);
  return (
    <output data-testid="academy-cache-value">
      {classes?.[0]?.classYear ?? "none"}
    </output>
  );
}

async function renderTopBarWithAcademyProbe() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  await queryClient.fetchQuery(academyClassesQueryOptions);
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: ["/settings"] }),
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <AcademyCacheProbe />
    </QueryClientProvider>,
  );
}

// Load Data lives in the shell top bar, so its outcome banner is asserted here
// rather than against the bridge panel that used to own the button.
describe("app top bar", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
    setLoadDataIpcMockMode("success");
  });

  afterEach(() => {
    resolveBusyLoadDataRequest();
    rejectBusyLoadDataRequest();
  });

  it("disables session history controls before navigation", async () => {
    renderWithProviders({ initialEntries: ["/settings"] });

    expect(await screen.findByRole("button", { name: "Back" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Forward" })).toBeDisabled();
  });

  it("does not expose Load Data before the active save context is available", async () => {
    renderWithProviders({ initialEntries: ["/settings"] });

    expect(screen.queryByRole("button", { name: "Load Data" })).toBeNull();
    expect(
      await screen.findByRole("button", { name: "Load Data" }),
    ).toBeEnabled();
  });

  it("tracks history traversal and truncates Forward after a branch", async () => {
    const user = userEvent.setup();
    const { router } = renderWithProviders();
    const back = await screen.findByRole("button", { name: "Back" });
    const forward = screen.getByRole("button", { name: "Forward" });

    await act(async () => router.history.push("/settings?tab=first#saves"));
    await act(async () => router.history.push("/search?view=general#filters"));

    expect(back).toBeEnabled();
    expect(forward).toBeDisabled();

    await user.click(back);
    await waitFor(() =>
      expect(router.history.location.href).toBe("/settings?tab=first#saves"),
    );
    expect(forward).toBeEnabled();

    await user.click(back);
    await waitFor(() => expect(router.history.location.href).toBe("/"));

    await user.click(forward);
    await waitFor(() =>
      expect(router.history.location.href).toBe("/settings?tab=first#saves"),
    );

    await user.click(back);
    await waitFor(() => expect(router.history.location.href).toBe("/"));
    await act(async () => router.history.push("/academy"));

    await waitFor(() => expect(forward).toBeDisabled());
  });

  it("labels session history controls and keeps them focusable", async () => {
    renderWithProviders();

    const search = await screen.findByRole("combobox", {
      name: "Search players",
    });
    const back = screen.getByRole("button", { name: "Back" });
    const forward = screen.getByRole("button", { name: "Forward" });

    expect(
      back.compareDocumentPosition(forward) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      forward.compareDocumentPosition(search) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(back).toHaveAttribute("title", "Back");
    expect(forward).toHaveAttribute("title", "Forward");
    back.focus();
    expect(back).toHaveFocus();
  });

  it("cleans up the history subscription after Strict Mode effect replay", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const router = createRouter({
      routeTree,
      context: { queryClient } satisfies RouterContext,
      defaultPreloadStaleTime: 0,
      history: createMemoryHistory({ initialEntries: ["/settings"] }),
    });
    const realSubscribe = router.history.subscribe.bind(router.history);
    let activeSubscriptions = 0;
    vi.spyOn(router.history, "subscribe").mockImplementation((listener) => {
      activeSubscriptions += 1;
      const unsubscribe = realSubscribe(listener);
      return () => {
        activeSubscriptions -= 1;
        unsubscribe();
      };
    });

    const { unmount } = render(
      <StrictMode>
        <QueryClientProvider client={queryClient}>
          <RouterContextProvider router={router}>
            <AppTopBar />
          </RouterContextProvider>
        </QueryClientProvider>
      </StrictMode>,
    );

    await screen.findByRole("button", { name: "Back" });
    expect(activeSubscriptions).toBe(1);

    unmount();
    expect(activeSubscriptions).toBe(0);
  });

  it("reports ingest success after load_data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
    expect(screen.getByText(/preparation 300ms/i)).toBeInTheDocument();
    expect(screen.getByText(/scoring 400ms/i)).toBeInTheDocument();
    expect(screen.getByText(/save 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/finalization 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/total 2\.1s/i)).toBeInTheDocument();
    expect(screen.queryByText(/ingest 400ms/i)).toBeNull();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it("sends unlimited maxAccepted", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await screen.findByText(/Loaded 3 players into the database/i);

    const args = getLastLoadDataIpcArgs() as Record<string, unknown>;
    expect(args).toMatchObject({ maxAccepted: null });
    expect(args.onProgress).toBeDefined();
    expect(typeof args.onProgress).toBe("object");
  });

  it("preserves player pages and does not use neutral key during Load Data", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders();
    const searchPage = searchKeys.players(0, 50);
    const squadPage = squadKeys.players(0, 50);
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    let preservedDuringInvoke = false;
    let neutralDuringInvoke = false;
    observeSnapshotIpcCall("loadData", () => {
      preservedDuringInvoke =
        queryClient.getQueryData(searchPage) !== undefined &&
        queryClient.getQueryData(squadPage) !== undefined;
      neutralDuringInvoke =
        queryClient.isMutating({
          mutationKey: playerResultContextMutationKey,
        }) > 0;
    });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await screen.findByText(/Loaded 3 players into the database/i);

    expect(preservedDuringInvoke).toBe(true);
    expect(neutralDuringInvoke).toBe(false);
    expect(queryClient.getQueryData(searchPage)).toBeUndefined();
    expect(queryClient.getQueryData(squadPage)).toBeUndefined();
  });

  it("clears player pages and exposes the transition before switching active saves", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders();
    const second = resolveCreateSaveIpcMock({ name: "Second save" });
    await queryClient.invalidateQueries({ queryKey: ["snapshot", "saves"] });
    const searchPage = searchKeys.players(0, 50);
    const squadPage = squadKeys.players(0, 50);
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    let mutationWasVisible = false;
    observeSnapshotIpcCall("setActiveSave", () => {
      expect(queryClient.getQueryData(searchPage)).toBeUndefined();
      expect(queryClient.getQueryData(squadPage)).toBeUndefined();
      mutationWasVisible =
        queryClient.isMutating({
          mutationKey: playerResultContextMutationKey,
        }) > 0;
    });

    await screen.findByRole("option", { name: "Second save" });
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      String(second.id),
    );

    await waitFor(() => expect(mutationWasVisible).toBe(true));
  });

  it("warns that a capped scan produced a partial ingest", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/the scan was capped at 500 players/i),
    ).toBeInTheDocument();
  });

  it("reports a scan failure from load_data", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();
    expect(
      screen.getByText(/scan produced zero player candidates/i),
    ).toBeInTheDocument();
  });

  it("reports an ingest failure from load_data", async () => {
    setLoadDataIpcMockMode("ingestFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Ingest failed/i)).toBeInTheDocument();
    expect(screen.getByText(/dump validation failed/i)).toBeInTheDocument();
  });

  it.each([
    ["success", /Loaded 3 players into the database/i],
    ["scanFailed", /Scan failed/i],
  ] as const)("dismisses a completed %s outcome", async (mode, message) => {
    setLoadDataIpcMockMode(mode);
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(message)).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Dismiss Load Data outcome" }),
    );

    expect(screen.queryByText(message)).not.toBeInTheDocument();
  });

  it("drops a failure banner once the user switches save", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    // The failure described the previous save's scan, not this one's.
    expect(screen.queryByText(/Scan failed/i)).not.toBeInTheDocument();
  });

  it("swaps the button label for the scan phase while the request is pending", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();
  });

  it("refetches an active Academy query after Load Data", async () => {
    const user = userEvent.setup();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    await renderTopBarWithAcademyProbe();

    expect(await screen.findByTestId("academy-cache-value")).toHaveTextContent(
      "2026",
    );
    setAcademyClasses([{ id: 8, classYear: 2027, memberCount: 0 }]);

    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "2027",
      ),
    );
  });

  it("invalidates cached Staff and Club DNA data after Load Data", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders();
    const staffProbeKey = [...staffKeys.all, "probe"];
    const clubDnaProbeKey = [...clubDnaKeys.all, "probe"];
    queryClient.setQueryData(staffProbeKey, []);
    queryClient.setQueryData(clubDnaProbeKey, []);

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    await waitFor(() => {
      expect(queryClient.getQueryState(staffProbeKey)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryState(clubDnaProbeKey)?.isInvalidated).toBe(
        true,
      );
    });
  });

  it("refetches an active Academy query after switching saves", async () => {
    const user = userEvent.setup();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    await renderTopBarWithAcademyProbe();

    expect(await screen.findByTestId("academy-cache-value")).toHaveTextContent(
      "2026",
    );
    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );

    setAcademyClasses([{ id: 8, classYear: 2027, memberCount: 0 }]);
    const releaseClassesFetch = deferAcademyClassesFetch();
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "1",
    );

    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "none",
      ),
    );
    releaseClassesFetch();
    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "2027",
      ),
    );
  });

  it("invalidates cached Staff and Club DNA data after switching saves", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const staffProbeKey = [...staffKeys.all, "probe"];
    const clubDnaProbeKey = [...clubDnaKeys.all, "probe"];
    queryClient.setQueryData(staffProbeKey, []);
    queryClient.setQueryData(clubDnaProbeKey, []);

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );

    await waitFor(() => {
      expect(queryClient.getQueryState(staffProbeKey)?.isInvalidated).toBe(
        true,
      );
      expect(queryClient.getQueryState(clubDnaProbeKey)?.isInvalidated).toBe(
        true,
      );
    });
  });

  it("A→B→A before settlement hides success outcome, late progress but reconciles DB truth on active A", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const searchPage = searchKeys.players(0, 50);
    const squadPage = squadKeys.players(0, 50);
    queryClient.setQueryData(searchPage, { players: ["old"] });
    queryClient.setQueryData(squadPage, { players: ["old"] });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeInTheDocument();

    // Emit progress for original save A
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });

    // A→B - stale command stays busy but phase is no longer owned, so generic label
    await user.type(await screen.findByLabelText("New save"), "Second");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );
    await screen.findByRole("option", { name: "Second" });
    expect(
      await screen.findByRole("button", { name: "Loading…" }),
    ).toBeDisabled();
    expect(
      screen.queryByRole("button", { name: "Scanning…" }),
    ).not.toBeInTheDocument();

    // Late progress for A must be ignored and keep generic busy label
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      });
    });
    expect(
      screen.getByRole("button", { name: "Loading…" }),
    ).toBeInTheDocument();

    // B→A before settlement - still stale revision, so still generic Loading…
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "1",
    );
    await screen.findByRole("option", { name: "Default save" });

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 10,
        total: 10,
      });
    });
    expect(
      screen.getByRole("button", { name: "Loading…" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Scanning…" }),
    ).not.toBeInTheDocument();

    // New roots after final switch will be reconciled away once effective publication settles on active A
    queryClient.setQueryData(searchPage, { players: ["new"] });
    queryClient.setQueryData(squadPage, { players: ["new"] });

    await act(async () => {
      resolveBusyLoadDataRequest();
    });

    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Loading…" }),
      ).not.toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("button", { name: "Scanning…" }),
    ).not.toBeInTheDocument();
    // Stale presentation stays hidden, but DB truth was reconciled: exact roots cleared / current owners invalidated
    expect(screen.queryByText(/Loaded 3 players/)).not.toBeInTheDocument();
    expect(queryClient.getQueryData(searchPage)).toBeUndefined();
    expect(queryClient.getQueryData(squadPage)).toBeUndefined();

    // Late progress after stale settlement must not revive
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "saving",
        completed: 10,
        total: 10,
      });
    });
    expect(screen.queryByText(/Loaded 3 players/)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Loading…" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Scanning…" }),
    ).not.toBeInTheDocument();
  });

  it("A→B→A before settlement hides error outcome and does not clear", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const searchPage = searchKeys.players(0, 50);
    queryClient.setQueryData(searchPage, { players: ["old"] });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await screen.findByRole("button", { name: "Scanning…" });

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });

    await user.type(await screen.findByLabelText("New save"), "Second");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );
    expect(
      await screen.findByRole("button", { name: "Loading…" }),
    ).toBeDisabled();
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "1",
    );
    expect(
      screen.getByRole("button", { name: "Loading…" }),
    ).toBeInTheDocument();

    // New roots after final switch must survive stale error
    queryClient.setQueryData(searchPage, { players: ["new"] });

    await act(async () => {
      rejectBusyLoadDataRequest({
        phase: "ingest",
        message: "dump validation failed",
      });
    });

    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Loading…" }),
      ).not.toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("button", { name: "Scanning…" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/Ingest failed/)).not.toBeInTheDocument();
    expect(
      screen.queryByText(/dump validation failed/),
    ).not.toBeInTheDocument();
    expect(queryClient.getQueryData(searchPage)).toEqual({ players: ["new"] });

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 5,
        total: 10,
      });
    });
    expect(screen.queryByText(/Ingest failed/)).not.toBeInTheDocument();
  });

  it("keeps Load Data button width stable via established token class", async () => {
    renderWithProviders();

    const button = await screen.findByRole("button", { name: "Load Data" });
    expect(button.className).toMatch(/min-w-36/);
  });

  it("shows scan indeterminate progress and each later phase as determinate", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    // Pending without progress shows Scanning… button and live region empty then scan indeterminate
    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    const liveRegion = document.querySelector(
      '[aria-live="polite"]',
    ) as HTMLElement;
    expect(within(liveRegion).getByText("Scanning…")).toBeInTheDocument();
    const scanBar = screen.getByRole("progressbar", { name: "Scanning…" });
    expect(scanBar).not.toHaveAttribute("value");
    expect(screen.queryByText(/%/)).toBeNull();

    // Preparing 0/10 -> 10/10
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 0,
        total: 10,
      });
    });
    expect(
      await screen.findByRole("button", { name: "Preparing…" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("progressbar", { name: "Preparing… 0 of 10" }),
    ).toHaveAttribute("value", "0");

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 10,
        total: 10,
      });
    });
    expect(
      screen.getByRole("progressbar", { name: "Preparing… 10 of 10" }),
    ).toHaveAttribute("max", "10");

    // Scoring
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 0,
        total: 5,
      });
    });
    expect(
      await screen.findByRole("button", { name: "Scoring…" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("progressbar", { name: "Scoring… 0 of 5" }),
    ).toBeInTheDocument();
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 5,
        total: 5,
      });
    });
    expect(
      screen.getByRole("progressbar", { name: "Scoring… 5 of 5" }),
    ).toHaveAttribute("value", "5");

    // Saving
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "saving",
        completed: 0,
        total: 5,
      });
    });
    expect(
      await screen.findByRole("button", { name: "Saving…" }),
    ).toBeDisabled();
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "saving",
        completed: 5,
        total: 5,
      });
    });
    expect(
      screen.getByRole("progressbar", { name: "Saving… 5 of 5" }),
    ).toBeInTheDocument();

    // Finalizing 0/1 -> 1/1
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "finalizing",
        completed: 0,
        total: 1,
      });
    });
    expect(
      await screen.findByRole("button", { name: "Finalizing…" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("progressbar", { name: "Finalizing… 0 of 1" }),
    ).toHaveAttribute("value", "0");
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "finalizing",
        completed: 1,
        total: 1,
      });
    });
    expect(
      screen.getByRole("progressbar", { name: "Finalizing… 1 of 1" }),
    ).toHaveAttribute("value", "1");

    // Only one progressbar visible, ordered replaces
    expect(screen.getAllByRole("progressbar")).toHaveLength(1);
    expect(screen.queryByText("Scanning…")).not.toBeInTheDocument();
    // ensure progressbar is outside live region (no double announcement)
    expect(scanBar.closest("[aria-live]")).toBeNull();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it("replaces pending progress with success and dismiss handles live region stability", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(
      screen.getByRole("progressbar", { name: "Scanning…" }),
    ).toBeInTheDocument();

    await act(async () => {
      resolveBusyLoadDataRequest();
    });
    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
    // dismiss preserves live region
    await user.click(
      screen.getByRole("button", { name: "Dismiss Load Data outcome" }),
    );
    expect(screen.queryByText(/Loaded 3 players/i)).not.toBeInTheDocument();
    // live region still present empty
    expect(document.querySelector('[aria-live="polite"]')).toBeInTheDocument();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });

  it("suppresses stale progress and keeps generic busy until the switched context refetches", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("progressbar", { name: "Scanning…" }),
    ).toBeInTheDocument();

    await user.type(await screen.findByLabelText("New save"), "Second");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    const saveSelect = await screen.findByRole("combobox", {
      name: "Active save",
    });
    await screen.findByRole("option", { name: "Second" });

    const invalidate = queryClient.invalidateQueries.bind(queryClient);
    let releaseRefetch: () => void = () => {};
    const refetchGate = new Promise<void>((resolve) => {
      releaseRefetch = resolve;
    });
    vi.spyOn(queryClient, "invalidateQueries").mockImplementation(
      async (filters, options) => {
        if (filters?.queryKey?.[0] === "snapshot") await refetchGate;
        return invalidate(filters, options);
      },
    );

    await user.selectOptions(saveSelect, "2");
    expect(
      await screen.findByRole("button", { name: "Loading…" }),
    ).toBeDisabled();
    expect(saveSelect).toHaveValue("1");
    expect(screen.queryByRole("progressbar", { name: "Scanning…" })).toBeNull();
    expect(screen.queryByText("Scanning…")).toBeNull();

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      });
    });
    expect(screen.getByRole("button", { name: "Loading…" })).toBeDisabled();
    expect(screen.queryByRole("progressbar")).toBeNull();

    await act(async () => releaseRefetch());
    await waitFor(() => expect(saveSelect).toHaveValue("2"));
    expect(screen.getByRole("button", { name: "Loading…" })).toBeDisabled();
  });

  it("disables Load Data while active-save selection is pending and captures B after settle", async () => {
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    const loadButton = await screen.findByRole("button", {
      name: "Load Data",
    });
    expect(loadButton).toBeEnabled();

    await user.type(await screen.findByLabelText("New save"), "Second");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await screen.findByRole("option", { name: "Second" });

    setActiveSaveIpcMockMode("busy");
    let loadDataInvokedDuringPending = false;
    observeSnapshotIpcCall("loadData", () => {
      loadDataInvokedDuringPending = true;
    });

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    await waitFor(() => expect(loadButton).toBeDisabled());
    expect(loadButton).toHaveAccessibleName("Load Data");
    expect(screen.queryByRole("button", { name: "Scanning…" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Loading…" })).toBeNull();

    await user.click(loadButton);
    expect(loadDataInvokedDuringPending).toBe(false);
    expect(getLastLoadDataIpcArgs()).toBeUndefined();

    await act(async () => {
      resolvePendingSetActiveSaveIpcMock();
    });

    await waitFor(() => expect(loadButton).toBeEnabled());
    await screen.findByRole("option", { name: "Second" });

    setLoadDataIpcMockMode("busy");
    await user.click(loadButton);

    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();

    await act(async () => {
      emitLoadDataProgress({
        saveId: 2,
        contextToken: "save-token-2",
        phase: "scan",
      });
    });
    expect(
      screen.getByRole("button", { name: "Scanning…" }),
    ).toBeInTheDocument();

    await act(async () => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      });
    });
    expect(
      screen.getByRole("button", { name: "Scanning…" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Preparing…" })).toBeNull();

    await act(async () => {
      resolveBusyLoadDataRequest();
    });
    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(loadButton).toBeEnabled();
    setActiveSaveIpcMockMode("success");
  });
});
