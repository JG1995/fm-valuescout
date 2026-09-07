import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import { PLAYABLE_POSITION_FAMILIARITY } from "@/utils/profile-position-roles";
import type { MoneyballRoleScore } from "../types/moneyball-profile";

type MoneyballRoleSummaryItemProps = {
  label: "Moneyball IP" | "Moneyball OOP";
  role: MoneyballRoleScore | null;
};

function MoneyballRoleSummaryItem({
  label,
  role,
}: MoneyballRoleSummaryItemProps) {
  const score = role?.score ?? null;

  return (
    <div className="flex min-w-0 items-start gap-3">
      {score === null ? (
        <span
          role="img"
          aria-label={`${label}: unavailable`}
          className="inline-flex size-12 items-center justify-center font-mono text-mono-lg text-on-surface-variant tabular-nums"
        >
          {formatMissable(null)}
        </span>
      ) : (
        <ScoreBadge score={score} roleName={label} variant="hero" />
      )}
      <div className="min-w-0">
        <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
          {label}
        </p>
        <p
          className="truncate text-body-md text-on-surface"
          title={role?.displayName}
        >
          {role?.displayName ?? formatMissable(null)}
        </p>
      </div>
    </div>
  );
}

function bestRoleForPhase(
  roleScores: readonly MoneyballRoleScore[],
  phase: "in_possession" | "out_of_possession",
) {
  let best: MoneyballRoleScore | null = null;

  for (const role of roleScores) {
    if (role.phase !== phase || role.score === null) continue;
    if (best === null || role.score > (best.score ?? -1)) best = role;
  }

  return best;
}

export function MoneyballRoleSummary({
  positions,
  roleScores,
}: {
  positions: Readonly<Record<string, number | null>>;
  roleScores: readonly MoneyballRoleScore[] | null;
}) {
  const playableRoleScores = (roleScores ?? []).filter((role) =>
    role.positionTags.some((position) => {
      const familiarity = positions[position];
      return (
        typeof familiarity === "number" &&
        Number.isFinite(familiarity) &&
        familiarity >= PLAYABLE_POSITION_FAMILIARITY
      );
    }),
  );

  return (
    <section
      aria-label="Moneyball tactical summaries"
      data-testid="moneyball-role-summaries"
      className="rounded-lg border border-outline-variant bg-surface-container px-4 py-3"
    >
      <h2 className="text-headline-sm text-on-surface">
        Moneyball tactical summaries
      </h2>
      <div className="mt-3 grid min-w-0 grid-cols-2 gap-3">
        <MoneyballRoleSummaryItem
          label="Moneyball IP"
          role={bestRoleForPhase(playableRoleScores, "in_possession")}
        />
        <MoneyballRoleSummaryItem
          label="Moneyball OOP"
          role={bestRoleForPhase(playableRoleScores, "out_of_possession")}
        />
      </div>
    </section>
  );
}
