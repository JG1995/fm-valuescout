import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotMetadata } from "../types/snapshot";

export function fetchSnapshotMetadata(saveId: number) {
  return invokeCommand<SnapshotMetadata[]>("list_snapshots", { saveId });
}
