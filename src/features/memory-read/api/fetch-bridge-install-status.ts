import { invokeCommand } from "@/lib/tauri-client";
import type { BridgeInstallStatus } from "../types/bridge-install";

export async function fetchBridgeInstallStatus(): Promise<BridgeInstallStatus> {
  return invokeCommand<BridgeInstallStatus>("get_bridge_install_status");
}
