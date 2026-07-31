import { queryOptions } from "@tanstack/react-query";
import { fetchPlannerClubs } from "./fetch-planner-clubs";
import { plannerKeys } from "./planner-keys";

export const plannerClubsQueryOptions = queryOptions({
  queryKey: plannerKeys.clubs(),
  queryFn: fetchPlannerClubs,
});
