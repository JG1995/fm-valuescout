export type SearchView = "general" | "moneyball";

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
