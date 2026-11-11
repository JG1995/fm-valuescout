import type { PlannerTeamSettingInput } from "../api/save-planner-teams";
import type { PlannerDepth } from "../types/depth";

/** The save's teams and strings as `save_planner_teams` takes them, unchanged. */
export function teamSettingInputs(
  depth: PlannerDepth,
): PlannerTeamSettingInput[] {
  return depth.teams.map((team) => ({
    team: team.team,
    displayName: team.displayName,
    maxAge: team.maxAge,
    strings: [...team.strings]
      .sort((left, right) => left.stringOrder - right.stringOrder)
      .map((plannerString) => ({
        id: plannerString.id,
        displayName: plannerString.displayName,
      })),
  }));
}
