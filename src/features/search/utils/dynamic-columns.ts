import { insertColumnsInGroups } from "@/components/player-table/table-groups";
import { isValidTacticColumnId, tacticColumnId } from "@/utils/tactic-ids";
import type { FilterRule } from "../types/filter-rule";
import type { SearchSortField } from "../types/search-sort";
import { BASIC_SEARCH_SORT_FIELDS } from "../types/search-sort";
import { completeFilterRules, getFilterField } from "./filter-registry";
import { SEARCH_TABLE_GROUPS } from "./search-table-groups";

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

/** The columns a saved filter keeps whatever its fields. */
const SAVED_FILTER_BASE_COLUMNS = ["age", "nationality", "value"];

const ROLE_SCORE_PREFIXES = ["role.", "potential_role."];

/**
 * Search's columns for an applied saved filter or tactic slot: Age,
 * Nationality, and Value; Role Score and Potential Role Score for each role
 * a rule scores; a tactic slot's Role Score and Potential; and each rule's
 * field, in header-group order.
 */
export function savedFilterColumnIds(
  filters: FilterRule[],
  laneId?: string,
): string[] {
  const fields = completeFilterRules(filters)
    .map((rule) => rule.field)
    .filter(isColumnField);
  const roleScores = fields.flatMap((field) => {
    const prefix = ROLE_SCORE_PREFIXES.find((candidate) =>
      field.startsWith(candidate),
    );
    if (!prefix) {
      return [];
    }
    const roleId = field.slice(prefix.length);
    return ROLE_SCORE_PREFIXES.map((rolePrefix) => `${rolePrefix}${roleId}`);
  });
  const slotScores =
    laneId === undefined
      ? []
      : (["current", "potential"] as const).flatMap(
          (group) => tacticColumnId(group, laneId) ?? [],
        );
  return insertColumnsInGroups(
    SAVED_FILTER_BASE_COLUMNS,
    [...roleScores, ...slotScores, ...fields],
    SEARCH_TABLE_GROUPS,
  );
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
