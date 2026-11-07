import { createRootRouteWithContext } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { AppShellLayout } from "@/app/components/app-shell-layout";
import { NotFoundPage } from "@/app/components/not-found-page";
import { RootError } from "@/app/components/root-error";
import type { RouterContext } from "@/app/router-context";

function RootComponent() {
  return (
    <>
      <AppShellLayout />
      {import.meta.env.DEV ? (
        // Offset so this launcher sits beside the React Query one instead of
        // under it — both share the only corner the app shell does not own.
        <TanStackRouterDevtools
          position="bottom-right"
          toggleButtonProps={{ style: { right: "4rem" } }}
        />
      ) : null}
    </>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  errorComponent: RootError,
  notFoundComponent: NotFoundPage,
});
