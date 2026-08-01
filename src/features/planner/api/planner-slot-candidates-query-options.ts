import { queryOptions } from "@tanstack/react-query";
import {
  fetchPlannerSlotCandidates,
  type PlannerSlotCandidateParams,
} from "./fetch-planner-slot-candidates";
import { plannerKeys } from "./planner-keys";

export function plannerSlotCandidatesQueryOptions(
  activeSaveId: number,
  params: PlannerSlotCandidateParams,
) {
  const search = params.search.trim();
  return queryOptions({
    queryKey: plannerKeys.slotCandidate(
      activeSaveId,
      params.team,
      params.laneId,
      search,
    ),
    queryFn: () => fetchPlannerSlotCandidates({ ...params, search }),
  });
}
