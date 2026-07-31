import { cn } from "@/utils/cn";
import {
  type ScoreTier,
  scoreBadgeAccessibleName,
  scoreToTier,
  tierLabel,
} from "./score-tier";

export type ScoreBadgeVariant = "table" | "card" | "hero" | "muted";

type ScoreBadgeProps = {
  score: number;
  roleName: string;
  variant?: ScoreBadgeVariant;
  className?: string;
};

const tierTextClass: Record<ScoreTier, string> = {
  1: "text-score-1",
  2: "text-score-2",
  3: "text-score-3",
  4: "text-score-4",
  5: "text-score-5",
};

const tierBorderClass: Record<ScoreTier, string> = {
  1: "border-score-1/40",
  2: "border-score-2/40",
  3: "border-score-3/40",
  4: "border-score-4/40",
  5: "border-score-5/40",
};

const variantClasses: Record<ScoreBadgeVariant, string> = {
  table: "font-mono text-mono-sm tabular-nums text-right",
  card: "inline-flex size-7 items-center justify-center rounded-full border bg-surface-container-high font-mono text-mono-md tabular-nums",
  hero: "inline-flex size-12 items-center justify-center font-mono text-mono-lg tabular-nums",
  muted:
    "inline-flex size-7 items-center justify-center rounded-full border border-outline-variant bg-surface-container-high font-mono text-mono-md text-on-surface-variant tabular-nums",
};

export function ScoreBadge({
  score,
  roleName,
  variant = "table",
  className,
}: ScoreBadgeProps) {
  const tier = scoreToTier(score);
  const label = tierLabel(tier);
  const accessibleName = scoreBadgeAccessibleName(roleName, score);
  const coloured = variant === "muted" ? undefined : tierTextClass[tier];
  const bordered = variant === "card" ? tierBorderClass[tier] : undefined;

  return (
    <span
      role="img"
      className={cn(variantClasses[variant], coloured, bordered, className)}
      title={label}
      aria-label={accessibleName}
    >
      {score}
    </span>
  );
}
