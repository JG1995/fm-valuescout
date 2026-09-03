import { isValidTacticColumnId } from "@/utils/tactic-ids";
import type { FilterRule } from "../types/filter-rule";
import type { SearchSortField } from "../types/search-sort";
import { BASIC_SEARCH_SORT_FIELDS } from "../types/search-sort";
import type { SearchView } from "../types/search-view";
import { completeFilterRules, getFilterField } from "./filter-registry";

const BASIC_COLUMN_FIELDS = new Set<string>(BASIC_SEARCH_SORT_FIELDS);

/** Non-basic complete filter fields that should appear as result columns. */
export function dynamicColumnFields(
  filters: FilterRule[],
  view: SearchView = "general",
): string[] {
  const fields: string[] = [];
  for (const rule of completeFilterRules(filters, view)) {
    if (BASIC_COLUMN_FIELDS.has(rule.field)) {
      continue;
    }
    if (!getFilterField(rule.field, view)) {
      continue;
    }
    if (fields.includes(rule.field)) {
      continue;
    }
    fields.push(rule.field);
  }
  return fields;
}

export function dynamicColumnLabel(
  fieldId: string,
  view: SearchView = "general",
): string {
  return getFilterField(fieldId, view)?.label ?? fieldId;
}

/** Sort is allowed for every known player metric, including hidden table fields. */
export function isVisibleSortField(
  value: unknown,
  _filters: FilterRule[],
  view: SearchView = "general",
  visibleColumnIds: readonly string[] = [],
): value is SearchSortField {
  if (typeof value !== "string") {
    return false;
  }
  if (isValidTacticColumnId(value)) {
    return visibleColumnIds.includes(value);
  }
  if ((BASIC_SEARCH_SORT_FIELDS as readonly string[]).includes(value)) {
    return (
      view === "general" ||
      view === "shortlist" ||
      (value !== "ca" && value !== "pa")
    );
  }
  return getFilterField(value, view) !== undefined;
}
