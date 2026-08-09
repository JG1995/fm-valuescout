import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export type PlannerScoreBasis = "current" | "potential";

export function optimizePlannerDepth(scoreBasis: PlannerScoreBasis) {
  return invokeCommand<PlannerDepth>("optimize_planner_depth", { scoreBasis });
}
