import { queryOptions } from "@tanstack/react-query";
import type {
  PlannerRoleReferencePhase,
  PlannerRoleReferenceScoreBasis,
} from "../types/role-reference";
import { fetchPlannerRoleReference } from "./fetch-planner-role-reference";
import { plannerKeys } from "./planner-keys";

export function plannerRoleReferenceQueryOptions(
  activeSaveId: number,
  phase: PlannerRoleReferencePhase,
  scoreBasis: PlannerRoleReferenceScoreBasis,
) {
  return queryOptions({
    queryKey: plannerKeys.roleReference(activeSaveId, phase, scoreBasis),
    queryFn: () => fetchPlannerRoleReference(phase, scoreBasis),
  });
}
