import { describe, expect, it } from "vitest";
import { queryClient } from "./router";

describe("queryClient desktop IPC defaults", () => {
  it("disables HTTP-oriented refetch and retry on production defaults", () => {
    const { queries } = queryClient.getDefaultOptions();

    expect(queries?.refetchOnWindowFocus).toBe(false);
    expect(queries?.refetchOnReconnect).toBe(false);
    expect(queries?.retry).toBe(false);
    expect(queries?.staleTime).toBe(60_000);
  });
});
