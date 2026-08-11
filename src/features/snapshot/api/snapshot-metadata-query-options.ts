import { queryOptions } from "@tanstack/react-query";
import { fetchSnapshotMetadata } from "./fetch-snapshot-metadata";
import { snapshotKeys } from "./snapshot-keys";

export function snapshotMetadataQueryOptions(saveId: number) {
  return queryOptions({
    queryKey: snapshotKeys.history(saveId),
    queryFn: () => fetchSnapshotMetadata(saveId),
  });
}
