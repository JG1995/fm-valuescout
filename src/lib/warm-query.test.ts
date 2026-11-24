import { QueryClient, queryOptions } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { warmQuery } from "./warm-query";

describe("warmQuery", () => {
  it("fetches a missing result in the background and leaves a cached one alone", async () => {
    const queryClient = new QueryClient();
    const queryFn = vi.fn().mockResolvedValue("squad");
    const options = queryOptions({
      queryKey: ["squad"],
      queryFn,
      staleTime: 0,
    });

    warmQuery(queryClient, options);
    await vi.waitFor(() =>
      expect(queryClient.getQueryData(options.queryKey)).toBe("squad"),
    );

    // Stale, but present: the screen paints it and refreshes it itself.
    warmQuery(queryClient, options);
    expect(queryFn).toHaveBeenCalledTimes(1);
  });

  it("refetches a cached result that something invalidated", async () => {
    const queryClient = new QueryClient();
    const queryFn = vi
      .fn()
      .mockResolvedValueOnce("before Load Data")
      .mockResolvedValueOnce("after Load Data");
    const options = queryOptions({
      queryKey: ["squad"],
      queryFn,
      staleTime: 60_000,
    });
    await queryClient.fetchQuery(options);
    await queryClient.invalidateQueries({ queryKey: options.queryKey });

    warmQuery(queryClient, options);

    await vi.waitFor(() =>
      expect(queryClient.getQueryData(options.queryKey)).toBe(
        "after Load Data",
      ),
    );
  });

  it("swallows a failed read so a hover never raises an unhandled rejection", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const options = queryOptions({
      queryKey: ["squad"],
      queryFn: () => Promise.reject(new Error("database is busy")),
    });

    warmQuery(queryClient, options);

    await vi.waitFor(() =>
      expect(queryClient.getQueryState(options.queryKey)?.status).toBe("error"),
    );
  });
});
