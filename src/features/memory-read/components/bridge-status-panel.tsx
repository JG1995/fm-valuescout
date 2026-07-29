import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";
import { requestPlayerDump } from "../api/request-player-dump";
import type { DumpRequestResult } from "../types/bridge-status";

function formatScanOutcome(result: DumpRequestResult): string {
  if (result.state === "ready") {
    const count = result.playersFound ?? 0;
    const truncatedNote =
      result.scanTruncated === true
        ? ` Partial dump (capped at ${result.maxAccepted ?? "unknown"} players).`
        : "";
    return result.dumpPresent
      ? `Dump ready (${count} players).${truncatedNote}`
      : `Scan finished but dump file is missing.`;
  }

  return result.error?.trim() ? `Scan failed: ${result.error}` : "Scan failed.";
}

export function BridgeStatusPanel() {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(bridgeStatusQueryOptions);

  const scan = useMutation({
    mutationFn: requestPlayerDump,
    onSettled: () => {
      void queryClient.invalidateQueries({
        queryKey: bridgeStatusQueryOptions.queryKey,
      });
    },
  });

  const bridgeLabel =
    data.state === "idle"
      ? "ready"
      : data.state === "scanning"
        ? "scanning"
        : data.state;

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
          {scan.isPending ? "Scanning…" : "Load Data"}
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
        <p className="text-on-background/80">Waiting for the FM bridge dump…</p>
      )}
      {scan.isSuccess && (
        <p className="text-on-background/80">{formatScanOutcome(scan.data)}</p>
      )}
      {scan.isError && (
        <p className="text-on-background/80">
          Could not request dump.{" "}
          <span className="text-on-background">{scan.error.message}</span>
        </p>
      )}
    </div>
  );
}
