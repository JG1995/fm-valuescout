import { TauriCommandError } from "@/lib/tauri-client";

export function loadDataErrorCopy(error: Error) {
  if (error instanceof TauriCommandError) {
    if (error.phase === "scan") {
      switch (error.kind) {
        case "unsupportedPlatform":
          return {
            title: "Windows required",
            body: "FM26 memory read is only supported on Windows. Run the app on your Windows host to load data.",
          };
        case "missing":
          return {
            title: "Bridge not detected",
            body: "Football Manager 26 must be running on Windows with the FmDataBridge plugin installed before Load Data can scan players.",
          };
        case "timeout":
          return {
            title: "Scan timed out",
            body: error.message,
          };
        default:
          return {
            title: "Scan failed",
            body: error.message,
          };
      }
    }

    if (error.phase === "ingest") {
      return {
        title: "Ingest failed",
        body: error.message,
      };
    }
  }

  return {
    title: "Could not load data",
    body: error.message,
  };
}
