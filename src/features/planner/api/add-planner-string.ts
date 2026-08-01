import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTeam } from "../types/club-family";
import type { PlannerDepth } from "../types/depth";

export function addPlannerString(team: PlannerTeam) {
  return invokeCommand<PlannerDepth>("add_planner_string", { team });
}
