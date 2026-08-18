import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerSlotCandidate } from "../types/depth";
import type { PlannerTeam } from "../types/team";

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
