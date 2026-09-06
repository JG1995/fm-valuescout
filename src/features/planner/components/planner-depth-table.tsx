import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import type {
  PlannerAssignment,
  PlannerDepth,
  PlannerDepthTeam,
  PlannerString,
} from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import {
  linkedPositionDescription,
  orderedTacticLanes,
  phaseDescription,
} from "../utils/tactic-editor";
import { joinPlannerTeamNames } from "../utils/team-display";
import type { PlannerSlotTarget } from "./planner-slot-fit-picker";

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

function AssignmentScore({
  label,
  score,
}: {
  label: "Current" | "Potential";
  score: number | null;
}) {
  return (
    <span className="inline-flex shrink-0 items-center">
      {score === null ? (
        <span
          role="img"
          aria-label={`${label} combined role score: unavailable`}
          className="font-mono text-mono-sm text-on-surface-variant tabular-nums"
        >
          —
        </span>
      ) : (
        <ScoreBadge
          score={score}
          roleName={`${label} combined role score`}
          className="shrink-0"
        />
      )}
    </span>
  );
}

function AssignmentScores({
  currentScore,
  potentialScore,
}: {
  currentScore: number | null;
  potentialScore: number | null;
}) {
  return (
    <span
      className="flex shrink-0 items-center gap-1"
      title="Current to potential combined role score"
    >
      <AssignmentScore label="Current" score={currentScore} />
      <span
        aria-hidden="true"
        className="text-label-sm text-on-surface-variant"
      >
        →
      </span>
      <AssignmentScore label="Potential" score={potentialScore} />
    </span>
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
  const stringLabel = plannerString.displayName;
  const assignment = assignmentForLane(plannerString, laneId);
  const name = assignmentName(assignment);
  const state = assignmentStateLabel(assignment);
  const score = assignment?.combinedScore ?? null;
  const potentialScore = assignment?.potentialCombinedScore ?? null;
  const ariaLabel = assignment
    ? `${teamLabel}, ${stringLabel}, ${laneName}, ${name}, ${state}, current score ${score ?? "—"}, potential score ${potentialScore ?? "—"}`
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
            {assignment ? (
              <AssignmentScores
                currentScore={score}
                potentialScore={potentialScore}
              />
            ) : (
              <span className="text-body-sm text-on-surface-variant">—</span>
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
  combined: boolean;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  onOpen: (target: PlannerSlotTarget) => void;
  cellRef: (
    team: PlannerTeam,
    stringId: number,
    laneId: string,
  ) => (element: HTMLButtonElement | null) => void;
  onCellFocus: (team: PlannerTeam, stringId: number, laneId: string) => void;
};

export function PlannerDepthTable({
  teamDepths,
  combined,
  tactic,
  options,
  onOpen,
  cellRef,
  onCellFocus,
}: PlannerDepthTableProps) {
  const teamDisplayName = (team: PlannerTeam) =>
    teamDepths.find((candidate) => candidate.team === team)?.displayName ??
    team;
  const matrixLabel = combined
    ? "All squads depth matrix"
    : `${teamDepths[0].displayName} squad depth matrix`;
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
      const headerId = `${idPrefix}-${team}-string-${plannerString.id}`;
      return (
        <th
          key={plannerString.id}
          id={headerId}
          scope="col"
          className={`${combined ? "sticky top-8 z-20" : ""} ${combined && (index === 0 || allStrings[index - 1]?.team !== team) ? "border-l-2" : ""} h-table-header-height min-w-52 border-b border-outline-variant bg-surface-container-high px-3 text-left font-mono text-mono-sm text-on-surface tabular-nums`}
        >
          {plannerString.displayName}
        </th>
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
            ? `${joinPlannerTeamNames(teamDepths.map((team) => team.displayName))} squad depth using the shared tactic`
            : `${teamDepths[0].displayName} squad depth using the shared tactic`}
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
                  return (
                    <th
                      key={teamDepth.team}
                      id={groupId}
                      colSpan={teamDepth.strings.length}
                      scope="colgroup"
                      aria-label={`${teamDepth.displayName} squad`}
                      className={`${index > 0 ? "border-l-2" : ""} sticky top-0 z-20 h-table-header-height border-b border-outline-variant bg-surface-container-lowest px-3 text-label-md text-on-surface`}
                    >
                      {teamDepth.displayName} squad
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
          {orderedTacticLanes(tactic.lanes).map((lane) => {
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
                    teamLabel={teamDisplayName(team)}
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
