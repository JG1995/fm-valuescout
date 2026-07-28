import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";

export function BridgeStatusPanel() {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(bridgeStatusQueryOptions);

  return (
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <p className="text-on-background/80">
        Bridge:{" "}
        <strong className="text-on-background">
          {data.state === "idle" ? "ready" : data.state}
        </strong>
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
        Keep Football Manager 26 running with the bridge plugin installed.
        Manual install steps are in <code>bridge/README.md</code>.
      </p>
      <Button
        type="button"
        variant="secondary"
        onClick={() =>
          queryClient.invalidateQueries({
            queryKey: bridgeStatusQueryOptions.queryKey,
          })
        }
      >
        Refresh bridge status
      </Button>
    </div>
  );
}
