import { invokeCommand } from "@/lib/tauri-client";
import type { StaffDetail } from "../types/staff-detail";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import { staffFilterRuleToIpc } from "../types/staff-filter-rule";
import {
  DEFAULT_STAFF_SORT_DIR,
  DEFAULT_STAFF_SORT_FIELD,
  type StaffSortDir,
  type StaffSortField,
} from "../types/staff-sort";
import type { StaffPage } from "../types/staff-summary";
import { completeStaffFilterRules } from "../utils/staff-filter-registry";

export const STAFF_PAGE_SIZE = 50;

export function fetchStaff(
  offset = 0,
  limit = STAFF_PAGE_SIZE,
  sortBy: StaffSortField = DEFAULT_STAFF_SORT_FIELD,
  sortDir: StaffSortDir = DEFAULT_STAFF_SORT_DIR,
  filters: StaffFilterRule[] = [],
  filterCombine: "and" | "or" = "and",
  requestedFields: string[] = [],
) {
  return fetchStaffCommand(
    "search_staff",
    offset,
    limit,
    sortBy,
    sortDir,
    filters,
    filterCombine,
    requestedFields,
  );
}

export function fetchMyStaff(
  offset = 0,
  limit = STAFF_PAGE_SIZE,
  sortBy: StaffSortField = DEFAULT_STAFF_SORT_FIELD,
  sortDir: StaffSortDir = DEFAULT_STAFF_SORT_DIR,
  requestedFields: string[] = [],
) {
  return fetchStaffCommand(
    "list_my_staff",
    offset,
    limit,
    sortBy,
    sortDir,
    [],
    "and",
    requestedFields,
  );
}

export function fetchStaffDetail(uid: number) {
  return invokeCommand<StaffDetail | null>("get_staff", { uid });
}

function fetchStaffCommand(
  command: "search_staff" | "list_my_staff",
  offset: number,
  limit: number,
  sortBy: StaffSortField,
  sortDir: StaffSortDir,
  filters: StaffFilterRule[],
  filterCombine: "and" | "or",
  requestedFields: string[],
) {
  const applied = completeStaffFilterRules(filters);
  return invokeCommand<StaffPage>(command, {
    offset,
    limit,
    sortBy,
    sortDir,
    requestedFields,
    ...(applied.length > 0
      ? {
          filters: applied.map(staffFilterRuleToIpc),
          filterCombine,
        }
      : {}),
  });
}
