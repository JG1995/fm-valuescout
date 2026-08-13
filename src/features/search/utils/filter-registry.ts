import {
  getPlayerMetric,
  PLAYER_METRICS,
  type PlayerMetric,
  type PlayerMetricKind,
  type PlayerMetricOperator,
} from "@/utils/player-metrics";
import type { FilterRule, FilterValue } from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";

export type FilterFieldKind = PlayerMetricKind;
export type FilterOperator = PlayerMetricOperator;
export type FilterFieldDef = PlayerMetric;

export const FILTER_FIELDS = PLAYER_METRICS;

export function getFilterField(fieldId: string): FilterFieldDef | undefined {
  return getPlayerMetric(fieldId);
}

export function defaultValueForField(fieldId: string): FilterValue {
  const field = getFilterField(fieldId);
  if (!field) {
    return { type: "text", value: "" };
  }
  switch (field.kind) {
    case "string":
      return { type: "text", value: "" };
    case "integer":
      return { type: "integer", value: 0 };
    case "boolean":
      return { type: "bool", value: true };
    case "enum":
      return { type: "text", value: field.enumOptions?.[0]?.value ?? "" };
  }
}

export function defaultOperatorForField(fieldId: string): string {
  const field = getFilterField(fieldId);
  return field?.operators[0]?.id ?? "contains";
}

export function createDefaultFilterRule(fieldId = "ca"): FilterRule {
  return {
    id: createFilterRuleId(),
    field: fieldId,
    op: defaultOperatorForField(fieldId),
    value: defaultValueForField(fieldId),
  };
}

export function isFilterRuleComplete(rule: FilterRule): boolean {
  const field = getFilterField(rule.field);
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

export function completeFilterRules(rules: FilterRule[]): FilterRule[] {
  return rules.filter(isFilterRuleComplete);
}
