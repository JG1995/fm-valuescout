import {
  isPositionFamilyId,
  type PositionFamilyId,
} from "@/utils/position-families";

export type SearchView = "general" | "moneyball";

/**
 * The field set a filter rule draws on: one of Search's views, or a tactic
 * slot's recruitment filters, which mix General fields with Moneyball
 * playing time and statistics.
 */
export type FilterView = SearchView | "slot";

/**
 * Whom Moneyball Search ranks each player's statistics against: the peers of
 * his own best position's family, or one family's peers for every player.
 */
export type MoneyballComparison = "own" | PositionFamilyId;

export function parseSearchView(value: unknown): SearchView {
  return value === "moneyball" ? "moneyball" : "general";
}

export function parseMoneyballComparison(value: unknown): MoneyballComparison {
  return isPositionFamilyId(value) ? value : "own";
}

export function defaultSearchSort(view: SearchView): string {
  return view === "moneyball" ? "moneyball.average_rating" : "ca";
}
