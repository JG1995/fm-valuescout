import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerClubFamily } from "./fetch-planner-club-family";
import { plannerKeys } from "./planner-keys";

export const plannerClubFamilyQueryOptions = queryOptions({
  queryKey: plannerKeys.clubFamily(),
  queryFn: fetchPlannerClubFamily,
});
