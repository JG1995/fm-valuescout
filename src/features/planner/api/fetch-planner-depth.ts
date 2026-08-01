import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function fetchPlannerDepth() {
  return invokeCommand<PlannerDepth>("get_planner_depth");
}
