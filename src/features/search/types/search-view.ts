export type SearchView = "general" | "moneyball";

/**
 * The field set a filter rule draws on: one of Search's views, or a tactic
 * slot's recruitment filters, which mix General fields with Moneyball
 * playing time and statistics.
 */
export type FilterView = SearchView | "slot";

export type ComparisonPool = "filtered" | "fullCsv";

export function parseSearchView(value: unknown): SearchView {
  return value === "moneyball" ? "moneyball" : "general";
}

export function parseComparisonPool(value: unknown): ComparisonPool {
  return value === "fullCsv" ? "fullCsv" : "filtered";
}

export function defaultSearchSort(view: SearchView): string {
  return view === "moneyball" ? "moneyball.average_rating" : "ca";
}
