import {
  isPositionFamilyId,
  type PositionFamilyId,
  positionFamilyLabel,
} from "@/utils/position-families";

/**
 * The field set a filter rule draws on: Player Search's, or a tactic slot's
 * recruitment filters, which leave out the fields the Dashboard cannot read.
 */
export type FilterView = "search" | "slot";

/**
 * Whom a statistic's percentile ranks each player against: the peers of his
 * own best position's family, one family's peers for everyone, or a tactic
 * slot's two families together (`winger+wide_midfielder`).
 */
export type MoneyballComparison =
  | "own"
  | PositionFamilyId
  | `${PositionFamilyId}+${PositionFamilyId}`;

export function parseMoneyballComparison(value: unknown): MoneyballComparison {
  if (typeof value !== "string") {
    return "own";
  }
  const families = value.split("+");
  return families.length <= 2 &&
    families.every(isPositionFamilyId) &&
    new Set(families).size === families.length
    ? (value as MoneyballComparison)
    : "own";
}

/** The comparison as a select label, such as "Wingers + Wide midfielders". */
export function moneyballComparisonLabel(
  comparison: MoneyballComparison,
): string {
  return comparison === "own"
    ? "Own position"
    : comparison.split("+").map(positionFamilyLabel).join(" + ");
}
