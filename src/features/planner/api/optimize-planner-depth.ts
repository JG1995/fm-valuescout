import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function optimizePlannerDepth() {
  return invokeCommand<PlannerDepth>("optimize_planner_depth");
}
