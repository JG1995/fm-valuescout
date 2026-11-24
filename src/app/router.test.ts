import type { ParsedLocation } from "@tanstack/react-router";
import { describe, expect, it } from "vitest";
import { queryClient, router, screenOf } from "./router";

function location(pathname: string, search: Record<string, unknown> = {}) {
  return { pathname, search } as ParsedLocation;
}

describe("queryClient desktop IPC defaults", () => {
  it("disables HTTP-oriented refetch and retry on production defaults", () => {
    const { queries } = queryClient.getDefaultOptions();

    expect(queries?.refetchOnWindowFocus).toBe(false);
    expect(queries?.refetchOnReconnect).toBe(false);
    expect(queries?.retry).toBe(false);
    expect(queries?.staleTime).toBe(60_000);
  });

  it("keeps an unobserved result for half an hour so a revisited page paints from cache", () => {
    expect(queryClient.getDefaultOptions().queries?.gcTime).toBe(30 * 60_000);
  });
});

describe("page cross-fade", () => {
  const transition = router.options.defaultViewTransition;
  const types = (from: ParsedLocation | undefined, to: ParsedLocation) => {
    if (
      typeof transition !== "object" ||
      typeof transition.types !== "function"
    ) {
      throw new Error("expected a view transition type resolver");
    }
    return transition.types({
      fromLocation: from,
      toLocation: to,
      pathChanged: from?.pathname !== to.pathname,
      hrefChanged: true,
      hashChanged: false,
    });
  };

  it("names a screen by its path and its view-switching search keys", () => {
    expect(screenOf(location("/my-club", { view: "planner" }))).not.toBe(
      screenOf(location("/my-club", { view: "squad" })),
    );
    expect(
      screenOf(location("/players/7", { section: "attributes" })),
    ).not.toBe(screenOf(location("/players/7", { section: "overview" })));
    expect(screenOf(location("/search", { sort: "ca", dir: "asc" }))).toBe(
      screenOf(location("/search", { sort: "pa", dir: "desc" })),
    );
  });

  it("fades between screens and never for a sort or filter change", () => {
    expect(types(location("/search"), location("/staff"))).toEqual(["screen"]);
    expect(
      types(
        location("/my-club", { view: "squad" }),
        location("/my-club", { view: "tactic" }),
      ),
    ).toEqual(["screen"]);
    expect(
      types(
        location("/search", { sort: "ca" }),
        location("/search", { sort: "pa", filters: "age" }),
      ),
    ).toBe(false);
  });

  it("does not fade the first load", () => {
    expect(types(undefined, location("/"))).toBe(false);
  });
});
