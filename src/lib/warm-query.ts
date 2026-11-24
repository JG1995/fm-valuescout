import type {
  EnsureQueryDataOptions,
  QueryClient,
  QueryKey,
} from "@tanstack/react-query";

/**
 * Starts a read the next screen will want without making navigation wait for
 * it. It fetches only when nothing is cached: a cached result, even a stale
 * one, paints at once and the screen's own observer refreshes it. A failure
 * surfaces where the screen reads the query.
 */
export function warmQuery<
  TQueryFnData,
  TError,
  TData,
  TQueryKey extends QueryKey,
>(
  queryClient: QueryClient,
  options: EnsureQueryDataOptions<TQueryFnData, TError, TData, TQueryKey>,
): void {
  void queryClient.ensureQueryData(options).catch(() => undefined);
}
