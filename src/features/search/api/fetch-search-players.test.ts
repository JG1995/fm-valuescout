import { beforeEach, describe, expect, it, vi } from "vitest";
import { invokeCommand } from "@/lib/tauri-client";
import { fetchSearchPlayers } from "./fetch-search-players";
import { searchPlayersQueryOptions } from "./search-players-query-options";

vi.mock("@/lib/tauri-client", () => ({ invokeCommand: vi.fn() }));

const context = {
  activeSave: { id: 1, contextToken: "save-1" },
  currentSnapshot: { id: 2, saveId: 1 },
};

function lastArgs() {
  return vi.mocked(invokeCommand).mock.lastCall?.[1];
}

describe("search player fetch", () => {
  beforeEach(() => {
    vi.mocked(invokeCommand).mockReset();
    vi.mocked(invokeCommand).mockResolvedValue({ players: [], total: 0 });
  });

  it("passes a table ID only when one is given", async () => {
    await fetchSearchPlayers();
    expect(lastArgs()).not.toHaveProperty("tableId");
    expect(lastArgs()).not.toHaveProperty("searchView");

    await fetchSearchPlayers(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      [],
      "own",
      null,
      "search",
    );
    expect(lastArgs()).toMatchObject({ tableId: "search" });
  });

  it("keeps the table ID out of the query key", async () => {
    const plain = searchPlayersQueryOptions(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      [],
      "own",
      null,
      context,
    );
    const named = searchPlayersQueryOptions(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      [],
      "own",
      null,
      context,
      "search",
    );
    expect(named.queryKey).toEqual(plain.queryKey);

    await named.queryFn?.({} as never);
    expect(lastArgs()).toMatchObject({ tableId: "search" });
  });
});
