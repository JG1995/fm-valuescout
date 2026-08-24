import { describe, expect, it } from "vitest";
import { getPlayerMetric, PLAYER_METRICS } from "./player-metrics";

const EXPECTED_ATTRIBUTE_METRIC_IDS_BY_CATEGORY = {
  "visible-attributes": [
    "attr.Crossing",
    "attr.Dribbling",
    "attr.Finishing",
    "attr.Heading",
    "attr.LongShots",
    "attr.Marking",
    "attr.OffTheBall",
    "attr.Passing",
    "attr.PenaltyTaking",
    "attr.Tackling",
    "attr.Vision",
    "attr.Handling",
    "attr.AerialReach",
    "attr.CommandOfArea",
    "attr.Communication",
    "attr.Kicking",
    "attr.Throwing",
    "attr.Anticipation",
    "attr.Decisions",
    "attr.OneOnOnes",
    "attr.Positioning",
    "attr.Reflexes",
    "attr.FirstTouch",
    "attr.Technique",
    "attr.Flair",
    "attr.Corners",
    "attr.Teamwork",
    "attr.WorkRate",
    "attr.LongThrows",
    "attr.Eccentricity",
    "attr.RushingOut",
    "attr.Punching",
    "attr.Acceleration",
    "attr.FreeKicks",
    "attr.Strength",
    "attr.Stamina",
    "attr.Pace",
    "attr.JumpingReach",
    "attr.Leadership",
    "attr.Balance",
    "attr.Bravery",
    "attr.Aggression",
    "attr.Agility",
    "attr.NaturalFitness",
    "attr.Determination",
    "attr.Composure",
    "attr.Concentration",
  ],
  "hidden-attributes": [
    "hidden.Dirtiness",
    "hidden.Consistency",
    "hidden.ImportantMatches",
    "hidden.InjuryProneness",
    "hidden.Versatility",
  ],
  personality: [
    "personality.Adaptability",
    "personality.Ambition",
    "personality.Loyalty",
    "personality.Pressure",
    "personality.Professionalism",
    "personality.Sportsmanship",
    "personality.Temperament",
    "personality.Controversy",
  ],
} as const;

describe("player metric table metadata", () => {
  it("gives every selectable metric a fixed table width and alignment", () => {
    expect(getPlayerMetric("name")).toMatchObject({
      align: "left",
      defaultWidth: 224,
      sortable: true,
    });
    expect(getPlayerMetric("attr.Acceleration")).toMatchObject({
      align: "right",
      defaultWidth: 88,
      sortable: true,
    });
    expect(getPlayerMetric("potential_role.goalkeeper_ip")).toMatchObject({
      align: "right",
      defaultWidth: 112,
      sortable: true,
    });
  });

  it("registers Club DNA as a sortable and filterable fixed score metric", () => {
    expect(getPlayerMetric("club_dna")).toMatchObject({
      id: "club_dna",
      label: "Club DNA",
      category: "ability-reputation",
      kind: "integer",
      align: "right",
      defaultWidth: 88,
      sortable: true,
      operators: [
        { id: "gt", label: "greater than" },
        { id: "lt", label: "less than" },
        { id: "eq", label: "equals" },
        { id: "neq", label: "does not equal" },
      ],
    });
  });

  it("uses the complete fixed catalog in metric metadata", () => {
    for (const [category, expectedIds] of Object.entries(
      EXPECTED_ATTRIBUTE_METRIC_IDS_BY_CATEGORY,
    )) {
      expect(
        PLAYER_METRICS.filter((metric) => metric.category === category).map(
          (metric) => metric.id,
        ),
      ).toEqual(expectedIds);
    }
  });

  it.each([
    ["goalkeeper_ip", "Goalkeepers"],
    ["centre_back_ip", "Central defense"],
    ["wing_back_ip", "Full-back and wing-back"],
    ["box_to_box_midfielder_ip", "Defensive midfield"],
    ["central_midfielder_ip", "Central midfield"],
    ["inside_winger_ip", "Wide midfield and wings"],
    ["channel_midfielder_ip", "Attacking midfield"],
    ["wide_forward_ip", "Forwards"],
  ])("keeps %s in its catalog playing area", (roleId, roleFamily) => {
    expect(
      PLAYER_METRICS.find((metric) => metric.id === `role.${roleId}`),
    ).toMatchObject({
      category: "current-role-scores",
      roleFamily,
    });
    expect(
      PLAYER_METRICS.find((metric) => metric.id === `potential_role.${roleId}`),
    ).toMatchObject({
      category: "potential-role-scores",
      roleFamily,
    });
  });
});
