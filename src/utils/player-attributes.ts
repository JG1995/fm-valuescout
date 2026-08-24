export type AttributeGroupId =
  | "technical"
  | "mental"
  | "physical"
  | "goalkeeping"
  | "hidden"
  | "personality";

export type AttributeSubgroup = {
  title: string;
  keys: readonly string[];
};

export type AttributeGroup = {
  id: AttributeGroupId;
  title: string;
  keys: readonly string[];
  subgroups?: readonly AttributeSubgroup[];
};

const TECHNICAL_SET_PIECE_KEYS = [
  "Corners",
  "FreeKicks",
  "LongThrows",
  "PenaltyTaking",
] as const;

export const VISIBLE_ATTRIBUTE_GROUPS: readonly AttributeGroup[] = [
  {
    id: "technical",
    title: "Technical",
    keys: [
      "Crossing",
      "Dribbling",
      "Finishing",
      "FirstTouch",
      "Heading",
      "LongShots",
      "Marking",
      "Passing",
      "Tackling",
      "Technique",
    ],
    subgroups: [{ title: "Set Pieces", keys: TECHNICAL_SET_PIECE_KEYS }],
  },
  {
    id: "mental",
    title: "Mental",
    keys: [
      "Aggression",
      "Anticipation",
      "Bravery",
      "Composure",
      "Concentration",
      "Decisions",
      "Determination",
      "Flair",
      "Leadership",
      "OffTheBall",
      "Positioning",
      "Teamwork",
      "Vision",
      "WorkRate",
    ],
  },
  {
    id: "physical",
    title: "Physical",
    keys: [
      "Acceleration",
      "Agility",
      "Balance",
      "JumpingReach",
      "NaturalFitness",
      "Pace",
      "Stamina",
      "Strength",
    ],
  },
  {
    id: "goalkeeping",
    title: "Goalkeeping",
    keys: [
      "AerialReach",
      "CommandOfArea",
      "Communication",
      "Eccentricity",
      "Handling",
      "Kicking",
      "OneOnOnes",
      "Punching",
      "Reflexes",
      "RushingOut",
      "Throwing",
    ],
  },
] as const;

export const VISIBLE_ATTRIBUTE_KEYS = [
  "Crossing",
  "Dribbling",
  "Finishing",
  "Heading",
  "LongShots",
  "Marking",
  "OffTheBall",
  "Passing",
  "PenaltyTaking",
  "Tackling",
  "Vision",
  "Handling",
  "AerialReach",
  "CommandOfArea",
  "Communication",
  "Kicking",
  "Throwing",
  "Anticipation",
  "Decisions",
  "OneOnOnes",
  "Positioning",
  "Reflexes",
  "FirstTouch",
  "Technique",
  "Flair",
  "Corners",
  "Teamwork",
  "WorkRate",
  "LongThrows",
  "Eccentricity",
  "RushingOut",
  "Punching",
  "Acceleration",
  "FreeKicks",
  "Strength",
  "Stamina",
  "Pace",
  "JumpingReach",
  "Leadership",
  "Balance",
  "Bravery",
  "Aggression",
  "Agility",
  "NaturalFitness",
  "Determination",
  "Composure",
  "Concentration",
] as const;

export const HIDDEN_ATTRIBUTE_KEYS = [
  "Dirtiness",
  "Consistency",
  "ImportantMatches",
  "InjuryProneness",
  "Versatility",
] as const;

export const PERSONALITY_ATTRIBUTE_KEYS = [
  "Adaptability",
  "Ambition",
  "Loyalty",
  "Pressure",
  "Professionalism",
  "Sportsmanship",
  "Temperament",
  "Controversy",
] as const;

export function labelFromPascal(key: string): string {
  return key.replaceAll(/([a-z])([A-Z])/g, "$1 $2");
}
