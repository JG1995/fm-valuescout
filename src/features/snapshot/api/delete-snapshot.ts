import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotDeleteResult } from "../types/snapshot";

export function deleteSnapshot(snapshotId: number, contextToken: string) {
  return invokeCommand<SnapshotDeleteResult>("delete_snapshot", {
    snapshotId,
    contextToken,
  });
}
