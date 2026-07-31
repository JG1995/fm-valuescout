export type ScoreTier = 1 | 2 | 3 | 4 | 5;

const TIER_LABELS: Record<ScoreTier, string> = {
  1: "Weak",
  2: "Fringe",
  3: "Rotation",
  4: "Starter",
  5: "Elite",
};

/** Map a 0–100 role score to the DESIGN.md score-ramp tier. */
export function scoreToTier(score: number): ScoreTier {
  if (score >= 85) return 5;
  if (score >= 70) return 4;
  if (score >= 55) return 3;
  if (score >= 40) return 2;
  return 1;
}

export function tierLabel(tier: ScoreTier): string {
  return TIER_LABELS[tier];
}

export function scoreBadgeAccessibleName(
  roleName: string,
  score: number,
): string {
  const label = tierLabel(scoreToTier(score));
  return `${roleName}: ${score}, ${label}`;
}
