import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { WorkspaceError } from "./workspace-error";

describe("WorkspaceError", () => {
  it("shows a thrown value that is not an Error", async () => {
    const rootRoute = createRootRoute({
      loader: () => {
        throw "Snapshot database is locked";
      },
      errorComponent: WorkspaceError,
    });
    const router = createRouter({
      routeTree: rootRoute,
      history: createMemoryHistory(),
    });

    render(<RouterProvider router={router} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Snapshot database is locked",
    );
  });
});
