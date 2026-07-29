import { useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { LoadDataOutcome } from "@/features/memory-read/components/load-data-outcome";
import { useLoadData } from "@/features/memory-read/hooks/use-load-data";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { ActiveSaveSelect } from "@/features/snapshot/components/active-save-select";
import { SnapshotFreshnessChip } from "@/features/snapshot/components/snapshot-freshness-chip";

export function AppTopBar() {
  const queryClient = useQueryClient();
  const { data: saves } = useQuery(savesQueryOptions);
  const activeSaveId = saves?.find((save) => save.isActive)?.id;

  const load = useLoadData({
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
    },
  });

  // An outcome for a save the user has since left describes data they are no
  // longer looking at. One rule covers both banners, because a failure is as
  // misleading as a stale player count.
  const [loadedSaveId, setLoadedSaveId] = useState<number>();
  const stale = loadedSaveId !== activeSaveId;

  return (
    <header
      data-testid="app-header"
      className="z-10 shrink-0 border-b border-outline-variant bg-surface-container"
    >
      <div className="flex h-header-height items-center gap-3 px-4">
        <ActiveSaveSelect />
        <SnapshotFreshnessChip />
        <div className="flex-1" />
        <Button
          size="lg"
          icon={RefreshCw}
          loading={load.isPending}
          loadingLabel="Scanning…"
          onClick={() => {
            setLoadedSaveId(activeSaveId);
            load.mutate();
          }}
        >
          Load Data
        </Button>
      </div>
      <LoadDataOutcome
        error={stale ? null : load.error}
        result={stale ? undefined : load.data}
      />
    </header>
  );
}
