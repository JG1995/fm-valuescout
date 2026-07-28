import type { ErrorComponentProps } from "@tanstack/react-router";
import {
  createRootRouteWithContext,
  ErrorComponent,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { AppShellLayout } from "@/app/components/app-shell-layout";
import { NotFoundPage } from "@/app/components/not-found-page";
import type { RouterContext } from "@/app/router-context";

function RootError({ error }: ErrorComponentProps) {
  return (
    <div className="p-4 text-on-background">
      <h1 className="text-lg font-semibold">Something went wrong</h1>
      <ErrorComponent error={error} />
    </div>
  );
}

function RootComponent() {
  return (
    <>
      <AppShellLayout />
      {import.meta.env.DEV ? (
        <TanStackRouterDevtools position="bottom-right" />
      ) : null}
    </>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  errorComponent: RootError,
  notFoundComponent: NotFoundPage,
});
