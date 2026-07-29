import { queryOptions } from "@tanstack/react-query";
import { fetchCurrentSnapshot } from "./fetch-current-snapshot";
import { snapshotKeys } from "./snapshot-keys";

export const currentSnapshotQueryOptions = queryOptions({
  queryKey: snapshotKeys.current(),
  queryFn: fetchCurrentSnapshot,
});
