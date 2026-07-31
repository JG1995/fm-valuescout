import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerTacticOptions } from "./fetch-planner-tactic-options";
import { plannerKeys } from "./planner-keys";

export const plannerTacticOptionsQueryOptions = queryOptions({
  queryKey: plannerKeys.tacticOptions(),
  queryFn: fetchPlannerTacticOptions,
});
