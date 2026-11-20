import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  SquadFmTraining,
  SquadFmTrainingResult,
} from "../types/squad-fm-training";
import { squadKeys } from "./squad-keys";

/**
 * The training Apply in FM would set for each player at the managed club. FM
 * and the bridge change outside the app, so every read is fresh.
 */
export function squadFmTrainingQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: squadKeys.fmTraining(activeSaveId),
    queryFn: () => invokeCommand<SquadFmTraining>("get_squad_fm_training"),
    staleTime: 0,
  });
}

/** Sets the chosen players' training in FM; Rust re-plans and ignores any other UID. */
export function applySquadFmTraining(playerUids: number[]) {
  return invokeCommand<SquadFmTrainingResult>("apply_squad_fm_training", {
    playerUids,
  });
}
