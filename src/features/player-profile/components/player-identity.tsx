import { Eye, EyeOff } from "lucide-react";
import type { ReactNode } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import { Button } from "@/components/ui/button/button";
import {
  formatMissable,
  formatMoney,
  formatPlayerDob,
  formatPreferredFoot,
} from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";

type SummaryFactProps = {
  label: string;
  value: ReactNode;
  numeric?: boolean;
};

export function SummaryFact({
  label,
  value,
  numeric = false,
}: SummaryFactProps) {
  return (
    <div className="flex min-w-0 items-center justify-between gap-3">
      <dt className="text-body-md text-on-surface-variant">{label}</dt>
      <dd
        className={
          numeric
            ? "shrink-0 font-mono text-mono-md text-on-surface tabular-nums"
            : "min-w-0 text-right text-body-md text-on-surface"
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

export function PlayerIdentity({ player }: { player: PlayerDetail }) {
  const flags = [
    flagLabel(player.transferListed, "Transfer listed"),
    flagLabel(player.loanListed, "Loan listed"),
    flagLabel(player.notForSale, "Not for sale"),
    flagLabel(player.setForRelease, "Set for release"),
    flagLabel(player.onLoan, "On loan"),
  ].filter((label): label is string => label !== null);

  return (
    <div className="min-w-0">
      <h1 className="break-words text-headline-lg text-on-surface">
        {player.name}
      </h1>
      <p className="mt-1 text-body-md text-on-surface">
        {formatMissable(player.club)}
      </p>
      {player.division ? (
        <p className="mt-0.5 text-body-sm text-on-surface-variant">
          {player.division}
        </p>
      ) : null}
      {flags.length > 0 ? (
        <p
          className="mt-2 truncate text-body-sm text-warning"
          title={flags.join(" · ")}
        >
          {flags.join(" · ")}
        </p>
      ) : null}
    </div>
  );
}

export function PlayerIdentityFacts({ player }: { player: PlayerDetail }) {
  return (
    <dl className="space-y-2 border-t border-outline-variant pt-3">
      <SummaryFact
        label="Nationality"
        value={<NationalityCell nationalities={player.nationalities} />}
      />
      <SummaryFact label="Age" value={formatMissable(player.age)} numeric />
      <SummaryFact
        label="Date of birth"
        value={formatPlayerDob(player.birthYear, player.birthDayOfYear, null)}
        numeric
      />
      <SummaryFact
        label="Height"
        value={player.heightCm === null ? "—" : `${player.heightCm} cm`}
        numeric
      />
      <SummaryFact
        label="Preferred foot"
        value={formatPreferredFoot(player.preferredFoot)}
      />
    </dl>
  );
}

export function PlayerMarketValueFact({ player }: { player: PlayerDetail }) {
  return (
    <SummaryFact
      label="Market value"
      value={
        player.marketValueGbp === null
          ? "—"
          : formatMoney(player.marketValueGbp)
      }
      numeric
    />
  );
}

function playerInitials(name: string) {
  const parts = name.split(/\s+/).filter(Boolean);
  const first = parts[0]?.charAt(0) ?? "";
  const last =
    parts.length > 1 ? (parts[parts.length - 1]?.charAt(0) ?? "") : "";
  return `${first}${last}`.toUpperCase() || "—";
}

function clubMonogram(club: string | null | undefined) {
  if (club === null || club === undefined || club === "") return "—";
  const monogram = club
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part.charAt(0).toUpperCase())
    .join("");
  return monogram || "—";
}

type PlayerIdentityRailProps = {
  player: PlayerDetail;
  actions?: ReactNode;
  hiddenInformationPending?: boolean;
  hiddenInformationError?: Error | null;
  onToggleHiddenInformation?: () => void;
};

export function PlayerIdentityRail({
  player,
  actions,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
}: PlayerIdentityRailProps) {
  const VisibilityIcon = player.hiddenInformationRevealed ? EyeOff : Eye;

  return (
    <aside
      aria-label="Player identity"
      data-testid="player-identity-rail"
      className="flex w-full shrink-0 flex-col overflow-y-auto rounded-lg border border-outline-variant bg-surface-container px-4 py-3 lg:h-full lg:w-80"
    >
      <div className="-mx-4 -mt-3 flex h-44 shrink-0 items-center justify-around rounded-t-lg bg-surface-container-lowest px-6">
        <span
          role="img"
          aria-label="Player portrait placeholder"
          title="Player portrait placeholder"
          className="flex size-28 shrink-0 items-center justify-center rounded-full bg-surface-container-high font-mono text-mono-lg text-on-surface-variant"
        >
          {playerInitials(player.name)}
        </span>
        <span
          role="img"
          aria-label="Club crest placeholder"
          title="Club crest placeholder"
          className="flex size-16 shrink-0 items-center justify-center rounded-lg bg-surface-container-high font-mono text-mono-md text-on-surface-variant"
        >
          {clubMonogram(player.club)}
        </span>
      </div>
      <div className="mt-4">
        <PlayerIdentity player={player} />
      </div>
      <div className="mt-3">
        <PlayerIdentityFacts player={player} />
      </div>
      <div className="mt-3 border-t border-outline-variant pt-3">
        <PlayerMarketValueFact player={player} />
      </div>
      <div className="mt-auto space-y-1 border-t border-outline-variant pt-3">
        <div data-testid="player-profile-action-slot" className="min-h-10">
          {actions}
        </div>
        <div data-testid="player-profile-display-control">
          <Button
            icon={VisibilityIcon}
            variant="ghost"
            className="w-full justify-start px-2"
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
            <p className="mt-1 text-body-sm text-error" role="alert">
              Could not update hidden information.
            </p>
          ) : null}
        </div>
      </div>
    </aside>
  );
}
