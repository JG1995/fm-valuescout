import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { setHealthSimulateError } from "@/features/health/api/health-simulate-error";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import { renderWithProviders } from "@/testing/render-with-providers";

describe("app shell routing", () => {
  beforeEach(() => {
    useLayoutStore.setState({ sidebarOpen: false });
    setHealthSimulateError(false);
  });

  it("renders the layout shell on the index route", async () => {
    renderWithProviders();

    expect(await screen.findByTestId("app-header")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Cursor React Tauri Template" }),
    ).toBeInTheDocument();
  });

  it("toggles sidebar visibility from the layout store", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const sidebar = await screen.findByTestId("app-sidebar");
    const toggle = screen.getByRole("button", { name: "Toggle sidebar" });

    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(sidebar).toHaveAttribute("data-open", "false");

    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(sidebar).toHaveAttribute("data-open", "true");

    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(sidebar).toHaveAttribute("data-open", "false");
  });

  it("renders the not-found page for unknown routes", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    const router = createRouter({
      routeTree,
      context: { queryClient } satisfies RouterContext,
      defaultPreloadStaleTime: 0,
      history: createMemoryHistory({ initialEntries: ["/does-not-exist"] }),
    });

    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(
      await screen.findByRole("heading", { name: "Page not found" }),
    ).toBeInTheDocument();
  });
});
