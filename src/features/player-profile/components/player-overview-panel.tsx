import { Eye, EyeOff } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail, PlayerRoleScore } from "../types/player-detail";
import {
  bestCurrentRolePair,
  bestRoleScore,
  type PositionRoleScore,
  type RolePhase,
  rolesForPhase,
  rolesForPlayablePositions,
} from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";
import { PlayerMarketValueFact, SummaryFact } from "./player-identity";

type BestRoleSummaryProps = {
  label: string;
  roleName: string | null;
  score: number | null;
  concealed?: boolean;
};

function BestRoleSummary({
  label,
  roleName,
  score,
  concealed = false,
}: BestRoleSummaryProps) {
  return (
    <div className="flex min-w-0 items-start gap-3">
      {score === null ? (
        <span
          role="img"
          aria-label={`${label}: ${concealed ? "concealed" : "unavailable"}`}
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
          title={concealed ? undefined : (roleName ?? undefined)}
        >
          {concealed ? "Concealed" : (roleName ?? formatMissable(null))}
        </p>
      </div>
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
      className="flex min-w-0 items-start gap-3 tabular-nums"
    >
      <div aria-hidden="true" className="flex shrink-0 items-center gap-1.5">
        {currentScore === null ? (
          <span className={unavailableTextClass}>{formatMissable(null)}</span>
        ) : (
          <ScoreBadge
            score={currentScore}
            roleName={`${labelBase} (Current)`}
            variant="hero"
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
          />
        )}
      </div>
      <div className="min-w-0">
        <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
          {`${fullPhase} (${rolePhaseLabel(phase)})`}
        </p>
        <p
          className="truncate text-body-md text-on-surface"
          title={roleName ?? undefined}
        >
          {roleName ?? formatMissable(null)}
        </p>
        <span className="sr-only">{accessibleDescription}</span>
      </div>
    </div>
  );
}

type PlayerOverviewPanelProps = {
  player: PlayerDetail;
  mode?: "general" | "moneyball";
  roleScores?: readonly PositionRoleScore[];
  actions?: ReactNode;
  hiddenInformationPending?: boolean;
  hiddenInformationError?: Error | null;
  onToggleHiddenInformation?: () => void;
  showAbilitySummary?: boolean;
  showTacticalFitSummary?: boolean;
};

export function PlayerOverviewPanel({
  player,
  mode = "general",
  roleScores,
  actions,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
  showAbilitySummary = false,
  showTacticalFitSummary = false,
}: PlayerOverviewPanelProps) {
  const showGeneralAnalysis = mode === "general";
  const showMoneyballAnalysis = mode === "moneyball";
  const showAbility = showGeneralAnalysis && showAbilitySummary;
  const analysisRoles = rolesForPlayablePositions(
    showMoneyballAnalysis ? (roleScores ?? []) : player.roleScores,
    player.positions,
  );
  const inPossessionRoles = rolesForPhase(analysisRoles, "in_possession");
  const outOfPossessionRoles = rolesForPhase(
    analysisRoles,
    "out_of_possession",
  );
  const currentIpRole = bestRoleScore(inPossessionRoles);
  const currentOopRole = bestRoleScore(outOfPossessionRoles);
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
            {showGeneralAnalysis ? (
              <>
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
                  <p
                    className="text-right text-body-sm text-error"
                    role="alert"
                  >
                    Could not update hidden information.
                  </p>
                ) : null}
              </>
            ) : null}
          </div>
          <div
            data-testid="player-profile-action-slot"
            className="flex min-h-10 min-w-0 flex-wrap justify-end overflow-visible"
          >
            {showGeneralAnalysis ? actions : null}
          </div>
        </div>

        <div
          data-testid="player-profile-summary-details"
          className="grid gap-x-4 gap-y-2 lg:grid-cols-2"
        >
          {showMoneyballAnalysis ? (
            <div
              data-testid="player-profile-role-summaries"
              className="grid min-w-0 grid-cols-2 grid-rows-2 gap-3 border-outline-variant lg:border-x lg:px-4"
            >
              <BestRoleSummary
                label="Moneyball IP"
                roleName={currentIpRole?.displayName ?? null}
                score={currentIpRole?.score ?? null}
              />
              <BestRoleSummary
                label="Moneyball OOP"
                roleName={currentOopRole?.displayName ?? null}
                score={currentOopRole?.score ?? null}
              />
              <div aria-hidden="true" className="min-h-12" />
              <div aria-hidden="true" className="min-h-12" />
            </div>
          ) : showTacticalFitSummary ? (
            <section
              aria-label="Tactical fit"
              data-testid="overview-tactical-fit"
              className="grid min-w-0 grid-cols-2 gap-3 border-outline-variant lg:border-x lg:px-4"
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
          ) : (
            <div aria-hidden="true" className="min-h-12" />
          )}

          <div
            aria-hidden={!showAbility}
            data-testid="player-profile-summary-analysis-details"
            className="min-h-9"
          >
            {showAbility ? (
              <section
                aria-label="Ability"
                data-testid="overview-ability"
                className="min-w-0"
              >
                <h2 className="text-label-md text-on-surface-variant uppercase tracking-[0.08em]">
                  Ability
                </h2>
                <dl className="mt-2 grid min-w-0 grid-cols-3 gap-3">
                  <SummaryFact label="CA" value={player.ca} numeric />
                  {player.hiddenInformationRevealed ? (
                    <SummaryFact
                      label="PA"
                      value={formatMissable(player.pa)}
                      numeric
                    />
                  ) : null}
                  <PlayerMarketValueFact player={player} />
                </dl>
              </section>
            ) : null}
          </div>
        </div>
      </div>
    </section>
  );
}
