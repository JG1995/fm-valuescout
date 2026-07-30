import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
} from "../types/search-sort";

export const searchKeys = {
  all: ["search"] as const,
  players: (
    offset: number,
    limit: number,
    sortBy: SearchSortField = DEFAULT_SEARCH_SORT_FIELD,
    sortDir: SearchSortDir = DEFAULT_SEARCH_SORT_DIR,
  ) =>
    [...searchKeys.all, "players", { offset, limit, sortBy, sortDir }] as const,
};
