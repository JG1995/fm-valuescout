import { isValidTacticColumnId } from "@/utils/tactic-ids";
import type { FilterRule } from "../types/filter-rule";
import type { SearchSortField } from "../types/search-sort";
import { BASIC_SEARCH_SORT_FIELDS } from "../types/search-sort";
import { completeFilterRules, getFilterField } from "./filter-registry";

const BASIC_COLUMN_FIELDS = new Set<string>(BASIC_SEARCH_SORT_FIELDS);

/** A known field with a table column; a filter-only field has none and is never sortable. */
function isColumnField(fieldId: string): boolean {
  const field = getFilterField(fieldId);
  return field !== undefined && field.sortable !== false;
}

/** Non-basic complete filter fields that should appear as result columns. */
export function dynamicColumnFields(filters: FilterRule[]): string[] {
  const fields: string[] = [];
  for (const rule of completeFilterRules(filters)) {
    if (BASIC_COLUMN_FIELDS.has(rule.field)) {
      continue;
    }
    if (!isColumnField(rule.field)) {
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

/** Sort is allowed for every known player metric, including hidden table fields. */
export function isVisibleSortField(
  value: unknown,
  visibleColumnIds: readonly string[] = [],
): value is SearchSortField {
  if (typeof value !== "string") {
    return false;
  }
  if (isValidTacticColumnId(value)) {
    return visibleColumnIds.includes(value);
  }
  if ((BASIC_SEARCH_SORT_FIELDS as readonly string[]).includes(value)) {
    return true;
  }
  return isColumnField(value);
}
