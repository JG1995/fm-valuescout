import { queryOptions } from "@tanstack/react-query";
import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";
import { squadRequestedFields } from "../utils/squad-columns";
import { fetchSquadPlayers, SQUAD_PAGE_SIZE } from "./fetch-squad-players";
import { type SquadPlayerPageContext, squadKeys } from "./squad-keys";

export { SQUAD_PAGE_SIZE };

export function squadPlayersQueryOptions(
  offset = 0,
  limit = SQUAD_PAGE_SIZE,
  sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
  sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
  requestedFields: string[] = [],
  context?: SquadPlayerPageContext,
) {
  return queryOptions({
    queryKey: squadKeys.players(
      offset,
      limit,
      sortBy,
      sortDir,
      requestedFields,
      context,
    ),
    queryFn: () =>
      fetchSquadPlayers(offset, limit, sortBy, sortDir, requestedFields),
  });
}

/**
 * Squad's first page for a column layout, exactly as the Squad table first
 * requests it, so the route loader warms the read the table observes.
 */
export function squadFirstPageQueryOptions(
  sortBy: SquadSortField,
  sortDir: SquadSortDir,
  columnIds: readonly string[],
  context: SquadPlayerPageContext,
) {
  return squadPlayersQueryOptions(
    0,
    SQUAD_PAGE_SIZE,
    sortBy,
    sortDir,
    squadRequestedFields(columnIds),
    context,
  );
}
