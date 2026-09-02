import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { setSearchPlayersOverride } from "@/testing/search-ipc-mock";
import {
  emitLoadDataProgress,
  getLastLoadDataProgressChannel,
  rejectBusyLoadDataRequest,
  resetSnapshotIpcMock,
  resolveBusyLoadDataRequest,
  resolveLoadDataIpcMock,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";
import { setSquadPlayersOverride } from "@/testing/squad-ipc-mock";
import type { ActiveSaveContext } from "./use-load-data";
import { useLoadData } from "./use-load-data";

function createQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
}

function wrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

describe("useLoadData", () => {
  beforeEach(() => {
    resetSnapshotIpcMock();
    setLoadDataIpcMockMode("success");
    setSearchPlayersOverride(null);
    setSquadPlayersOverride(null);
  });

  afterEach(() => {
    // Ensure busy deferred is cleared regardless of success or reject path
    resolveBusyLoadDataRequest();
    rejectBusyLoadDataRequest();
  });

  it("does not use neutral mutation key and does not clear before invoke", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    let clearedBeforeInvoke = false;
    clearExactRoots.mockImplementation(async () => {
      clearedBeforeInvoke = true;
    });

    let neutralDuringInvoke = false;
    setLoadDataIpcMockMode("busy");

    const mutatePromise = act(async () => {
      result.current.mutate(null);
    });

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    neutralDuringInvoke =
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
      0;
    const searchDuringInvoke = queryClient.getQueryData(searchPage);
    const squadDuringInvoke = queryClient.getQueryData(squadPage);

    expect(searchDuringInvoke).toBeDefined();
    expect(squadDuringInvoke).toBeDefined();
    expect(neutralDuringInvoke).toBe(false);
    expect(clearedBeforeInvoke).toBe(false);
    expect(clearExactRoots).not.toHaveBeenCalled();

    await act(async () => {
      resolveBusyLoadDataRequest();
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(clearExactRoots).toHaveBeenCalledOnce();
    expect(invalidateCurrent).toHaveBeenCalledOnce();

    await mutatePromise;
  });

  it("preserves Search/Squad caches during delayed scan/preparation and does not block", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");

    act(() => {
      result.current.mutate(null);
    });

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(result.current.isPending).toBe(true);
    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    expect(queryClient.getQueryData(squadPage)).toBeDefined();
    expect(
      queryClient.isMutating({ mutationKey: playerResultContextMutationKey }) >
        0,
    ).toBe(false);

    const channel = getLastLoadDataProgressChannel();
    expect(channel).toBeDefined();
    const progressEvents = [
      { saveId: 1, contextToken: "save-token-1", phase: "scan" },
      {
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      },
    ];
    for (const ev of progressEvents) {
      act(() => {
        emitLoadDataProgress(ev);
      });
    }

    expect(result.current.progress).toMatchObject({
      phase: "preparing",
      completed: 5,
    });

    act(() => {
      emitLoadDataProgress({
        saveId: 999,
        contextToken: "other",
        phase: "scoring",
        completed: 1,
        total: 10,
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "preparing" });

    await act(async () => {
      resolveBusyLoadDataRequest();
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
  });

  it("failure preserves caches and invalidates only bridge", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    const snapshotCurrentKey = ["snapshot", "current"] as const;
    const snapshotSavesKey = ["snapshot", "saves"] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    queryClient.setQueryData(snapshotCurrentKey, { id: 1 });
    queryClient.setQueryData(snapshotSavesKey, [{ id: 1 }]);

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("ingestFailed");

    await act(async () => {
      result.current.mutate(null);
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    expect(queryClient.getQueryData(squadPage)).toBeDefined();
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(invalidateHistory).not.toHaveBeenCalled();
    expect(result.current.progress).toBeNull();
    // Bridge must be invalidated even on failure (same-feature truthful owner)
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(snapshotCurrentKey)?.isInvalidated).toBe(
      false,
    );
    expect(queryClient.getQueryState(snapshotSavesKey)?.isInvalidated).toBe(
      false,
    );
  });

  it("context-matching effective-current success clears exact roots only after result and invalidates current owners and bridge", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });

    const order: string[] = [];
    const clearExactRoots = vi.fn(async () => {
      order.push("clear");
      queryClient.removeQueries({ queryKey: ["search", "players"] });
      queryClient.removeQueries({ queryKey: ["planner", "squad", "players"] });
    });
    const invalidateCurrent = vi.fn(() => {
      order.push("invalidate");
    });
    const invalidateHistory = vi.fn(() => {
      order.push("invalidateHistory");
    });
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("success");

    await act(async () => {
      result.current.mutate(null);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(clearExactRoots).toHaveBeenCalledOnce();
    expect(invalidateCurrent).toHaveBeenCalledOnce();
    expect(invalidateHistory).not.toHaveBeenCalled();
    expect(order).toEqual(["clear", "invalidate"]);
    expect(queryClient.getQueryData(searchPage)).toBeUndefined();
    expect(queryClient.getQueryData(squadPage)).toBeUndefined();
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
  });

  it("historical non-winner does not clear current roots and only refreshes exact history and bridge", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    const historyKey = ["snapshot", "history", 1] as const;
    const currentKey = ["snapshot", "current"] as const;
    const savesKey = ["snapshot", "saves"] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    queryClient.setQueryData(currentKey, { id: 99 });
    queryClient.setQueryData(savesKey, [{ id: 1 }]);
    queryClient.setQueryData(historyKey, [{ id: 1 }]);
    queryClient.setQueryData(bridgeKey, { status: "ready" });

    await resolveLoadDataIpcMock();
    setLoadDataIpcMockMode("historicalSuccess");

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    // Use real queryClient invalidation for history to verify exact key
    const historySpy = vi.fn((saveId: number) => {
      void queryClient.invalidateQueries({
        queryKey: ["snapshot", "history", saveId] as const,
      });
    });

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: historySpy,
        }),
      { wrapper: wrapper(queryClient) },
    );

    await act(async () => {
      result.current.mutate(null);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data?.storedSnapshot.contextToken).not.toBe(
      result.current.data?.effectiveSnapshot.contextToken,
    );
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(historySpy).toHaveBeenCalledOnce();
    expect(historySpy).toHaveBeenCalledWith(1);
    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    expect(queryClient.getQueryData(squadPage)).toBeDefined();
    expect(queryClient.getQueryState(historyKey)?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(currentKey)?.isInvalidated).not.toBe(true);
    expect(queryClient.getQueryState(savesKey)?.isInvalidated).not.toBe(true);
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
  });

  it("save switch uses ref: late progress and stale success hide progress and are marked terminal", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();

    let activeSaveContext: ActiveSaveContext | null = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      {
        initialProps: { ctx: activeSaveContext },
        wrapper: wrapper(queryClient),
      },
    );

    setLoadDataIpcMockMode("busy");

    act(() => {
      result.current.mutate(null);
    });

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    activeSaveContext = { id: 2, contextToken: "save-token-2" };
    rerender({ ctx: activeSaveContext });

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      });
    });
    // Switching context hides the old phase immediately; late progress stays hidden.
    expect(result.current.progress).toBeNull();

    await act(async () => {
      resolveBusyLoadDataRequest();
    });

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    // Effective-current settled while on B: exact history for captured A refreshed, not current roots.
    expect(invalidateHistory).toHaveBeenCalledWith(1);
    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    // Stale settlement must hide progress and not expose data, and present as idle.
    expect(result.current.progress).toBeNull();
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.isError).toBe(false);
    expect(result.current.isIdle).toBe(true);
    expect(result.current.isPending).toBe(false);
    expect(result.current.status).toBe("idle");
    expect(result.current.isCommandPending).toBe(false);
    expect(result.current.data).toBeUndefined();
    const bridgeKey2 = ["memory-read", "bridge-status", "status"] as const;
    expect(queryClient.getQueryState(bridgeKey2)?.isInvalidated).toBe(true);

    // Late channel event after stale terminal must not revive progress
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "saving",
        completed: 10,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();

    // Return to old save: settled generation must not become live again
    activeSaveContext = { id: 1, contextToken: "save-token-1" };
    rerender({ ctx: activeSaveContext });
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "finalizing",
        completed: 1,
        total: 1,
      });
    });
    expect(result.current.progress).toBeNull();
    expect(result.current.isSuccess).toBe(false);
  });

  it("A→B→A before settlement permanently stales success and late progress never revives", async () => {
    const queryClient = createQueryClient();
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    let activeSaveContext: ActiveSaveContext | null = {
      id: 1,
      contextToken: "save-token-1",
    };
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();

    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      {
        initialProps: { ctx: activeSaveContext },
        wrapper: wrapper(queryClient),
      },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    // A→B
    activeSaveContext = { id: 2, contextToken: "save-token-2" };
    rerender({ ctx: activeSaveContext });
    act(() => {
      emitLoadDataProgress({
        saveId: 2,
        contextToken: "save-token-2",
        phase: "scan",
      });
    });
    // Old progress is hidden and the new context has no Load Data request.
    expect(result.current.progress).toBeNull();

    // B→A (back to original) before settlement
    activeSaveContext = { id: 1, contextToken: "save-token-1" };
    rerender({ ctx: activeSaveContext });
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "preparing",
        completed: 5,
        total: 10,
      });
    });
    // Revision changed A(0)→B(1)→A(2), so the captured phase stays hidden.
    expect(result.current.progress).toBeNull();

    await act(async () => {
      resolveBusyLoadDataRequest();
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    // Effective-current publication settled while back on active A: DB truth reconciled
    // (exact roots cleared + current owners invalidated) but stale presentation
    // keeps outcome hidden.
    expect(clearExactRoots).toHaveBeenCalledTimes(1);
    expect(invalidateCurrent).toHaveBeenCalledTimes(1);
    expect(invalidateHistory).not.toHaveBeenCalled();
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.data).toBeUndefined();
    expect(result.current.status).toBe("idle");
    expect(result.current.isIdle).toBe(true);
    expect(result.current.isPending).toBe(false);
    expect(result.current.progress).toBeNull();

    // Late progress after stale settlement must not revive
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "saving",
        completed: 10,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();
    expect(result.current.isSuccess).toBe(false);
  });

  it("A→B→A before settlement permanently stales real error and no outcome revival", async () => {
    const queryClient = createQueryClient();
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    let activeSaveContext: ActiveSaveContext | null = {
      id: 1,
      contextToken: "save-token-1",
    };
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();

    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      {
        initialProps: { ctx: activeSaveContext },
        wrapper: wrapper(queryClient),
      },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    activeSaveContext = { id: 2, contextToken: "save-token-2" };
    rerender({ ctx: activeSaveContext });
    activeSaveContext = { id: 1, contextToken: "save-token-1" };
    rerender({ ctx: activeSaveContext });

    await act(async () => {
      rejectBusyLoadDataRequest({
        phase: "ingest",
        message: "dump validation failed",
      });
      await new Promise((r) => setTimeout(r, 0));
    });

    // Stale error must not be exposed
    expect(result.current.isError).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.progress).toBeNull();
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(invalidateHistory).not.toHaveBeenCalled();
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 5,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();
  });

  it("stale error terminal hides progress and late events cannot revive", async () => {
    const queryClient = createQueryClient();
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    let activeSaveContext: ActiveSaveContext | null = {
      id: 1,
      contextToken: "save-token-1",
    };
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      {
        initialProps: { ctx: activeSaveContext },
        wrapper: wrapper(queryClient),
      },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });
    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    activeSaveContext = { id: 2, contextToken: "save-token-2" };
    rerender({ ctx: activeSaveContext });

    await act(async () => {
      rejectBusyLoadDataRequest({
        phase: "ingest",
        message: "dump validation failed",
      });
      await new Promise((r) => setTimeout(r, 0));
    });

    // Stale error must hide
    expect(result.current.isError).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.progress).toBeNull();
    expect(clearExactRoots).not.toHaveBeenCalled();

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 5,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();
  });

  it("late channel events after success do not overwrite current progress", async () => {
    const queryClient = createQueryClient();
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");

    act(() => {
      result.current.mutate(null);
    });

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    await act(async () => {
      resolveBusyLoadDataRequest();
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.progress).toBeNull();

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scoring",
        completed: 5,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();
  });

  it("does not compare only numeric IDs; uses context token for effective-current check", async () => {
    const queryClient = createQueryClient();
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();
    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    await act(async () => {
      resolveBusyLoadDataRequest({
        requestId: "req-mock",
        playersFound: 3,
        scanTruncated: false,
        maxAccepted: null,
        storedSnapshot: {
          id: 1,
          contextToken: "snapshot-token-1",
          saveId: 1,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        effectiveSnapshot: {
          id: 1,
          contextToken: "snapshot-token-1-replacement",
          saveId: 1,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        timings: {
          scanMs: 1200,
          prepareMs: 300,
          scoringMs: 400,
          saveMs: 200,
          finalizeMs: 200,
          totalMs: 2100,
          ingestMs: 400,
        },
      } as never);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateHistory).toHaveBeenCalledWith(1);
  });

  it("active context switch while awaiting clearExactRoots keeps newly established roots and does not invalidate", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(searchPage, { players: ["old"] });
    queryClient.setQueryData(squadPage, { players: ["old"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });

    const activeSaveContextInitial: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };
    let activeSaveContext: ActiveSaveContext | null = activeSaveContextInitial;
    // Guard-aware clear with realistic async cancel delay
    const clearExactRoots = vi.fn(async (guard?: () => boolean) => {
      await new Promise<void>((r) => setTimeout(r, 100));
      if (guard && !guard()) return;
      queryClient.removeQueries({ queryKey: ["search", "players"] });
      queryClient.removeQueries({ queryKey: ["planner", "squad", "players"] });
    });
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn();

    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      {
        initialProps: { ctx: activeSaveContext },
        wrapper: wrapper(queryClient),
      },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    // Start settlement (will await clear)
    act(() => {
      resolveBusyLoadDataRequest();
    });

    await waitFor(() => expect(clearExactRoots).toHaveBeenCalled());

    // Switch context and establish new roots before guard is evaluated (within clear delay)
    act(() => {
      activeSaveContext = { id: 2, contextToken: "save-token-2" };
      rerender({ ctx: activeSaveContext });
    });
    queryClient.setQueryData(searchPage, { players: ["new"] });
    queryClient.setQueryData(squadPage, { players: ["new"] });

    // Wait for clear to finish (100ms delay)
    await act(async () => {
      await new Promise((r) => setTimeout(r, 120));
    });

    // Stale settlement must not have removed newly established roots
    expect(queryClient.getQueryData(searchPage)).toEqual({ players: ["new"] });
    expect(queryClient.getQueryData(squadPage)).toEqual({ players: ["new"] });
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.progress).toBeNull();
    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
  });

  it("busy mock retains invocation-time save identity after active-save switch", async () => {
    const queryClient = createQueryClient();
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    queryClient.setQueryData(bridgeKey, { status: "ready" });

    // Capture identity at invocation: save 1, then switch to save 2 before settlement
    const activeSaveContext = { id: 1, contextToken: "save-token-1" } as const;
    let currentCtx: ActiveSaveContext | null = { ...activeSaveContext };
    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn((saveId: number) => {
      // history invalidation should use captured id 1, not switched id 2
      expect(saveId).toBe(1);
    });

    const { result, rerender } = renderHook(
      ({ ctx }: { ctx: ActiveSaveContext | null }) =>
        useLoadData({
          activeSaveContext: ctx,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { initialProps: { ctx: currentCtx }, wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    // Switch active save before busy resolves
    currentCtx = { id: 2, contextToken: "save-token-2" };
    rerender({ ctx: currentCtx });

    // Intercept resolve to capture result identity
    await act(async () => {
      // Use custom result to prove captured save is used, not current
      resolveBusyLoadDataRequest({
        requestId: "req-mock",
        playersFound: 3,
        scanTruncated: false,
        maxAccepted: null,
        storedSnapshot: {
          id: 99,
          contextToken: "snapshot-token-99",
          saveId: 1,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        effectiveSnapshot: {
          id: 99,
          contextToken: "snapshot-token-100",
          saveId: 1,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        timings: {
          scanMs: 1200,
          prepareMs: 300,
          scoringMs: 400,
          saveMs: 200,
          finalizeMs: 200,
          totalMs: 2100,
          ingestMs: 400,
        },
      } as never);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    // Result should be stale (presentationLive false) so not exposed, but history handler saw captured id 1
    expect(invalidateHistory).toHaveBeenCalledTimes(1);
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.data).toBeUndefined();
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
  });

  it("mismatched effective-current saveId B hides outcome, preserves A/B roots, invalidates B history and bridge, no clear/broad invalidation", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    const historyBKey = ["snapshot", "history", 2] as const;
    const historyAKey = ["snapshot", "history", 1] as const;
    const currentKey = ["snapshot", "current"] as const;
    const savesKey = ["snapshot", "saves"] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    queryClient.setQueryData(historyAKey, [{ id: 1 }]);
    queryClient.setQueryData(historyBKey, [{ id: 2 }]);
    queryClient.setQueryData(currentKey, { id: 99 });
    queryClient.setQueryData(savesKey, [{ id: 1 }, { id: 2 }]);

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn((saveId: number) => {
      void queryClient.invalidateQueries({
        queryKey: ["snapshot", "history", saveId] as const,
      });
    });

    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    act(() => {
      emitLoadDataProgress({
        saveId: 1,
        contextToken: "save-token-1",
        phase: "scan",
      });
    });
    expect(result.current.progress).toMatchObject({ phase: "scan" });

    await act(async () => {
      resolveBusyLoadDataRequest({
        requestId: "req-mock",
        playersFound: 3,
        scanTruncated: false,
        maxAccepted: null,
        storedSnapshot: {
          id: 10,
          contextToken: "snapshot-token-10",
          saveId: 2,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        effectiveSnapshot: {
          id: 10,
          contextToken: "snapshot-token-10",
          saveId: 2,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        timings: {
          scanMs: 1200,
          prepareMs: 300,
          scoringMs: 400,
          saveMs: 200,
          finalizeMs: 200,
          totalMs: 2100,
          ingestMs: 400,
        },
      } as never);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
    expect(invalidateHistory).toHaveBeenCalledWith(2);
    expect(invalidateHistory).not.toHaveBeenCalledWith(1);
    expect(queryClient.getQueryState(historyBKey)?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(historyAKey)?.isInvalidated).not.toBe(
      true,
    );
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    expect(queryClient.getQueryData(squadPage)).toBeDefined();
    expect(queryClient.getQueryState(currentKey)?.isInvalidated).not.toBe(true);
    expect(queryClient.getQueryState(savesKey)?.isInvalidated).not.toBe(true);
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.data).toBeUndefined();
    expect(result.current.isIdle).toBe(true);
    expect(result.current.status).toBe("idle");
    expect(result.current.progress).toBeNull();
    expect(result.current.isPending).toBe(false);
    expect(result.current.isCommandPending).toBe(false);
    expect(result.current.isError).toBe(false);
    expect(result.current.error).toBeNull();

    act(() => {
      emitLoadDataProgress({
        saveId: 2,
        contextToken: "save-token-2",
        phase: "scoring",
        completed: 5,
        total: 10,
      });
    });
    expect(result.current.progress).toBeNull();
    expect(result.current.isSuccess).toBe(false);
  });

  it("mixed stored/effective saveId corruption hides outcome and invalidates both distinct histories without duplicate", async () => {
    const queryClient = createQueryClient();
    const searchPage = ["search", "players", { offset: 0, limit: 50 }] as const;
    const squadPage = [
      "planner",
      "squad",
      "players",
      { offset: 0, limit: 50 },
    ] as const;
    const bridgeKey = ["memory-read", "bridge-status", "status"] as const;
    const historyAKey = ["snapshot", "history", 1] as const;
    const historyBKey = ["snapshot", "history", 2] as const;
    queryClient.setQueryData(searchPage, { players: ["search"] });
    queryClient.setQueryData(squadPage, { players: ["squad"] });
    queryClient.setQueryData(bridgeKey, { status: "ready" });
    queryClient.setQueryData(historyAKey, [{ id: 1 }]);
    queryClient.setQueryData(historyBKey, [{ id: 2 }]);

    const clearExactRoots = vi.fn(async () => undefined);
    const invalidateCurrent = vi.fn();
    const invalidateHistory = vi.fn((saveId: number) => {
      void queryClient.invalidateQueries({
        queryKey: ["snapshot", "history", saveId] as const,
      });
    });

    const activeSaveContext: ActiveSaveContext = {
      id: 1,
      contextToken: "save-token-1",
    };

    const { result } = renderHook(
      () =>
        useLoadData({
          activeSaveContext,
          clearExactRoots,
          invalidateCurrentOwners: invalidateCurrent,
          invalidateHistoryOwners: invalidateHistory,
        }),
      { wrapper: wrapper(queryClient) },
    );

    setLoadDataIpcMockMode("busy");
    act(() => {
      result.current.mutate(null);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    await act(async () => {
      resolveBusyLoadDataRequest({
        requestId: "req-mock",
        playersFound: 3,
        scanTruncated: false,
        maxAccepted: null,
        storedSnapshot: {
          id: 10,
          contextToken: "snapshot-token-10",
          saveId: 1,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        effectiveSnapshot: {
          id: 10,
          contextToken: "snapshot-token-10",
          saveId: 2,
          schemaVersion: 6,
          generatedAtUtc: "2026-07-28T15:00:00.000Z",
          gameVersion: "26.0.0",
          supportedGameVersion: "26.0.0",
          bridgeVersion: "0.1.0",
          protocolVersion: 1,
          gameDate: "2026-07-01",
          gameDateSource: "inGame",
          scanTruncated: false,
          maxAccepted: null,
          playerCount: 3,
          loadedAtUtc: "2026-07-28T15:05:00.000Z",
        },
        timings: {
          scanMs: 1200,
          prepareMs: 300,
          scoringMs: 400,
          saveMs: 200,
          finalizeMs: 200,
          totalMs: 2100,
          ingestMs: 400,
        },
      } as never);
    });
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0));
    });

    expect(queryClient.getQueryState(bridgeKey)?.isInvalidated).toBe(true);
    expect(invalidateHistory).toHaveBeenCalledTimes(2);
    expect(invalidateHistory).toHaveBeenCalledWith(1);
    expect(invalidateHistory).toHaveBeenCalledWith(2);
    expect(queryClient.getQueryState(historyAKey)?.isInvalidated).toBe(true);
    expect(queryClient.getQueryState(historyBKey)?.isInvalidated).toBe(true);
    expect(clearExactRoots).not.toHaveBeenCalled();
    expect(invalidateCurrent).not.toHaveBeenCalled();
    expect(queryClient.getQueryData(searchPage)).toBeDefined();
    expect(queryClient.getQueryData(squadPage)).toBeDefined();
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.data).toBeUndefined();
    expect(result.current.isIdle).toBe(true);
    expect(result.current.status).toBe("idle");
    expect(result.current.progress).toBeNull();
  });
});
