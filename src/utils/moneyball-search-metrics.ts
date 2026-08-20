import type { MoneyballMetric } from "./moneyball-metrics";
import { MONEYBALL_METRICS } from "./moneyball-metrics";

export type MoneyballSearchMetric = {
  id: string;
  label: string;
  category: string;
  kind: "string" | "integer" | "number";
  align: "left" | "right";
  defaultWidth: number;
  sortable: true;
  operators: readonly { id: string; label: string }[];
  metric?: MoneyballMetric;
  context?: true;
};

const NUMBER_OPERATORS = [
  { id: "gt", label: "greater than" },
  { id: "lt", label: "less than" },
  { id: "eq", label: "equals" },
  { id: "neq", label: "does not equal" },
] as const;
const TEXT_OPERATORS = [
  { id: "contains", label: "contains" },
  { id: "not_contains", label: "does not contain" },
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
] as const;

const basic = (id: string, label: string, kind: "string" | "integer") => ({
  id,
  label,
  category:
    id === "name" || id === "age" || id === "nationality"
      ? "Identity"
      : "Club and value",
  kind,
  align: kind === "integer" ? ("right" as const) : ("left" as const),
  defaultWidth: kind === "integer" ? 96 : 144,
  sortable: true as const,
  operators: kind === "integer" ? NUMBER_OPERATORS : TEXT_OPERATORS,
});

const context = (id: string, label: string): MoneyballSearchMetric => ({
  id,
  label,
  category: "Context",
  kind: "integer",
  align: "right",
  defaultWidth: 104,
  sortable: true,
  operators: NUMBER_OPERATORS,
  context: true,
});

export const MONEYBALL_SEARCH_METRICS: readonly MoneyballSearchMetric[] = [
  basic("name", "Name", "string"),
  basic("age", "Age / DOB", "integer"),
  basic("nationality", "Nationality", "string"),
  basic("club", "Club", "string"),
  basic("division", "Division", "string"),
  basic("value", "Value", "integer"),
  basic("position", "Position", "string"),
  context("moneyball.starts", "Starts"),
  context("moneyball.substitute_appearances", "Sub appearances"),
  context("moneyball.minutes", "Minutes"),
  ...MONEYBALL_METRICS.map((metric) => ({
    id: `moneyball.${metric.id}`,
    label: metric.label,
    category:
      metric.categoryId.slice(0, 1).toUpperCase() + metric.categoryId.slice(1),
    kind: metric.kind === "count" ? ("integer" as const) : ("number" as const),
    align: "right" as const,
    defaultWidth: metric.width === "wide" ? 176 : 112,
    sortable: true as const,
    operators: NUMBER_OPERATORS,
    metric,
  })),
];

export const DEFAULT_MONEYBALL_TABLE_COLUMN_IDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
  "value",
  "moneyball.minutes",
  "moneyball.average_rating",
  "moneyball.goals_per_90",
  "moneyball.assists_per_90",
  "moneyball.xg_per_90",
  "moneyball.xa_per_90",
] as const;

export function getMoneyballSearchMetric(id: string) {
  return MONEYBALL_SEARCH_METRICS.find((metric) => metric.id === id);
}

export function formatMoneyballMetric(
  metric: MoneyballMetric,
  value: number | null | undefined,
) {
  if (value === null || value === undefined) return "—";
  if (metric.kind === "percentage") {
    const percentage =
      metric.percentageScale === "fraction" ? value * 100 : value;
    return `${percentage.toFixed(metric.precision)}%`;
  }
  const formatted = value.toFixed(metric.precision);
  return metric.kind === "distance"
    ? `${formatted}${metric.id.endsWith("_per_90") ? " km / 90" : " km"}`
    : formatted;
}
