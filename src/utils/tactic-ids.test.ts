import { describe, expect, it } from "vitest";
import rustTacticSource from "../../src-tauri/src/features/planner/tactic.rs?raw";
import { TACTIC_LANE_IDS as PLANNER_TACTIC_LANE_IDS } from "../features/planner/types/tactic";
import {
  allTacticColumnIdsForGroup,
  isFullTacticGroup,
  isTacticColumnId,
  isValidTacticColumnId,
  sanitizeTacticIds,
  TACTIC_COLUMN_DEFAULT_WIDTH,
  TACTIC_CURRENT_PREFIX,
  TACTIC_LANE_IDS,
  TACTIC_POTENTIAL_PREFIX,
  tacticColumnId,
  tacticGroupForId,
  tacticLaneIdForId,
} from "./tactic-ids";

describe("tactic column IDs", () => {
  it("matches the Rust default lane IDs and Planner re-export exactly", () => {
    const rustLaneBlock = rustTacticSource.match(
      /DEFAULT_LANE_IDS:[^=]+=[\s\S]*?\[([\s\S]*?)\];/,
    )?.[1];
    expect(rustLaneBlock).toBeDefined();

    const rustLaneIds = [...(rustLaneBlock ?? "").matchAll(/"([^"]+)"/g)].map(
      ([, laneId]) => laneId,
    );

    expect(TACTIC_LANE_IDS).toHaveLength(11);
    expect(TACTIC_LANE_IDS).toEqual(rustLaneIds);
    expect(PLANNER_TACTIC_LANE_IDS).toBe(TACTIC_LANE_IDS);
  });

  it("recognizes tactic prefixes but validates only canonical lane suffixes", () => {
    expect(isTacticColumnId("tactic_current.goalkeeper")).toBe(true);
    expect(isTacticColumnId("tactic_potential.not_a_lane")).toBe(true);
    expect(isTacticColumnId("role.goalkeeper_ip")).toBe(false);
    expect(isValidTacticColumnId("tactic_current.goalkeeper")).toBe(true);
    expect(isValidTacticColumnId("tactic_potential.right_winger")).toBe(true);
    expect(isValidTacticColumnId("tactic_current.not_a_lane")).toBe(false);
    expect(isValidTacticColumnId("tactic_current.")).toBe(false);
  });

  it("extracts groups and lane IDs without accepting other column families", () => {
    expect(tacticGroupForId("tactic_current.left_back")).toBe("current");
    expect(tacticGroupForId("tactic_potential.left_back")).toBe("potential");
    expect(tacticGroupForId("role.left_back")).toBeNull();
    expect(tacticLaneIdForId("tactic_current.left_back")).toBe("left_back");
    expect(tacticLaneIdForId("role.left_back")).toBeNull();
  });

  it("requires all 11 canonical IDs for a full group", () => {
    const currentIds = allTacticColumnIdsForGroup("current", TACTIC_LANE_IDS);

    expect(isFullTacticGroup(currentIds.slice(0, 10), "current")).toBe(false);
    expect(isFullTacticGroup(currentIds, "current")).toBe(true);
    expect(isFullTacticGroup(currentIds, "potential")).toBe(false);
  });

  it("filters invalid and non-tactic IDs while preserving valid order", () => {
    expect(
      sanitizeTacticIds([
        "name",
        "tactic_potential.right_winger",
        "tactic_current.not_a_lane",
        "tactic_current.goalkeeper",
      ]),
    ).toEqual(["tactic_potential.right_winger", "tactic_current.goalkeeper"]);
  });

  it("keeps tactic prefixes separate from catalog column families", () => {
    const catalogPrefixes = [
      "role.",
      "potential_role.",
      "moneyball_role.",
      "moneyball.",
      "attr.",
      "pos.",
    ];

    expect(TACTIC_CURRENT_PREFIX).not.toBe(TACTIC_POTENTIAL_PREFIX);
    for (const catalogPrefix of catalogPrefixes) {
      expect(TACTIC_CURRENT_PREFIX.startsWith(catalogPrefix)).toBe(false);
      expect(TACTIC_POTENTIAL_PREFIX.startsWith(catalogPrefix)).toBe(false);
    }
    expect(tacticColumnId("current", "goalkeeper")).toBe(
      "tactic_current.goalkeeper",
    );
    expect(TACTIC_COLUMN_DEFAULT_WIDTH).toBe(112);
  });

  it("does not construct synthetic IDs for unknown WebView lanes", () => {
    expect(tacticColumnId("current", "not_a_lane")).toBeNull();
    expect(tacticColumnId("potential", "")).toBeNull();
    expect(tacticColumnId("current", "goalkeeper ")).toBeNull();

    expect(allTacticColumnIdsForGroup("current", ["not_a_lane"])).toEqual([]);
    expect(
      allTacticColumnIdsForGroup("potential", [
        "goalkeeper",
        "not_a_lane",
        "right_winger",
      ]),
    ).toEqual(["tactic_potential.goalkeeper", "tactic_potential.right_winger"]);
    expect(
      allTacticColumnIdsForGroup("current", ["not_a_lane", "also_bad"]),
    ).toEqual([]);
  });
});
