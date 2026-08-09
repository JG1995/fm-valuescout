import type { PlayerRoleScore } from "../types/player-detail";

export type PositionFamilyId =
  | "goalkeeper"
  | "centre-back"
  | "full-back"
  | "defensive-midfield"
  | "central-midfield"
  | "wide-midfield"
  | "attacking-midfield"
  | "striker";

export type PositionFamily = {
  id: PositionFamilyId;
  title: string;
  tags: readonly string[];
};

/** Ordered pitch groups — primary family is the first matching tag in this order. */
export const POSITION_FAMILIES: readonly PositionFamily[] = [
  { id: "goalkeeper", title: "Goalkeeper", tags: ["GK"] },
  { id: "centre-back", title: "Centre-back", tags: ["DC"] },
  {
    id: "full-back",
    title: "Full-back / Wing-back",
    tags: ["DL", "DR", "WBL", "WBR"],
  },
  { id: "defensive-midfield", title: "Defensive midfield", tags: ["DM"] },
  { id: "central-midfield", title: "Central midfield", tags: ["MC"] },
  {
    id: "wide-midfield",
    title: "Wide midfield / Winger",
    tags: ["ML", "MR", "AML", "AMR"],
  },
  { id: "attacking-midfield", title: "Attacking midfield", tags: ["AMC"] },
  { id: "striker", title: "Striker", tags: ["ST"] },
] as const;

const TAG_TO_FAMILY = new Map<string, PositionFamily>();
for (const family of POSITION_FAMILIES) {
  for (const tag of family.tags) {
    TAG_TO_FAMILY.set(tag, family);
  }
}

export type RoleFamilyGroup = {
  family: PositionFamily;
  roles: PlayerRoleScore[];
};

export type ScoredRole = PlayerRoleScore & { score: number };
export type PotentialScoredRole = PlayerRoleScore & { potentialScore: number };

/** Resolve primary family from the first position tag that maps to a family. */
export function primaryFamily(
  positionTags: readonly string[],
): PositionFamily | null {
  for (const tag of positionTags) {
    const family = TAG_TO_FAMILY.get(tag);
    if (family) {
      return family;
    }
  }
  return null;
}

/**
 * Group role scores by position family. Preserves input (catalog) order within
 * each family. Roles with no known tags are omitted.
 */
export function groupRolesByFamily(
  roleScores: readonly PlayerRoleScore[],
): RoleFamilyGroup[] {
  const buckets = new Map<PositionFamilyId, PlayerRoleScore[]>();

  for (const role of roleScores) {
    const family = primaryFamily(role.positionTags);
    if (!family) {
      continue;
    }
    const list = buckets.get(family.id);
    if (list) {
      list.push(role);
    } else {
      buckets.set(family.id, [role]);
    }
  }

  return POSITION_FAMILIES.flatMap((family) => {
    const roles = buckets.get(family.id);
    return roles && roles.length > 0 ? [{ family, roles }] : [];
  });
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
