import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";
import type { PlannerTeam } from "../types/team";

export type PlannerStringSettingInput = {
  id: number | null;
  displayName: string;
};

export type PlannerTeamSettingInput = {
  team: PlannerTeam;
  displayName: string;
  /** The oldest age the team's squads take, or null for no limit. */
  maxAge: number | null;
  strings: PlannerStringSettingInput[];
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
