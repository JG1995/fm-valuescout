import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerTacticOptions } from "./fetch-planner-tactic-options";
import { type PlannerContext, plannerKeys } from "./planner-keys";

export function plannerTacticOptionsQueryOptions(context: PlannerContext) {
  return queryOptions({
    queryKey: plannerKeys.tacticOptions(context),
    queryFn: () => fetchPlannerTacticOptions(context),
  });
}
