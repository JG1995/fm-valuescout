import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTactic } from "../types/tactic";
import type { PlannerContext } from "./planner-keys";

export function fetchPlannerTactic(context: PlannerContext) {
  return invokeCommand<PlannerTactic>("get_planner_tactic", {
    saveId: context.saveId,
    contextToken: context.contextToken,
  });
}
