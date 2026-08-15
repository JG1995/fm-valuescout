import { ArrowDown, ArrowUp } from "lucide-react";
import { useState } from "react";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import { attributeValueTier } from "../utils/attribute-groups";
import {
  defaultProfilePosition,
  PROFILE_POSITION_ROWS,
  type RoleSort,
  rolesForProfilePosition,
} from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";

type PlayerRolesPanelProps = {
  player: PlayerDetail;
  hiddenInformationRevealed: boolean;
};

type RoleScoreProps = {
  roleName: string;
  basis: "Current" | "Potential";
  score: number | null;
};

function RoleScore({ roleName, basis, score }: RoleScoreProps) {
  if (score === null) {
    return (
      <span
        role="img"
        aria-label={`${roleName} (${basis}): unavailable`}
        className="font-mono text-mono-sm text-on-surface-variant tabular-nums"
      >
        {formatMissable(null)}
      </span>
    );
  }

  return (
    <ScoreBadge
      score={score}
      roleName={`${roleName} (${basis})`}
      variant="card"
    />
  );
}

function RoleSortHeader({
  label,
  basis,
  sort,
  onSort,
  className,
}: {
  label: "Current" | "Potential";
  basis: RoleSort["basis"];
  sort: RoleSort;
  onSort: (basis: RoleSort["basis"]) => void;
  className: string;
}) {
  const active = sort.basis === basis;
  const SortIcon = sort.direction === "ascending" ? ArrowUp : ArrowDown;

  return (
    <th
      scope="col"
      aria-sort={active ? sort.direction : undefined}
      className={className}
    >
      <button
        type="button"
        className={`inline-flex min-h-8 w-full items-center justify-center gap-1 rounded-md px-1 text-label-sm transition-colors duration-150 ease-out ${
          active
            ? "text-primary"
            : "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
        }`}
        onClick={() => onSort(basis)}
      >
        <span>{label}</span>
        <span aria-hidden="true" className="inline-flex size-3 items-center">
          {active ? <SortIcon size={12} strokeWidth={1.5} /> : null}
        </span>
      </button>
    </th>
  );
}

function PositionPitch({
  player,
  selectedPosition,
  onSelectPosition,
}: {
  player: PlayerDetail;
  selectedPosition: string;
  onSelectPosition: (position: string) => void;
}) {
  return (
    <fieldset className="relative isolate overflow-hidden rounded-lg border border-outline-variant bg-surface-container-lowest p-3">
      <legend className="sr-only">Select a pitch position</legend>
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-3 rounded-sm border border-outline/45"
      >
        <span className="absolute top-1/2 right-0 left-0 border-t border-outline/45" />
        <span className="absolute top-1/2 left-1/2 size-14 -translate-x-1/2 -translate-y-1/2 rounded-full border border-outline/45" />
        <span className="absolute top-0 left-1/2 h-10 w-1/2 -translate-x-1/2 border-x border-b border-outline/45" />
        <span className="absolute bottom-0 left-1/2 h-10 w-1/2 -translate-x-1/2 border-x border-t border-outline/45" />
      </div>
      <div className="relative z-10 grid h-full grid-cols-3 content-between gap-1.5">
        {PROFILE_POSITION_ROWS.flatMap((row, rowIndex) =>
          row.map((position, columnIndex) => {
            const key = `${rowIndex}:${columnIndex}`;
            if (position === null) {
              return <span aria-hidden="true" key={key} className="min-h-11" />;
            }

            const familiarity = player.positions[position];
            const knownFamiliarity =
              typeof familiarity === "number" && familiarity > 0;
            const selected = position === selectedPosition;
            const accessibleName = knownFamiliarity
              ? `${position}, familiarity ${familiarity}`
              : `${position}, no recorded familiarity`;

            return (
              <button
                key={key}
                type="button"
                aria-label={accessibleName}
                aria-pressed={selected}
                data-tier={
                  knownFamiliarity ? attributeValueTier(familiarity) : undefined
                }
                className={`min-h-11 rounded-md border px-1 py-1 text-center transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "border-primary bg-primary-container text-on-primary-container ring-2 ring-primary/50"
                    : knownFamiliarity
                      ? "border-outline bg-surface-container-high hover:bg-surface-container-highest data-[tier=1]:text-score-1 data-[tier=2]:text-score-2 data-[tier=3]:text-score-3 data-[tier=4]:text-score-4"
                      : "border-outline-variant bg-surface-container/85 text-on-surface-variant hover:bg-surface-container-high"
                }`}
                onClick={() => onSelectPosition(position)}
              >
                <span className="block text-label-md">{position}</span>
                <span className="block font-mono text-[10px] tabular-nums">
                  {knownFamiliarity ? familiarity : "—"}
                </span>
              </button>
            );
          }),
        )}
      </div>
    </fieldset>
  );
}

export function PlayerRolesPanel({
  player,
  hiddenInformationRevealed,
}: PlayerRolesPanelProps) {
  const [selectedPosition, setSelectedPosition] = useState(() =>
    defaultProfilePosition(player.positions, player.roleScores),
  );
  const [sort, setSort] = useState<RoleSort>({
    basis: "current",
    direction: "descending",
  });
  const effectiveSort = {
    ...sort,
    basis: hiddenInformationRevealed ? sort.basis : "current",
  } satisfies RoleSort;
  const roles = rolesForProfilePosition(
    player.roleScores,
    selectedPosition,
    effectiveSort,
  );
  const familiarity = player.positions[selectedPosition];
  const onSort = (basis: RoleSort["basis"]) => {
    setSort((current) => ({
      basis,
      direction:
        current.basis === basis && current.direction === "descending"
          ? "ascending"
          : "descending",
    }));
  };

  return (
    <Panel
      title="Role fit"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          Select a position
        </span>
      }
      className="flex min-h-0 flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <section
        aria-label={`Role fit for ${selectedPosition}`}
        className="grid h-full min-h-0 gap-4 sm:grid-cols-[minmax(180px,0.8fr)_minmax(240px,1.2fr)]"
      >
        <PositionPitch
          player={player}
          selectedPosition={selectedPosition}
          onSelectPosition={setSelectedPosition}
        />
        <div className="flex min-h-0 min-w-0 flex-col">
          <div className="min-h-0 flex-1 overflow-y-auto pr-1">
            <table className="w-full table-fixed border-collapse">
              <caption className="sr-only">
                Role scores for {selectedPosition}
              </caption>
              <colgroup>
                <col />
                <col className="w-[72px]" />
                {hiddenInformationRevealed ? (
                  <col className="w-[80px]" />
                ) : null}
              </colgroup>
              <thead className="sticky top-0 z-10 bg-surface-container">
                <tr className="border-b border-outline-variant">
                  <th scope="col" className="pb-2 text-left font-normal">
                    <h3 className="text-headline-sm text-on-surface">
                      {selectedPosition}
                    </h3>
                    <p className="text-body-sm text-on-surface-variant">
                      {roles.length} {roles.length === 1 ? "role" : "roles"}
                      {typeof familiarity === "number" && familiarity > 0
                        ? ` · familiarity ${familiarity}`
                        : ""}
                    </p>
                    <span className="sr-only">Role</span>
                  </th>
                  <RoleSortHeader
                    label="Current"
                    basis="current"
                    sort={effectiveSort}
                    onSort={onSort}
                    className="w-[72px] pb-1 text-center align-bottom"
                  />
                  {hiddenInformationRevealed ? (
                    <RoleSortHeader
                      label="Potential"
                      basis="potential"
                      sort={effectiveSort}
                      onSort={onSort}
                      className="w-[80px] pb-1 text-center align-bottom"
                    />
                  ) : null}
                </tr>
              </thead>
              <tbody>
                {roles.length === 0 ? (
                  <tr>
                    <td
                      colSpan={hiddenInformationRevealed ? 3 : 2}
                      className="h-24 text-center text-body-sm text-on-surface-variant"
                    >
                      No catalog roles use this position.
                    </td>
                  </tr>
                ) : null}
                {roles.map((role) => (
                  <tr
                    key={role.roleId}
                    className="h-12 border-b border-outline-variant/70"
                  >
                    <td className="min-w-0 pr-2">
                      <p className="truncate text-body-md text-on-surface">
                        {role.displayName}
                      </p>
                      <p className="text-[11px] text-on-surface-variant">
                        {rolePhaseLabel(role.phase)}
                      </p>
                    </td>
                    <td className="text-center">
                      <RoleScore
                        roleName={role.displayName}
                        basis="Current"
                        score={role.score}
                      />
                    </td>
                    {hiddenInformationRevealed ? (
                      <td className="text-center">
                        <RoleScore
                          roleName={role.displayName}
                          basis="Potential"
                          score={role.potentialScore}
                        />
                      </td>
                    ) : null}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </Panel>
  );
}
