import { describe, expect, it } from "vitest";
import { defaultDirForSortField } from "./search-sort";

describe("defaultDirForSortField", () => {
  it("sorts lower-is-better Moneyball statistics best first", () => {
    expect(defaultDirForSortField("moneyball.fouls_made_per_90")).toBe("asc");
    expect(defaultDirForSortField("moneyball.minutes_per_goal")).toBe("asc");
    expect(defaultDirForSortField("moneyball.goals_per_90")).toBe("desc");
  });
});
