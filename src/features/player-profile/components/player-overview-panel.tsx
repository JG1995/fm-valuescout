import { Panel } from "@/components/ui/panel/panel";
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
} from "../utils/position-families";

type FieldProps = {
  label: string;
  value: string | number;
  numeric?: boolean;
};

function Field({ label, value, numeric = false }: FieldProps) {
  return (
    <div className="min-w-0">
      <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
        {label}
      </p>
      <p
        className={
          numeric
            ? "font-mono text-mono-sm text-on-surface tabular-nums"
            : "truncate text-body-md text-on-surface"
        }
        title={typeof value === "string" ? value : undefined}
      >
        {value}
      </p>
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
};

function BestRoleSummary({
  label,
  basis,
  roleName,
  score,
}: BestRoleSummaryProps) {
  const accessibleLabel = `${label} (${basis})`;

  return (
    <div className="flex min-w-0 items-center gap-4">
      {score === null ? (
        <span
          role="img"
          aria-label={`${accessibleLabel}: unavailable`}
          className="font-mono text-mono-lg text-on-surface-variant tabular-nums"
        >
          {formatMissable(null)}
        </span>
      ) : (
        <ScoreBadge score={score} roleName={accessibleLabel} variant="hero" />
      )}
      <div className="min-w-0">
        <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
          {accessibleLabel}
        </p>
        <p className="truncate text-body-md text-on-surface">
          {roleName ?? formatMissable(null)}
        </p>
      </div>
    </div>
  );
}

type PlayerOverviewPanelProps = {
  player: PlayerDetail;
};

export function PlayerOverviewPanel({ player }: PlayerOverviewPanelProps) {
  const nationality =
    player.nationalities.length > 0 ? player.nationalities.join(", ") : "—";
  const flags = [
    flagLabel(player.transferListed, "Transfer listed"),
    flagLabel(player.loanListed, "Loan listed"),
    flagLabel(player.notForSale, "Not for sale"),
    flagLabel(player.setForRelease, "Set for release"),
    flagLabel(player.onLoan, "On loan"),
  ].filter((label): label is string => label !== null);
  const bestRole = bestRoleScore(player.roleScores);
  const bestPotentialRole = bestPotentialRoleScore(player.roleScores);

  return (
    <Panel title="Overview">
      <div className="space-y-6">
        <div className="grid gap-4 sm:grid-cols-2">
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
          />
        </div>
        <div className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3 lg:grid-cols-4">
          <Field label="Name" value={player.name} />
          <Field
            label="Age / DOB"
            value={formatPlayerDob(
              player.birthYear,
              player.birthDayOfYear,
              player.age,
            )}
          />
          <Field label="Nationality" value={nationality} />
          <Field label="Club" value={formatMissable(player.club)} />
          <Field label="Division" value={formatMissable(player.division)} />
          <Field label="CA" value={player.ca} numeric />
          <Field label="PA" value={player.pa} numeric />
          <Field
            label="Value"
            value={
              player.marketValueGbp === null
                ? "—"
                : formatMoney(player.marketValueGbp)
            }
            numeric
          />
          <Field
            label="Height"
            value={player.heightCm === null ? "—" : `${player.heightCm} cm`}
            numeric
          />
          <Field
            label="Preferred foot"
            value={formatPreferredFoot(player.preferredFoot)}
          />
          {flags.length > 0 ? (
            <Field label="Status" value={flags.join(" · ")} />
          ) : null}
        </div>
      </div>
    </Panel>
  );
}
