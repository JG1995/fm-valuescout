import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import { searchKeys } from "@/features/search/api/search-keys";
import { squadKeys } from "@/features/squad/api/squad-keys";
import { clearPlayerResultContext } from "./player-result-context";

describe("clearPlayerResultContext", () => {
  it("cancels player-page roots before removing them without clearing adjacent queries", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const searchPage = searchKeys.players(0, 50);
    const squadPage = squadKeys.players(0, 50);
    const suggestion = searchKeys.suggest("alex", 5);
    const plannerQuery = ["planner", "depth"] as const;
    queryClient.setQueryData(searchPage, { players: ["old-search"] });
    queryClient.setQueryData(squadPage, { players: ["old-squad"] });
    queryClient.setQueryData(suggestion, ["Alex"]);
    queryClient.setQueryData(plannerQuery, { teams: [] });

    const calls: string[] = [];
    const cancelQueries = queryClient.cancelQueries.bind(queryClient);
    const removeQueries = queryClient.removeQueries.bind(queryClient);
    vi.spyOn(queryClient, "cancelQueries").mockImplementation((filters) => {
      calls.push(
        JSON.stringify(filters?.queryKey) ===
          JSON.stringify(searchKeys.playerPages())
          ? "cancel-search"
          : "cancel-squad",
      );
      return cancelQueries(filters);
    });
    vi.spyOn(queryClient, "removeQueries").mockImplementation((filters) => {
      calls.push(
        JSON.stringify(filters?.queryKey) ===
          JSON.stringify(searchKeys.playerPages())
          ? "remove-search"
          : "remove-squad",
      );
      return removeQueries(filters);
    });
    let resolveLateSearch!: (value: { players: string[] }) => void;
    let resolveLateSquad!: (value: { players: string[] }) => void;
    const lateSearch = new Promise<{ players: string[] }>((resolve) => {
      resolveLateSearch = resolve;
    });
    const lateSquad = new Promise<{ players: string[] }>((resolve) => {
      resolveLateSquad = resolve;
    });
    void queryClient
      .fetchQuery({
        queryKey: searchKeys.players(50, 50),
        queryFn: () => lateSearch,
      })
      .catch(() => undefined);
    void queryClient
      .fetchQuery({
        queryKey: squadKeys.players(50, 50),
        queryFn: () => lateSquad,
      })
      .catch(() => undefined);

    await clearPlayerResultContext(queryClient);
    resolveLateSearch({ players: ["late-search"] });
    resolveLateSquad({ players: ["late-squad"] });
    await Promise.resolve();

    expect(calls).toEqual([
      "cancel-search",
      "cancel-squad",
      "remove-search",
      "remove-squad",
    ]);

    expect(queryClient.getQueryData(searchPage)).toBeUndefined();
    expect(queryClient.getQueryData(squadPage)).toBeUndefined();
    expect(
      queryClient.getQueryData(searchKeys.players(50, 50)),
    ).toBeUndefined();
    expect(queryClient.getQueryData(squadKeys.players(50, 50))).toBeUndefined();
    expect(queryClient.getQueryData(suggestion)).toEqual(["Alex"]);
    expect(queryClient.getQueryData(plannerQuery)).toEqual({ teams: [] });
  });
});
