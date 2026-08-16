import type { QueryKey } from "@tanstack/react-query";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";

export const staffKeys = {
  all: ["staff"] as const,
  list: (
    scope: "search" | "my-staff",
    offset: number,
    limit: number,
    sort: StaffSortField,
    dir: StaffSortDir,
    filters: StaffFilterRule[],
    combine: "and" | "or",
    requestedFields: string[],
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
    ] as const satisfies QueryKey,
} as const;
