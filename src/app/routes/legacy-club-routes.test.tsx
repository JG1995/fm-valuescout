import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { routeTree } from "@/routeTree.gen";

function renderLegacyPlannerRoute(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const history = createMemoryHistory({ initialEntries: [initialEntry] });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history,
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return { history, router };
}

describe("legacy club routes", () => {
  it("replaces Planner with My Club while preserving workspace and Squad sort", async () => {
    const { history, router } = renderLegacyPlannerRoute(
      "/planner?view=planner&sort=name&dir=asc",
    );

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/my-club");
      expect(router.state.location.search).toEqual({
        view: "planner",
        squadSort: "name",
        squadDir: "asc",
      });
    });
    expect(history.canGoBack()).toBe(false);
    expect(
      await screen.findByRole("heading", { level: 1, name: "My Club" }),
    ).toBeInTheDocument();
  });

  it("normalizes retired Planner workspaces while preserving valid sort direction", async () => {
    const { router } = renderLegacyPlannerRoute(
      "/planner?view=clubs&sort=unknown&dir=asc",
    );

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/my-club");
      expect(router.state.location.search).toEqual({ squadDir: "asc" });
    });
    expect(
      await screen.findByRole("heading", { level: 1, name: "My Club" }),
    ).toBeInTheDocument();
  });

  it("preserves a valid legacy direction without a sort field", async () => {
    const { router } = renderLegacyPlannerRoute("/planner?dir=asc");

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/my-club");
      expect(router.state.location.search).toEqual({ squadDir: "asc" });
    });
    expect(
      await screen.findByRole("heading", { level: 1, name: "My Club" }),
    ).toBeInTheDocument();
  });

  it("replaces the legacy Settings managed-club anchor with My Club", async () => {
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: () => {},
    });
    const { history, router } = renderLegacyPlannerRoute(
      "/settings#managed-club",
    );

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/my-club");
      expect(router.state.location.hash).toBe("managed-club");
    });
    expect(history.canGoBack()).toBe(false);
  });
});
