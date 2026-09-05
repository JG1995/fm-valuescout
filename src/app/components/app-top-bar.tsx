import { useIsMutating, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "@tanstack/react-router";
import { ArrowLeft, ArrowRight, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { clearPlayerResultContext } from "@/app/player-result-context";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import {
  LoadDataOutcome,
  loadDataPhaseLabels,
} from "@/features/memory-read/components/load-data-outcome";
import { useLoadData } from "@/features/memory-read/hooks/use-load-data";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { GlobalPlayerSearch } from "@/features/search/components/global-player-search";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { ActiveSaveSelect } from "@/features/snapshot/components/active-save-select";
import { SnapshotFreshnessChip } from "@/features/snapshot/components/snapshot-freshness-chip";
import { staffKeys } from "@/features/staff/api/staff-keys";

export function AppTopBar() {
  const queryClient = useQueryClient();
  const router = useRouter();
  const isContextMutating =
    useIsMutating({ mutationKey: playerResultContextMutationKey }) > 0;
  const [historyIndexes, setHistoryIndexes] = useState(() => {
    const index = router.history.location.state.__TSR_index;
    return { current: index, max: index };
  });
  const { data: saves } = useQuery(savesQueryOptions);
  const activeSave = saves?.find((save) => save.isActive);

  const clearResults = useCallback(
    (guard?: () => boolean) => clearPlayerResultContext(queryClient, guard),
    [queryClient],
  );
  const activeSaveContext = activeSave
    ? { id: activeSave.id, contextToken: activeSave.contextToken }
    : null;

  const invalidateCurrentOwners = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
    void queryClient.invalidateQueries({ queryKey: searchKeys.all });
    void queryClient.invalidateQueries({ queryKey: playerKeys.all });
    void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
    void queryClient.invalidateQueries({ queryKey: clubDnaKeys.all });
    void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
    void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    void queryClient.invalidateQueries({ queryKey: academyKeys.all });
    void queryClient.invalidateQueries({ queryKey: staffKeys.all });
  }, [queryClient]);
  const invalidateHistoryOwners = useCallback(
    (saveId: number) => {
      void queryClient.invalidateQueries({
        queryKey: snapshotKeys.history(saveId),
      });
    },
    [queryClient],
  );
  const load = useLoadData({
    activeSaveContext,
    clearExactRoots: clearResults,
    invalidateCurrentOwners,
    invalidateHistoryOwners,
  });

  const [loadedSave, setLoadedSave] = useState<
    { id: number; contextToken: string } | undefined
  >();
  const stale =
    isContextMutating ||
    !loadedSave ||
    loadedSave.id !== activeSave?.id ||
    loadedSave.contextToken !== activeSave?.contextToken;

  useEffect(
    () =>
      router.history.subscribe(({ action, location }) => {
        const index = location.state.__TSR_index;
        setHistoryIndexes((previous) => ({
          current: index,
          max: action.type === "PUSH" ? index : Math.max(previous.max, index),
        }));
      }),
    [router],
  );

  return (
    <header
      data-testid="app-header"
      className="z-10 shrink-0 border-b border-outline-variant bg-surface-container"
    >
      <div className="flex h-header-height items-center gap-3 px-4">
        <div className="flex shrink-0 items-center gap-1">
          <Button
            aria-label="Back"
            disabled={historyIndexes.current === 0}
            icon={ArrowLeft}
            size="icon"
            variant="ghost"
            onClick={() => router.history.back()}
          />
          <Button
            aria-label="Forward"
            disabled={historyIndexes.current >= historyIndexes.max}
            icon={ArrowRight}
            size="icon"
            variant="ghost"
            onClick={() => router.history.forward()}
          />
        </div>
        <GlobalPlayerSearch />
        <ActiveSaveSelect
          onBeforeContextChange={clearResults}
          onSwitched={() => {
            void queryClient.invalidateQueries({ queryKey: searchKeys.all });
            void queryClient.invalidateQueries({ queryKey: playerKeys.all });
            void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
            void queryClient.invalidateQueries({ queryKey: clubDnaKeys.all });
            void queryClient.invalidateQueries({
              queryKey: managedClubKeys.all,
            });
            void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
            void queryClient.resetQueries({ queryKey: academyKeys.all });
            void queryClient.invalidateQueries({ queryKey: staffKeys.all });
          }}
        />
        <SnapshotFreshnessChip />
        <Button
          size="lg"
          icon={RefreshCw}
          loading={load.isCommandPending}
          loadingLabel={
            isContextMutating && load.isCommandPending
              ? "Loading…"
              : load.isPending && load.progress
                ? loadDataPhaseLabels[load.progress.phase]
                : load.isPending
                  ? "Scanning…"
                  : load.isCommandPending
                    ? "Loading…"
                    : undefined
          }
          className="min-w-36"
          disabled={!activeSave || isContextMutating}
          onClick={() => {
            setLoadedSave(
              activeSave
                ? {
                    id: activeSave.id,
                    contextToken: activeSave.contextToken,
                  }
                : undefined,
            );
            load.mutate(null);
          }}
        >
          Load Data
        </Button>
      </div>
      <LoadDataOutcome
        error={stale ? null : load.error}
        result={stale ? undefined : load.data}
        progress={stale ? null : load.progress}
        onDismiss={load.reset}
      />
    </header>
  );
}
