import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotMetadata } from "../types/snapshot";

export function setSnapshotKept(
  snapshotId: number,
  contextToken: string,
  kept: boolean,
) {
  return invokeCommand<SnapshotMetadata>("set_snapshot_kept", {
    snapshotId,
    contextToken,
    kept,
  });
}
