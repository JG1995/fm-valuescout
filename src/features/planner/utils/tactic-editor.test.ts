import { describe, expect, it } from "vitest";
import type { TacticLane, TacticOptions } from "../types/tactic";
import { swapPhasePlacement } from "./tactic-editor";

function lane(
  laneId: string,
  ipPosition: string,
  ipRoleId: string,
  oopPosition = "MC",
  oopRoleId = "pressing_central_midfielder_oop",
  overrides: Partial<TacticLane> = {},
): TacticLane {
  return {
    laneId,
    ipWeight: 0.5,
    importanceRank: null,
    preferredFoot: "any",
    footPreference: "preferred",
    ipPosition,
    ipRoleId,
    oopPosition,
    oopRoleId,
    ...overrides,
  };
}

const options: TacticOptions = {
  placements: ["MCR", "MC", "MCL", "DM", "ML", "ST", "STC", "STCL"],
  roles: [
    {
      roleId: "central_midfielder_ip",
      displayName: "Central Midfielder",
      phase: "in_possession",
      positionTags: ["MC"],
    },
    {
      roleId: "advanced_playmaker_ip",
      displayName: "Advanced Playmaker",
      phase: "in_possession",
      positionTags: ["MC"],
    },
    {
      roleId: "box_to_box_midfielder_ip",
      displayName: "Box-to-Box Midfielder",
      phase: "in_possession",
      positionTags: ["DM", "MC"],
    },
    {
      roleId: "centre_forward_ip",
      displayName: "Centre Forward",
      phase: "in_possession",
      positionTags: ["ST"],
    },
    {
      roleId: "pressing_central_midfielder_oop",
      displayName: "Pressing Central Midfielder",
      phase: "out_of_possession",
      positionTags: ["MC"],
    },
  ],
};

describe("swapPhasePlacement", () => {
  it("applies an unoccupied placement directly", () => {
    const lanes = [
      lane("a", "MCR", "central_midfielder_ip"),
      lane("b", "MCL", "central_midfielder_ip"),
    ];

    const next = swapPhasePlacement(lanes, "a", "ip", "MC", options);

    expect(next.find((candidate) => candidate.laneId === "a")).toMatchObject({
      ipPosition: "MC",
      ipRoleId: "central_midfielder_ip",
    });
    expect(next.find((candidate) => candidate.laneId === "b")).toMatchObject({
      ipPosition: "MCL",
      ipRoleId: "central_midfielder_ip",
    });
  });

  it("treats legacy ST and canonical STC as the same placement", () => {
    const lanes = [
      lane("a", "ST", "centre_forward_ip"),
      lane("b", "STCL", "centre_forward_ip"),
    ];

    const next = swapPhasePlacement(lanes, "b", "ip", "STC", options);

    expect(next.find((candidate) => candidate.laneId === "b")).toMatchObject({
      ipPosition: "STC",
      ipRoleId: "centre_forward_ip",
    });
    expect(next.find((candidate) => candidate.laneId === "a")).toMatchObject({
      ipPosition: "STCL",
      ipRoleId: "centre_forward_ip",
    });
  });

  it("swaps exactly the two lanes placements in the edited phase only", () => {
    const lanes = [
      lane("a", "MCR", "central_midfielder_ip", "MCR"),
      lane("b", "MCL", "central_midfielder_ip", "MCL"),
      lane("c", "MC", "advanced_playmaker_ip", "MC"),
    ];

    const next = swapPhasePlacement(lanes, "a", "ip", "MCL", options);

    expect(next.find((candidate) => candidate.laneId === "a")?.ipPosition).toBe(
      "MCL",
    );
    expect(next.find((candidate) => candidate.laneId === "b")?.ipPosition).toBe(
      "MCR",
    );
    expect(next.find((candidate) => candidate.laneId === "c")?.ipPosition).toBe(
      "MC",
    );
    for (const candidate of next) {
      expect(candidate.oopPosition).toBe(
        lanes.find((original) => original.laneId === candidate.laneId)
          ?.oopPosition,
      );
    }
    expect(next.map((candidate) => candidate.laneId)).toEqual(["a", "b", "c"]);
  });

  it("keeps non-placement settings on their original lanes", () => {
    const lanes = [
      lane(
        "a",
        "MCR",
        "central_midfielder_ip",
        "MCR",
        "pressing_central_midfielder_oop",
        {
          ipWeight: 0.8,
          importanceRank: 2,
          preferredFoot: "left",
          footPreference: "strict",
        },
      ),
      lane(
        "b",
        "MCL",
        "central_midfielder_ip",
        "MCL",
        "pressing_central_midfielder_oop",
        {
          ipWeight: 0.3,
          importanceRank: 5,
          preferredFoot: "right",
          footPreference: "preferred",
        },
      ),
    ];

    const next = swapPhasePlacement(lanes, "a", "ip", "MCL", options);

    expect(next.find((candidate) => candidate.laneId === "a")).toMatchObject({
      ipPosition: "MCL",
      ipWeight: 0.8,
      importanceRank: 2,
      preferredFoot: "left",
      footPreference: "strict",
    });
    expect(next.find((candidate) => candidate.laneId === "b")).toMatchObject({
      ipPosition: "MCR",
      ipWeight: 0.3,
      importanceRank: 5,
      preferredFoot: "right",
      footPreference: "preferred",
    });
  });

  it("preserves compatible roles on both swapped lanes", () => {
    const lanes = [
      lane("a", "MCR", "central_midfielder_ip"),
      lane("b", "MCL", "advanced_playmaker_ip"),
    ];

    const next = swapPhasePlacement(lanes, "a", "ip", "MCL", options);

    expect(next.find((candidate) => candidate.laneId === "a")?.ipRoleId).toBe(
      "central_midfielder_ip",
    );
    expect(next.find((candidate) => candidate.laneId === "b")?.ipRoleId).toBe(
      "advanced_playmaker_ip",
    );
  });

  it("clears an incompatible role on the affected lane only", () => {
    const lanes = [
      lane("a", "MC", "advanced_playmaker_ip"),
      lane("b", "DM", "box_to_box_midfielder_ip"),
    ];

    const next = swapPhasePlacement(lanes, "a", "ip", "DM", options);

    expect(next.find((candidate) => candidate.laneId === "a")).toMatchObject({
      ipPosition: "DM",
      ipRoleId: "",
    });
    expect(next.find((candidate) => candidate.laneId === "b")).toMatchObject({
      ipPosition: "MC",
      ipRoleId: "box_to_box_midfielder_ip",
    });
  });
});
