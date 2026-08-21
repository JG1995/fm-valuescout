import { invokeCommand } from "@/lib/tauri-client";
import type {
  PlannerRoleReference,
  PlannerRoleReferencePhase,
  PlannerRoleReferenceScoreBasis,
} from "../types/role-reference";

export function fetchPlannerRoleReference(
  phase: PlannerRoleReferencePhase,
  scoreBasis: PlannerRoleReferenceScoreBasis,
) {
  return invokeCommand<PlannerRoleReference>("get_planner_role_reference", {
    phase,
    scoreBasis,
  });
}
