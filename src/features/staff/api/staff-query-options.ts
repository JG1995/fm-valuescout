import { queryOptions } from "@tanstack/react-query";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";
import {
  DEFAULT_STAFF_SORT_DIR,
  DEFAULT_STAFF_SORT_FIELD,
} from "../types/staff-sort";
import { fetchMyStaff, fetchStaff, STAFF_PAGE_SIZE } from "./fetch-staff";
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
  shortlistOnly = false,
  preferredJob?: string,
  unemployedOnly = false,
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
      preferredJob,
      unemployedOnly,
      shortlistOnly,
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
        shortlistOnly,
        preferredJob,
        unemployedOnly,
      ),
  });
}

export function staffMyStaffQueryOptions(
  offset = 0,
  limit = STAFF_PAGE_SIZE,
  sortBy: StaffSortField = DEFAULT_STAFF_SORT_FIELD,
  sortDir: StaffSortDir = DEFAULT_STAFF_SORT_DIR,
  requestedFields: string[] = [],
) {
  return queryOptions({
    queryKey: staffKeys.list(
      "my-staff",
      offset,
      limit,
      sortBy,
      sortDir,
      [],
      "and",
      requestedFields,
    ),
    queryFn: () =>
      fetchMyStaff(offset, limit, sortBy, sortDir, requestedFields),
  });
}
