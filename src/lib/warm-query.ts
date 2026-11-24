import type {
  FetchQueryOptions,
  QueryClient,
  QueryKey,
} from "@tanstack/react-query";

/**
 * Starts a read the next screen will want without making navigation wait for
 * it. It fetches when nothing is cached or when the cached result was
 * invalidated. A result that is merely old is left alone: the screen paints it
 * at once and its own observer refreshes it, so hovering a link never reruns
 * an expensive read. A failure surfaces where the screen reads the query.
 */
export function warmQuery<
  TQueryFnData,
  TError,
  TData,
  TQueryKey extends QueryKey,
>(
  queryClient: QueryClient,
  options: FetchQueryOptions<TQueryFnData, TError, TData, TQueryKey>,
): void {
  const state = queryClient.getQueryState(options.queryKey);
  if (state?.data !== undefined && !state.isInvalidated) return;
  void queryClient.prefetchQuery(options);
}
