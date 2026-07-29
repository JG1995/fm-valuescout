import { queryOptions } from "@tanstack/react-query";
import { fetchSanityPlayers } from "./fetch-sanity-players";
import { snapshotKeys } from "./snapshot-keys";

export const sanityPlayersQueryOptions = queryOptions({
  queryKey: snapshotKeys.sanityPlayers(),
  queryFn: () => fetchSanityPlayers(),
});
