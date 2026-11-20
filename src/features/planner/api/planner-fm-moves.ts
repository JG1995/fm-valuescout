import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerFmMoveResult, PlannerFmMoves } from "../types/fm-moves";
import { plannerKeys } from "./planner-keys";

/**
 * The moves that would put each placed player in the FM team his Planner squad
 * stands for. FM and the bridge change outside the app, so every read is fresh.
 */
export function plannerFmMovesQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: plannerKeys.fmMoves(activeSaveId),
    queryFn: () => invokeCommand<PlannerFmMoves>("get_planner_fm_moves"),
    staleTime: 0,
  });
}

/** Moves the chosen players in FM; Rust re-plans and ignores any other UID. */
export function applyPlannerFmMoves(playerUids: number[]) {
  return invokeCommand<PlannerFmMoveResult>("apply_planner_fm_moves", {
    playerUids,
  });
}
