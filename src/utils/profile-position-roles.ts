export const PROFILE_POSITION_ROWS: readonly (readonly (string | null)[])[] = [
  [null, "ST", null],
  ["AML", "AMC", "AMR"],
  ["ML", "MC", "MR"],
  ["WBL", "DM", "WBR"],
  ["DL", "DC", "DR"],
  [null, "GK", null],
] as const;

export const PROFILE_POSITION_TAGS = PROFILE_POSITION_ROWS.flatMap((row) =>
  row.filter((position): position is string => position !== null),
);

const PROFILE_POSITION_TAG_SET = new Set(PROFILE_POSITION_TAGS);

export type PositionRoleScore = {
  roleId: string;
  displayName: string;
  phase: string;
  positionTags: string[];
  score: number | null;
};
export type PositionFamiliarity = number | null;
export type PositionFamiliarityMap = Readonly<
  Record<string, PositionFamiliarity>
>;

export const PLAYABLE_POSITION_FAMILIARITY = 15;

export function isGoalkeeper(positions: PositionFamiliarityMap): boolean {
  return (
    typeof positions.GK === "number" &&
    positions.GK >= PLAYABLE_POSITION_FAMILIARITY
  );
}

/** Pick the strongest recorded position, then fall back to the best role score. */
export function defaultProfilePosition(
  positions: PositionFamiliarityMap,
  roleScores: readonly PositionRoleScore[],
): string {
  let selected: string | null = null;
  let familiarity = 0;

  for (const position of PROFILE_POSITION_TAGS) {
    const value = positions[position];
    if (isPositiveFamiliarity(value) && value > familiarity) {
      selected = position;
      familiarity = value;
    }
  }

  if (selected) return selected;

  const bestRole = roleScores.reduce<PositionRoleScore | null>((best, role) => {
    if (role.score === null) return best;
    return best === null || best.score === null || role.score > best.score
      ? role
      : best;
  }, null);
  const bestRolePosition = bestRole?.positionTags.find((position) =>
    PROFILE_POSITION_TAG_SET.has(position),
  );
  if (bestRolePosition) return bestRolePosition;

  for (const role of roleScores) {
    const rolePosition = role.positionTags.find((position) =>
      PROFILE_POSITION_TAG_SET.has(position),
    );
    if (rolePosition) return rolePosition;
  }

  return "MC";
}

/** Filter to an exact pitch position and rank one score column with nulls last. */
export function rolesForScorePosition<T extends PositionRoleScore>(
  roleScores: readonly T[],
  position: string,
  direction: "ascending" | "descending" = "descending",
): T[] {
  return roleScores
    .filter((role) => role.positionTags.includes(position))
    .map((role, index) => ({ role, index }))
    .sort((left, right) => {
      if (left.role.score === null) {
        return right.role.score === null ? left.index - right.index : 1;
      }
      if (right.role.score === null) return -1;
      const scoreDifference =
        direction === "descending"
          ? right.role.score - left.role.score
          : left.role.score - right.role.score;
      return scoreDifference || left.index - right.index;
    })
    .map(({ role }) => role);
}

function isPositiveFamiliarity(
  value: PositionFamiliarity | undefined,
): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}
