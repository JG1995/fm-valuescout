import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { RouterProvider } from "@tanstack/react-router";
import { queryClient, router } from "@/app/router";
import { DestinationWarmer } from "@/app/warm-destinations";

export function AppProvider() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <DestinationWarmer router={router} />
      {import.meta.env.DEV ? (
        <ReactQueryDevtools
          buttonPosition="bottom-right"
          initialIsOpen={false}
        />
      ) : null}
    </QueryClientProvider>
  );
}
