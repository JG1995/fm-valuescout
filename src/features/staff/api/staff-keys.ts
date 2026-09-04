import type { QueryKey } from "@tanstack/react-query";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";

export const staffKeys = {
  all: ["staff"] as const,
  detail: (uid: number) => [...staffKeys.all, "detail", uid] as const,
  assignmentTargets: (contextKey: string) =>
    [...staffKeys.all, "assignment-targets", contextKey] as const,
  list: (
    scope: "search" | "my-staff",
    offset: number,
    limit: number,
    sort: StaffSortField,
    dir: StaffSortDir,
    filters: StaffFilterRule[],
    combine: "and" | "or",
    requestedFields: string[],
    preferredJob?: string,
    unemployedOnly?: boolean,
    shortlistOnly?: boolean,
  ) =>
    [
      ...staffKeys.all,
      "list",
      scope,
      offset,
      limit,
      sort,
      dir,
      filters,
      combine,
      requestedFields,
      preferredJob,
      unemployedOnly,
      shortlistOnly,
    ] as const satisfies QueryKey,
} as const;
