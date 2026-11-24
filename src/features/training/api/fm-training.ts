import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { FmTraining, FmTrainingResult } from "../types/fm-training";

export const trainingKeys = {
  // The plan follows the Planner's assignments and tactic, so the Planner's
  // invalidation boundary also refreshes it.
  all: ["planner", "training"] as const,
  fmTraining: (activeSaveId: number) =>
    [...trainingKeys.all, "fm", activeSaveId] as const,
};

/**
 * The training Apply in FM would set for each player at the managed club. FM
 * and the bridge change outside the app, so every read is fresh.
 */
export function fmTrainingQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: trainingKeys.fmTraining(activeSaveId),
    queryFn: () => invokeCommand<FmTraining>("get_squad_fm_training"),
    staleTime: 0,
  });
}

/** Sets the chosen players' training in FM; Rust re-plans and ignores any other UID. */
export function applyFmTraining(playerUids: number[]) {
  return invokeCommand<FmTrainingResult>("apply_squad_fm_training", {
    playerUids,
  });
}
