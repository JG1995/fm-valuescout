import {
  defaultDirForStaffSortField,
  getStaffMetric,
} from "../utils/staff-metrics";

export type StaffSortField = string;
export type StaffSortDir = "asc" | "desc";

export const DEFAULT_STAFF_SORT_FIELD = "ca";
export const DEFAULT_STAFF_SORT_DIR: StaffSortDir = "desc";

export function isStaffSortField(value: unknown): value is StaffSortField {
  return typeof value === "string" && getStaffMetric(value) !== undefined;
}

export function isStaffSortDir(value: unknown): value is StaffSortDir {
  return value === "asc" || value === "desc";
}

export { defaultDirForStaffSortField };
