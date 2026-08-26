import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTeamRemovalImpact } from "../types/team-removal-impact";
import type { PlannerTeamSettingInput } from "./save-planner-teams";

export function fetchPlannerTeamRemovalImpacts(
  teams: PlannerTeamSettingInput[],
) {
  return invokeCommand<PlannerTeamRemovalImpact[]>(
    "get_planner_team_removal_impacts",
    { teams },
  );
}
