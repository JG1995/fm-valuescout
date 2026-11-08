import {
  isPercentileOperator,
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
import type {
  FilterRule,
  FilterRuleIpc,
  FilterValue,
} from "../types/filter-rule";
import { createFilterRuleId, filterRuleToIpc } from "../types/filter-rule";
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
    // Numbers start empty so a rule is never applied with an unchosen value.
    case "integer":
      return { type: "integer", value: Number.NaN };
    case "number":
      return { type: "number", value: Number.NaN };
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

/** A draft rule with no field chosen yet; the editor drops it on Done. */
export function createBlankFilterRule(): FilterRule {
  return {
    id: createFilterRuleId(),
    field: "",
    op: "",
    value: { type: "text", value: "" },
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
  if (
    isPercentileOperator(rule.op) &&
    !(
      typeof rule.value.value === "number" &&
      rule.value.value >= 0 &&
      rule.value.value <= 100
    )
  ) {
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

/** True for a Moneyball statistic shown as a percentage, compared on its raw value. */
export function isRawPercentageRule(
  rule: FilterRule,
  view: SearchView = "general",
): boolean {
  const field = getFilterField(rule.field, view);
  return (
    field !== undefined &&
    "metric" in field &&
    field.metric?.kind === "percentage" &&
    !isPercentileOperator(rule.op)
  );
}

/**
 * Search IPC rule. Moneyball ratio statistics display ×100 but are stored as
 * 0–1 fractions, so a raw comparison typed on the displayed scale is divided
 * by 100. Percentile operators already use 0–100.
 */
export function filterRuleToSearchIpc(
  rule: FilterRule,
  view: SearchView = "general",
): FilterRuleIpc {
  const ipc = filterRuleToIpc(rule);
  const field = getFilterField(rule.field, view);
  const fraction =
    field !== undefined &&
    "metric" in field &&
    field.metric?.percentageScale === "fraction";
  return fraction &&
    !isPercentileOperator(rule.op) &&
    typeof ipc.value === "number"
    ? { ...ipc, value: ipc.value / 100 }
    : ipc;
}

export function completeFilterRules(
  rules: FilterRule[],
  view: SearchView = "general",
): FilterRule[] {
  return rules.filter((rule) => isFilterRuleComplete(rule, view));
}
