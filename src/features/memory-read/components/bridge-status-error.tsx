import { CircleAlert } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { TauriCommandError } from "@/lib/tauri-client";

type BridgeStatusErrorProps = {
  error: Error;
  onRetry: () => void;
};

function bridgeStatusErrorCopy(error: Error) {
  if (error instanceof TauriCommandError) {
    switch (error.kind) {
      case "missing":
        return {
          title: "Bridge not detected",
          body: (
            <>
              Football Manager 26 must be running on Windows with the
              FmDataBridge plugin installed. Use <strong>Install plugin</strong>{" "}
              above, or copy <code>FmDataBridge.dll</code> into your FM26{" "}
              <code>BepInEx/plugins/</code> folder — see{" "}
              <code>bridge/README.md</code> for manual steps.
            </>
          ),
        };
      case "unsupportedPlatform":
        return {
          title: "Windows required",
          body: "FM26 memory read is only supported on Windows. Run the app on your Windows host to connect to the bridge.",
        };
      case "unsupportedVersion":
        return {
          title: "Bridge version mismatch",
          body: "The installed bridge plugin uses an unsupported protocol version. Update the plugin to match this app.",
        };
      default:
        break;
    }
  }

  return {
    title: "Could not read bridge status",
    body: error.message,
  };
}

export function BridgeStatusError({ error, onRetry }: BridgeStatusErrorProps) {
  const copy = bridgeStatusErrorCopy(error);

  return (
    <Panel>
      <EmptyState
        icon={CircleAlert}
        title={copy.title}
        action={
          <Button variant="secondary" onClick={onRetry}>
            Retry
          </Button>
        }
      >
        {copy.body}
      </EmptyState>
    </Panel>
  );
}
