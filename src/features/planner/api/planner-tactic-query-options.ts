import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerTactic } from "./fetch-planner-tactic";
import { type PlannerContext, plannerKeys } from "./planner-keys";

export function plannerTacticQueryOptions(context: PlannerContext) {
  return queryOptions({
    queryKey: plannerKeys.tactic(context),
    queryFn: () => fetchPlannerTactic(context),
  });
}
