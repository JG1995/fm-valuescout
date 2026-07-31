import { invokeCommand } from "@/lib/tauri-client";
import type { TacticOptions } from "../types/tactic";

export function fetchPlannerTacticOptions() {
  return invokeCommand<TacticOptions>("get_planner_tactic_options");
}
