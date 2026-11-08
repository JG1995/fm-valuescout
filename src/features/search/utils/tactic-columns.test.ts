import { describe, expect, it } from "vitest";
import {
  TACTIC_CURRENT_PREFIX,
  TACTIC_CUSTOM_PREFIX,
  TACTIC_POTENTIAL_PREFIX,
} from "@/utils/tactic-ids";
import {
  buildTacticColumnOrder,
  orderedLaneIdsForColumns,
} from "./tactic-columns";

const shuffledLanes = [
  { laneId: "left_back", ipPosition: "DL", oopPosition: "ML" },
  {
    laneId: "left_centre_back",
    ipPosition: "DC",
    oopPosition: "DCL",
  },
  { laneId: "goalkeeper", ipPosition: "GK", oopPosition: "GK" },
  {
    laneId: "right_centre_back",
    ipPosition: "DC",
    oopPosition: "DCR",
  },
  { laneId: "right_back", ipPosition: "DR", oopPosition: "MR" },
] as const;

const orderedLaneIds = [
  "goalkeeper",
  "right_back",
  "right_centre_back",
  "left_centre_back",
  "left_back",
];

describe("tactic column ordering", () => {
  it("orders lanes by IP position and uses OOP position as the tie-breaker", () => {
    expect(orderedLaneIdsForColumns(shuffledLanes)).toEqual(orderedLaneIds);
  });

  it("builds deterministic straight orders for one active group", () => {
    expect(buildTacticColumnOrder(orderedLaneIds, ["current"])).toEqual(
      orderedLaneIds.map((laneId) => `${TACTIC_CURRENT_PREFIX}${laneId}`),
    );
    expect(buildTacticColumnOrder(orderedLaneIds, ["potential"])).toEqual(
      orderedLaneIds.map((laneId) => `${TACTIC_POTENTIAL_PREFIX}${laneId}`),
    );
    expect(buildTacticColumnOrder(orderedLaneIds, ["custom"])).toEqual(
      orderedLaneIds.map((laneId) => `${TACTIC_CUSTOM_PREFIX}${laneId}`),
    );
    expect(buildTacticColumnOrder(orderedLaneIds, [])).toEqual([]);
  });

  it("interleaves the active groups per lane in view order", () => {
    expect(
      buildTacticColumnOrder(orderedLaneIds, ["current", "potential"]),
    ).toEqual(
      orderedLaneIds.flatMap((laneId) => [
        `${TACTIC_CURRENT_PREFIX}${laneId}`,
        `${TACTIC_POTENTIAL_PREFIX}${laneId}`,
      ]),
    );
    expect(
      buildTacticColumnOrder(orderedLaneIds, ["current", "custom"]),
    ).toEqual(
      orderedLaneIds.flatMap((laneId) => [
        `${TACTIC_CURRENT_PREFIX}${laneId}`,
        `${TACTIC_CUSTOM_PREFIX}${laneId}`,
      ]),
    );
  });

  it("does not emit synthetic IDs for unknown lanes", () => {
    expect(buildTacticColumnOrder(["not_a_lane"], ["current"])).toEqual([]);
    expect(
      buildTacticColumnOrder(["not_a_lane"], ["current", "custom"]),
    ).toEqual([]);
    expect(
      buildTacticColumnOrder(
        ["goalkeeper", "not_a_lane", "right_back"],
        ["current", "potential"],
      ),
    ).toEqual([
      "tactic_current.goalkeeper",
      "tactic_potential.goalkeeper",
      "tactic_current.right_back",
      "tactic_potential.right_back",
    ]);
  });
});
