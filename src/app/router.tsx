import { QueryClient } from "@tanstack/react-query";
import { createRouter, type ParsedLocation } from "@tanstack/react-router";
import { WorkspaceError } from "@/app/components/workspace-error";
import { routeTree } from "@/routeTree.gen";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,
      // A page left for a while must come back with its last result on screen
      // instead of a loading state. Stale results still refetch in the
      // background, and half an hour bounds the paged table windows.
      gcTime: 30 * 60_000,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      retry: false,
    },
  },
});

/**
 * The screen a location shows: its path plus the search keys that switch a
 * route between whole views. Sort, filter, and column changes keep the screen.
 */
export function screenOf(location: ParsedLocation | undefined): string | null {
  if (!location) return null;
  const search = location.search as Record<string, unknown>;
  return [location.pathname, search.view, search.section, search.classId]
    .map((part) => (part === undefined ? "" : String(part)))
    .join("|");
}

export const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreloadStaleTime: 0,
  defaultPreload: "intent",
  defaultPendingComponent: () => (
    <p
      role="status"
      className="animate-enter-late p-4 text-body-md text-on-surface-variant"
    >
      Loading workspace…
    </p>
  ),
  defaultErrorComponent: WorkspaceError,
  // Cross-fade the page area when the screen changes, and never for a sort,
  // filter, or column change on the screen the user is already reading.
  defaultViewTransition: {
    types: ({ fromLocation, toLocation }) =>
      fromLocation && screenOf(fromLocation) !== screenOf(toLocation)
        ? ["screen"]
        : false,
  },
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
