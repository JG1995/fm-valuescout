/** FM-style visible attribute groups — dump PascalCase keys (bridge AttributeEntries). */

export type AttributeGroupId =
  | "technical"
  | "mental"
  | "physical"
  | "goalkeeping";

export type AttributeGroup = {
  id: AttributeGroupId;
  title: string;
  keys: readonly string[];
};

export const VISIBLE_ATTRIBUTE_GROUPS: readonly AttributeGroup[] = [
  {
    id: "technical",
    title: "Technical",
    keys: [
      "Corners",
      "Crossing",
      "Dribbling",
      "Finishing",
      "FirstTouch",
      "FreeKicks",
      "Heading",
      "LongShots",
      "LongThrows",
      "Marking",
      "Passing",
      "PenaltyTaking",
      "Tackling",
      "Technique",
    ],
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

export type AttributeRow = {
  key: string;
  label: string;
  value: number | null;
  potentialValue?: number | null;
};

export type AttributeTier = 1 | 2 | 3 | 4;

const ATTRIBUTE_TIER_LABELS: Record<AttributeTier, string> = {
  1: "Weak",
  2: "Average",
  3: "Good",
  4: "Excellent",
};

/** Map FM's 1–20 attribute scale to its four familiar display bands. */
export function attributeValueTier(value: number): AttributeTier {
  if (value >= 16) return 4;
  if (value >= 11) return 3;
  if (value >= 6) return 2;
  return 1;
}

export function attributeTierLabel(tier: AttributeTier): string {
  return ATTRIBUTE_TIER_LABELS[tier];
}

export function labelFromPascal(key: string): string {
  return key.replaceAll(/([a-z])([A-Z])/g, "$1 $2");
}

/** Resolve group keys against a value map; missing keys and explicit nulls stay null. */
export function attributeRows(
  keys: readonly string[],
  values: Record<string, number | null>,
  potentialValues?: Record<string, number | null>,
): AttributeRow[] {
  return keys.map((key) => {
    const value = Object.hasOwn(values, key) ? values[key] : null;
    if (potentialValues === undefined) {
      return { key, label: labelFromPascal(key), value };
    }
    return {
      key,
      label: labelFromPascal(key),
      value,
      potentialValue: Object.hasOwn(potentialValues, key)
        ? potentialValues[key]
        : null,
    };
  });
}
