import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";
import { loadData } from "../api/load-data";
import type { LoadDataResult } from "../types/load-data";
import { loadDataErrorCopy } from "./load-data-error";

function formatLoadOutcome(result: LoadDataResult): string {
  const count = result.snapshot.playerCount;
  const truncatedNote =
    result.snapshot.scanTruncated === true
      ? ` Partial ingest (capped at ${result.snapshot.maxAccepted ?? "unknown"} players).`
      : "";
  return `Loaded ${count} players into the database.${truncatedNote}`;
}

export function BridgeStatusPanel({
  activeSaveId,
  onLoadDataSettled,
}: {
  activeSaveId?: number;
  onLoadDataSettled?: () => void;
}) {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(bridgeStatusQueryOptions);

  const scan = useMutation({
    mutationFn: loadData,
    onSettled: () => {
      void queryClient.invalidateQueries({
        queryKey: bridgeStatusQueryOptions.queryKey,
      });
      onLoadDataSettled?.();
    },
  });

  const bridgeLabel =
    data.state === "idle"
      ? "ready"
      : data.state === "scanning"
        ? "scanning"
        : data.state;

  const loadDataError = scan.isError ? loadDataErrorCopy(scan.error) : null;

  return (
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <p className="text-on-background/80">
        Bridge: <strong className="text-on-background">{bridgeLabel}</strong>
      </p>
      <p className="text-on-background/80">
        Plugin version:{" "}
        <strong className="text-on-background">{data.pluginVersion}</strong>
      </p>
      <p className="text-on-background/80">
        FM modules:{" "}
        <strong className="text-on-background">
          {data.gamePluginModulePresent && data.gameAssemblyModulePresent
            ? "detected"
            : "not fully loaded"}
        </strong>
      </p>
      <p className="text-sm text-on-background/60">
        Keep Football Manager 26 running with the bridge plugin installed. Use
        the install section above or see <code>bridge/README.md</code> for
        manual steps.
      </p>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          disabled={scan.isPending}
          onClick={() => scan.mutate()}
        >
          {scan.isPending ? "Loading…" : "Load Data"}
        </Button>
        <Button
          type="button"
          variant="secondary"
          disabled={scan.isPending}
          onClick={() =>
            queryClient.invalidateQueries({
              queryKey: bridgeStatusQueryOptions.queryKey,
            })
          }
        >
          Refresh bridge status
        </Button>
      </div>
      {scan.isPending && (
        <p className="text-on-background/80">Scanning and ingesting FM data…</p>
      )}
      {scan.isSuccess && scan.data.snapshot.saveId === activeSaveId && (
        <p className="text-on-background/80">{formatLoadOutcome(scan.data)}</p>
      )}
      {loadDataError && (
        <div className="space-y-1 text-on-background/80">
          <p className="font-medium text-on-background">
            {loadDataError.title}
          </p>
          <p>{loadDataError.body}</p>
        </div>
      )}
    </div>
  );
}
