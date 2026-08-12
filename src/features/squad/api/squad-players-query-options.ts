import { queryOptions } from "@tanstack/react-query";
import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";
import { fetchSquadPlayers, SQUAD_PAGE_SIZE } from "./fetch-squad-players";
import { squadKeys } from "./squad-keys";

export { SQUAD_PAGE_SIZE };

export function squadPlayersQueryOptions(
  offset = 0,
  limit = SQUAD_PAGE_SIZE,
  sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
  sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
) {
  return queryOptions({
    queryKey: squadKeys.players(offset, limit, sortBy, sortDir),
    queryFn: () => fetchSquadPlayers(offset, limit, sortBy, sortDir),
  });
}
