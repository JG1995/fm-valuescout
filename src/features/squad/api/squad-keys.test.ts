import { describe, expect, it } from "vitest";
import { squadKeys } from "./squad-keys";
import { squadPlayersQueryOptions } from "./squad-players-query-options";

describe("squadKeys.players", () => {
  it("uses a stable player-page root", () => {
    const key = squadKeys.players(0, 50, "ca", "desc", [], {
      activeSave: { id: 1, contextToken: "save-token" },
      currentSnapshot: { id: 2, saveId: 1 },
      managedClub: { clubName: "Metro FC", status: "available" },
    });

    expect(key.slice(0, squadKeys.playerPages().length)).toEqual(
      squadKeys.playerPages(),
    );
  });

  it("keeps requested fields and mounted context in the page cache identity", () => {
    const context = {
      activeSave: { id: 1, contextToken: "save-token" },
      currentSnapshot: { id: 2, saveId: 1 },
      managedClub: { clubName: "Metro FC", status: "available" },
    } as const;
    const positionThenRole = squadKeys.players(
      0,
      50,
      "ca",
      "desc",
      ["position", "role.goalkeeper_ip"],
      context,
    );
    const roleThenPosition = squadKeys.players(
      0,
      50,
      "ca",
      "desc",
      ["role.goalkeeper_ip", "position"],
      context,
    );

    expect(positionThenRole).not.toEqual(roleThenPosition);
    expect(positionThenRole).not.toEqual(
      squadKeys.players(0, 50, "ca", "desc", [], {
        ...context,
        managedClub: { clubName: "Riverside", status: "available" },
      }),
    );
  });

  it("uses the exact first-page key without changing IPC arguments", () => {
    const options = squadPlayersQueryOptions(0, 50, "name", "asc");

    expect(options.queryKey).toEqual(squadKeys.players(0, 50, "name", "asc"));
  });
});
