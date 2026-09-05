import { invokeCommand } from "@/lib/tauri-client";
import type { SnapshotMetadata } from "../types/snapshot";

export type SnapshotGameDateUpdateResult = {
  snapshot: SnapshotMetadata;
  previousCurrentSnapshotId: number | null;
  currentSnapshotId: number | null;
};

export function updateSnapshotDate(
  snapshotId: number,
  contextToken: string,
  gameDate: string,
) {
  return invokeCommand<SnapshotGameDateUpdateResult>(
    "update_snapshot_game_date",
    {
      snapshotId,
      contextToken,
      gameDate,
    },
  );
}
