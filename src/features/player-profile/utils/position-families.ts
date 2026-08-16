import type { PlayerRoleScore } from "../types/player-detail";

export const PROFILE_POSITION_ROWS: readonly (readonly (string | null)[])[] = [
  [null, "ST", null],
  ["AML", "AMC", "AMR"],
  ["ML", "MC", "MR"],
  ["WBL", "DM", "WBR"],
  ["DL", "DC", "DR"],
  [null, "SW", null],
  [null, "GK", null],
] as const;

export const PROFILE_POSITION_TAGS = PROFILE_POSITION_ROWS.flatMap((row) =>
  row.filter((position): position is string => position !== null),
);

const PROFILE_POSITION_TAG_SET = new Set(PROFILE_POSITION_TAGS);

export type ScoredRole = PlayerRoleScore & { score: number };
export type PotentialScoredRole = PlayerRoleScore & { potentialScore: number };
export type PositionFamiliarity = number | null;
export type PositionFamiliarityMap = Readonly<
  Record<string, PositionFamiliarity>
>;
export type RolePhase = "in_possession" | "out_of_possession";
export type RoleSort = {
  basis: "current" | "potential";
  direction: "ascending" | "descending";
};

const DEFAULT_ROLE_SORT: RoleSort = {
  basis: "current",
  direction: "descending",
};

export const PLAYABLE_POSITION_FAMILIARITY = 15;

export function isGoalkeeper(positions: PositionFamiliarityMap): boolean {
  return (
    typeof positions.GK === "number" &&
    positions.GK >= PLAYABLE_POSITION_FAMILIARITY
  );
}

/** Pick the player's strongest recorded position, then fall back to best-role fit. */
export function defaultProfilePosition(
  positions: PositionFamiliarityMap,
  roleScores: readonly PlayerRoleScore[],
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

  if (selected) {
    return selected;
  }

  const bestRole = bestRoleScore(roleScores);
  const bestRolePosition = bestRole?.positionTags.find((position) =>
    PROFILE_POSITION_TAG_SET.has(position),
  );
  if (bestRolePosition) {
    return bestRolePosition;
  }

  for (const role of roleScores) {
    const rolePosition = role.positionTags.find((position) =>
      PROFILE_POSITION_TAG_SET.has(position),
    );
    if (rolePosition) {
      return rolePosition;
    }
  }

  return "MC";
}

/** Filter to an exact pitch position and rank known scores before unavailable ones. */
export function rolesForProfilePosition(
  roleScores: readonly PlayerRoleScore[],
  position: string,
  sort: RoleSort = DEFAULT_ROLE_SORT,
): PlayerRoleScore[] {
  return roleScores
    .filter((role) => role.positionTags.includes(position))
    .map((role, index) => ({ role, index }))
    .sort((left, right) => {
      const leftScore =
        sort.basis === "current" ? left.role.score : left.role.potentialScore;
      const rightScore =
        sort.basis === "current" ? right.role.score : right.role.potentialScore;
      if (leftScore === null)
        return rightScore === null ? left.index - right.index : 1;
      if (rightScore === null) return -1;
      const scoreDifference =
        sort.direction === "descending"
          ? rightScore - leftScore
          : leftScore - rightScore;
      return scoreDifference || left.index - right.index;
    })
    .map(({ role }) => role);
}

/** Keep roles attached to at least one position the player can play. */
export function rolesForPlayablePositions(
  roleScores: readonly PlayerRoleScore[],
  positions: PositionFamiliarityMap,
): PlayerRoleScore[] {
  return roleScores.filter((role) =>
    role.positionTags.some(
      (position) =>
        isPositiveFamiliarity(positions[position]) &&
        positions[position] >= PLAYABLE_POSITION_FAMILIARITY,
    ),
  );
}

function isPositiveFamiliarity(
  value: PositionFamiliarity | undefined,
): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

/** Keep only roles from the requested catalog phase. */
export function rolesForPhase(
  roleScores: readonly PlayerRoleScore[],
  phase: RolePhase,
): PlayerRoleScore[] {
  return roleScores.filter((role) => role.phase === phase);
}

/** Highest non-null score; ties keep the earlier catalog entry. */
export function bestRoleScore(
  roleScores: readonly PlayerRoleScore[],
): ScoredRole | null {
  let best: ScoredRole | null = null;
  for (const role of roleScores) {
    if (role.score === null) {
      continue;
    }
    if (best === null || role.score > best.score) {
      best = { ...role, score: role.score };
    }
  }
  return best;
}

/** Highest non-null potential score; ties keep the earlier catalog entry. */
export function bestPotentialRoleScore(
  roleScores: readonly PlayerRoleScore[],
): PotentialScoredRole | null {
  let best: PotentialScoredRole | null = null;
  for (const role of roleScores) {
    if (role.potentialScore === null) {
      continue;
    }
    if (best === null || role.potentialScore > best.potentialScore) {
      best = { ...role, potentialScore: role.potentialScore };
    }
  }
  return best;
}
