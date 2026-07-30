export const SEARCH_SORT_FIELDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
  "ca",
  "pa",
  "value",
] as const;

export type SearchSortField = (typeof SEARCH_SORT_FIELDS)[number];
export type SearchSortDir = "asc" | "desc";

export const DEFAULT_SEARCH_SORT_FIELD: SearchSortField = "ca";
export const DEFAULT_SEARCH_SORT_DIR: SearchSortDir = "desc";

export function isSearchSortField(value: unknown): value is SearchSortField {
  return (
    typeof value === "string" &&
    (SEARCH_SORT_FIELDS as readonly string[]).includes(value)
  );
}

export function isSearchSortDir(value: unknown): value is SearchSortDir {
  return value === "asc" || value === "desc";
}

/** Default direction when switching to a new column. */
export function defaultDirForSortField(field: SearchSortField): SearchSortDir {
  switch (field) {
    case "name":
    case "nationality":
    case "club":
    case "division":
      return "asc";
    case "age":
    case "ca":
    case "pa":
    case "value":
      return "desc";
  }
}
