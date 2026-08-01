import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTeam } from "../types/club-family";
import type { PlannerSlotCandidate } from "../types/depth";

export type PlannerSlotCandidateParams = {
  team: PlannerTeam;
  laneId: string;
  search: string;
};

export function fetchPlannerSlotCandidates(params: PlannerSlotCandidateParams) {
  return invokeCommand<PlannerSlotCandidate[]>(
    "get_planner_slot_candidates",
    params,
  );
}
