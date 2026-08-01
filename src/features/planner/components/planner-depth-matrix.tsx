import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
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
}: {
  teamDepth: PlannerDepthTeam;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  onOpen: (target: PlannerSlotTarget) => void;
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
              <th
                scope="col"
                className="min-w-52 border-b border-outline-variant px-3 py-2 text-right font-mono text-mono-sm text-on-surface tabular-nums"
                key={plannerString.id}
              >
                {ordinal(plannerString.stringOrder)}
              </th>
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
  const closeTimerRef = useRef<number | null>(null);
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
    </Panel>
  );
}
