import {
  MONEYBALL_SEARCH_METRICS,
  type MoneyballSearchMetric,
} from "@/utils/moneyball-search-metrics";
import {
  getPlayerMetric,
  PLAYER_METRICS,
  type PlayerMetric,
  type PlayerMetricKind,
  type PlayerMetricOperator,
} from "@/utils/player-metrics";
import type { FilterRule, FilterValue } from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";

export type FilterFieldKind = PlayerMetricKind | "number";
export type FilterOperator = PlayerMetricOperator;
export type FilterFieldDef = PlayerMetric | MoneyballSearchMetric;

export const FILTER_FIELDS = PLAYER_METRICS;

export function filterFieldsForView(
  view: SearchView,
): readonly FilterFieldDef[] {
  return view === "moneyball" ? MONEYBALL_SEARCH_METRICS : FILTER_FIELDS;
}

export function getFilterField(
  fieldId: string,
  view: SearchView = "general",
): FilterFieldDef | undefined {
  return view === "moneyball"
    ? MONEYBALL_SEARCH_METRICS.find((field) => field.id === fieldId)
    : getPlayerMetric(fieldId);
}

export function defaultValueForField(
  fieldId: string,
  view: SearchView = "general",
): FilterValue {
  const field = getFilterField(fieldId, view);
  if (!field) {
    return { type: "text", value: "" };
  }
  switch (field.kind) {
    case "string":
      return { type: "text", value: "" };
    case "integer":
      return { type: "integer", value: 0 };
    case "number":
      return { type: "number", value: 0 };
    case "boolean":
      return { type: "bool", value: true };
    case "enum":
      return { type: "text", value: field.enumOptions?.[0]?.value ?? "" };
  }
}

export function defaultOperatorForField(
  fieldId: string,
  view: SearchView = "general",
): string {
  const field = getFilterField(fieldId, view);
  return field?.operators[0]?.id ?? "contains";
}

export function createDefaultFilterRule(
  fieldId = "ca",
  view: SearchView = "general",
): FilterRule {
  return {
    id: createFilterRuleId(),
    field: fieldId,
    op: defaultOperatorForField(fieldId, view),
    value: defaultValueForField(fieldId, view),
  };
}

export function isFilterRuleComplete(
  rule: FilterRule,
  view: SearchView = "general",
): boolean {
  const field = getFilterField(rule.field, view);
  if (!field) {
    return false;
  }
  if (!field.operators.some((operator) => operator.id === rule.op)) {
    return false;
  }

  switch (field.kind) {
    case "string":
      return rule.value.type === "text" && rule.value.value.trim().length > 0;
    case "integer":
      return rule.value.type === "integer" && Number.isFinite(rule.value.value);
    case "number":
      return rule.value.type === "number" && Number.isFinite(rule.value.value);
    case "boolean":
      return rule.value.type === "bool";
    case "enum":
      return (
        rule.value.type === "text" &&
        (field.enumOptions?.some(
          (option) => option.value === rule.value.value,
        ) ??
          false)
      );
  }
}

export function completeFilterRules(
  rules: FilterRule[],
  view: SearchView = "general",
): FilterRule[] {
  return rules.filter((rule) => isFilterRuleComplete(rule, view));
}
