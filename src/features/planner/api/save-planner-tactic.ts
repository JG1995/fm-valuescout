import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerTactic } from "../types/tactic";
import type { PlannerContext } from "./planner-keys";

export function savePlannerTactic(
  context: PlannerContext,
  tactic: PlannerTactic,
) {
  return invokeCommand<PlannerTactic>("save_planner_tactic", {
    saveId: context.saveId,
    contextToken: context.contextToken,
    tactic,
  });
}
