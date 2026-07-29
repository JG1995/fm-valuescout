import { invokeCommand } from "@/lib/tauri-client";
import type { BridgeInstallStatus } from "../types/bridge-install";

export function removeBridgePlugin() {
  return invokeCommand<BridgeInstallStatus>("remove_bridge_plugin");
}
