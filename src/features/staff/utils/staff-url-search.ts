import type { StaffFilterRule } from "../types/staff-filter-rule";
import {
  getStaffFilterField,
  parseStaffFilters as parseStaffFilterRules,
  STAFF_MAX_FILTER_RULES,
} from "./staff-filter-registry";
import { defaultDirForStaffSortField, isStaffMetricId } from "./staff-metrics";

export type StaffView = "search" | "my-staff" | "shortlist";
export type StaffSortDir = "asc" | "desc";
export type StaffSortField = string;

export type StaffFilterRuleUrl = {
  id?: string;
  field: string;
  op: string;
  value: string | number;
};

export function parseStaffView(value: unknown): StaffView {
  return value === "my-staff" || value === "shortlist" ? value : "search";
}

export function parseStaffSort(value: unknown): StaffSortField {
  return typeof value === "string" && isStaffMetricId(value) ? value : "ca";
}

export function parseStaffSortDir(value: unknown): StaffSortDir {
  return value === "asc" ? "asc" : "desc";
}

export function defaultStaffSortDir(value: unknown): StaffSortDir {
  return defaultDirForStaffSortField(parseStaffSort(value));
}

export function parseStaffCombine(value: unknown): "and" | "or" {
  return value === "or" ? "or" : "and";
}

/**
 * Tri-state URL override for the shortlist filter: explicit true/false act
 * as deep-link overrides, while missing and invalid values leave the choice
 * unset so the per-save Zustand choice (or the presence-derived default)
 * restores normal page visits.
 */
export function parseShortlistOnly(value: unknown): boolean | undefined {
  if (value === true || value === "true") {
    return true;
  }
  if (value === false || value === "false") {
    return false;
  }
  return undefined;
}

export function parseStaffFilters(value: unknown): StaffFilterRule[] {
  return parseStaffFilterRules(value);
}

export function staffFiltersForUrl(
  rules: StaffFilterRule[],
): StaffFilterRuleUrl[] {
  return rules.slice(0, STAFF_MAX_FILTER_RULES).map((rule) => ({
    id: rule.id,
    field: rule.field,
    op: rule.op,
    value: rule.value.value,
  }));
}

export function isStaffFilterField(field: string): boolean {
  return getStaffFilterField(field) !== undefined;
}
