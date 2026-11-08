import type { MoneyballMetric } from "./moneyball-metrics";
import { MONEYBALL_METRICS } from "./moneyball-metrics";
import {
  MONEYBALL_ROLE_CATALOG,
  type MoneyballRolePositionFamily,
} from "./moneyball-role-catalog";
import { getPlayerMetric } from "./player-metrics";
import { POSITION_ORDER } from "./position-order";

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

const TEXT_OPERATORS = [
  { id: "contains", label: "contains" },
  { id: "not_contains", label: "does not contain" },
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
] as const;
const ENUM_OPERATORS = [
  { id: "in", label: "is any of" },
  { id: "not_in", label: "is none of" },
] as const;

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

const basic = (id: string, label: string, kind: "string" | "integer") => ({
  id,
  label,
  category:
    id === "name" || id === "age" || id === "nationality" || id === "height"
      ? "Identity"
      : "Club and value",
  kind,
  align: kind === "integer" ? ("right" as const) : ("left" as const),
  defaultWidth: id === "height" ? 88 : kind === "integer" ? 96 : 144,
  sortable: true as const,
  operators: kind === "integer" ? NUMBER_OPERATORS : TEXT_OPERATORS,
});

/** A General Search field offered unchanged in Moneyball under a Moneyball category. */
const shared = (id: string, category: string): MoneyballSearchMetric => {
  const metric = getPlayerMetric(id);
  if (!metric) {
    throw new Error(`unknown General Search field: ${id}`);
  }
  return {
    id,
    label: metric.label,
    category,
    kind: metric.kind,
    align: metric.align,
    defaultWidth: metric.defaultWidth,
    sortable: true,
    operators: metric.operators,
    enumOptions: metric.enumOptions,
  };
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
  label: role.label,
  category: `Moneyball roles · ${MONEYBALL_ROLE_FAMILY_LABELS[role.positionFamily]}`,
  kind: "integer" as const,
  align: "right" as const,
  defaultWidth: 128,
  sortable: true as const,
  operators: NUMBER_OPERATORS,
  role: true as const,
  roleId: role.id,
});

export const MONEYBALL_SEARCH_METRICS: readonly MoneyballSearchMetric[] = [
  basic("name", "Name", "string"),
  basic("age", "Age", "integer"),
  basic("nationality", "Nationality", "string"),
  basic("height", "Height", "integer"),
  basic("club", "Club", "string"),
  basic("division", "Division", "string"),
  basic("parent_club", "Parent club", "string"),
  {
    id: "preferred_foot",
    label: "Preferred foot",
    category: "Identity",
    kind: "enum",
    align: "left",
    defaultWidth: 128,
    sortable: true,
    operators: ENUM_OPERATORS,
    enumOptions: [
      { value: "left", label: "Left" },
      { value: "right", label: "Right" },
      { value: "either", label: "Either" },
    ],
  },
  basic("value", "Value", "integer"),
  {
    id: "position",
    label: "Position",
    category: "Identity",
    kind: "enum",
    align: "left",
    defaultWidth: 144,
    sortable: true,
    operators: ENUM_OPERATORS,
    enumOptions: POSITION_ORDER.map((key) => ({ value: key, label: key })),
  },
  // Ability and contract context for recruitment. FM attributes, hidden and
  // personality values, position suitability, FM role scores, and Club DNA
  // stay General-only, so Moneyball remains a performance view.
  ...["ca", "pa", "reputation", "world_reputation"].map((id) =>
    shared(id, "Ability and reputation"),
  ),
  ...[
    "wage",
    "contract_year",
    "transfer_listed",
    "loan_listed",
    "not_for_sale",
    "set_for_release",
    "on_loan",
    "team_level",
  ].map((id) => shared(id, "Club and value")),
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

export const DEFAULT_MONEYBALL_TABLE_COLUMN_IDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
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
