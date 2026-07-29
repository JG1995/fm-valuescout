import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
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

  const installPending = install.isPending;
  const removePending = remove.isPending;
  const actionsDisabled = installPending || removePending;

  return (
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <p className="font-medium text-on-background">Bridge plugin install</p>
      <p className="text-on-background/80">
        Plugin DLL:{" "}
        <strong className="text-on-background">
          {data.pluginPresent ? "installed" : "not installed"}
        </strong>
      </p>
      <p className="text-on-background/80">
        BepInEx:{" "}
        <strong className="text-on-background">
          {data.bepinexPresent ? "found" : "not found"}
        </strong>
      </p>
      <p className="text-sm text-on-background/60">
        Target: <code>{data.pluginsPath}</code>
      </p>
      <p className="text-sm text-on-background/60">
        Installs or updates <code>FmDataBridge.dll</code> in your Steam FM26{" "}
        <code>BepInEx/plugins</code> folder. Restart Football Manager after
        install so BepInEx loads the new DLL. Windows may prompt for permission
        or antivirus approval when writing into the game folder.
      </p>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          disabled={actionsDisabled || !data.bepinexPresent}
          onClick={() => install.mutate()}
        >
          {installPending
            ? "Installing…"
            : data.pluginPresent
              ? "Update plugin"
              : "Install plugin"}
        </Button>
        <Button
          type="button"
          variant="secondary"
          disabled={actionsDisabled || !data.pluginPresent}
          onClick={() => remove.mutate()}
        >
          {removePending ? "Removing…" : "Remove plugin"}
        </Button>
      </div>
      {install.isSuccess && (
        <p className="text-on-background/80">
          Plugin installed. Restart Football Manager 26 to load it.
        </p>
      )}
      {remove.isSuccess && (
        <p className="text-on-background/80">
          Plugin removed from <code>BepInEx/plugins</code>.
        </p>
      )}
      {install.isError && (
        <p className="text-on-background/80">
          Install failed.{" "}
          <span className="text-on-background">
            {installErrorCopy(install.error)}
          </span>
        </p>
      )}
      {remove.isError && (
        <p className="text-on-background/80">
          Remove failed.{" "}
          <span className="text-on-background">
            {installErrorCopy(remove.error)}
          </span>
        </p>
      )}
    </div>
  );
}
