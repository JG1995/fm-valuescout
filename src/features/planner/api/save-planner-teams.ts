import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTeam } from "../types/club-family";
import type { PlannerDepth } from "../types/depth";

export type PlannerTeamSettingInput = {
  team: PlannerTeam;
  displayName: string;
};

export function savePlannerTeams(
  teams: PlannerTeamSettingInput[],
  confirmPopulatedRemoval: boolean,
) {
  return invokeCommand<PlannerDepth>("save_planner_teams", {
    teams,
    confirmPopulatedRemoval,
  });
}
