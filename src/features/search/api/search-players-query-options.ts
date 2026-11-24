import { queryOptions } from "@tanstack/react-query";
import { getSearchMetric } from "@/utils/search-metrics";
import { isValidTacticColumnId } from "@/utils/tactic-ids";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
  isBasicSearchSortField,
} from "../types/search-sort";
import type { MoneyballComparison } from "../types/search-view";
import {
  fetchSearchPlayers,
  SEARCH_PAGE_SIZE,
  type SearchTableId,
} from "./fetch-search-players";
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
  comparison: MoneyballComparison = "own",
  shortlistId: number | null = null,
  context?: SearchPlayerPageContext,
  tableId?: SearchTableId,
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
      comparison,
      shortlistId,
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
        comparison,
        shortlistId,
        tableId,
      ),
  });
}

/**
 * The fields the table requests for its visible columns: every known column
 * except the basic ones every row carries, in a stable order.
 */
export function searchRequestedFields(columnIds: readonly string[]): string[] {
  return columnIds
    .flatMap((id) => {
      const fieldId = isValidTacticColumnId(id) ? id : getSearchMetric(id)?.id;
      return fieldId === undefined || isBasicSearchSortField(fieldId)
        ? []
        : [fieldId];
    })
    .sort();
}

export type SearchFirstPageRequest = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  requestedFields: string[];
  comparison: MoneyballComparison;
  shortlistId: number | null;
  pageContext: SearchPlayerPageContext;
};

/** The Search table's first page; the route loader warms the same query. */
export function searchFirstPageQueryOptions(
  request: SearchFirstPageRequest,
  tableId?: SearchTableId,
) {
  return searchPlayersQueryOptions(
    0,
    SEARCH_PAGE_SIZE,
    request.sortBy,
    request.sortDir,
    request.filters,
    request.filterCombine,
    request.requestedFields,
    request.comparison,
    request.shortlistId,
    request.pageContext,
    tableId,
  );
}
