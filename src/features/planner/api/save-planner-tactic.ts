import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTactic } from "../types/tactic";

export function savePlannerTactic(tactic: PlannerTactic) {
  return invokeCommand<PlannerTactic>("save_planner_tactic", { tactic });
}
