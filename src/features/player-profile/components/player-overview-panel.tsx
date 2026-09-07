import { Eye, EyeOff } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable, formatMoney } from "@/utils/format";
import type { PlayerDetail, PlayerRoleScore } from "../types/player-detail";
import {
  bestCurrentRolePair,
  type RolePhase,
} from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";

type OverviewFactCardProps = {
  label: string;
  value: ReactNode;
};

function OverviewFactCard({ label, value }: OverviewFactCardProps) {
  return (
    <div className="min-w-0 rounded-lg border border-outline-variant bg-surface-container-high p-4">
      <h2 className="text-label-lg text-on-surface-variant">{label}</h2>
      <p className="mt-3 font-mono text-headline-lg text-on-surface tabular-nums">
        {value}
      </p>
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
  const unavailableTextClass =
    "inline-flex size-12 items-center justify-center font-mono text-mono-lg text-on-surface-variant";
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
    <div
      data-testid={testId}
      className="min-w-0 rounded-lg border border-outline-variant bg-surface-container-high p-4 tabular-nums"
    >
      <p className="text-label-lg text-on-surface-variant">{summaryLabel}</p>
      <p
        className="mt-2 text-body-lg font-semibold text-on-surface"
        title={roleName ?? undefined}
      >
        {roleName ?? formatMissable(null)}
      </p>
      <div aria-hidden="true" className="mt-1 flex items-center gap-1.5">
        {currentScore === null ? (
          <span className={unavailableTextClass}>{formatMissable(null)}</span>
        ) : (
          <ScoreBadge
            score={currentScore}
            roleName={`${labelBase} (Current)`}
            variant="hero"
            className="text-headline-lg"
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
            className="text-headline-lg"
          />
        )}
      </div>
      <p className="text-label-sm text-on-surface-variant">
        {`${rolePhaseLabel(phase)} · Current → Potential`}
      </p>
      <span className="sr-only">{accessibleDescription}</span>
    </div>
  );
}

type PlayerOverviewPanelProps = {
  player: PlayerDetail;
  actions?: ReactNode;
  hiddenInformationPending?: boolean;
  hiddenInformationError?: Error | null;
  onToggleHiddenInformation?: () => void;
  showAbilitySummary?: boolean;
  showTacticalFitSummary?: boolean;
};

export function PlayerOverviewPanel({
  player,
  actions,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
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
  const VisibilityIcon = player.hiddenInformationRevealed ? EyeOff : Eye;

  return (
    <section
      aria-label={`${player.name} summary`}
      className="rounded-lg border border-outline-variant bg-surface-container px-4 py-3"
    >
      <div className="space-y-3">
        <div className="flex min-h-10 min-w-0 flex-wrap items-start justify-between gap-2 overflow-visible">
          <div
            data-testid="player-profile-display-control"
            className="space-y-2"
          >
            <Button
              icon={VisibilityIcon}
              variant="secondary"
              aria-label="Reveal hidden information"
              aria-pressed={player.hiddenInformationRevealed}
              disabled={hiddenInformationPending}
              loading={hiddenInformationPending}
              loadingLabel="Updating…"
              onClick={onToggleHiddenInformation}
            >
              {player.hiddenInformationRevealed
                ? "Hide hidden info"
                : "Reveal hidden info"}
            </Button>
            {hiddenInformationError ? (
              <p className="text-right text-body-sm text-error" role="alert">
                Could not update hidden information.
              </p>
            ) : null}
          </div>
          <div
            data-testid="player-profile-action-slot"
            className="flex min-h-10 min-w-0 flex-wrap justify-end overflow-visible"
          >
            {actions}
          </div>
        </div>

        {showTacticalFitSummary || showAbility ? (
          <div
            data-testid="player-profile-summary-details"
            className="grid gap-3 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-[repeat(3,minmax(0,1fr))_repeat(2,minmax(0,1.2fr))]"
          >
            {showAbility ? (
              <div
                data-testid="player-profile-summary-analysis-details"
                className="contents"
              >
                <section
                  aria-label="Ability"
                  data-testid="overview-ability"
                  className="contents"
                >
                  <OverviewFactCard label="Current Ability" value={player.ca} />
                  {player.hiddenInformationRevealed ? (
                    <OverviewFactCard
                      label="Potential Ability"
                      value={formatMissable(player.pa)}
                    />
                  ) : null}
                  <OverviewFactCard
                    label="Market Value"
                    value={
                      player.marketValueGbp === null
                        ? formatMissable(null)
                        : formatMoney(player.marketValueGbp)
                    }
                  />
                </section>
              </div>
            ) : null}

            {showTacticalFitSummary ? (
              <section
                aria-label="Tactical fit"
                data-testid="overview-tactical-fit"
                className="contents"
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
      </div>
    </section>
  );
}
