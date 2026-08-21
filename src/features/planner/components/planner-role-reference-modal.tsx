import { useQuery } from "@tanstack/react-query";
import { ArrowDown, ArrowUp, CircleAlert, UsersRound } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Modal } from "@/components/ui/modal/modal";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import { plannerRoleReferenceQueryOptions } from "../api/planner-role-reference-query-options";
import type {
  PlannerRoleReferencePhase,
  PlannerRoleReferencePlayer,
  PlannerRoleReferenceScoreBasis,
} from "../types/role-reference";
import type { PlannerTactic, TacticOptions } from "../types/tactic";
import {
  phasePositionLabel,
  roleLabel,
  type TacticPhase,
} from "../utils/tactic-editor";
import { PlannerTacticPitch } from "./planner-tactic-pitch";

type RoleReferenceSortColumn = "name" | "current" | "potential";
type RoleReferenceSortDirection = "ascending" | "descending";

type RoleReferenceSort = {
  column: RoleReferenceSortColumn;
  direction: RoleReferenceSortDirection;
};

type PlannerRoleReferenceModalProps = {
  activeSaveId: number;
  open: boolean;
  tactic: PlannerTactic;
  options: TacticOptions;
  onClose: () => void;
  returnFocusTo?: HTMLElement | null;
};

const DEFAULT_SORT: RoleReferenceSort = {
  column: "current",
  direction: "descending",
};

const PHASE_OPTIONS: Array<{
  value: PlannerRoleReferencePhase;
  label: string;
  pitch: TacticPhase;
}> = [
  { value: "in_possession", label: "In Possession", pitch: "ip" },
  { value: "out_of_possession", label: "Out of Possession", pitch: "oop" },
];

const BASIS_OPTIONS: Array<{
  value: PlannerRoleReferenceScoreBasis;
  label: string;
}> = [
  { value: "current", label: "Current" },
  { value: "potential", label: "Potential" },
];

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function comparePlayers(
  left: PlannerRoleReferencePlayer,
  right: PlannerRoleReferencePlayer,
  sort: RoleReferenceSort,
) {
  if (sort.column === "name") {
    const nameComparison = left.name.localeCompare(right.name, undefined, {
      sensitivity: "base",
    });
    if (nameComparison !== 0) {
      return sort.direction === "ascending" ? nameComparison : -nameComparison;
    }
    return left.playerUid - right.playerUid;
  }

  const leftScore =
    sort.column === "current" ? left.currentScore : left.potentialScore;
  const rightScore =
    sort.column === "current" ? right.currentScore : right.potentialScore;
  if (leftScore === null && rightScore === null) {
    return (
      left.name.localeCompare(right.name, undefined, { sensitivity: "base" }) ||
      left.playerUid - right.playerUid
    );
  }
  if (leftScore === null) {
    return 1;
  }
  if (rightScore === null) {
    return -1;
  }
  return sort.direction === "ascending"
    ? leftScore - rightScore
    : rightScore - leftScore;
}

function sortPlayers(
  players: PlannerRoleReferencePlayer[],
  sort: RoleReferenceSort,
) {
  return [...players].sort((left, right) => {
    const compared = comparePlayers(left, right, sort);
    if (compared !== 0) {
      return compared;
    }
    return (
      left.name.localeCompare(right.name, undefined, { sensitivity: "base" }) ||
      left.playerUid - right.playerUid
    );
  });
}

function PhaseToggle({
  value,
  selected,
  onSelect,
}: {
  value: PlannerRoleReferencePhase;
  selected: boolean;
  onSelect: () => void;
}) {
  const option = PHASE_OPTIONS.find((candidate) => candidate.value === value);
  if (!option) {
    return null;
  }
  return (
    <label
      className={`relative cursor-pointer rounded-full px-3 py-1.5 text-label-md transition-colors duration-150 ease-out has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-primary ${
        selected
          ? "bg-primary text-on-primary"
          : "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      }`}
    >
      <input
        type="radio"
        name="planner-role-reference-phase"
        checked={selected}
        onChange={onSelect}
        className="absolute inset-0 size-full cursor-pointer opacity-0"
      />
      <span>{option.label}</span>
    </label>
  );
}

function BasisToggle({
  value,
  selected,
  onSelect,
}: {
  value: PlannerRoleReferenceScoreBasis;
  selected: boolean;
  onSelect: () => void;
}) {
  const option = BASIS_OPTIONS.find((candidate) => candidate.value === value);
  if (!option) {
    return null;
  }
  return (
    <label
      className={`relative cursor-pointer rounded-full px-3 py-1.5 text-label-md transition-colors duration-150 ease-out has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-primary ${
        selected
          ? "bg-primary text-on-primary"
          : "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      }`}
    >
      <input
        type="radio"
        name="planner-role-reference-score-basis"
        checked={selected}
        onChange={onSelect}
        className="absolute inset-0 size-full cursor-pointer opacity-0"
      />
      <span>{option.label}</span>
    </label>
  );
}

function SortHeader({
  column,
  label,
  sort,
  onSort,
  className,
}: {
  column: RoleReferenceSortColumn;
  label: string;
  sort: RoleReferenceSort;
  onSort: (column: RoleReferenceSortColumn) => void;
  className?: string;
}) {
  const active = sort.column === column;
  const SortIcon = sort.direction === "ascending" ? ArrowUp : ArrowDown;
  return (
    <th
      scope="col"
      aria-sort={active ? sort.direction : undefined}
      className={className}
    >
      <button
        type="button"
        className={`inline-flex min-h-8 w-full items-center gap-1 rounded-md px-1 text-label-sm transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
          column === "name" ? "justify-start" : "justify-end"
        } ${
          active
            ? "text-primary"
            : "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
        }`}
        onClick={() => onSort(column)}
      >
        <span>{label}</span>
        <span aria-hidden="true" className="inline-flex size-3 items-center">
          {active ? <SortIcon size={12} strokeWidth={1.5} /> : null}
        </span>
      </button>
    </th>
  );
}

function RoleScoreCell({
  score,
  roleName,
  basis,
}: {
  score: number | null;
  roleName: string;
  basis: "Current" | "Potential";
}) {
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
      variant="table"
    />
  );
}

function PlayerTable({
  players,
  roleName,
  sort,
  onSort,
  caption,
}: {
  players: PlannerRoleReferencePlayer[];
  roleName: string;
  sort: RoleReferenceSort;
  onSort: (column: RoleReferenceSortColumn) => void;
  caption: string;
}) {
  return (
    <table className="w-full table-fixed border-collapse">
      <caption className="sr-only">{caption}</caption>
      <colgroup>
        <col />
        <col className="w-[76px]" />
        <col className="w-[84px]" />
      </colgroup>
      <thead className="sticky top-0 z-10 bg-surface-container-highest">
        <tr className="border-b border-outline-variant">
          <SortHeader column="name" label="Name" sort={sort} onSort={onSort} />
          <SortHeader
            column="current"
            label="Current"
            sort={sort}
            onSort={onSort}
            className="text-right"
          />
          <SortHeader
            column="potential"
            label="Potential"
            sort={sort}
            onSort={onSort}
            className="text-right"
          />
        </tr>
      </thead>
      <tbody>
        {players.map((player) => (
          <tr
            key={player.playerUid}
            className="h-11 border-b border-outline-variant/70"
          >
            <th scope="row" className="min-w-0 truncate pr-2 text-left">
              <span
                className="block truncate text-body-md text-on-surface"
                title={player.name}
              >
                {player.name}
              </span>
            </th>
            <td className="text-right">
              <RoleScoreCell
                score={player.currentScore}
                roleName={roleName}
                basis="Current"
              />
            </td>
            <td className="text-right">
              <RoleScoreCell
                score={player.potentialScore}
                roleName={roleName}
                basis="Potential"
              />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function PlannerRoleReferenceModal({
  activeSaveId,
  open,
  tactic,
  options,
  onClose,
  returnFocusTo,
}: PlannerRoleReferenceModalProps) {
  const firstLaneId = tactic.lanes[0]?.laneId ?? "";
  const [phase, setPhase] =
    useState<PlannerRoleReferencePhase>("in_possession");
  const [scoreBasis, setScoreBasis] =
    useState<PlannerRoleReferenceScoreBasis>("current");
  const [selectedLaneId, setSelectedLaneId] = useState(firstLaneId);
  const [highlightedLaneId, setHighlightedLaneId] = useState<string | null>(
    null,
  );
  const [sort, setSort] = useState<RoleReferenceSort>(DEFAULT_SORT);
  const query = useQuery({
    ...plannerRoleReferenceQueryOptions(activeSaveId, phase, scoreBasis),
    enabled: open,
  });

  useEffect(() => {
    if (!open) {
      return;
    }
    setPhase("in_possession");
    setScoreBasis("current");
    setSelectedLaneId(firstLaneId);
    setHighlightedLaneId(null);
    setSort(DEFAULT_SORT);
  }, [firstLaneId, open]);

  useEffect(() => {
    if (!query.data) {
      return;
    }
    if (query.data.lanes.some((lane) => lane.laneId === selectedLaneId)) {
      return;
    }
    setSelectedLaneId(query.data.lanes[0]?.laneId ?? firstLaneId);
  }, [firstLaneId, query.data, selectedLaneId]);

  const selectedPhase =
    PHASE_OPTIONS.find((option) => option.value === phase) ?? PHASE_OPTIONS[0];
  const selectedLane = tactic.lanes.find(
    (lane) => lane.laneId === selectedLaneId,
  );
  const selectedReferenceLane = query.data?.lanes.find(
    (lane) => lane.laneId === selectedLaneId,
  );
  const roleName = selectedLane
    ? roleLabel(selectedLane, selectedPhase.pitch, options)
    : "Selected role";
  const positionName = selectedLane
    ? phasePositionLabel(selectedLane, selectedPhase.pitch, tactic.lanes)
    : "Position";
  const sortedPlayers = useMemo(
    () => sortPlayers(selectedReferenceLane?.players ?? [], sort),
    [selectedReferenceLane?.players, sort],
  );
  const sortedNoEligible = useMemo(
    () => sortPlayers(query.data?.noEligible ?? [], sort),
    [query.data?.noEligible, sort],
  );
  const hasCohortPlayers =
    query.data?.lanes.some((lane) => lane.players.length > 0) ||
    (query.data?.noEligible.length ?? 0) > 0;

  const onSort = (column: RoleReferenceSortColumn) => {
    setSort((current) => ({
      column,
      direction:
        current.column === column && current.direction === "descending"
          ? "ascending"
          : current.column === column
            ? "descending"
            : column === "name"
              ? "ascending"
              : "descending",
    }));
  };

  return (
    <Modal
      open={open}
      title="Best role fit reference"
      variant="informational"
      onClose={onClose}
      returnFocusTo={returnFocusTo}
      className="max-w-[720px]"
    >
      <div className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <fieldset className="min-w-0">
            <legend className="text-label-md text-on-surface">
              Tactic phase
            </legend>
            <div
              className="mt-1 inline-flex rounded-full bg-surface-container p-0.5"
              role="radiogroup"
              aria-label="Tactic phase"
            >
              {PHASE_OPTIONS.map((option) => (
                <PhaseToggle
                  key={option.value}
                  value={option.value}
                  selected={phase === option.value}
                  onSelect={() => setPhase(option.value)}
                />
              ))}
            </div>
          </fieldset>
          <fieldset className="min-w-0">
            <legend className="text-label-md text-on-surface">
              Score basis
            </legend>
            <div
              className="mt-1 inline-flex rounded-full bg-surface-container p-0.5"
              role="radiogroup"
              aria-label="Score basis"
            >
              {BASIS_OPTIONS.map((option) => (
                <BasisToggle
                  key={option.value}
                  value={option.value}
                  selected={scoreBasis === option.value}
                  onSelect={() => {
                    setScoreBasis(option.value);
                    setSort({
                      column: option.value,
                      direction: "descending",
                    });
                  }}
                />
              ))}
            </div>
          </fieldset>
        </div>

        <div className="grid min-h-0 gap-4 md:grid-cols-[minmax(220px,0.85fr)_minmax(0,1.15fr)]">
          <div className="min-w-0">
            <PlannerTacticPitch
              phase={selectedPhase.pitch}
              lanes={tactic.lanes}
              options={options}
              selectedLaneId={selectedLaneId}
              highlightedLaneId={highlightedLaneId}
              onHighlight={setHighlightedLaneId}
              onSelectLane={(laneId) => {
                setSelectedLaneId(laneId);
                setHighlightedLaneId(laneId);
              }}
            />
          </div>
          <section
            aria-label={`${positionName} ${roleName} players`}
            className="min-w-0 rounded-lg border border-outline-variant bg-surface-container"
          >
            <div className="border-b border-outline-variant px-3 py-2">
              <h3
                className="truncate text-headline-sm text-on-surface"
                title={`${positionName} · ${roleName}`}
              >
                {positionName} · {roleName}
              </h3>
              <p className="text-body-sm text-on-surface-variant">
                {selectedPhase.label} · ranked independently of squad need
              </p>
            </div>
            <div className="min-h-0 px-3">
              {query.isPending ? (
                <div
                  className="flex min-h-40 items-center justify-center text-body-sm text-on-surface-variant"
                  role="status"
                >
                  Loading role-fit reference…
                </div>
              ) : query.isError ? (
                <div role="alert">
                  <EmptyState
                    icon={CircleAlert}
                    title="Role-fit reference unavailable"
                  >
                    {errorMessage(query.error)}
                  </EmptyState>
                </div>
              ) : !hasCohortPlayers ? (
                <EmptyState
                  icon={UsersRound}
                  title="No players at your managed club"
                >
                  No players are available in the current snapshot for this
                  managed club.
                </EmptyState>
              ) : selectedReferenceLane && sortedPlayers.length > 0 ? (
                <PlayerTable
                  players={sortedPlayers}
                  roleName={`${positionName} · ${roleName}`}
                  sort={sort}
                  onSort={onSort}
                  caption={`Players best suited to ${positionName} ${roleName}`}
                />
              ) : (
                <EmptyState icon={UsersRound} title="No eligible players">
                  No eligible players have a score for this role on the selected
                  basis.
                </EmptyState>
              )}

              {!query.isPending &&
              !query.isError &&
              sortedNoEligible.length > 0 ? (
                <section
                  className="border-t border-outline-variant py-3"
                  aria-label="No eligible role"
                >
                  <h3 className="mb-2 text-label-lg text-on-surface">
                    No eligible role
                  </h3>
                  <PlayerTable
                    players={sortedNoEligible}
                    roleName="Unavailable role fit"
                    sort={sort}
                    onSort={onSort}
                    caption="Players without an eligible score for this tactic"
                  />
                </section>
              ) : null}
            </div>
          </section>
        </div>
      </div>
    </Modal>
  );
}
