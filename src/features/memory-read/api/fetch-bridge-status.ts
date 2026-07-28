import { invokeCommand } from "@/lib/tauri-client";
import type { BridgeStatus } from "../types/bridge-status";

export async function fetchBridgeStatus(): Promise<BridgeStatus> {
  return invokeCommand<BridgeStatus>("get_bridge_status");
}
