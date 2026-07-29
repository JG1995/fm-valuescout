import { invokeCommand } from "@/lib/tauri-client";
import type { BridgeInstallStatus } from "../types/bridge-install";

export function installBridgePlugin() {
  return invokeCommand<BridgeInstallStatus>("install_bridge_plugin");
}
