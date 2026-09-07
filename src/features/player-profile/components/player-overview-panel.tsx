import { Eye, EyeOff } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  bestPotentialRoleScore,
  bestRoleScore,
  type PositionRoleScore,
  rolesForPhase,
  rolesForPlayablePositions,
} from "../utils/position-families";
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

type PlayerOverviewPanelProps = {
  player: PlayerDetail;
  mode?: "general" | "moneyball";
  roleScores?: readonly PositionRoleScore[];
  actions?: ReactNode;
  hiddenInformationPending?: boolean;
  hiddenInformationError?: Error | null;
  onToggleHiddenInformation?: () => void;
};

export function PlayerOverviewPanel({
  player,
  mode = "general",
  roleScores,
  actions,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
}: PlayerOverviewPanelProps) {
  const showGeneralAnalysis = mode === "general";
  const showMoneyballAnalysis = mode === "moneyball";
  const analysisRoles = rolesForPlayablePositions(
    showMoneyballAnalysis ? (roleScores ?? []) : player.roleScores,
    player.positions,
  );
  const generalRoles = showGeneralAnalysis
    ? rolesForPlayablePositions(player.roleScores, player.positions)
    : [];
  const inPossessionRoles = rolesForPhase(analysisRoles, "in_possession");
  const outOfPossessionRoles = rolesForPhase(
    analysisRoles,
    "out_of_possession",
  );
  const currentIpRole = bestRoleScore(inPossessionRoles);
  const currentOopRole = bestRoleScore(outOfPossessionRoles);
  const generalInPossessionRoles = rolesForPhase(generalRoles, "in_possession");
  const generalOutOfPossessionRoles = rolesForPhase(
    generalRoles,
    "out_of_possession",
  );
  const potentialIpRole = player.hiddenInformationRevealed
    ? bestPotentialRoleScore(generalInPossessionRoles)
    : null;
  const potentialOopRole = player.hiddenInformationRevealed
    ? bestPotentialRoleScore(generalOutOfPossessionRoles)
    : null;
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
          <div
            data-testid="player-profile-role-summaries"
            className="grid min-w-0 grid-cols-2 grid-rows-2 gap-3 border-outline-variant lg:border-x lg:px-4"
          >
            <BestRoleSummary
              label={showGeneralAnalysis ? "Current IP" : "Moneyball IP"}
              roleName={currentIpRole?.displayName ?? null}
              score={currentIpRole?.score ?? null}
            />
            <BestRoleSummary
              label={showGeneralAnalysis ? "Current OOP" : "Moneyball OOP"}
              roleName={currentOopRole?.displayName ?? null}
              score={currentOopRole?.score ?? null}
            />
            {showGeneralAnalysis ? (
              <>
                <BestRoleSummary
                  label="Potential IP"
                  roleName={potentialIpRole?.displayName ?? null}
                  score={potentialIpRole?.potentialScore ?? null}
                  concealed={!player.hiddenInformationRevealed}
                />
                <BestRoleSummary
                  label="Potential OOP"
                  roleName={potentialOopRole?.displayName ?? null}
                  score={potentialOopRole?.potentialScore ?? null}
                  concealed={!player.hiddenInformationRevealed}
                />
              </>
            ) : (
              <>
                <div aria-hidden="true" className="min-h-12" />
                <div aria-hidden="true" className="min-h-12" />
              </>
            )}
          </div>

          <div
            aria-hidden={!showGeneralAnalysis}
            data-testid="player-profile-summary-analysis-details"
            className="min-h-9"
          >
            {showGeneralAnalysis ? (
              <dl className="grid min-w-0 grid-cols-3 gap-3">
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
            ) : null}
          </div>
        </div>
      </div>
    </section>
  );
}
