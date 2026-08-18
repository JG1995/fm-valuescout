import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import {
  CircleCheck,
  CircleDashed,
  CircleX,
  Radar,
  RefreshCw,
} from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import {
  formatAbsoluteUtc,
  formatMissable,
  formatRelativeAge,
} from "@/utils/format";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";

/** The plugin's protocol states (`bridge/Protocol/BridgeProtocol.cs`). An
 *  unknown state stays neutral rather than claiming health it cannot vouch for. */
const stateChip = {
  idle: { label: "ready", tone: "success", icon: CircleCheck },
  ready: { label: "ready", tone: "success", icon: CircleCheck },
  scanning: { label: "scanning", tone: "info", icon: Radar },
  failed: { label: "failed", tone: "error", icon: CircleX },
} as const;

export function BridgeStatusPanel() {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(bridgeStatusQueryOptions);

  const chip = stateChip[data.state as keyof typeof stateChip] ?? {
    label: data.state,
    tone: "neutral",
    icon: CircleDashed,
  };
  const modulesDetected =
    data.gamePluginModulePresent && data.gameAssemblyModulePresent;

  return (
    <Panel
      title="Bridge"
      actions={
        <Button
          size="icon"
          variant="ghost"
          icon={RefreshCw}
          aria-label="Refresh bridge status"
          onClick={() =>
            queryClient.invalidateQueries({
              queryKey: bridgeStatusQueryOptions.queryKey,
            })
          }
        />
      }
    >
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip tone={chip.tone} icon={chip.icon}>
          {`Bridge: ${chip.label}`}
        </StatusChip>
        <StatusChip
          tone={modulesDetected ? "success" : "warning"}
          icon={modulesDetected ? CircleCheck : CircleDashed}
        >
          {`FM modules: ${modulesDetected ? "detected" : "not fully loaded"}`}
        </StatusChip>
      </div>
      <p className="mt-3 text-body-sm text-on-surface-variant">
        Plugin version: {formatMissable(data.pluginVersion)} · protocol{" "}
        {data.protocolVersion} · updated{" "}
        <span title={formatAbsoluteUtc(data.updatedAtUtc)}>
          {formatRelativeAge(data.updatedAtUtc)}
        </span>
      </p>
      <p className="mt-2 max-w-prose text-body-sm text-on-surface-variant">
        Keep Football Manager 26 running with the bridge plugin installed. Use
        the plugin install controls in this section or see{" "}
        <code className="font-mono text-mono-sm">bridge/README.md</code> for
        manual steps.
      </p>
    </Panel>
  );
}
