import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import type {
  PlannerAssignment,
  PlannerDepth,
  PlannerDepthTeam,
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
  plannerString: PlannerDepthTeam["strings"][number],
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
}: {
  team: PlannerTeam;
  teamLabel: string;
  laneId: string;
  laneName: string;
  rowHeaderId: string;
  teamHeaderId: string;
  stringHeaderId: string;
  teamStart: boolean;
  plannerString: PlannerDepthTeam["strings"][number];
  onOpen: (target: PlannerSlotTarget) => void;
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
      headers={[rowHeaderId, teamHeaderId, stringHeaderId].join(" ")}
      className={`${teamStart ? "border-l-2" : ""} w-52 max-w-52 min-w-52 border-b border-outline-variant px-3 py-1.5 align-middle`}
    >
      <button
        type="button"
        data-planner-team={team}
        className="block w-full rounded-md border border-outline-variant bg-surface-container-high px-2 py-1.5 text-left transition-colors duration-150 ease-out hover:bg-surface-container-highest focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
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
            {assignment ? (
              <AssignmentScores
                currentScore={score}
                potentialScore={potentialScore}
              />
            ) : (
              <span className="shrink-0 text-body-sm font-medium text-on-surface">
                Assign
              </span>
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

type PlannerSquadBoardProps = {
  teamDepths: PlannerDepthTeam[];
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  onOpen: (target: PlannerSlotTarget) => void;
};

export function PlannerSquadBoard({
  teamDepths,
  tactic,
  options,
  onOpen,
}: PlannerSquadBoardProps) {
  const teamDisplayName = (team: PlannerTeam) =>
    teamDepths.find((candidate) => candidate.team === team)?.displayName ??
    team;
  const allStrings = teamDepths.flatMap((teamDepth) =>
    teamDepth.strings.map((plannerString) => ({
      team: teamDepth.team,
      plannerString,
    })),
  );
  const idPrefix = "planner-board";

  return (
    <section
      className="max-h-[min(70vh,720px)] overflow-x-auto overflow-y-auto rounded-lg border border-outline-variant"
      aria-label="Squad depth board"
      data-testid="planner-squad-board"
    >
      <table
        className="w-max border-collapse text-left"
        aria-label="Squad depth board"
      >
        <caption className="sr-only">
          {`${joinPlannerTeamNames(teamDepths.map((team) => team.displayName))} squad depth using the shared tactic`}
        </caption>
        <thead>
          <tr className="bg-surface-container-lowest">
            <th
              rowSpan={2}
              scope="col"
              className="sticky top-0 left-0 z-30 h-table-header-height w-52 max-w-52 min-w-52 border-b border-r border-outline-variant bg-surface-container-lowest px-3 text-label-md text-on-surface"
            >
              Tactical slot
            </th>
            {teamDepths.map((teamDepth, index) => (
              <th
                key={teamDepth.team}
                id={`planner-team-${teamDepth.team}-header`}
                colSpan={teamDepth.strings.length}
                scope="colgroup"
                className={`${index > 0 ? "border-l-2" : ""} sticky top-0 z-20 h-table-header-height border-b border-outline-variant bg-surface-container-lowest px-3 text-label-md text-on-surface`}
              >
                {teamDepth.displayName}
              </th>
            ))}
          </tr>
          <tr className="bg-surface-container-high">
            {allStrings.map(({ team, plannerString }, index) => (
              <th
                key={plannerString.id}
                id={`${idPrefix}-${team}-string-${plannerString.id}`}
                scope="col"
                className={`${index === 0 || allStrings[index - 1]?.team !== team ? "border-l-2" : ""} h-table-header-height w-52 max-w-52 min-w-52 border-b border-outline-variant bg-surface-container-high px-3 text-left font-mono text-mono-sm text-on-surface tabular-nums`}
              >
                {plannerString.displayName}
              </th>
            ))}
          </tr>
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
                  className="sticky left-0 z-10 h-table-row-height-two-line w-52 max-w-52 min-w-52 border-b border-r border-outline-variant bg-surface-container px-3 py-1.5 align-middle"
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
                      index === 0 || allStrings[index - 1]?.team !== team
                    }
                    rowHeaderId={`${idPrefix}-position-${lane.laneId}`}
                    teamHeaderId={`planner-team-${team}-header`}
                    stringHeaderId={`${idPrefix}-${team}-string-${plannerString.id}`}
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
