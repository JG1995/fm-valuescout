import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function clearPlannerAssignment(stringId: number, laneId: string) {
  return invokeCommand<PlannerDepth>("clear_planner_assignment", {
    stringId,
    laneId,
  });
}
