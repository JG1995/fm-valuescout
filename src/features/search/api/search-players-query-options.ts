import { queryOptions } from "@tanstack/react-query";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
import type { ComparisonPool, SearchView } from "../types/search-view";
import { fetchSearchPlayers, SEARCH_PAGE_SIZE } from "./fetch-search-players";
import { type SearchPlayerPageContext, searchKeys } from "./search-keys";

export { SEARCH_PAGE_SIZE };

export function searchPlayersQueryOptions(
  offset = 0,
  limit = SEARCH_PAGE_SIZE,
  sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
  sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
  filters: FilterRule[] = [],
  filterCombine: FilterCombineMode = "and",
  requestedFields: string[] = [],
  searchView: SearchView = "general",
  comparisonPool: ComparisonPool = "filtered",
  context?: SearchPlayerPageContext,
) {
  return queryOptions({
    queryKey: searchKeys.players(
      offset,
      limit,
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
      searchView,
      comparisonPool,
      context,
    ),
    queryFn: () =>
      fetchSearchPlayers(
        offset,
        limit,
        sortBy,
        sortDir,
        filters,
        filterCombine,
        requestedFields,
        searchView,
        comparisonPool,
      ),
  });
}
