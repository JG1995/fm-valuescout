import { Pencil, Pin, Plus, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button/button";
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
  label: "Current" | "Expected";
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
  expectedScore,
  potentialScore,
}: {
  currentScore: number | null;
  expectedScore: number | null;
  potentialScore: number | null;
}) {
  return (
    <span
      className="flex shrink-0 items-center gap-1"
      title={`Current to expected combined role score. Potential: ${potentialScore ?? "—"}`}
    >
      <AssignmentScore label="Current" score={currentScore} />
      <span
        aria-hidden="true"
        className="text-label-sm text-on-surface-variant"
      >
        →
      </span>
      <AssignmentScore label="Expected" score={expectedScore} />
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
  const expectedScore = assignment?.expectedCombinedScore ?? null;
  const potentialScore = assignment?.potentialCombinedScore ?? null;
  const ariaLabel = assignment
    ? [
        `${teamLabel}, ${stringLabel}, ${laneName}, ${name}, ${state}, current score ${score ?? "—"}, expected score ${expectedScore ?? "—"}, potential score ${potentialScore ?? "—"}`,
        assignment.age === null ? null : `aged ${assignment.age}`,
        assignment.pinned ? "pinned" : null,
        assignment.overAge ? "over the age limit" : null,
      ]
        .filter((part) => part !== null)
        .join(", ")
    : `${teamLabel}, ${stringLabel}, ${laneName}, Empty`;

  return (
    <td
      headers={[rowHeaderId, teamHeaderId, stringHeaderId].join(" ")}
      className={`${teamStart ? "border-l-2" : ""} w-60 max-w-60 min-w-60 border-b border-outline-variant px-3 py-1.5 align-middle`}
    >
      <button
        type="button"
        data-planner-team={team}
        className={`block min-h-8 w-full rounded-md border px-2 py-1.5 text-left transition-colors duration-150 ease-out hover:bg-surface-container-highest focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${assignment ? "border-outline-variant bg-surface-container-high" : "border-transparent"}`}
        aria-label={ariaLabel}
        onClick={() =>
          onOpen({
            team,
            stringId: plannerString.id,
            stringOrder: plannerString.stringOrder,
            laneId,
            laneName,
            occupant: assignment
              ? {
                  playerUid: assignment.playerUid,
                  name,
                  pinned: assignment.pinned,
                }
              : null,
          })
        }
      >
        <span className="block">
          {assignment ? (
            // One line: the name gives way first, so the age, the pin, and the
            // scores always show.
            <span className="flex min-w-0 items-center gap-1.5 text-body-sm">
              <span className="min-w-0 truncate text-on-surface" title={name}>
                {name}
              </span>
              {assignment.age === null ? null : (
                <span
                  className={`flex shrink-0 items-center gap-1 ${assignment.overAge ? "text-warning" : "text-on-surface-variant"}`}
                  title={assignment.overAge ? "Over the age limit" : undefined}
                >
                  · {assignment.age}
                  {assignment.overAge ? (
                    <TriangleAlert aria-hidden="true" className="size-3" />
                  ) : null}
                </span>
              )}
              {assignment.pinned ? (
                <span
                  className="shrink-0 text-on-surface-variant"
                  title="Pinned: Optimize keeps this player here"
                >
                  <Pin aria-hidden="true" className="size-3" />
                </span>
              ) : null}
              <span className="ml-auto flex shrink-0">
                <AssignmentScores
                  currentScore={score}
                  expectedScore={expectedScore}
                  potentialScore={potentialScore}
                />
              </span>
            </span>
          ) : (
            <span className="text-body-sm font-medium text-on-surface underline decoration-outline underline-offset-4">
              Assign
            </span>
          )}
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
  /** Whether the strings can be added to or edited right now. */
  stringsDisabled: boolean;
  onOpen: (target: PlannerSlotTarget) => void;
  onAddString: (team: PlannerTeam) => void;
  onEditString: (team: PlannerTeam, plannerString: PlannerString) => void;
};

export function PlannerSquadBoard({
  teamDepths,
  tactic,
  options,
  stringsDisabled,
  onOpen,
  onAddString,
  onEditString,
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
      className="max-h-[min(70vh,720px)] w-fit max-w-full overflow-x-auto overflow-y-auto rounded-lg border border-outline-variant"
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
                // The header's name stays the team's: every cell is announced
                // with it. The age limit is its description.
                aria-label={teamDepth.displayName}
                aria-description={
                  teamDepth.maxAge === null
                    ? undefined
                    : `Max age ${teamDepth.maxAge}`
                }
                className={`${index > 0 ? "border-l-2" : ""} sticky top-0 z-20 h-table-header-height border-b border-outline-variant bg-surface-container-lowest pr-1 pl-3 text-label-md text-on-surface`}
              >
                <span className="flex items-center justify-between gap-2">
                  <span className="min-w-0 truncate">
                    {teamDepth.displayName}
                    {teamDepth.maxAge === null ? null : (
                      <span
                        className="ml-2 text-body-sm font-normal text-on-surface-variant"
                        title={`Max age ${teamDepth.maxAge}`}
                      >
                        ≤{teamDepth.maxAge}
                      </span>
                    )}
                  </span>
                  <Button
                    size="icon"
                    variant="ghost"
                    icon={Plus}
                    aria-label={`Add string to ${teamDepth.displayName}`}
                    disabled={stringsDisabled}
                    onClick={() => onAddString(teamDepth.team)}
                  />
                </span>
              </th>
            ))}
          </tr>
          <tr className="bg-surface-container-high">
            {allStrings.map(({ team, plannerString }, index) => (
              <th
                key={plannerString.id}
                id={`${idPrefix}-${team}-string-${plannerString.id}`}
                scope="col"
                aria-label={plannerString.displayName}
                className={`${index === 0 || allStrings[index - 1]?.team !== team ? "border-l-2" : ""} h-table-header-height w-60 max-w-60 min-w-60 border-b border-outline-variant bg-surface-container-high px-3 text-left text-body-sm font-medium text-on-surface`}
              >
                <button
                  type="button"
                  aria-label={`Edit ${teamDisplayName(team)} ${plannerString.displayName}`}
                  title="Rename, reorder, or remove this string"
                  disabled={stringsDisabled}
                  className="flex w-full items-center justify-between gap-2 rounded-sm text-left underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
                  onClick={() => onEditString(team, plannerString)}
                >
                  <span className="min-w-0 truncate">
                    {plannerString.displayName}
                  </span>
                  <Pencil
                    aria-hidden="true"
                    className="size-3 shrink-0 text-on-surface-variant"
                  />
                </button>
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
