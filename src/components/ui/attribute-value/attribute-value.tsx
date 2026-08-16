import { formatMissable } from "@/utils/format";

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

export function AttributeValue({
  value,
}: {
  value: number | null | undefined;
}) {
  if (value === null || value === undefined) {
    return (
      <span className="text-on-surface-variant">{formatMissable(value)}</span>
    );
  }

  const tier = attributeValueTier(value);
  return (
    <span
      data-tier={tier}
      title={attributeTierLabel(tier)}
      className="inline-flex min-w-7 justify-center rounded-sm bg-surface-container-high px-1.5 py-0.5 data-[tier=1]:bg-score-1/10 data-[tier=1]:text-score-1 data-[tier=2]:bg-score-2/10 data-[tier=2]:text-score-2 data-[tier=3]:bg-score-3/10 data-[tier=3]:text-score-3 data-[tier=4]:bg-score-4/10 data-[tier=4]:text-score-4"
    >
      {value}
    </span>
  );
}
