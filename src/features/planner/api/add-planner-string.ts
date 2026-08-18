import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";
import type { PlannerTeam } from "../types/team";

export function addPlannerString(team: PlannerTeam) {
  return invokeCommand<PlannerDepth>("add_planner_string", { team });
}
