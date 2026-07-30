import { queryOptions } from "@tanstack/react-query";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
import { fetchSearchPlayers, SEARCH_PAGE_SIZE } from "./fetch-search-players";
import { searchKeys } from "./search-keys";

export { SEARCH_PAGE_SIZE };

export function searchPlayersQueryOptions(
  offset = 0,
  limit = SEARCH_PAGE_SIZE,
  sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
  sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
  filters: FilterRule[] = [],
  filterCombine: FilterCombineMode = "and",
) {
  return queryOptions({
    queryKey: searchKeys.players(
      offset,
      limit,
      sortBy,
      sortDir,
      filters,
      filterCombine,
    ),
    queryFn: () =>
      fetchSearchPlayers(
        offset,
        limit,
        sortBy,
        sortDir,
        filters,
        filterCombine,
      ),
  });
}
