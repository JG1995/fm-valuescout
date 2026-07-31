import { queryOptions } from "@tanstack/react-query";
import { fetchGetPlayer } from "./fetch-get-player";
import { playerKeys } from "./player-keys";

export function getPlayerQueryOptions(uid: number) {
  return queryOptions({
    queryKey: playerKeys.detail(uid),
    queryFn: () => fetchGetPlayer(uid),
  });
}
