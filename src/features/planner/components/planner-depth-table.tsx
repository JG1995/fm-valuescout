import { Ellipsis, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import type { PlannerTeam } from "../types/club-family";
import type {
  PlannerAssignment,
  PlannerDepth,
  PlannerDepthTeam,
  PlannerString,
} from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import {
  linkedPositionDescription,
  phaseDescription,
} from "../utils/tactic-editor";
import { PlannerClearTeamControl } from "./planner-clear-team-control";
import type { PlannerSlotTarget } from "./planner-slot-fit-picker";

function ordinal(value: number): string {
  const number = value + 1;
  const suffix =
    number % 100 >= 11 && number % 100 <= 13
      ? "th"
      : number % 10 === 1
        ? "st"
        : number % 10 === 2
          ? "nd"
          : number % 10 === 3
            ? "rd"
            : "th";
  return `${number}${suffix} string`;
}

function assignmentForLane(
  plannerString: PlannerString,
  laneId: string,
): PlannerAssignment | undefined {
  return plannerString.assignments.find(
    (assignment) => assignment.laneId === laneId,
  );
}

function assignmentName(assignment: PlannerAssignment | undefined): string {
  if (!assignment) {
    return "Empty";
  }
  return assignment.currentName ?? assignment.lastKnownName;
}

function assignmentStateLabel(
  assignment: PlannerAssignment | undefined,
): string {
  if (!assignment) {
    return "Empty";
  }
  if (assignment.state === "outside_pool") {
    return "Outside pool";
  }
  if (assignment.state === "unresolved") {
    return "Unresolved";
  }
  return "Resolved";
}

function PlannerStringHeader({
  team,
  plannerString,
  headerId,
  combined,
  teamStart,
  canRemove,
  menuOpen,
  onOpenMenu,
  onCloseMenu,
  onAdd,
  onRemove,
  addDisabled,
  triggerRef,
  onFocus,
}: {
  team: PlannerTeam;
  plannerString: PlannerString;
  headerId: string;
  combined: boolean;
  teamStart: boolean;
  canRemove: boolean;
  menuOpen: boolean;
  onOpenMenu: () => void;
  onCloseMenu: () => void;
  onAdd: () => void;
  onRemove: () => void;
  addDisabled: boolean;
  triggerRef: (element: HTMLButtonElement | null) => void;
  onFocus: () => void;
}) {
  const label = ordinal(plannerString.stringOrder);

  return (
    <th
      id={headerId}
      scope="col"
      className={`${combined ? "sticky top-8 z-20" : ""} ${teamStart ? "border-l-2" : ""} h-table-header-height min-w-52 border-b border-outline-variant bg-surface-container-high px-3 text-right font-mono text-mono-sm text-on-surface tabular-nums`}
      onContextMenu={(event) => {
        event.preventDefault();
        onOpenMenu();
      }}
    >
      <div className="relative flex items-center justify-between gap-2">
        <span>{label}</span>
        <button
          ref={triggerRef}
          type="button"
          data-planner-team={team}
          data-planner-string-id={plannerString.id}
          aria-label={`Manage ${label}`}
          aria-expanded={menuOpen}
          aria-haspopup="menu"
          className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
          onFocus={onFocus}
          onClick={() => (menuOpen ? onCloseMenu() : onOpenMenu())}
        >
          <Ellipsis aria-hidden="true" size={16} strokeWidth={1.5} />
        </button>
        {menuOpen ? (
          <div
            role="menu"
            aria-label={`${label} actions`}
            className="absolute right-0 top-full z-20 mt-1 w-44 rounded-md border border-outline-variant bg-surface-container-highest p-1 text-left shadow-overlay"
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                onCloseMenu();
              }
            }}
          >
            <button
              type="button"
              role="menuitem"
              disabled={addDisabled}
              className="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-label-md text-on-surface hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
              onClick={onAdd}
            >
              <Plus aria-hidden="true" size={16} strokeWidth={1.5} />
              Add string
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!canRemove}
              className="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-label-md text-error hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
              onClick={onRemove}
            >
              <Trash2 aria-hidden="true" size={16} strokeWidth={1.5} />
              Remove string
            </button>
          </div>
        ) : null}
      </div>
    </th>
  );
}

function AssignmentCell({
  team,
  teamLabel,
  laneId,
  laneName,
  rowHeaderId,
  teamHeaderId,
  stringHeaderId,
  teamStart,
  plannerString,
  onOpen,
  cellRef,
  onFocus,
}: {
  team: PlannerTeam;
  teamLabel: string;
  laneId: string;
  laneName: string;
  rowHeaderId: string;
  teamHeaderId?: string;
  stringHeaderId: string;
  teamStart: boolean;
  plannerString: PlannerString;
  onOpen: (target: PlannerSlotTarget) => void;
  cellRef: (element: HTMLButtonElement | null) => void;
  onFocus: () => void;
}) {
  const stringLabel = ordinal(plannerString.stringOrder);
  const assignment = assignmentForLane(plannerString, laneId);
  const name = assignmentName(assignment);
  const state = assignmentStateLabel(assignment);
  const score = assignment?.combinedScore ?? null;
  const ariaLabel = assignment
    ? `${teamLabel}, ${stringLabel}, ${laneName}, ${name}, ${state}, score ${score ?? "—"}`
    : `${teamLabel}, ${stringLabel}, ${laneName}, Empty`;

  return (
    <td
      headers={[rowHeaderId, teamHeaderId, stringHeaderId]
        .filter(Boolean)
        .join(" ")}
      className={`${teamStart ? "border-l-2" : ""} h-table-row-height-two-line min-w-52 border-b border-outline-variant px-3 py-1.5 align-middle`}
    >
      <button
        ref={cellRef}
        type="button"
        data-planner-team={team}
        className="w-full rounded-md text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
        aria-label={ariaLabel}
        onFocus={onFocus}
        onClick={() =>
          onOpen({
            team,
            stringId: plannerString.id,
            stringOrder: plannerString.stringOrder,
            laneId,
            laneName,
            occupantName: assignment ? name : null,
          })
        }
      >
        <span className="block">
          <span className="flex min-w-0 items-center justify-between gap-2">
            <span
              className="min-w-0 flex-1 truncate text-body-sm text-on-surface"
              title={name}
            >
              {name}
            </span>
            {score === null ? (
              <span className="shrink-0 font-mono text-mono-sm text-on-surface-variant">
                —
              </span>
            ) : (
              <ScoreBadge
                score={score}
                roleName="Combined role score"
                className="shrink-0"
              />
            )}
          </span>
          {assignment?.state === "outside_pool" ? (
            <span className="block text-label-sm text-warning">
              Outside pool
            </span>
          ) : null}
          {assignment?.state === "unresolved" ? (
            <span className="block text-label-sm text-warning">Unresolved</span>
          ) : null}
        </span>
      </button>
    </td>
  );
}

type PlannerDepthTableProps = {
  teamDepths: PlannerDepthTeam[];
  teamLabels: Record<PlannerTeam, string>;
  combined: boolean;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  onOpen: (target: PlannerSlotTarget) => void;
  openStringId: number | null;
  onOpenStringMenu: (stringId: number) => void;
  onCloseStringMenu: () => void;
  onAddString: (team: PlannerTeam, stringId: number) => void;
  onRemoveString: (plannerString: PlannerString) => void;
  addDisabled: boolean;
  clearDisabled: boolean;
  clearPending: boolean;
  clearTeamTarget: PlannerTeam | null;
  clearTeamOpen: boolean;
  clearTeamError: string | null;
  onRequestClearTeam: (team: PlannerTeam) => void;
  onClearTeamFocus: (team: PlannerTeam) => void;
  onCloseClearTeam: () => void;
  onConfirmClearTeam: (team: PlannerTeam) => void;
  stringHeaderRef: (
    stringId: number,
  ) => (element: HTMLButtonElement | null) => void;
  onStringHeaderFocus: (team: PlannerTeam, stringId: number) => void;
  cellRef: (
    team: PlannerTeam,
    stringId: number,
    laneId: string,
  ) => (element: HTMLButtonElement | null) => void;
  onCellFocus: (team: PlannerTeam, stringId: number, laneId: string) => void;
};

export function PlannerDepthTable({
  teamDepths,
  teamLabels,
  combined,
  tactic,
  options,
  onOpen,
  openStringId,
  onOpenStringMenu,
  onCloseStringMenu,
  onAddString,
  onRemoveString,
  addDisabled,
  clearDisabled,
  clearPending,
  clearTeamTarget,
  clearTeamOpen,
  clearTeamError,
  onRequestClearTeam,
  onClearTeamFocus,
  onCloseClearTeam,
  onConfirmClearTeam,
  stringHeaderRef,
  onStringHeaderFocus,
  cellRef,
  onCellFocus,
}: PlannerDepthTableProps) {
  const matrixLabel = combined
    ? "All squads depth matrix"
    : `${teamLabels[teamDepths[0].team]} squad depth matrix`;
  const allStrings = teamDepths.flatMap((teamDepth) =>
    teamDepth.strings.map((plannerString) => ({
      team: teamDepth.team,
      plannerString,
    })),
  );
  const idPrefix = combined
    ? "planner-combined"
    : `planner-${teamDepths[0].team}`;
  const renderStringHeaders = () =>
    allStrings.map(({ team, plannerString }, index) => {
      const teamDepth = teamDepths.find((candidate) => candidate.team === team);
      const headerId = `${idPrefix}-${team}-string-${plannerString.id}`;
      return (
        <PlannerStringHeader
          key={plannerString.id}
          team={team}
          plannerString={plannerString}
          headerId={headerId}
          combined={combined}
          teamStart={
            combined && (index === 0 || allStrings[index - 1]?.team !== team)
          }
          canRemove={(teamDepth?.strings.length ?? 0) > 1}
          menuOpen={openStringId === plannerString.id}
          onOpenMenu={() => onOpenStringMenu(plannerString.id)}
          onCloseMenu={onCloseStringMenu}
          onAdd={() => onAddString(team, plannerString.id)}
          onRemove={() => onRemoveString(plannerString)}
          addDisabled={addDisabled}
          triggerRef={stringHeaderRef(plannerString.id)}
          onFocus={() => onStringHeaderFocus(team, plannerString.id)}
        />
      );
    });

  return (
    <section
      className="max-h-[min(70vh,720px)] overflow-auto rounded-lg border border-outline-variant"
      aria-label={matrixLabel}
      data-layout-mode={combined ? "combined" : "selected"}
    >
      <table
        className="min-w-max w-full border-collapse text-left"
        aria-label={matrixLabel}
      >
        <caption className="sr-only">
          {combined
            ? "Senior, Reserves, and Youth squad depth using the shared tactic"
            : `${teamLabels[teamDepths[0].team]} squad depth using the shared tactic`}
        </caption>
        <thead>
          {combined ? (
            <>
              <tr className="bg-surface-container-lowest">
                <th
                  rowSpan={2}
                  scope="col"
                  className="sticky top-0 left-0 z-30 h-table-header-height min-w-52 border-b border-r border-outline-variant bg-surface-container-lowest px-3 text-label-md text-on-surface"
                >
                  Tactical position
                </th>
                {teamDepths.map((teamDepth, index) => {
                  const groupId = `planner-team-${teamDepth.team}-header`;
                  const clearTarget =
                    clearTeamTarget === teamDepth.team ? clearTeamTarget : null;
                  return (
                    <th
                      key={teamDepth.team}
                      id={groupId}
                      colSpan={teamDepth.strings.length}
                      scope="colgroup"
                      aria-label={`${teamLabels[teamDepth.team]} squad`}
                      className={`${index > 0 ? "border-l-2" : ""} sticky top-0 z-20 h-table-header-height border-b border-outline-variant bg-surface-container-lowest px-3 text-label-md text-on-surface`}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <span>{teamLabels[teamDepth.team]} squad</span>
                        <PlannerClearTeamControl
                          team={teamDepth.team}
                          target={clearTarget}
                          open={clearTeamOpen && clearTarget !== null}
                          pending={clearPending}
                          disabled={clearDisabled}
                          error={clearTeamError}
                          onRequest={() => onRequestClearTeam(teamDepth.team)}
                          onFocus={() => onClearTeamFocus(teamDepth.team)}
                          onClose={onCloseClearTeam}
                          onConfirm={onConfirmClearTeam}
                        />
                      </div>
                    </th>
                  );
                })}
              </tr>
              <tr className="bg-surface-container-high">
                {renderStringHeaders()}
              </tr>
            </>
          ) : (
            <tr className="bg-surface-container-high">
              <th
                scope="col"
                className="sticky top-0 left-0 z-30 h-table-header-height min-w-52 border-b border-r border-outline-variant bg-surface-container-high px-3 text-label-md text-on-surface"
              >
                Tactical position
              </th>
              {renderStringHeaders()}
            </tr>
          )}
        </thead>
        <tbody>
          {tactic.lanes.map((lane) => {
            const ipDescription = phaseDescription(
              lane,
              "ip",
              tactic.lanes,
              options,
            );
            const oopDescription = phaseDescription(
              lane,
              "oop",
              tactic.lanes,
              options,
            );
            const positionDescription = linkedPositionDescription(
              lane,
              tactic.lanes,
              options,
            );
            return (
              <tr
                key={lane.laneId}
                className="h-table-row-height-two-line"
                aria-label={positionDescription}
              >
                <th
                  id={`${idPrefix}-position-${lane.laneId}`}
                  scope="row"
                  className="sticky left-0 z-10 h-table-row-height-two-line min-w-52 border-b border-r border-outline-variant bg-surface-container px-3 py-1.5 align-middle"
                >
                  <span className="block min-w-0 text-body-sm text-on-surface-variant">
                    <span
                      className="block min-w-0 truncate"
                      title={`IP: ${ipDescription}`}
                    >
                      IP: {ipDescription}
                    </span>
                    <span
                      className="block min-w-0 truncate"
                      title={`OOP: ${oopDescription}`}
                    >
                      OOP: {oopDescription}
                    </span>
                  </span>
                </th>
                {allStrings.map(({ team, plannerString }, index) => (
                  <AssignmentCell
                    key={plannerString.id}
                    team={team}
                    teamLabel={teamLabels[team]}
                    laneId={lane.laneId}
                    laneName={positionDescription}
                    teamStart={
                      combined &&
                      (index === 0 || allStrings[index - 1]?.team !== team)
                    }
                    rowHeaderId={`${idPrefix}-position-${lane.laneId}`}
                    teamHeaderId={
                      combined ? `planner-team-${team}-header` : undefined
                    }
                    stringHeaderId={`${idPrefix}-${team}-string-${plannerString.id}`}
                    plannerString={plannerString}
                    onOpen={onOpen}
                    cellRef={cellRef(team, plannerString.id, lane.laneId)}
                    onFocus={() =>
                      onCellFocus(team, plannerString.id, lane.laneId)
                    }
                  />
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </section>
  );
}

type PlannerStringRemovalConfirmationProps = {
  target: PlannerString | null;
  open: boolean;
  pending: boolean;
  onClose: () => void;
  onConfirm: (plannerString: PlannerString) => void;
};

export function PlannerStringRemovalConfirmation({
  target,
  open,
  pending,
  onClose,
  onConfirm,
}: PlannerStringRemovalConfirmationProps) {
  if (!target) {
    return null;
  }

  return (
    <Modal
      open={open}
      title={`Remove ${ordinal(target.stringOrder)}?`}
      variant="destructive"
      onClose={onClose}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={pending} onClick={() => onConfirm(target)}>
            {pending ? "Removing…" : "Remove string"}
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface-variant">
        This removes {ordinal(target.stringOrder)} and its{" "}
        {target.assignments.length} assignment
        {target.assignments.length === 1 ? "" : "s"}:{" "}
        {target.assignments
          .map((assignment) => assignmentName(assignment))
          .join(", ")}
        .
      </p>
    </Modal>
  );
}
