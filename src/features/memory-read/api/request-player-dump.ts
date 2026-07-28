import { invokeCommand } from "@/lib/tauri-client";
import type { DumpRequestResult } from "../types/bridge-status";

export function requestPlayerDump() {
  return invokeCommand<DumpRequestResult>("request_player_dump");
}
