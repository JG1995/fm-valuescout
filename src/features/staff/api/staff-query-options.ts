import { queryOptions } from "@tanstack/react-query";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";
import {
  DEFAULT_STAFF_SORT_DIR,
  DEFAULT_STAFF_SORT_FIELD,
} from "../types/staff-sort";
import { fetchStaff, STAFF_PAGE_SIZE } from "./fetch-staff";
import { staffKeys } from "./staff-keys";

export { STAFF_PAGE_SIZE };

export function staffSearchQueryOptions(
  offset = 0,
  limit = STAFF_PAGE_SIZE,
  sortBy: StaffSortField = DEFAULT_STAFF_SORT_FIELD,
  sortDir: StaffSortDir = DEFAULT_STAFF_SORT_DIR,
  filters: StaffFilterRule[] = [],
  filterCombine: "and" | "or" = "and",
  requestedFields: string[] = [],
) {
  return queryOptions({
    queryKey: staffKeys.list(
      "search",
      offset,
      limit,
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
    ),
    queryFn: () =>
      fetchStaff(
        offset,
        limit,
        sortBy,
        sortDir,
        filters,
        filterCombine,
        requestedFields,
      ),
  });
}
