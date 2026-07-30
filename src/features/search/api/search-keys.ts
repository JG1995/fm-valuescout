import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
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
  ) =>
    [
      ...searchKeys.all,
      "players",
      {
        offset,
        limit,
        sortBy,
        sortDir,
        filters: completeFilterRules(filters),
        filterCombine,
      },
    ] as const,
};
