import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";
import { loadData } from "../api/load-data";
import type { LoadDataProgress, LoadDataResult } from "../types/load-data";

export type ActiveSaveContext = { id: number; contextToken: string };

type UseLoadDataOptions = {
  activeSaveContext: ActiveSaveContext | null;
  clearExactRoots: (guard?: () => boolean) => Promise<void>;
  invalidateCurrentOwners: () => void;
  invalidateHistoryOwners: (captured: ActiveSaveContext) => void;
};

type MutationContext = {
  generation: number;
  revision: number;
  captured: ActiveSaveContext | null;
};

function sameContext(
  a: ActiveSaveContext | null,
  b: ActiveSaveContext | null,
): boolean {
  if (a === null && b === null) return true;
  if (!a || !b) return false;
  return a.id === b.id && a.contextToken === b.contextToken;
}

export function useLoadData(options: UseLoadDataOptions) {
  const queryClient = useQueryClient();

  // Context revision increments on every id/token transition.
  const contextRevisionRef = useRef(0);
  const prevContextRef = useRef<ActiveSaveContext | null>(
    options.activeSaveContext,
  );
  const activeSaveContextRef = useRef(options.activeSaveContext);

  const currentCtx = options.activeSaveContext;
  const prevCtx = prevContextRef.current;
  if (!sameContext(prevCtx, currentCtx)) {
    contextRevisionRef.current += 1;
    prevContextRef.current = currentCtx;
  }
  activeSaveContextRef.current = currentCtx;

  const [progress, setProgress] = useState<LoadDataProgress | null>(null);
  const generationRef = useRef(0);
  const activeRequestRef = useRef<MutationContext | null>(null);
  const settledGenerationsRef = useRef<Set<number>>(new Set());

  const isCurrent = (
    captured: ActiveSaveContext | null,
    revision: number,
  ): boolean => {
    if (revision !== contextRevisionRef.current) return false;
    return sameContext(captured, activeSaveContextRef.current);
  };

  const invalidateBridge = () => {
    void queryClient.invalidateQueries({
      queryKey: bridgeStatusQueryOptions.queryKey,
    });
  };

  const mutation = useMutation<
    LoadDataResult,
    Error,
    number | null,
    MutationContext
  >({
    onMutate: () => {
      const generation = ++generationRef.current;
      const revision = contextRevisionRef.current;
      const captured = options.activeSaveContext;
      const ctx: MutationContext = { generation, revision, captured };
      activeRequestRef.current = ctx;
      settledGenerationsRef.current.delete(generation);
      setProgress(null);
      return ctx;
    },
    mutationFn: async (maxAccepted: number | null) => {
      const generation = generationRef.current;
      const ctx = activeRequestRef.current;
      const captured = ctx?.captured ?? null;
      const revision = ctx?.revision ?? contextRevisionRef.current;

      const onProgress = (event: LoadDataProgress) => {
        if (generation !== generationRef.current) return;
        if (settledGenerationsRef.current.has(generation)) return;
        if (!captured) return;
        if (
          event.saveId !== captured.id ||
          event.contextToken !== captured.contextToken
        )
          return;
        if (!isCurrent(captured, revision)) return;
        setProgress(event);
      };

      return loadData(maxAccepted, onProgress);
    },
    onSuccess: async (data, _variables, context) => {
      const ctx =
        (context as MutationContext | undefined) ?? activeRequestRef.current;
      const generation = ctx?.generation ?? generationRef.current;
      const captured = ctx?.captured ?? null;

      // Bridge status changes for every terminal settlement.
      invalidateBridge();

      if (settledGenerationsRef.current.has(generation)) return;
      settledGenerationsRef.current.add(generation);

      const isEffectiveCurrent =
        data.storedSnapshot.contextToken ===
        data.effectiveSnapshot.contextToken;

      // Historical non-winner: always reconcile exact history regardless of presentation staleness.
      if (!isEffectiveCurrent) {
        if (!captured) {
          if (generation === generationRef.current) setProgress(null);
          return;
        }
        options.invalidateHistoryOwners(captured);
        if (generation === generationRef.current) setProgress(null);
        return;
      }

      // Effective-current path
      if (!captured) {
        if (generation === generationRef.current) setProgress(null);
        return;
      }

      const exactContextMatchAtSettlement = sameContext(
        captured,
        activeSaveContextRef.current,
      );

      if (!exactContextMatchAtSettlement) {
        // Settled while another save is active: history only, do not clear current roots.
        options.invalidateHistoryOwners(captured);
        if (generation === generationRef.current) setProgress(null);
        return;
      }

      // Effective-current replacement settling on currently active matching save:
      // reconcile even if presentation revision is stale (A→B→A). Capture reconciliation revision
      // and guard with generation + reconciliation revision + exact context.
      const reconciliationRevision = contextRevisionRef.current;

      const shouldContinue = () =>
        generationRef.current === generation &&
        contextRevisionRef.current === reconciliationRevision &&
        sameContext(captured, activeSaveContextRef.current);

      await options.clearExactRoots(shouldContinue);

      if (!shouldContinue()) {
        if (generation === generationRef.current) setProgress(null);
        return;
      }
      options.invalidateCurrentOwners();

      if (generation === generationRef.current) setProgress(null);
    },
    onError: (_error, _variables, context) => {
      const ctx =
        (context as MutationContext | undefined) ?? activeRequestRef.current;
      const generation = ctx?.generation ?? generationRef.current;
      const revision = ctx?.revision ?? contextRevisionRef.current;
      const captured = ctx?.captured ?? null;

      invalidateBridge();

      if (settledGenerationsRef.current.has(generation)) return;
      settledGenerationsRef.current.add(generation);

      if (generation === generationRef.current) setProgress(null);

      // Error never clears roots or invalidates snapshots; bridge already handled.
      void captured;
      void revision;
    },
  });

  // Presentation liveness gates UI state only. Reconciliation uses exact current context + reconciliation revision.
  const activeReq = activeRequestRef.current;
  const presentationLive =
    activeReq !== null &&
    activeReq.generation === generationRef.current &&
    activeReq.revision === contextRevisionRef.current &&
    sameContext(activeReq.captured, activeSaveContextRef.current);

  const exposedData = presentationLive ? mutation.data : undefined;
  const exposedError = presentationLive ? mutation.error : null;
  const exposedIsSuccess = presentationLive ? mutation.isSuccess : false;
  const exposedIsError = presentationLive ? mutation.isError : false;
  const exposedIsIdle = presentationLive ? mutation.isIdle : true;
  const exposedIsPending = presentationLive ? mutation.isPending : false;
  const exposedIsPaused = presentationLive ? mutation.isPaused : false;
  const exposedStatus = presentationLive ? mutation.status : ("idle" as const);
  const exposedProgress = presentationLive ? progress : null;

  return {
    ...mutation,
    data: exposedData,
    error: exposedError,
    isSuccess: exposedIsSuccess,
    isError: exposedIsError,
    isIdle: exposedIsIdle,
    isPending: exposedIsPending,
    isPaused: exposedIsPaused,
    status: exposedStatus,
    progress: exposedProgress,
    isCommandPending: mutation.isPending,
  };
}
