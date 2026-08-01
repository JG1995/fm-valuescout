import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Ellipsis, Plus, Trash2 } from "lucide-react";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { addPlannerString } from "../api/add-planner-string";
import { plannerKeys } from "../api/planner-keys";
import { removePlannerString } from "../api/remove-planner-string";
import { PLANNER_TEAMS, type PlannerTeam } from "../types/club-family";
import type {
  PlannerAssignment,
  PlannerDepth,
  PlannerDepthTeam,
  PlannerString,
} from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import { laneLabel, phasePosition, roleLabel } from "../utils/tactic-editor";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

function nextTeam(team: PlannerTeam, key: string): PlannerTeam | null {
  const index = PLANNER_TEAMS.indexOf(team);
  if (index < 0) {
    return null;
  }
  if (key === "Home") {
    return PLANNER_TEAMS[0];
  }
  if (key === "End") {
    return PLANNER_TEAMS[PLANNER_TEAMS.length - 1];
  }
  if (key === "ArrowRight" || key === "ArrowDown") {
    return PLANNER_TEAMS[(index + 1) % PLANNER_TEAMS.length];
  }
  if (key === "ArrowLeft" || key === "ArrowUp") {
    return PLANNER_TEAMS[
      (index - 1 + PLANNER_TEAMS.length) % PLANNER_TEAMS.length
    ];
  }
  return null;
}

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

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
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
      className="min-w-52 border-b border-outline-variant px-3 py-2 text-right font-mono text-mono-sm text-on-surface tabular-nums"
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
          onContextMenu={(event) => {
            event.preventDefault();
            onOpenMenu();
          }}
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
  laneId,
  laneName,
  plannerString,
  onOpen,
}: {
  team: PlannerTeam;
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
    ? `${TEAM_LABELS[team]}, ${stringLabel}, ${laneName}, ${name}, ${state}, score ${score ?? "—"}`
    : `${TEAM_LABELS[team]}, ${stringLabel}, ${laneName}, Empty`;

  return (
    <td className="min-w-52 border-b border-outline-variant px-3 py-2 align-top">
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
        <span className="block space-y-1">
          <span
            className="block truncate text-body-sm text-on-surface"
            title={name}
          >
            {name}
          </span>
          {assignment?.state === "outside_pool" ? (
            <span className="block text-label-sm text-warning">
              Outside pool
            </span>
          ) : null}
          {assignment?.state === "unresolved" ? (
            <span className="block text-label-sm text-warning">Unresolved</span>
          ) : null}
          {score === null ? (
            <span className="font-mono text-mono-sm text-on-surface-variant">
              —
            </span>
          ) : (
            <ScoreBadge score={score} roleName="Combined role score" />
          )}
        </span>
      </button>
    </td>
  );
}

function PlannerDepthTable({
  teamDepth,
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
}: {
  teamDepth: PlannerDepthTeam;
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
}) {
  const teamLabel = TEAM_LABELS[teamDepth.team];

  return (
    <section
      className="overflow-x-auto rounded-lg border border-outline-variant"
      aria-label={`${teamLabel} squad depth matrix`}
    >
      <table
        className="min-w-max w-full border-collapse text-left"
        aria-label={`${teamLabel} squad depth matrix`}
      >
        <caption className="sr-only">
          {teamLabel} squad depth using the shared tactic
        </caption>
        <thead>
          <tr className="bg-surface-container-high">
            <th
              scope="col"
              className="sticky left-0 z-10 min-w-52 border-b border-r border-outline-variant bg-surface-container-high px-3 py-2 text-label-md text-on-surface"
            >
              Tactic lane
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
          {tactic.lanes.map((lane, index) => (
            <tr key={lane.laneId} aria-label={laneLabel(lane.laneId)}>
              <th
                scope="row"
                className="sticky left-0 z-10 min-w-52 border-b border-r border-outline-variant bg-surface-container px-3 py-2 align-top"
              >
                <span className="block text-label-md text-on-surface">
                  {laneLabel(lane.laneId)}
                </span>
                <span className="block font-mono text-mono-sm text-on-surface-variant tabular-nums">
                  Lane {index + 1}
                </span>
                <span className="block text-body-sm text-on-surface-variant">
                  IP: {phasePosition(lane, "ip")} ·{" "}
                  {roleLabel(lane, "ip", options)}
                </span>
                <span className="block text-body-sm text-on-surface-variant">
                  OOP: {phasePosition(lane, "oop")} ·{" "}
                  {roleLabel(lane, "oop", options)}
                </span>
              </th>
              {teamDepth.strings.map((plannerString) => (
                <AssignmentCell
                  key={plannerString.id}
                  team={teamDepth.team}
                  laneId={lane.laneId}
                  laneName={laneLabel(lane.laneId)}
                  plannerString={plannerString}
                  onOpen={onOpen}
                />
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

export function PlannerDepthMatrix({
  depth,
  tactic,
  options,
  activeSaveId,
}: {
  depth: PlannerDepth;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  activeSaveId: number;
}) {
  const [selectedTeam, setSelectedTeam] = useState<PlannerTeam>("senior");
  const [picker, setPicker] = useState<PlannerSlotTarget | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [pickerError, setPickerError] = useState<string | null>(null);
  const [stringError, setStringError] = useState<string | null>(null);
  const [openStringId, setOpenStringId] = useState<number | null>(null);
  const [removalTarget, setRemovalTarget] = useState<PlannerString | null>(
    null,
  );
  const [removalOpen, setRemovalOpen] = useState(false);
  const queryClient = useQueryClient();
  const closeTimerRef = useRef<number | null>(null);
  const removalTimerRef = useRef<number | null>(null);
  const stringHeaderRefs = useRef(new Map<number, HTMLButtonElement>());
  const tabRefs = useRef<Record<PlannerTeam, HTMLButtonElement | null>>({
    senior: null,
    reserves: null,
    youth: null,
  });

  useEffect(() => {
    return () => {
      if (closeTimerRef.current !== null) {
        window.clearTimeout(closeTimerRef.current);
      }
      if (removalTimerRef.current !== null) {
        window.clearTimeout(removalTimerRef.current);
      }
    };
  }, []);

  const openPicker = (target: PlannerSlotTarget) => {
    if (closeTimerRef.current !== null) {
      window.clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }
    setPickerError(null);
    setPicker(target);
    setPickerOpen(true);
  };

  const closePicker = () => {
    setPickerOpen(false);
    closeTimerRef.current = window.setTimeout(() => {
      setPicker(null);
      closeTimerRef.current = null;
    }, 200);
  };

  const handleTabKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const next = nextTeam(selectedTeam, event.key);
    if (!next) {
      return;
    }
    event.preventDefault();
    setSelectedTeam(next);
    tabRefs.current[next]?.focus();
  };

  const returnToStringHeader = (
    stringId: number,
    clearRemovalTarget = false,
  ) => {
    if (removalTimerRef.current !== null) {
      window.clearTimeout(removalTimerRef.current);
    }
    removalTimerRef.current = window.setTimeout(() => {
      stringHeaderRefs.current.get(stringId)?.focus();
      if (clearRemovalTarget) {
        setRemovalTarget(null);
      }
      removalTimerRef.current = null;
    }, 200);
  };

  const completeStringAction = (stringId: number) => {
    setOpenStringId(null);
    setRemovalOpen(false);
    returnToStringHeader(stringId, true);
  };

  const addString = useMutation({
    mutationFn: ({ team }: { team: PlannerTeam; originStringId: number }) =>
      addPlannerString(team),
    onSuccess: (nextDepth, { team }) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      setOpenStringId(null);
      const teamDepth = nextDepth.teams.find(
        (candidate) => candidate.team === team,
      );
      const addedString = teamDepth?.strings.at(-1);
      if (addedString) {
        returnToStringHeader(addedString.id);
      }
    },
    onError: (error, { originStringId }) => {
      setStringError(errorMessage(error));
      setOpenStringId(null);
      returnToStringHeader(originStringId);
    },
  });

  const removeString = useMutation({
    mutationFn: ({
      plannerString,
      confirmPopulated,
    }: {
      plannerString: PlannerString;
      confirmPopulated: boolean;
    }) => removePlannerString(plannerString.id, confirmPopulated),
    onSuccess: (nextDepth, variables) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      const team = depth.teams.find((candidate) =>
        candidate.strings.some(
          (plannerString) => plannerString.id === variables.plannerString.id,
        ),
      );
      const remainingStrings = nextDepth.teams.find(
        (candidate) => candidate.team === team?.team,
      )?.strings;
      const focusTarget = remainingStrings?.at(
        Math.min(
          variables.plannerString.stringOrder,
          (remainingStrings.length ?? 1) - 1,
        ),
      );
      completeStringAction(focusTarget?.id ?? variables.plannerString.id);
    },
    onError: (error, variables) => {
      setStringError(errorMessage(error));
      completeStringAction(variables.plannerString.id);
    },
  });

  const requestRemoveString = (plannerString: PlannerString) => {
    setStringError(null);
    if (plannerString.assignments.length === 0) {
      removeString.mutate({ plannerString, confirmPopulated: false });
      return;
    }
    setOpenStringId(null);
    setRemovalTarget(plannerString);
    setRemovalOpen(true);
  };

  const closeRemoval = () => {
    if (!removalTarget) {
      return;
    }
    completeStringAction(removalTarget.id);
  };

  if (depth.teams.length === 0) {
    return null;
  }

  return (
    <Panel title="Squad depth" flush>
      <div className="space-y-4 p-4">
        {pickerError ? (
          <p className="text-body-sm text-error" role="alert">
            {pickerError}
          </p>
        ) : null}
        {stringError ? (
          <p className="text-body-sm text-error" role="alert">
            {stringError}
          </p>
        ) : null}
        <div
          role="tablist"
          aria-label="Squad planner teams"
          className="inline-flex rounded-full bg-surface-container-high p-0.5"
          onKeyDown={handleTabKeyDown}
        >
          {PLANNER_TEAMS.map((team) => {
            const selected = team === selectedTeam;
            return (
              <button
                key={team}
                ref={(element) => {
                  tabRefs.current[team] = element;
                }}
                type="button"
                role="tab"
                id={`${team}-depth-tab`}
                aria-selected={selected}
                aria-controls={`${team}-depth-panel`}
                tabIndex={selected ? 0 : -1}
                className={`cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "bg-primary text-on-primary"
                    : "text-on-surface-variant hover:text-on-surface"
                }`}
                onClick={() => setSelectedTeam(team)}
              >
                {TEAM_LABELS[team]}
              </button>
            );
          })}
        </div>
        {PLANNER_TEAMS.map((team) => {
          const teamDepth = depth.teams.find(
            (candidate) => candidate.team === team,
          );
          if (!teamDepth) {
            return null;
          }
          return (
            <div
              id={`${team}-depth-panel`}
              key={team}
              role="tabpanel"
              aria-labelledby={`${team}-depth-tab`}
              hidden={team !== selectedTeam}
            >
              <PlannerDepthTable
                teamDepth={teamDepth}
                tactic={tactic}
                options={options}
                onOpen={openPicker}
                openStringId={openStringId}
                onOpenStringMenu={setOpenStringId}
                onCloseStringMenu={() => setOpenStringId(null)}
                onAddString={(team, originStringId) => {
                  setStringError(null);
                  addString.mutate({ team, originStringId });
                }}
                onRemoveString={requestRemoveString}
                addDisabled={addString.isPending}
                stringHeaderRef={(stringId) => (element) => {
                  if (element) {
                    stringHeaderRefs.current.set(stringId, element);
                  } else {
                    stringHeaderRefs.current.delete(stringId);
                  }
                }}
              />
            </div>
          );
        })}
      </div>
      {picker ? (
        <PlannerSlotFitPicker
          activeSaveId={activeSaveId}
          open={pickerOpen}
          target={picker}
          onClose={closePicker}
          onMutationError={setPickerError}
        />
      ) : null}
      {removalTarget ? (
        <Modal
          open={removalOpen}
          title={`Remove ${ordinal(removalTarget.stringOrder)}?`}
          variant="destructive"
          onClose={closeRemoval}
          footer={
            <>
              <Button variant="secondary" onClick={closeRemoval}>
                Cancel
              </Button>
              <Button
                disabled={removeString.isPending}
                onClick={() =>
                  removeString.mutate({
                    plannerString: removalTarget,
                    confirmPopulated: true,
                  })
                }
              >
                {removeString.isPending ? "Removing…" : "Remove string"}
              </Button>
            </>
          }
        >
          <p className="text-body-md text-on-surface-variant">
            This removes {ordinal(removalTarget.stringOrder)} and its{" "}
            {removalTarget.assignments.length} assignment
            {removalTarget.assignments.length === 1 ? "" : "s"}:{" "}
            {removalTarget.assignments
              .map((assignment) => assignmentName(assignment))
              .join(", ")}
            .
          </p>
        </Modal>
      ) : null}
    </Panel>
  );
}
