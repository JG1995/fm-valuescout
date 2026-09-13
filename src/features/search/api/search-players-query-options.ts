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
  shortlistOnly = false,
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
      shortlistOnly,
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
        shortlistOnly,
      ),
  });
}

/**
 * One-row General+shortlist probe reusing the player search query surface.
 * Its response `state` distinguishes a save with no stored shortlist
 * (`no_shortlist`) from a stored shortlist with no current-snapshot matches
 * (`ready`), so the route can derive the shortlist filter default from
 * actual presence. Keying by the mounted save/snapshot context keeps a
 * switched save from initializing the choice from a previous save's data.
 */
export function playerShortlistProbeQueryOptions(
  context?: SearchPlayerPageContext,
) {
  return searchPlayersQueryOptions(
    0,
    1,
    DEFAULT_SEARCH_SORT_FIELD,
    DEFAULT_SEARCH_SORT_DIR,
    [],
    "and",
    [],
    "general",
    "filtered",
    true,
    context,
  );
}
