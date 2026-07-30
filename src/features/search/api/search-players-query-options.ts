import { queryOptions } from "@tanstack/react-query";
import { fetchSearchPlayers, SEARCH_PAGE_SIZE } from "./fetch-search-players";
import { searchKeys } from "./search-keys";

export { SEARCH_PAGE_SIZE };

export function searchPlayersQueryOptions(
  offset = 0,
  limit = SEARCH_PAGE_SIZE,
) {
  return queryOptions({
    queryKey: searchKeys.players(offset, limit),
    queryFn: () => fetchSearchPlayers(offset, limit),
  });
}
