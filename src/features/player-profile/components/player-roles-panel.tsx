import { ArrowDown, ArrowUp } from "lucide-react";
import { useState } from "react";
import { ProfilePositionPicker } from "@/components/profile-position-picker/profile-position-picker";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  defaultProfilePosition,
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
        <ProfilePositionPicker
          positions={player.positions}
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
