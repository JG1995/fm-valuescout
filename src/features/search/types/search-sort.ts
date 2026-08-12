export const BASIC_SEARCH_SORT_FIELDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
  "ca",
  "pa",
  "value",
] as const;

/** @deprecated Prefer BASIC_SEARCH_SORT_FIELDS when you mean the fixed columns. */
export const SEARCH_SORT_FIELDS = BASIC_SEARCH_SORT_FIELDS;

export type BasicSearchSortField = (typeof BASIC_SEARCH_SORT_FIELDS)[number];

/** Basic column id or a filter field id used as a dynamic column sort key. */
export type SearchSortField = string;
export type SearchSortDir = "asc" | "desc";

export const DEFAULT_SEARCH_SORT_FIELD: SearchSortField = "ca";
export const DEFAULT_SEARCH_SORT_DIR: SearchSortDir = "desc";

export function isBasicSearchSortField(
  value: unknown,
): value is BasicSearchSortField {
  return (
    typeof value === "string" &&
    (BASIC_SEARCH_SORT_FIELDS as readonly string[]).includes(value)
  );
}

/**
 * Shape check for URL/IPC sort keys. Dynamic ids are confirmed against the
 * filter registry and visible columns in the search route.
 */
export function isSearchSortField(value: unknown): value is SearchSortField {
  if (typeof value !== "string" || value.length === 0) {
    return false;
  }
  if (isBasicSearchSortField(value)) {
    return true;
  }
  return /^[a-z0-9_.]+$/i.test(value);
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
    case "parent_club":
    case "preferred_foot":
    case "team_level":
    case "position":
      return "asc";
    default:
      return "desc";
  }
}
