import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function clearPlannerDepth(confirmed: boolean) {
  return invokeCommand<PlannerDepth>("clear_planner_depth", { confirmed });
}
