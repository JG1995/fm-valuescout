import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTactic } from "../types/tactic";

export function fetchPlannerTactic() {
  return invokeCommand<PlannerTactic>("get_planner_tactic");
}
