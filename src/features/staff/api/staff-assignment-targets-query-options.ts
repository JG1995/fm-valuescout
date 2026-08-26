import { queryOptions } from "@tanstack/react-query";
import type { StaffAssignmentContext } from "../types/staff-assignment";
import { fetchStaffAssignmentTargets } from "./fetch-staff-assignment-targets";
import { staffKeys } from "./staff-keys";

export function staffAssignmentTargetsQueryOptions(
  context: StaffAssignmentContext,
  contextKey: string,
) {
  return queryOptions({
    queryKey: staffKeys.assignmentTargets(contextKey),
    queryFn: () => fetchStaffAssignmentTargets(context.saveContextToken),
  });
}
