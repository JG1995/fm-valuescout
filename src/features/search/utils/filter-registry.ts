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
/**
 * A field that filters but has no table column, so the metric catalogs leave it
 * out. It is never sortable, which keeps it out of the sort picker.
 */
type FilterOnlyField = Pick<
  PlayerMetric,
  "id" | "label" | "kind" | "operators" | "enumOptions"
> & { category: string; sortable: false };
export type FilterFieldDef =
  | PlayerMetric
  | MoneyballSearchMetric
  | FilterOnlyField;

/** The rule the "Only realistic signings" toggle adds and removes. */
export const REALISTIC_SIGNING_FIELD_ID = "realistic_signing";

/** The picker category the field joins in each view. */
const realisticSigningField = (category: string): FilterOnlyField => ({
  id: REALISTIC_SIGNING_FIELD_ID,
  label: "Realistic signing",
  category,
  kind: "boolean",
  sortable: false,
  operators: [
    { id: "is", label: "is" },
    { id: "is_not", label: "is not" },
  ],
});

const REALISTIC_SIGNING_FIELDS: Record<SearchView, FilterOnlyField> = {
  general: realisticSigningField("club-contract"),
  moneyball: realisticSigningField("Club and value"),
};

const GENERAL_FILTER_FIELDS: readonly FilterFieldDef[] = [
  ...PLAYER_METRICS,
  REALISTIC_SIGNING_FIELDS.general,
];
const MONEYBALL_FILTER_FIELDS: readonly FilterFieldDef[] = [
  ...MONEYBALL_SEARCH_METRICS,
  REALISTIC_SIGNING_FIELDS.moneyball,
];

export function filterFieldsForView(
  view: SearchView,
): readonly FilterFieldDef[] {
  return view === "moneyball" ? MONEYBALL_FILTER_FIELDS : GENERAL_FILTER_FIELDS;
}

export function getFilterField(
  fieldId: string,
  view: SearchView = "general",
): FilterFieldDef | undefined {
  if (fieldId === REALISTIC_SIGNING_FIELD_ID) {
    return REALISTIC_SIGNING_FIELDS[view];
  }
  return view === "moneyball"
    ? MONEYBALL_SEARCH_METRICS.find((field) => field.id === fieldId)
    : getPlayerMetric(fieldId);
}

/** The value shape an operator takes: one value, a `[min, max]` range, or a list of choices. */
export function valueShapeForOperator(op: string): "single" | "range" | "list" {
  if (op === "between" || op === "pct_between") {
    return "range";
  }
  return op === "in" || op === "not_in" ? "list" : "single";
}

/** An empty value for the operator's shape; percentiles start at 90 (or 80–100). */
export function defaultValueForRule(
  fieldId: string,
  op: string,
  view: SearchView = "general",
): FilterValue {
  const shape = valueShapeForOperator(op);
  if (shape === "list") {
    return { type: "list", value: [] };
  }
  if (shape === "range") {
    return {
      type: "range",
      value: op === "pct_between" ? [80, 100] : [Number.NaN, Number.NaN],
    };
  }
  if (isPercentileOperator(op)) {
    return { type: "number", value: 90 };
  }
  return defaultValueForField(fieldId, view);
}

function defaultValueForField(
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
  const shape = valueShapeForOperator(rule.op);
  const inScale = (value: number) =>
    Number.isFinite(value) &&
    (!isPercentileOperator(rule.op) || (value >= 0 && value <= 100));
  if (shape === "range") {
    if (rule.value.type !== "range") {
      return false;
    }
    const [min, max] = rule.value.value;
    return inScale(min) && inScale(max) && min <= max;
  }
  if (shape === "list") {
    return (
      rule.value.type === "list" &&
      rule.value.value.length > 0 &&
      rule.value.value.every(
        (choice) =>
          field.enumOptions?.some((option) => option.value === choice) ?? false,
      )
    );
  }
  if (
    isPercentileOperator(rule.op) &&
    !(typeof rule.value.value === "number" && inScale(rule.value.value))
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
  if (!fraction || isPercentileOperator(rule.op)) {
    return ipc;
  }
  if (typeof ipc.value === "number") {
    return { ...ipc, value: ipc.value / 100 };
  }
  return Array.isArray(ipc.value) &&
    ipc.value.every((bound) => typeof bound === "number")
    ? { ...ipc, value: ipc.value.map((bound) => (bound as number) / 100) }
    : ipc;
}

export function completeFilterRules(
  rules: FilterRule[],
  view: SearchView = "general",
): FilterRule[] {
  return rules.filter((rule) => isFilterRuleComplete(rule, view));
}
