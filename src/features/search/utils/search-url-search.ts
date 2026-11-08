import { isPercentileOperator } from "@/utils/moneyball-search-metrics";
import type {
  FilterCombineMode,
  FilterRule,
  FilterValue,
} from "../types/filter-rule";
import { createFilterRuleId, filterValueToIpc } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import {
  type FilterFieldDef,
  getFilterField,
  valueShapeForOperator,
} from "./filter-registry";

/** Mirrors Rust `MAX_FILTER_RULES` in `features/search/filter.rs`. */
export const MAX_FILTER_RULES = 32;

export type FilterRuleUrl = {
  id?: string;
  field: string;
  op: string;
  value: string | number | boolean | number[] | string[];
};

export function parseSearchCombine(value: unknown): FilterCombineMode {
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

/** URL values are flat, but older links stored `{ type, value }` objects. */
function unwrapTaggedValue(raw: unknown): unknown {
  if (raw !== null && typeof raw === "object" && !Array.isArray(raw)) {
    return (raw as Record<string, unknown>).value;
  }
  return raw;
}

function valueFromRaw(
  fieldId: string,
  op: string,
  raw: unknown,
  view: SearchView,
): FilterValue | undefined {
  const field = getFilterField(fieldId, view);
  if (!field) {
    return undefined;
  }
  const value = unwrapTaggedValue(raw);
  const isNumber = (item: unknown): item is number =>
    typeof item === "number" && Number.isFinite(item);

  switch (valueShapeForOperator(op)) {
    case "range":
      return Array.isArray(value) && value.length === 2 && value.every(isNumber)
        ? { type: "range", value: [value[0], value[1]] }
        : undefined;
    case "list":
      return Array.isArray(value) &&
        value.every((item) => typeof item === "string")
        ? { type: "list", value: [...value] }
        : undefined;
    case "single":
      break;
  }
  if (isPercentileOperator(op)) {
    return isNumber(value) ? { type: "number", value } : undefined;
  }
  switch (field.kind) {
    case "string":
    case "enum":
      return typeof value === "string" ? { type: "text", value } : undefined;
    case "integer":
      return isNumber(value) ? { type: "integer", value } : undefined;
    case "number":
      return isNumber(value) ? { type: "number", value } : undefined;
    case "boolean":
      return typeof value === "boolean" ? { type: "bool", value } : undefined;
  }
}

/**
 * Older links used exclusive `gt` / `lt`, `neq`, and single-value `is` /
 * `is_not` for choices. Map them onto today's operators: whole numbers shift
 * by one, decimals become inclusive, choices become one-item lists, and
 * `neq` (no longer offered) is dropped.
 */
function migrateLegacyRule(
  field: FilterFieldDef,
  op: string,
  raw: unknown,
): { op: string; raw: unknown } | undefined {
  const value = unwrapTaggedValue(raw);
  if (field.kind === "integer" || field.kind === "number") {
    if (op === "neq") {
      return undefined;
    }
    if ((op === "gt" || op === "lt") && typeof value === "number") {
      const step = field.kind === "integer" ? 1 : 0;
      return op === "gt"
        ? { op: "gte", raw: value + step }
        : { op: "lte", raw: value - step };
    }
  }
  if (field.kind === "enum" && (op === "is" || op === "is_not")) {
    return { op: op === "is" ? "in" : "not_in", raw: [value] };
  }
  return { op, raw };
}

function parseOneFilterRule(
  entry: unknown,
  view: SearchView,
): FilterRule | undefined {
  if (entry === null || typeof entry !== "object" || Array.isArray(entry)) {
    return undefined;
  }

  const record = entry as Record<string, unknown>;
  const field = typeof record.field === "string" ? record.field : undefined;
  const recordOp = typeof record.op === "string" ? record.op : undefined;
  if (!field || !recordOp) {
    return undefined;
  }

  const fieldDef = getFilterField(field, view);
  const migrated = fieldDef
    ? migrateLegacyRule(fieldDef, recordOp, record.value)
    : undefined;
  if (
    !migrated ||
    !fieldDef?.operators.some((candidate) => candidate.id === migrated.op)
  ) {
    return undefined;
  }

  const value = valueFromRaw(field, migrated.op, migrated.raw, view);
  if (!value) {
    return undefined;
  }

  const id =
    typeof record.id === "string" && record.id.length > 0
      ? record.id
      : createFilterRuleId();

  return { id, field, op: migrated.op, value };
}

export function parseSearchFilters(
  value: unknown,
  view: SearchView = "general",
): FilterRule[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const parsed: FilterRule[] = [];
  for (const entry of value) {
    if (parsed.length >= MAX_FILTER_RULES) {
      break;
    }
    const rule = parseOneFilterRule(entry, view);
    if (rule) {
      parsed.push(rule);
    }
  }
  return parsed;
}

export function searchFiltersForUrl(rules: FilterRule[]): FilterRuleUrl[] {
  return rules.slice(0, MAX_FILTER_RULES).map((rule) => ({
    id: rule.id,
    field: rule.field,
    op: rule.op,
    value: filterValueToIpc(rule.value),
  }));
}

export function capFilterRules(rules: FilterRule[]): FilterRule[] {
  return rules.slice(0, MAX_FILTER_RULES);
}
