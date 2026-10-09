import { QueryClient } from "@tanstack/react-query";
import { createRouter } from "@tanstack/react-router";
import { WorkspaceError } from "@/app/components/workspace-error";
import { routeTree } from "@/routeTree.gen";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      retry: false,
    },
  },
});

export const router = createRouter({
  routeTree,
  context: { queryClient },
  defaultPreloadStaleTime: 0,
  defaultPreload: "intent",
  defaultPendingComponent: () => (
    <p role="status" className="p-4 text-body-md text-on-surface-variant">
      Loading workspace…
    </p>
  ),
  defaultErrorComponent: WorkspaceError,
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
