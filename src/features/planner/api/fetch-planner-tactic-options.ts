import { invokeCommand } from "@/lib/tauri-client";
import type { TacticOptions } from "../types/tactic";
import type { PlannerContext } from "./planner-keys";

export function fetchPlannerTacticOptions(context: PlannerContext) {
  return invokeCommand<TacticOptions>("get_planner_tactic_options", {
    saveId: context.saveId,
    contextToken: context.contextToken,
  });
}
