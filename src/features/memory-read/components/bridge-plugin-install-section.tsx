import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { CircleCheck, CircleDashed, CircleX } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { TauriCommandError } from "@/lib/tauri-client";
import { bridgeInstallQueryOptions } from "../api/bridge-install-query-options";
import { installBridgePlugin } from "../api/install-bridge-plugin";
import { removeBridgePlugin } from "../api/remove-bridge-plugin";

function installErrorCopy(error: Error): string {
  if (error instanceof TauriCommandError) {
    switch (error.kind) {
      case "bepinexMissing":
        return "BepInEx is not installed in your FM26 folder. Install BepInEx 6 IL2CPP first, then try again.";
      case "writeFailed":
        return "Could not write the plugin DLL. Check folder permissions or antivirus settings.";
      case "sourceMissing":
        return "The bundled plugin DLL is missing from this app build.";
      case "unsupportedPlatform":
        return "Plugin install is only supported on Windows.";
      default:
        break;
    }
  }

  return error.message;
}

export function BridgePluginInstallSection() {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(bridgeInstallQueryOptions);

  const invalidateInstallStatus = () => {
    void queryClient.invalidateQueries({
      queryKey: bridgeInstallQueryOptions.queryKey,
    });
  };

  const install = useMutation({
    mutationFn: installBridgePlugin,
    onSuccess: () => {
      remove.reset();
    },
    onSettled: invalidateInstallStatus,
  });

  const remove = useMutation({
    mutationFn: removeBridgePlugin,
    onSuccess: () => {
      install.reset();
    },
    onSettled: invalidateInstallStatus,
  });

  const actionsDisabled = install.isPending || remove.isPending;

  return (
    <Panel title="Bridge plugin install">
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip
          tone={data.pluginPresent ? "success" : "neutral"}
          icon={data.pluginPresent ? CircleCheck : CircleDashed}
        >
          {`Plugin DLL: ${data.pluginPresent ? "installed" : "not installed"}`}
        </StatusChip>
        <StatusChip
          tone={data.bepinexPresent ? "success" : "error"}
          icon={data.bepinexPresent ? CircleCheck : CircleX}
        >
          {`BepInEx: ${data.bepinexPresent ? "found" : "not found"}`}
        </StatusChip>
      </div>

      <p className="mt-3 text-body-sm text-on-surface-variant">
        Target:{" "}
        <code className="font-mono text-mono-sm">{data.pluginsPath}</code>
      </p>
      <p className="mt-2 max-w-prose text-body-sm text-on-surface-variant">
        Installs or updates{" "}
        <code className="font-mono text-mono-sm">FmDataBridge.dll</code> in your
        Steam FM26{" "}
        <code className="font-mono text-mono-sm">BepInEx/plugins</code> folder.
        Restart Football Manager after install so BepInEx loads the new DLL.
        Windows may prompt for permission or antivirus approval when writing
        into the game folder.
      </p>

      <div className="mt-4 flex flex-wrap gap-2">
        <Button
          variant="secondary"
          disabled={actionsDisabled || !data.bepinexPresent}
          loading={install.isPending}
          loadingLabel="Installing…"
          onClick={() => install.mutate()}
        >
          {data.pluginPresent ? "Update plugin" : "Install plugin"}
        </Button>
        <Button
          variant="secondary"
          disabled={actionsDisabled || !data.pluginPresent}
          loading={remove.isPending}
          loadingLabel="Removing…"
          onClick={() => remove.mutate()}
        >
          Remove plugin
        </Button>
      </div>

      {install.isSuccess && (
        <p className="mt-3 text-body-sm text-on-surface">
          Plugin installed. Restart Football Manager 26 to load it.
        </p>
      )}
      {remove.isSuccess && (
        <p className="mt-3 text-body-sm text-on-surface">
          Plugin removed from{" "}
          <code className="font-mono text-mono-sm">BepInEx/plugins</code>.
        </p>
      )}
      {install.isError && (
        <p className="mt-3 text-body-sm text-error">
          Install failed. {installErrorCopy(install.error)}
        </p>
      )}
      {remove.isError && (
        <p className="mt-3 text-body-sm text-error">
          Remove failed. {installErrorCopy(remove.error)}
        </p>
      )}
    </Panel>
  );
}
