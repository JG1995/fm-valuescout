import { queryOptions } from "@tanstack/react-query";
import { fetchSaves } from "./fetch-saves";
import { snapshotKeys } from "./snapshot-keys";

export const savesQueryOptions = queryOptions({
  queryKey: snapshotKeys.saves(),
  queryFn: fetchSaves,
});
