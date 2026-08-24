import {
  type AttributeGroup,
  type AttributeGroupId,
  type AttributeSubgroup,
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "@/utils/player-attributes";

export type { AttributeGroup, AttributeGroupId, AttributeSubgroup };
export {
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
};

export const OUTFIELD_ATTRIBUTE_GROUPS = VISIBLE_ATTRIBUTE_GROUPS.slice(0, 3);
export const GOALKEEPING_ATTRIBUTE_GROUP = VISIBLE_ATTRIBUTE_GROUPS[3];

const GOALKEEPER_BALL_PLAYING_ATTRIBUTE_KEYS: readonly string[] = [
  "FirstTouch",
  "Passing",
  "Technique",
] as const;

export const GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS =
  OUTFIELD_ATTRIBUTE_GROUPS.slice(0, 1).map((group) => ({
    ...group,
    keys: group.keys.filter(
      (key) => !GOALKEEPER_BALL_PLAYING_ATTRIBUTE_KEYS.includes(key),
    ),
  }));

const GOALKEEPER_PRIMARY_ATTRIBUTE_GROUP: AttributeGroup = {
  ...GOALKEEPING_ATTRIBUTE_GROUP,
  keys: [
    "AerialReach",
    "CommandOfArea",
    "Communication",
    "Eccentricity",
    "FirstTouch",
    "Handling",
    "Kicking",
    "OneOnOnes",
    "Passing",
    "Punching",
    "Reflexes",
    "RushingOut",
    "Technique",
    "Throwing",
  ],
};

export const GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS = [
  GOALKEEPER_PRIMARY_ATTRIBUTE_GROUP,
  OUTFIELD_ATTRIBUTE_GROUPS[1],
  OUTFIELD_ATTRIBUTE_GROUPS[2],
] as const;

export type AttributeRow = {
  key: string;
  label: string;
  value: number | null;
  potentialValue?: number | null;
};

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
