import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotMetadata } from "../types/snapshot";

export function renameSnapshot(
  snapshotId: number,
  contextToken: string,
  customName: string | null,
) {
  return invokeCommand<SnapshotMetadata>("rename_snapshot", {
    snapshotId,
    contextToken,
    customName,
  });
}
