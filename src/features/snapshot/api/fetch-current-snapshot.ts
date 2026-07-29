import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotSummary } from "../types/snapshot";

export function fetchCurrentSnapshot() {
  return invokeCommand<SnapshotSummary | null>("get_current_snapshot");
}
