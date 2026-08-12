import { invokeCommand } from "@/lib/tauri-client";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import { filterRuleToIpc } from "../types/filter-rule";
import type { SearchPlayersPage } from "../types/player-summary";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";
import { completeFilterRules } from "../utils/filter-registry";

/** Matches Rust `DEFAULT_PAGE_LIMIT` — window size for virtualized fetch. */
export const SEARCH_PAGE_SIZE = 50;

export function fetchSearchPlayers(
  offset = 0,
  limit = SEARCH_PAGE_SIZE,
  sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
  sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
  filters: FilterRule[] = [],
  filterCombine: FilterCombineMode = "and",
  requestedFields: string[] = [],
) {
  const applied = completeFilterRules(filters);
  return invokeCommand<SearchPlayersPage>("search_players", {
    offset,
    limit,
    sortBy,
    sortDir,
    requestedFields,
    ...(applied.length > 0
      ? {
          filters: applied.map(filterRuleToIpc),
          filterCombine,
        }
      : {}),
  });
}
