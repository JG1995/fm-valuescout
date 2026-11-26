import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { OptimizeFor, PlannerDevelopment } from "../types/development";
import { plannerKeys } from "./planner-keys";

/**
 * What Optimize squads builds for the save, and the slot each young player is
 * developing towards. It follows the tactic and the data, not who sits where.
 */
export function plannerDevelopmentQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: plannerKeys.development(activeSaveId),
    queryFn: () => invokeCommand<PlannerDevelopment>("get_planner_development"),
  });
}

/** Sets whether Optimize squads builds the strongest squads or builds for development. */
export function setPlannerOptimizeFor(optimizeFor: OptimizeFor) {
  return invokeCommand<null>("set_planner_optimize_for", { optimizeFor });
}
