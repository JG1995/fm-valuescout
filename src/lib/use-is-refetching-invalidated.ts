import { type QueryKey, useIsFetching } from "@tanstack/react-query";

/**
 * Whether a query under `queryKey` is refetching data that something
 * invalidated. Route preloads, loader prefetches, and newly mounted observers
 * also refetch data that is merely stale; that data has not changed, so a page
 * must not treat those refetches as a context change.
 */
export function useIsRefetchingInvalidated(queryKey: QueryKey) {
  return (
    useIsFetching({
      queryKey,
      predicate: (query) => query.state.isInvalidated,
    }) > 0
  );
}
