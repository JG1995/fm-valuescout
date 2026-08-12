import { describe, expect, it } from "vitest";
import { squadKeys } from "./squad-keys";

describe("squadKeys.players", () => {
  it("keeps requested field order in the page cache identity", () => {
    const positionThenRole = squadKeys.players(0, 50, "ca", "desc", [
      "position",
      "role.goalkeeper_ip",
    ]);
    const roleThenPosition = squadKeys.players(0, 50, "ca", "desc", [
      "role.goalkeeper_ip",
      "position",
    ]);

    expect(positionThenRole).not.toEqual(roleThenPosition);
  });
});
