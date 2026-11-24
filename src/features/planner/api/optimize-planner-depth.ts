import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

/**
 * Refills the squads. Without position familiarity, players are placed by
 * role scores and preferred foot alone.
 */
export function optimizePlannerDepth(positionFamiliarity: boolean) {
  return invokeCommand<PlannerDepth>("optimize_planner_depth", {
    positionFamiliarity,
  });
}
