import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerDepth } from "./fetch-planner-depth";
import { plannerKeys } from "./planner-keys";

export const plannerDepthQueryOptions = queryOptions({
  queryKey: plannerKeys.depth(),
  queryFn: fetchPlannerDepth,
});
