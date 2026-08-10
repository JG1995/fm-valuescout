export type ScoreTier = 1 | 2 | 3 | 4;

const TIER_LABELS: Record<ScoreTier, string> = {
  1: "Weak",
  2: "Average",
  3: "Good",
  4: "Excellent",
};

/** Map a 0–100 role score to the DESIGN.md score-ramp tier. */
export function scoreToTier(score: number): ScoreTier {
  if (score >= 81) return 4;
  if (score >= 61) return 3;
  if (score >= 41) return 2;
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
