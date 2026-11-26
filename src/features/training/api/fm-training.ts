import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  FmTraining,
  FmTrainingResult,
  TrainingBasis,
} from "../types/fm-training";

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

/** Sets whether the save's training follows Planner slots or development slots. */
export function setTrainingBasis(basis: TrainingBasis) {
  return invokeCommand<null>("set_training_basis", { basis });
}

/** Sets the oldest age at which the save's players still count as developing. */
export function setDevelopmentMaxAge(maxAge: number) {
  return invokeCommand<null>("set_development_max_age", { maxAge });
}

/**
 * Sets whether development slots are suggested only among the positions a
 * player can play, or in any position.
 */
export function setDevelopmentPositionFamiliarity(
  positionFamiliarity: boolean,
) {
  return invokeCommand<null>("set_development_position_familiarity", {
    positionFamiliarity,
  });
}

/**
 * Gives a player a development slot in place of the suggested one; a null
 * lane returns him to the suggestion.
 */
export function setDevelopmentSlot(playerUid: number, laneId: string | null) {
  return invokeCommand<null>("set_development_slot", { playerUid, laneId });
}
