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
  plannerString,
  canRemove,
  menuOpen,
  onOpenMenu,
  onCloseMenu,
  onAdd,
  onRemove,
  addDisabled,
  triggerRef,
}: {
  plannerString: PlannerString;
  canRemove: boolean;
  menuOpen: boolean;
  onOpenMenu: () => void;
  onCloseMenu: () => void;
  onAdd: () => void;
  onRemove: () => void;
  addDisabled: boolean;
  triggerRef: (element: HTMLButtonElement | null) => void;
}) {
  const label = ordinal(plannerString.stringOrder);

  return (
    <th
      scope="col"
      className="h-table-header-height min-w-52 border-b border-outline-variant px-3 text-right font-mono text-mono-sm text-on-surface tabular-nums"
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
          aria-label={`Manage ${label}`}
          aria-expanded={menuOpen}
          aria-haspopup="menu"
          className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
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
  plannerString,
  onOpen,
}: {
  team: PlannerTeam;
  teamLabel: string;
  laneId: string;
  laneName: string;
  plannerString: PlannerString;
  onOpen: (target: PlannerSlotTarget) => void;
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
    <td className="h-table-row-height-two-line min-w-52 border-b border-outline-variant px-3 py-1.5 align-middle">
      <button
        type="button"
        className="w-full rounded-md text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
        aria-label={ariaLabel}
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
  teamDepth: PlannerDepthTeam;
  teamLabel: string;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  onOpen: (target: PlannerSlotTarget) => void;
  openStringId: number | null;
  onOpenStringMenu: (stringId: number) => void;
  onCloseStringMenu: () => void;
  onAddString: (team: PlannerTeam, stringId: number) => void;
  onRemoveString: (plannerString: PlannerString) => void;
  addDisabled: boolean;
  stringHeaderRef: (
    stringId: number,
  ) => (element: HTMLButtonElement | null) => void;
};

export function PlannerDepthTable({
  teamDepth,
  teamLabel,
  tactic,
  options,
  onOpen,
  openStringId,
  onOpenStringMenu,
  onCloseStringMenu,
  onAddString,
  onRemoveString,
  addDisabled,
  stringHeaderRef,
}: PlannerDepthTableProps) {
  return (
    <section
      className="max-h-[min(70vh,720px)] overflow-auto rounded-lg border border-outline-variant"
      aria-label={`${teamLabel} squad depth matrix`}
    >
      <table
        className="min-w-max w-full border-collapse text-left"
        aria-label={`${teamLabel} squad depth matrix`}
      >
        <caption className="sr-only">
          {teamLabel} squad depth using the shared tactic
        </caption>
        <thead className="sticky top-0 z-20">
          <tr className="bg-surface-container-high">
            <th
              scope="col"
              className="sticky left-0 z-30 h-table-header-height min-w-52 border-b border-r border-outline-variant bg-surface-container-high px-3 text-label-md text-on-surface"
            >
              Tactical position
            </th>
            {teamDepth.strings.map((plannerString) => (
              <PlannerStringHeader
                key={plannerString.id}
                plannerString={plannerString}
                canRemove={teamDepth.strings.length > 1}
                menuOpen={openStringId === plannerString.id}
                onOpenMenu={() => onOpenStringMenu(plannerString.id)}
                onCloseMenu={onCloseStringMenu}
                onAdd={() => onAddString(teamDepth.team, plannerString.id)}
                onRemove={() => onRemoveString(plannerString)}
                addDisabled={addDisabled}
                triggerRef={stringHeaderRef(plannerString.id)}
              />
            ))}
          </tr>
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
                {teamDepth.strings.map((plannerString) => (
                  <AssignmentCell
                    key={plannerString.id}
                    team={teamDepth.team}
                    teamLabel={teamLabel}
                    laneId={lane.laneId}
                    laneName={positionDescription}
                    plannerString={plannerString}
                    onOpen={onOpen}
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
