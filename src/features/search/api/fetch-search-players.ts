import { invokeCommand } from "@/lib/tauri-client";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchPlayersPage } from "../types/player-summary";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
import type { ComparisonPool, SearchView } from "../types/search-view";
import {
  completeFilterRules,
  filterRuleToSearchIpc,
} from "../utils/filter-registry";

/** Matches Rust `DEFAULT_PAGE_LIMIT` — window size for virtualized fetch. */
export const SEARCH_PAGE_SIZE = 50;

/**
 * Names the table whose newest request may stop an older one in Rust. Only a
 * panel's requested first page passes it.
 */
export type SearchTableId = "search" | "moneyball-search";

/** Rust's error for a table request that a newer one superseded. */
const SUPERSEDED_MESSAGE = "Superseded by a newer request";

/** A superseded request is never shown as a failure. */
export function isSupersededError(error: unknown): boolean {
  return error instanceof Error && error.message === SUPERSEDED_MESSAGE;
}

export function fetchSearchPlayers(
  offset = 0,
  limit = SEARCH_PAGE_SIZE,
  sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
  sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
  filters: FilterRule[] = [],
  filterCombine: FilterCombineMode = "and",
  requestedFields: string[] = [],
  searchView: SearchView = "general",
  comparisonPool: ComparisonPool = "filtered",
  shortlistId: number | null = null,
  tableId?: SearchTableId,
) {
  const applied = completeFilterRules(filters, searchView);
  return invokeCommand<SearchPlayersPage>("search_players", {
    offset,
    limit,
    sortBy,
    sortDir,
    requestedFields,
    searchView,
    comparisonPool,
    shortlistId,
    ...(tableId ? { tableId } : {}),
    ...(applied.length > 0
      ? {
          filters: applied.map((rule) =>
            filterRuleToSearchIpc(rule, searchView),
          ),
          filterCombine,
        }
      : {}),
  });
}
