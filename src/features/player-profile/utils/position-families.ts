import {
  PLAYABLE_POSITION_FAMILIARITY,
  type PositionFamiliarity,
  type PositionFamiliarityMap,
  type PositionRoleScore,
} from "@/utils/profile-position-roles";
import type { PlayerRoleScore } from "../types/player-detail";

export type {
  PositionFamiliarity,
  PositionFamiliarityMap,
  PositionRoleScore,
} from "@/utils/profile-position-roles";
export {
  defaultProfilePosition,
  isGoalkeeper,
  PLAYABLE_POSITION_FAMILIARITY,
  PROFILE_POSITION_ROWS,
  PROFILE_POSITION_TAGS,
  rolesForScorePosition,
} from "@/utils/profile-position-roles";

export type ScoredRole<T extends PositionRoleScore = PositionRoleScore> = T & {
  score: number;
};
export type PotentialScoredRole = PlayerRoleScore & { potentialScore: number };
export type RolePhase = "in_possession" | "out_of_possession";
export type RoleSort = {
  basis: "current" | "potential";
  direction: "ascending" | "descending";
};

const DEFAULT_ROLE_SORT: RoleSort = {
  basis: "current",
  direction: "descending",
};

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
export function rolesForPlayablePositions<T extends PositionRoleScore>(
  roleScores: readonly T[],
  positions: PositionFamiliarityMap,
): T[] {
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
export function rolesForPhase<T extends PositionRoleScore>(
  roleScores: readonly T[],
  phase: RolePhase,
): T[] {
  return roleScores.filter((role) => role.phase === phase);
}

/** Highest non-null score; ties keep the earlier catalog entry. */
export function bestRoleScore<T extends PositionRoleScore>(
  roleScores: readonly T[],
): ScoredRole<T> | null {
  let best: ScoredRole<T> | null = null;
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
