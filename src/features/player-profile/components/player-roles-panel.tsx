import { useState } from "react";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import { attributeValueTier } from "../utils/attribute-groups";
import {
  defaultProfilePosition,
  PROFILE_POSITION_ROWS,
  rolesForProfilePosition,
} from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";

type PlayerRolesPanelProps = {
  player: PlayerDetail;
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
                      ? "border-outline bg-surface-container-high hover:bg-surface-container-highest data-[tier=1]:text-score-1 data-[tier=2]:text-score-2 data-[tier=3]:text-score-3 data-[tier=4]:text-score-4 data-[tier=5]:text-score-5"
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

export function PlayerRolesPanel({ player }: PlayerRolesPanelProps) {
  const [selectedPosition, setSelectedPosition] = useState(() =>
    defaultProfilePosition(player.positions, player.roleScores),
  );
  const roles = rolesForProfilePosition(player.roleScores, selectedPosition);
  const familiarity = player.positions[selectedPosition];

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
          <div className="flex items-end justify-between gap-3 border-b border-outline-variant pb-2">
            <div>
              <h3 className="text-headline-sm text-on-surface">
                {selectedPosition}
              </h3>
              <p className="text-body-sm text-on-surface-variant">
                {roles.length} {roles.length === 1 ? "role" : "roles"}
                {typeof familiarity === "number" && familiarity > 0
                  ? ` · familiarity ${familiarity}`
                  : ""}
              </p>
            </div>
            <div className="grid shrink-0 grid-cols-[52px_62px] text-center text-label-sm text-on-surface-variant">
              <span>Current</span>
              <span>Potential</span>
            </div>
          </div>
          <ul className="min-h-0 flex-1 overflow-y-auto pr-1">
            {roles.length === 0 ? (
              <li className="flex min-h-24 items-center justify-center text-center text-body-sm text-on-surface-variant">
                No catalog roles use this position.
              </li>
            ) : null}
            {roles.map((role) => (
              <li
                key={role.roleId}
                className="grid min-h-12 grid-cols-[minmax(0,1fr)_52px_62px] items-center gap-1 border-b border-outline-variant/70"
              >
                <div className="min-w-0 pr-2">
                  <p className="truncate text-body-md text-on-surface">
                    {role.displayName}
                  </p>
                  <p className="text-[11px] text-on-surface-variant">
                    {rolePhaseLabel(role.phase)}
                  </p>
                </div>
                <span className="flex justify-center">
                  <RoleScore
                    roleName={role.displayName}
                    basis="Current"
                    score={role.score}
                  />
                </span>
                <span className="flex justify-center">
                  <RoleScore
                    roleName={role.displayName}
                    basis="Potential"
                    score={role.potentialScore}
                  />
                </span>
              </li>
            ))}
          </ul>
        </div>
      </section>
    </Panel>
  );
}
