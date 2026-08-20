import { queryOptions } from "@tanstack/react-query";
import { fetchPlayerMoneyball } from "./fetch-player-moneyball";
import { moneyballKeys } from "./moneyball-keys";

export function getPlayerMoneyballQueryOptions(uid: number) {
  return queryOptions({
    queryKey: moneyballKeys.profile(uid),
    queryFn: () => fetchPlayerMoneyball(uid),
  });
}
