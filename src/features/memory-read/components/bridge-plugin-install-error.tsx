import { Button } from "@/components/ui/button/button";
import { TauriCommandError } from "@/lib/tauri-client";

type BridgePluginInstallErrorProps = {
  error: Error;
  onRetry: () => void;
};

function bridgePluginInstallErrorCopy(error: Error) {
  if (error instanceof TauriCommandError) {
    switch (error.kind) {
      case "unsupportedPlatform":
        return {
          title: "Windows required for plugin install",
          body: "Installing the FM26 bridge plugin is only supported on Windows. Run the app on your Windows host to install or update the plugin.",
        };
      default:
        break;
    }
  }

  return {
    title: "Could not read plugin install status",
    body: error.message,
  };
}

export function BridgePluginInstallError({
  error,
  onRetry,
}: BridgePluginInstallErrorProps) {
  const copy = bridgePluginInstallErrorCopy(error);

  return (
    <div className="space-y-3 rounded-md border border-error/40 bg-error-container/20 p-4">
      <p className="font-medium text-on-background">{copy.title}</p>
      <p className="text-on-background/80">{copy.body}</p>
      <Button type="button" variant="secondary" onClick={onRetry}>
        Retry
      </Button>
    </div>
  );
}
