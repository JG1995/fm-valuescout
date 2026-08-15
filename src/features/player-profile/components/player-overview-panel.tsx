import { Eye, EyeOff } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import {
  formatMissable,
  formatMoney,
  formatPlayerDob,
  formatPreferredFoot,
} from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  bestPotentialRoleScore,
  bestRoleScore,
  rolesForPlayablePositions,
} from "../utils/position-families";

type SummaryFactProps = {
  label: string;
  value: string | number;
  numeric?: boolean;
};

function SummaryFact({ label, value, numeric = false }: SummaryFactProps) {
  return (
    <div className="min-w-0">
      <dt className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
        {label}
      </dt>
      <dd
        className={
          numeric
            ? "font-mono text-mono-md text-on-surface tabular-nums"
            : "truncate text-body-md text-on-surface"
        }
        title={typeof value === "string" ? value : undefined}
      >
        {value}
      </dd>
    </div>
  );
}

function flagLabel(value: boolean | null | undefined, yes: string) {
  return value === true ? yes : null;
}

type BestRoleSummaryProps = {
  label: string;
  basis: "Current" | "Potential";
  roleName: string | null;
  score: number | null;
  concealed?: boolean;
};

function BestRoleSummary({
  label,
  basis,
  roleName,
  score,
  concealed = false,
}: BestRoleSummaryProps) {
  const accessibleLabel = `${label} (${basis})`;

  return (
    <div className="flex min-w-0 items-start gap-3">
      {score === null ? (
        <span
          role="img"
          aria-label={`${accessibleLabel}: ${concealed ? "concealed" : "unavailable"}`}
          className="inline-flex size-12 items-center justify-center font-mono text-mono-lg text-on-surface-variant tabular-nums"
        >
          {concealed ? "Concealed" : formatMissable(null)}
        </span>
      ) : (
        <ScoreBadge score={score} roleName={accessibleLabel} variant="hero" />
      )}
      <div className="min-w-0">
        <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
          {accessibleLabel}
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
  actions: ReactNode;
  hiddenInformationPending: boolean;
  hiddenInformationError: Error | null;
  onToggleHiddenInformation: () => void;
};

export function PlayerOverviewPanel({
  player,
  actions,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
}: PlayerOverviewPanelProps) {
  const nationality =
    player.nationalities.length > 0 ? player.nationalities.join(", ") : "—";
  const flags = [
    flagLabel(player.transferListed, "Transfer listed"),
    flagLabel(player.loanListed, "Loan listed"),
    flagLabel(player.notForSale, "Not for sale"),
    flagLabel(player.setForRelease, "Set for release"),
    flagLabel(player.onLoan, "On loan"),
  ].filter((label): label is string => label !== null);
  const playableRoles = rolesForPlayablePositions(
    player.roleScores,
    player.positions,
  );
  const bestRole = bestRoleScore(playableRoles);
  const bestPotentialRole = player.hiddenInformationRevealed
    ? bestPotentialRoleScore(playableRoles)
    : null;
  const VisibilityIcon = player.hiddenInformationRevealed ? EyeOff : Eye;

  return (
    <section
      aria-label={`${player.name} summary`}
      className="rounded-lg border border-outline-variant bg-surface-container p-4"
    >
      <div className="grid gap-x-4 gap-y-2 lg:grid-cols-[minmax(260px,1.15fr)_minmax(300px,1fr)_minmax(260px,0.9fr)] lg:items-start">
        <div className="min-w-0">
          <h1
            className="truncate text-headline-lg text-on-surface"
            title={player.name}
          >
            {player.name}
          </h1>
          <p className="mt-0.5 truncate text-body-md text-on-surface-variant">
            {formatMissable(player.club)}
            {player.division ? ` · ${player.division}` : ""}
          </p>
          {flags.length > 0 ? (
            <p
              className="mt-2 truncate text-body-sm text-warning"
              title={flags.join(" · ")}
            >
              {flags.join(" · ")}
            </p>
          ) : null}
        </div>

        <div className="min-w-0 lg:col-span-2 lg:flex lg:self-end lg:justify-end">
          <div className="space-y-2">
            <div className="flex flex-wrap items-center justify-end gap-2">
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
              {actions}
            </div>
            {hiddenInformationError ? (
              <p className="text-right text-body-sm text-error" role="alert">
                Could not update hidden information.
              </p>
            ) : null}
          </div>
        </div>

        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4 lg:grid-cols-2 xl:grid-cols-4">
          <SummaryFact
            label="Age / DOB"
            value={formatPlayerDob(
              player.birthYear,
              player.birthDayOfYear,
              player.age,
            )}
          />
          <SummaryFact label="Nationality" value={nationality} />
          <SummaryFact
            label="Height"
            value={player.heightCm === null ? "—" : `${player.heightCm} cm`}
            numeric
          />
          <SummaryFact
            label="Foot"
            value={formatPreferredFoot(player.preferredFoot)}
          />
        </dl>

        <div className="grid min-w-0 grid-cols-2 gap-3 border-outline-variant lg:border-x lg:px-4">
          <BestRoleSummary
            label="Best role"
            basis="Current"
            roleName={bestRole?.displayName ?? null}
            score={bestRole?.score ?? null}
          />
          <BestRoleSummary
            label="Best potential role"
            basis="Potential"
            roleName={bestPotentialRole?.displayName ?? null}
            score={bestPotentialRole?.potentialScore ?? null}
            concealed={!player.hiddenInformationRevealed}
          />
        </div>

        <dl className="grid min-w-0 grid-cols-3 gap-3">
          <SummaryFact label="CA" value={player.ca} numeric />
          {player.hiddenInformationRevealed ? (
            <SummaryFact label="PA" value={formatMissable(player.pa)} numeric />
          ) : null}
          <SummaryFact
            label="Value"
            value={
              player.marketValueGbp === null
                ? "—"
                : formatMoney(player.marketValueGbp)
            }
            numeric
          />
        </dl>
      </div>
    </section>
  );
}
