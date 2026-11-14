import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerLoanCandidate } from "../types/depth";
import { plannerKeys } from "./planner-keys";

/** The managed club's possible loan players, most to gain from a loan first. */
export function plannerLoanCandidatesQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: plannerKeys.loanCandidates(activeSaveId),
    queryFn: () =>
      invokeCommand<PlannerLoanCandidate[]>("get_planner_loan_candidates"),
  });
}
