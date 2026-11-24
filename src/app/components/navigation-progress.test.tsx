import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { act, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { NavigationProgress } from "./navigation-progress";

function renderWithSlowRoute() {
  let finishLoading = () => {};
  const loading = new Promise<void>((resolve) => {
    finishLoading = resolve;
  });
  const rootRoute = createRootRoute({
    component: () => (
      <>
        <NavigationProgress />
        <Outlet />
      </>
    ),
  });
  const routeTree = rootRoute.addChildren([
    createRoute({
      getParentRoute: () => rootRoute,
      path: "/",
      component: () => <p>Home</p>,
    }),
    createRoute({
      getParentRoute: () => rootRoute,
      path: "/slow",
      loader: () => loading,
      component: () => <p>Slow page</p>,
    }),
  ]);
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ["/"] }),
  });
  render(<RouterProvider router={router} />);
  return { router, finishLoading };
}

describe("NavigationProgress", () => {
  it("shows while a navigation waits for its loader and leaves once the page renders", async () => {
    const { router, finishLoading } = renderWithSlowRoute();
    expect(await screen.findByText("Home")).toBeInTheDocument();
    expect(screen.queryByRole("progressbar")).toBeNull();

    act(() => {
      // The app's registered route types do not know this test tree.
      router.history.push("/slow");
    });

    expect(
      await screen.findByRole("progressbar", { name: "Loading page" }),
    ).toBeInTheDocument();
    // The page the user is reading stays up until the next one is ready.
    expect(screen.getByText("Home")).toBeInTheDocument();

    await act(async () => {
      finishLoading();
    });

    expect(await screen.findByText("Slow page")).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole("progressbar")).toBeNull());
  });
});
