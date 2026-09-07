import type { ReactNode } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
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
  );
}

export function PlayerIdentityFacts({ player }: { player: PlayerDetail }) {
  return (
    <dl className="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4 lg:grid-cols-2 xl:grid-cols-4">
      <SummaryFact
        label="Age / DOB"
        value={formatPlayerDob(
          player.birthYear,
          player.birthDayOfYear,
          player.age,
        )}
      />
      <SummaryFact
        label="Nationality"
        value={<NationalityCell nationalities={player.nationalities} />}
      />
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
  );
}

export function PlayerMarketValueFact({ player }: { player: PlayerDetail }) {
  return (
    <SummaryFact
      label="Value"
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

export function PlayerIdentityRail({ player }: { player: PlayerDetail }) {
  return (
    <aside
      aria-label="Player identity"
      data-testid="player-identity-rail"
      className="w-full shrink-0 rounded-lg border border-outline-variant bg-surface-container px-4 py-3 lg:w-72"
    >
      <div className="flex items-center justify-between gap-3">
        <span
          role="img"
          aria-label="Player portrait placeholder"
          title="Player portrait placeholder"
          className="flex size-16 shrink-0 items-center justify-center rounded-full bg-surface-container-high font-mono text-mono-md text-on-surface-variant"
        >
          {playerInitials(player.name)}
        </span>
        <span
          role="img"
          aria-label="Club crest placeholder"
          title="Club crest placeholder"
          className="flex size-10 shrink-0 items-center justify-center rounded-md bg-surface-container-high font-mono text-mono-sm text-on-surface-variant"
        >
          {clubMonogram(player.club)}
        </span>
      </div>
      <div className="mt-3">
        <PlayerIdentity player={player} />
      </div>
      <div className="mt-3">
        <PlayerIdentityFacts player={player} />
      </div>
      <div className="mt-2">
        <PlayerMarketValueFact player={player} />
      </div>
    </aside>
  );
}
