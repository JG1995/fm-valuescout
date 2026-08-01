import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTeam } from "../types/club-family";
import type { PlannerDepth } from "../types/depth";

export function clearPlannerTeam(team: PlannerTeam, confirmed: boolean) {
  return invokeCommand<PlannerDepth>("clear_planner_team", { team, confirmed });
}
