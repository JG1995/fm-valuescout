import { describe, expect, it } from "vitest";
import { searchKeys } from "./search-keys";
import { searchPlayersQueryOptions } from "./search-players-query-options";

describe("searchKeys.players", () => {
  it("uses a stable player-page root while keeping suggestions outside it", () => {
    const key = searchKeys.players(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      [],
      "general",
      "filtered",
      null,
      {
        activeSave: { id: 1, contextToken: "save-token" },
        currentSnapshot: { id: 2, saveId: 1 },
      },
    );

    expect(key.slice(0, searchKeys.playerPages().length)).toEqual(
      searchKeys.playerPages(),
    );
    expect(searchKeys.suggest("alex", 5).slice(0, 2)).toEqual([
      "search",
      "suggest",
    ]);
  });

  it("keeps requested field order and mounted context in the page cache identity", () => {
    const contextOne = {
      activeSave: { id: 1, contextToken: "one" },
      currentSnapshot: { id: 2, saveId: 1 },
    };
    const contextTwo = {
      ...contextOne,
      activeSave: { id: 1, contextToken: "two" },
    };
    const positionThenRole = searchKeys.players(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      ["position", "role.goalkeeper_ip"],
      "general",
      "filtered",
      null,
      contextOne,
    );
    const roleThenPosition = searchKeys.players(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      ["role.goalkeeper_ip", "position"],
      "general",
      "filtered",
      null,
      contextOne,
    );

    expect(positionThenRole).not.toEqual(roleThenPosition);
    expect(positionThenRole).not.toEqual(
      searchKeys.players(
        0,
        50,
        "ca",
        "desc",
        [],
        "and",
        [],
        "general",
        "filtered",
        null,
        contextTwo,
      ),
    );
  });

  it("uses the exact first-page key without changing IPC arguments", () => {
    const options = searchPlayersQueryOptions(0, 50, "name", "asc");

    expect(options.queryKey).toEqual(searchKeys.players(0, 50, "name", "asc"));
  });

  it("keeps the selected shortlist in the page cache identity", () => {
    const context = {
      activeSave: { id: 1, contextToken: "one" },
      currentSnapshot: { id: 2, saveId: 1 },
    };
    const pageKey = (shortlistId: number | null) =>
      searchKeys.players(
        0,
        50,
        "ca",
        "desc",
        [],
        "and",
        [],
        "general",
        "filtered",
        shortlistId,
        context,
      );

    expect(pageKey(1)).not.toEqual(pageKey(null));
    expect(pageKey(1)).not.toEqual(pageKey(2));
  });
});
