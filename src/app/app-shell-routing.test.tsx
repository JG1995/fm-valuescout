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
    useLayoutStore.setState({ railExpanded: false });
    setHealthSimulateError(false);
  });

  it("renders the layout shell on the index route", async () => {
    renderWithProviders();

    expect(await screen.findByTestId("app-header")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
  });

  it("keeps the nav rail reachable and its label hidden while collapsed", async () => {
    renderWithProviders();

    const rail = await screen.findByRole("navigation", { name: "Primary" });

    expect(rail).toHaveAttribute("data-expanded", "false");
    expect(screen.getByRole("link", { name: "Dashboard" })).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  it("expands the nav rail to show item labels", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const rail = await screen.findByTestId("app-nav-rail");
    const toggle = screen.getByRole("button", { name: "Toggle navigation" });

    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText("FM ValueScout", { selector: "span" }),
    ).toBeNull();

    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(rail).toHaveAttribute("data-expanded", "true");
    expect(
      screen.getByText("FM ValueScout", { selector: "span" }),
    ).toBeInTheDocument();

    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(rail).toHaveAttribute("data-expanded", "false");
  });

  it("offers a skip link to the main region", async () => {
    renderWithProviders();

    const skipLink = await screen.findByRole("link", {
      name: "Skip to content",
    });

    expect(skipLink).toHaveAttribute("href", "#main-content");
    expect(screen.getByRole("main")).toHaveAttribute("id", "main-content");
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

  it("lists Squad in the nav rail", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("link", { name: "Squad" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Planner" })).toBeNull();
  });

  it("lists Youth Academy in the nav rail", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("link", { name: "Youth Academy" }),
    ).toHaveAttribute("href", "/academy");
  });
});
