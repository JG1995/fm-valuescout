import { describe, expect, it } from "vitest";
import { searchKeys } from "./search-keys";

describe("searchKeys.players", () => {
  it("keeps requested field order in the page cache identity", () => {
    const positionThenRole = searchKeys.players(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      ["position", "role.goalkeeper_ip"],
    );
    const roleThenPosition = searchKeys.players(
      0,
      50,
      "ca",
      "desc",
      [],
      "and",
      ["role.goalkeeper_ip", "position"],
    );

    expect(positionThenRole).not.toEqual(roleThenPosition);
  });
});
