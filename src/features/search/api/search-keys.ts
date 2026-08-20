import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
import type { ComparisonPool, SearchView } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";

export const searchKeys = {
  all: ["search"] as const,
  players: (
    offset: number,
    limit: number,
    sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
    sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
    filters: FilterRule[] = [],
    filterCombine: FilterCombineMode = "and",
    requestedFields: string[] = [],
    searchView: SearchView = "general",
    comparisonPool: ComparisonPool = "filtered",
  ) =>
    [
      ...searchKeys.all,
      "players",
      {
        offset,
        limit,
        sortBy,
        sortDir,
        filters: completeFilterRules(filters, searchView),
        filterCombine,
        requestedFields,
        searchView,
        comparisonPool,
      },
    ] as const,
  suggest: (query: string, limit: number) =>
    [...searchKeys.all, "suggest", { query, limit }] as const,
};
