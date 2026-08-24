import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "@tanstack/react-router";
import { ArrowLeft, ArrowRight, RefreshCw } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { fieldClasses } from "@/components/ui/field/field-styles";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { LoadDataOutcome } from "@/features/memory-read/components/load-data-outcome";
import { useLoadData } from "@/features/memory-read/hooks/use-load-data";
import { useLoadDataPreferences } from "@/features/memory-read/stores/use-load-data-preferences";
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
import { cn } from "@/utils/cn";

export function AppTopBar() {
  const queryClient = useQueryClient();
  const router = useRouter();
  const [historyIndexes, setHistoryIndexes] = useState(() => {
    const index = router.history.location.state.__TSR_index;
    return { current: index, max: index };
  });
  const { data: saves } = useQuery(savesQueryOptions);
  const activeSave = saves?.find((save) => save.isActive);
  const capCheckboxId = useId();
  const capLimitId = useId();

  const playerCapEnabled = useLoadDataPreferences(
    (state) => state.playerCapEnabled,
  );
  const playerCap = useLoadDataPreferences((state) => state.playerCap);
  const setPlayerCapEnabled = useLoadDataPreferences(
    (state) => state.setPlayerCapEnabled,
  );
  const setPlayerCap = useLoadDataPreferences((state) => state.setPlayerCap);

  const load = useLoadData({
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
      void queryClient.invalidateQueries({ queryKey: searchKeys.all });
      void queryClient.invalidateQueries({ queryKey: playerKeys.all });
      void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
      void queryClient.invalidateQueries({ queryKey: clubDnaKeys.all });
      void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
      void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
      void queryClient.invalidateQueries({ queryKey: academyKeys.all });
      void queryClient.invalidateQueries({ queryKey: staffKeys.all });
    },
  });

  // An outcome for a save the user has since left describes data they are no
  // longer looking at. One rule covers both banners, because a failure is as
  // misleading as a stale player count.
  const [loadedSave, setLoadedSave] = useState<
    { id: number; contextToken: string } | undefined
  >();
  const stale =
    !loadedSave ||
    loadedSave.id !== activeSave?.id ||
    loadedSave.contextToken !== activeSave?.contextToken;

  const capValid = Number.isInteger(playerCap) && playerCap > 0;

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
        <div className="flex items-center gap-2">
          <label
            className="flex cursor-pointer items-center gap-1.5 text-label-md text-on-surface-variant"
            htmlFor={capCheckboxId}
          >
            <input
              checked={playerCapEnabled}
              className="size-3.5 accent-primary"
              id={capCheckboxId}
              type="checkbox"
              onChange={(event) => {
                setPlayerCapEnabled(event.target.checked);
              }}
            />
            Cap players
          </label>
          {playerCapEnabled ? (
            <input
              aria-label="Player limit"
              className={cn(fieldClasses, "w-20")}
              id={capLimitId}
              min={1}
              step={1}
              type="number"
              value={playerCap}
              onChange={(event) => {
                const next = Number(event.target.value);
                setPlayerCap(Number.isFinite(next) ? next : 0);
              }}
            />
          ) : null}
          <Button
            size="lg"
            icon={RefreshCw}
            loading={load.isPending}
            loadingLabel="Scanning…"
            disabled={playerCapEnabled && !capValid}
            onClick={() => {
              setLoadedSave(
                activeSave
                  ? {
                      id: activeSave.id,
                      contextToken: activeSave.contextToken,
                    }
                  : undefined,
              );
              load.mutate(playerCapEnabled ? playerCap : null);
            }}
          >
            Load Data
          </Button>
        </div>
      </div>
      <LoadDataOutcome
        error={stale ? null : load.error}
        result={stale ? undefined : load.data}
        onDismiss={load.reset}
      />
    </header>
  );
}
