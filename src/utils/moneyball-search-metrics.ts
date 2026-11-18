import type { MoneyballMetric } from "./moneyball-metrics";
import { MONEYBALL_METRICS } from "./moneyball-metrics";
import {
  MONEYBALL_ROLE_CATALOG,
  type MoneyballRolePositionFamily,
} from "./moneyball-role-catalog";

export type MoneyballSearchMetric = {
  id: string;
  label: string;
  category: string;
  kind: "string" | "integer" | "number" | "boolean" | "enum";
  align: "left" | "right";
  defaultWidth: number;
  sortable: true;
  operators: readonly { id: string; label: string }[];
  enumOptions?: readonly { value: string; label: string }[];
  metric?: MoneyballMetric;
  context?: true;
  role?: true;
  roleId?: string;
};

const NUMBER_OPERATORS = [
  { id: "gte", label: "at least" },
  { id: "lte", label: "at most" },
  { id: "between", label: "between" },
  { id: "eq", label: "equals" },
] as const;
/** Decimal statistics: an exact match is almost never meaningful. */
const DECIMAL_OPERATORS = [
  { id: "gte", label: "at least" },
  { id: "lte", label: "at most" },
  { id: "between", label: "between" },
] as const;
/**
 * Compare a statistic's 0–100 comparison-pool percentile instead of its raw
 * value. The editor's Value | Percentile switch names the scale, so the
 * labels match the raw operators.
 */
const PERCENTILE_OPERATORS = [
  { id: "pct_gte", label: "at least" },
  { id: "pct_lte", label: "at most" },
  { id: "pct_between", label: "between" },
] as const;

export function isPercentileOperator(op: string) {
  return PERCENTILE_OPERATORS.some((operator) => operator.id === op);
}

const MONEYBALL_ROLE_FAMILY_LABELS: Record<
  MoneyballRolePositionFamily,
  string
> = {
  attacking_midfielder: "Attacking midfield",
  central_defender: "Central defense",
  central_midfielder: "Central midfield",
  defensive_midfielder: "Defensive midfield",
  full_back: "Full-back",
  goalkeeper: "Goalkeeper",
  striker: "Striker",
  wide_midfielder: "Wide midfield",
  wing_back: "Wing-back",
  winger: "Winger",
};

const context = (id: string, label: string): MoneyballSearchMetric => ({
  id,
  label,
  category: "Playing time",
  kind: "integer",
  align: "right",
  defaultWidth: 104,
  sortable: true,
  operators: NUMBER_OPERATORS,
  context: true,
});

const roleMetric = (role: (typeof MONEYBALL_ROLE_CATALOG)[number]) => ({
  id: `moneyball_role.${role.id}`,
  label: `Moneyball Score · ${role.label}`,
  category: `Moneyball Scores · ${MONEYBALL_ROLE_FAMILY_LABELS[role.positionFamily]}`,
  kind: "integer" as const,
  align: "right" as const,
  defaultWidth: 128,
  sortable: true as const,
  operators: NUMBER_OPERATORS,
  role: true as const,
  roleId: role.id,
});

/**
 * Player Search's season fields: playing time, statistics, and each Moneyball
 * role's Moneyball Score. They sit beside the player fields of
 * `PLAYER_METRICS`.
 */
export const MONEYBALL_SEARCH_METRICS: readonly MoneyballSearchMetric[] = [
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
    operators: [
      ...(metric.kind === "count" ? NUMBER_OPERATORS : DECIMAL_OPERATORS),
      ...PERCENTILE_OPERATORS,
    ],
    metric,
  })),
  ...MONEYBALL_ROLE_CATALOG.map(roleMetric),
];

const MONEYBALL_SEARCH_METRIC_BY_ID = new Map(
  MONEYBALL_SEARCH_METRICS.map((metric) => [metric.id, metric]),
);

export function getMoneyballSearchMetric(id: string) {
  return MONEYBALL_SEARCH_METRIC_BY_ID.get(id);
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
