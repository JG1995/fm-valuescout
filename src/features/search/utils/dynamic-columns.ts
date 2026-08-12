import type { FilterRule } from "../types/filter-rule";
import type { SearchSortField } from "../types/search-sort";
import { BASIC_SEARCH_SORT_FIELDS } from "../types/search-sort";
import { completeFilterRules, getFilterField } from "./filter-registry";

const BASIC_COLUMN_FIELDS = new Set<string>(BASIC_SEARCH_SORT_FIELDS);

/** Non-basic complete filter fields that should appear as result columns. */
export function dynamicColumnFields(filters: FilterRule[]): string[] {
  const fields: string[] = [];
  for (const rule of completeFilterRules(filters)) {
    if (
      BASIC_COLUMN_FIELDS.has(rule.field) ||
      rule.field === "position" ||
      rule.field.startsWith("potential_role.")
    ) {
      continue;
    }
    if (!getFilterField(rule.field)) {
      continue;
    }
    if (fields.includes(rule.field)) {
      continue;
    }
    fields.push(rule.field);
  }
  return fields;
}

export function dynamicColumnLabel(fieldId: string): string {
  return getFilterField(fieldId)?.label ?? fieldId;
}

/** Sort is allowed for basic columns and currently visible dynamic columns. */
export function isVisibleSortField(
  value: unknown,
  filters: FilterRule[],
): value is SearchSortField {
  if (typeof value !== "string") {
    return false;
  }
  if ((BASIC_SEARCH_SORT_FIELDS as readonly string[]).includes(value)) {
    return true;
  }
  return dynamicColumnFields(filters).includes(value);
}
