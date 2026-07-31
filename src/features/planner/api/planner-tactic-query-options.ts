import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerTactic } from "./fetch-planner-tactic";
import { plannerKeys } from "./planner-keys";

export const plannerTacticQueryOptions = queryOptions({
  queryKey: plannerKeys.tactic(),
  queryFn: fetchPlannerTactic,
});
