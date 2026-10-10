import type { ReactNode } from "react";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable, formatMoney } from "@/utils/format";
import type { PlayerDetail, PlayerRoleScore } from "../types/player-detail";
import {
  bestCurrentRolePair,
  type RolePhase,
} from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";

function OverviewFact({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="min-w-0">
      <dt className="text-body-sm text-on-surface-variant">{label}</dt>
      <dd className="mt-1 font-mono text-mono-lg text-on-surface tabular-nums">
        {value}
      </dd>
    </div>
  );
}

type TacticalFitPairProps = {
  phase: RolePhase;
  pair: (PlayerRoleScore & { score: number }) | null;
  concealed: boolean;
};

function TacticalFitPair({ phase, pair, concealed }: TacticalFitPairProps) {
  const fullPhase =
    phase === "in_possession" ? "In possession" : "Out of possession";
  const testId =
    phase === "in_possession"
      ? "overview-tactical-fit-ip"
      : "overview-tactical-fit-oop";
  const summaryLabel =
    phase === "in_possession"
      ? "Best In-Possession Role"
      : "Best Out-of-Possession Role";
  const roleName = pair?.displayName ?? null;
  const currentScore = pair?.score ?? null;
  const potentialScore = concealed ? null : (pair?.potentialScore ?? null);
  const labelBase = roleName ?? fullPhase;
  const unavailableTextClass = "font-mono text-mono-lg text-on-surface-variant";
  const currentText =
    currentScore === null ? "unavailable" : formatMissable(currentScore);
  const potentialText = concealed
    ? "concealed"
    : potentialScore === null
      ? "unavailable"
      : formatMissable(potentialScore);
  const accessibleDescription =
    roleName === null
      ? `${fullPhase}: Current ${currentText}, Potential ${potentialText}`
      : `${labelBase}, ${fullPhase}: Current ${currentText}, Potential ${potentialText}`;

  return (
    <div data-testid={testId} className="flex min-w-0 flex-col tabular-nums">
      <p className="text-body-md text-on-surface-variant">{summaryLabel}</p>
      <p
        className="mt-1 break-words text-body-md font-semibold leading-snug text-on-surface"
        title={roleName ?? undefined}
      >
        {roleName ?? formatMissable(null)}
      </p>
      <div aria-hidden="true" className="mt-auto flex items-center gap-3">
        {currentScore === null ? (
          <span className={unavailableTextClass}>{formatMissable(null)}</span>
        ) : (
          <ScoreBadge
            score={currentScore}
            roleName={`${labelBase} (Current)`}
            variant="hero"
            className="size-auto! text-mono-lg!"
          />
        )}
        <span className="text-on-surface-variant">→</span>
        {concealed || potentialScore === null ? (
          <span className={unavailableTextClass}>{formatMissable(null)}</span>
        ) : (
          <ScoreBadge
            score={potentialScore}
            roleName={`${labelBase} (Potential)`}
            variant="hero"
            className="size-auto! text-mono-lg!"
          />
        )}
      </div>
      <p className="text-body-sm text-on-surface-variant">
        {`${rolePhaseLabel(phase)} · Current → Potential`}
      </p>
      <span className="sr-only">{accessibleDescription}</span>
    </div>
  );
}

type PlayerOverviewPanelProps = {
  player: PlayerDetail;
  showAbilitySummary?: boolean;
  showTacticalFitSummary?: boolean;
};

export function PlayerOverviewPanel({
  player,
  showAbilitySummary = false,
  showTacticalFitSummary = false,
}: PlayerOverviewPanelProps) {
  const showAbility = showAbilitySummary;
  const ipPair = bestCurrentRolePair(
    player.roleScores,
    player.positions,
    "in_possession",
  );
  const oopPair = bestCurrentRolePair(
    player.roleScores,
    player.positions,
    "out_of_possession",
  );
  return (
    <section aria-label={`${player.name} summary`} className="min-w-0">
      {showTacticalFitSummary || showAbility ? (
        <div
          data-testid="player-profile-summary-details"
          className="grid gap-4 lg:grid-cols-[minmax(320px,1fr)_minmax(0,1.5fr)]"
        >
          {showAbility ? (
            <div
              data-testid="player-profile-summary-analysis-details"
              className="contents"
            >
              <section
                aria-label="Ability"
                data-testid="overview-ability"
                className="rounded-lg border border-outline-variant bg-surface-container p-4"
              >
                <dl className="grid grid-cols-3 gap-4">
                  <OverviewFact label="Current Ability" value={player.ca} />
                  {player.hiddenInformationRevealed ? (
                    <OverviewFact
                      label="Potential Ability"
                      value={formatMissable(player.pa)}
                    />
                  ) : null}
                  <OverviewFact
                    label="Market Value"
                    value={
                      player.marketValueGbp === null
                        ? formatMissable(null)
                        : formatMoney(player.marketValueGbp)
                    }
                  />
                </dl>
              </section>
            </div>
          ) : null}

          {showTacticalFitSummary ? (
            <section
              aria-label="Tactical fit"
              data-testid="overview-tactical-fit"
              className="grid grid-cols-2 gap-4 rounded-lg border border-outline-variant bg-surface-container-low px-4 py-2"
            >
              <TacticalFitPair
                phase="in_possession"
                pair={ipPair}
                concealed={!player.hiddenInformationRevealed}
              />
              <TacticalFitPair
                phase="out_of_possession"
                pair={oopPair}
                concealed={!player.hiddenInformationRevealed}
              />
            </section>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
