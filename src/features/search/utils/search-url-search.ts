import type {
  FilterCombineMode,
  FilterRule,
  FilterValue,
} from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { getFilterField } from "./filter-registry";

/** Mirrors Rust `MAX_FILTER_RULES` in `features/search/filter.rs`. */
export const MAX_FILTER_RULES = 32;

export type FilterRuleUrl = {
  id?: string;
  field: string;
  op: string;
  value: string | number | boolean;
};

export function parseSearchCombine(value: unknown): FilterCombineMode {
  return value === "or" ? "or" : "and";
}

function isFilterValue(value: unknown): value is FilterValue {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const record = value as Record<string, unknown>;
  if (record.type === "text" && typeof record.value === "string") {
    return true;
  }
  if (
    (record.type === "integer" || record.type === "number") &&
    typeof record.value === "number" &&
    Number.isFinite(record.value)
  ) {
    return true;
  }
  if (record.type === "bool" && typeof record.value === "boolean") {
    return true;
  }
  return false;
}

function valueFromRaw(
  fieldId: string,
  raw: unknown,
  view: SearchView,
): FilterValue | undefined {
  const field = getFilterField(fieldId, view);
  if (!field) {
    return undefined;
  }

  if (isFilterValue(raw)) {
    switch (field.kind) {
      case "string":
      case "enum":
        return raw.type === "text" ? raw : undefined;
      case "integer":
        return raw.type === "integer" ? raw : undefined;
      case "number":
        return raw.type === "number" ? raw : undefined;
      case "boolean":
        return raw.type === "bool" ? raw : undefined;
    }
  }

  switch (field.kind) {
    case "string":
    case "enum":
      return typeof raw === "string" ? { type: "text", value: raw } : undefined;
    case "integer":
      return typeof raw === "number" && Number.isFinite(raw)
        ? { type: "integer", value: raw }
        : undefined;
    case "number":
      return typeof raw === "number" && Number.isFinite(raw)
        ? { type: "number", value: raw }
        : undefined;
    case "boolean":
      return typeof raw === "boolean"
        ? { type: "bool", value: raw }
        : undefined;
  }
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
  const op = typeof record.op === "string" ? record.op : undefined;
  if (!field || !op) {
    return undefined;
  }

  const fieldDef = getFilterField(field, view);
  if (!fieldDef?.operators.some((candidate) => candidate.id === op)) {
    return undefined;
  }

  const value = valueFromRaw(field, record.value, view);
  if (!value) {
    return undefined;
  }

  const id =
    typeof record.id === "string" && record.id.length > 0
      ? record.id
      : createFilterRuleId();

  return { id, field, op, value };
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
    value: rule.value.value,
  }));
}

export function capFilterRules(rules: FilterRule[]): FilterRule[] {
  return rules.slice(0, MAX_FILTER_RULES);
}
