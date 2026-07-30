import { queryOptions } from "@tanstack/react-query";
import {
  fetchSuggestPlayers,
  SUGGEST_DEFAULT_LIMIT,
} from "./fetch-suggest-players";
import { searchKeys } from "./search-keys";

export function suggestPlayersQueryOptions(
  query: string,
  limit = SUGGEST_DEFAULT_LIMIT,
) {
  const trimmed = query.trim();
  return queryOptions({
    queryKey: searchKeys.suggest(trimmed, limit),
    queryFn: () => fetchSuggestPlayers(trimmed, limit),
    enabled: trimmed.length > 0,
  });
}
