import { describe, expect, it } from "vitest";
import {
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "./player-attributes";

const EXPECTED_VISIBLE_CATALOG = [
  {
    id: "technical",
    label: "Technical",
    attributes: [
      ["attr.Crossing", "Crossing"],
      ["attr.Dribbling", "Dribbling"],
      ["attr.Finishing", "Finishing"],
      ["attr.FirstTouch", "First Touch"],
      ["attr.Heading", "Heading"],
      ["attr.LongShots", "Long Shots"],
      ["attr.Marking", "Marking"],
      ["attr.Passing", "Passing"],
      ["attr.Tackling", "Tackling"],
      ["attr.Technique", "Technique"],
    ],
    subgroups: [
      {
        label: "Set Pieces",
        attributes: [
          ["attr.Corners", "Corners"],
          ["attr.FreeKicks", "Free Kicks"],
          ["attr.LongThrows", "Long Throws"],
          ["attr.PenaltyTaking", "Penalty Taking"],
        ],
      },
    ],
  },
  {
    id: "mental",
    label: "Mental",
    attributes: [
      ["attr.Aggression", "Aggression"],
      ["attr.Anticipation", "Anticipation"],
      ["attr.Bravery", "Bravery"],
      ["attr.Composure", "Composure"],
      ["attr.Concentration", "Concentration"],
      ["attr.Decisions", "Decisions"],
      ["attr.Determination", "Determination"],
      ["attr.Flair", "Flair"],
      ["attr.Leadership", "Leadership"],
      ["attr.OffTheBall", "Off The Ball"],
      ["attr.Positioning", "Positioning"],
      ["attr.Teamwork", "Teamwork"],
      ["attr.Vision", "Vision"],
      ["attr.WorkRate", "Work Rate"],
    ],
  },
  {
    id: "physical",
    label: "Physical",
    attributes: [
      ["attr.Acceleration", "Acceleration"],
      ["attr.Agility", "Agility"],
      ["attr.Balance", "Balance"],
      ["attr.JumpingReach", "Jumping Reach"],
      ["attr.NaturalFitness", "Natural Fitness"],
      ["attr.Pace", "Pace"],
      ["attr.Stamina", "Stamina"],
      ["attr.Strength", "Strength"],
    ],
  },
  {
    id: "goalkeeping",
    label: "Goalkeeping",
    attributes: [
      ["attr.AerialReach", "Aerial Reach"],
      ["attr.CommandOfArea", "Command Of Area"],
      ["attr.Communication", "Communication"],
      ["attr.Eccentricity", "Eccentricity"],
      ["attr.Handling", "Handling"],
      ["attr.Kicking", "Kicking"],
      ["attr.OneOnOnes", "One On Ones"],
      ["attr.Punching", "Punching"],
      ["attr.Reflexes", "Reflexes"],
      ["attr.RushingOut", "Rushing Out"],
      ["attr.Throwing", "Throwing"],
    ],
  },
] as const;

const EXPECTED_HIDDEN_ATTRIBUTES = [
  ["hidden.Dirtiness", "Dirtiness"],
  ["hidden.Consistency", "Consistency"],
  ["hidden.ImportantMatches", "Important Matches"],
  ["hidden.InjuryProneness", "Injury Proneness"],
  ["hidden.Versatility", "Versatility"],
] as const;

const EXPECTED_PERSONALITY_ATTRIBUTES = [
  ["personality.Adaptability", "Adaptability"],
  ["personality.Ambition", "Ambition"],
  ["personality.Loyalty", "Loyalty"],
  ["personality.Pressure", "Pressure"],
  ["personality.Professionalism", "Professionalism"],
  ["personality.Sportsmanship", "Sportsmanship"],
  ["personality.Temperament", "Temperament"],
  ["personality.Controversy", "Controversy"],
] as const;

describe("player attributes", () => {
  it("keeps the complete canonical FM catalog, IDs, labels, and order", () => {
    expect(toCatalog(VISIBLE_ATTRIBUTE_GROUPS)).toEqual(
      EXPECTED_VISIBLE_CATALOG,
    );
    expect(toAttributes(HIDDEN_ATTRIBUTE_KEYS, "hidden")).toEqual(
      EXPECTED_HIDDEN_ATTRIBUTES,
    );
    expect(toAttributes(PERSONALITY_ATTRIBUTE_KEYS, "personality")).toEqual(
      EXPECTED_PERSONALITY_ATTRIBUTES,
    );
  });
});

function toCatalog(groups: typeof VISIBLE_ATTRIBUTE_GROUPS) {
  return groups.map((group) => ({
    id: group.id,
    label: group.title,
    attributes: toAttributes(group.keys, "attr"),
    ...(group.subgroups === undefined
      ? {}
      : {
          subgroups: group.subgroups.map((subgroup) => ({
            label: subgroup.title,
            attributes: toAttributes(subgroup.keys, "attr"),
          })),
        }),
  }));
}

function toAttributes(keys: readonly string[], prefix: string) {
  return keys.map((key) => [`${prefix}.${key}`, labelFromPascal(key)]);
}
