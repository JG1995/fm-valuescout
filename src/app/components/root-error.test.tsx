import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RootError } from "./root-error";

describe("RootError", () => {
  it("shows the reason and a way back without the router's default panel", async () => {
    const rootRoute = createRootRoute({
      loader: () => {
        throw new Error("Snapshot database is locked");
      },
      errorComponent: RootError,
    });
    const router = createRouter({
      routeTree: rootRoute,
      history: createMemoryHistory(),
    });

    render(<RouterProvider router={router} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Snapshot database is locked",
    );
    expect(screen.getAllByRole("heading")).toHaveLength(1);
    expect(
      screen.getByRole("heading", { level: 1, name: "Something went wrong" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("main")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeEnabled();
    expect(
      screen.getByRole("link", { name: "Open Player Search" }),
    ).toHaveAttribute("href", "/search");
    expect(screen.queryByRole("button", { name: /show error/i })).toBeNull();
  });
});
