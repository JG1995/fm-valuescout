import type { LucideIcon } from "lucide-react";
import {
  ArrowRightLeft,
  ArrowUpDown,
  Building2,
  Check,
  RotateCcw,
  TriangleAlert,
  UserPlus,
  UserX,
} from "lucide-react";
import { Fragment, useId } from "react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { Tooltip } from "@/components/ui/tooltip/tooltip";
import type {
  CoachRequirement,
  StaffAssignmentAction,
  StaffAssignmentOptimization,
  StaffAssignmentReplacement,
  StaffAssignmentScope,
  StaffAssignmentSlot,
} from "../types/staff-assignment";

type StaffAssignmentResultsProps = {
  result: StaffAssignmentOptimization;
  onRequestConfiguration: () => void;
  onReviewShortlist: () => void;
  onOpenStaff: (uid: number) => void;
  onSetSkip: (uid: number, skipped: boolean) => void;
  skipDisabled: boolean;
};

type VacancySlot = Extract<StaffAssignmentSlot, { kind: "vacancy" }>;

const coachRequirementLabels: Record<CoachRequirement, string> = {
  attacking_technical: "Attacking Technical",
  attacking_tactical: "Attacking Tactical",
  defending_technical: "Defending Technical",
  defending_tactical: "Defending Tactical",
  possession_technical: "Possession Technical",
  possession_tactical: "Possession Tactical",
  fitness: "Fitness",
  goalkeeping: "Goalkeeping",
};

const actionPresentation: Record<
  StaffAssignmentAction["kind"],
  {
    label: string;
    tone: "success" | "info" | "neutral";
    icon: LucideIcon;
  }
> = {
  keep: { label: "Keep", tone: "success", icon: Check },
  move_team: { label: "Move team", tone: "neutral", icon: ArrowUpDown },
  change_job: { label: "Change job", tone: "neutral", icon: ArrowRightLeft },
  on_staff: { label: "Your staff", tone: "neutral", icon: Building2 },
  recruit: { label: "Recruit", tone: "info", icon: UserPlus },
};

function plural(count: number, one: string, many: string) {
  return `${count} ${count === 1 ? one : many}`;
}

const defaultTeamNames = {
  senior: "First team",
  reserves: "Reserves",
  youth: "Youth",
} as const;

type TeamName = (scope: StaffAssignmentScope) => string | null;

function actionDetail(action: StaffAssignmentAction, teamName: TeamName) {
  switch (action.kind) {
    case "move_team":
      return `Currently ${teamName(action.currentTeam)}`;
    case "change_job":
      return action.currentTeam
        ? `Currently ${teamName(action.currentTeam)} ${action.currentJob}`
        : `Currently ${action.currentJob}`;
    case "recruit":
      return action.club ? `From ${action.club}` : "Unemployed";
    case "keep":
    case "on_staff":
      return null;
  }
}

/** Whom a newcomer to a job replaces, or that the job gains a place when nobody leaves it. */
function replacementDetail(
  replaces: StaffAssignmentReplacement | null,
  teamName: TeamName,
) {
  if (!replaces) {
    return "Adds a place";
  }
  const name = replaces.name ?? `Staff ${replaces.uid}`;
  if (!replaces.movesTo) {
    return `Replaces ${name}`;
  }
  const team = teamName(replaces.movesTo.scope);
  return `Replaces ${name}, who moves to ${team ? `${team} ` : ""}${replaces.movesTo.job}`;
}

/** Why nobody filled a slot, from the optimizer's counts for its job. */
function vacancyReason({ evidence }: VacancySlot) {
  if (evidence.joinedCandidateCount === 0) {
    return "No one on your shortlist or staff fits this job.";
  }
  if (evidence.eligibleScoreCount === 0) {
    return `${plural(evidence.joinedCandidateCount, "person fits", "people fit")} this job, but their ratings are missing.`;
  }
  return "Everyone who fits this job is already picked for another slot.";
}

function groupAdjacentSlots(slots: StaffAssignmentSlot[]) {
  const groups: { displayName: string; slots: StaffAssignmentSlot[] }[] = [];
  for (const slot of slots) {
    const previous = groups.at(-1);
    if (previous?.displayName === slot.scopeDisplayName) {
      previous.slots.push(slot);
    } else {
      groups.push({ displayName: slot.scopeDisplayName, slots: [slot] });
    }
  }
  return groups;
}

/** A job's label, numbered only when the team has more than one of that job. */
function slotLabel(slot: StaffAssignmentSlot, slots: StaffAssignmentSlot[]) {
  const shared = slots.some(
    (other) =>
      other !== slot &&
      other.scope === slot.scope &&
      other.jobId === slot.jobId,
  );
  return shared ? `${slot.jobLabel} ${slot.slotNumber}` : slot.jobLabel;
}

export function StaffAssignmentResults({
  result,
  onRequestConfiguration,
  onReviewShortlist,
  onOpenStaff,
  onSetSkip,
  skipDisabled,
}: StaffAssignmentResultsProps) {
  const skippedHeadingId = useId();
  const surplusHeadingId = useId();
  const teamName: TeamName = (scope) =>
    scope === "club"
      ? null
      : (result.teamNames[scope] ?? defaultTeamNames[scope]);
  const recommendations = result.slots.filter(
    (slot) => slot.kind === "recommendation",
  );
  const vacancyCount = result.slots.length - recommendations.length;
  const recruitCount = recommendations.filter(
    (slot) => slot.action.kind === "recruit",
  ).length;
  const metrics = [
    ["Filled", `${recommendations.length} of ${result.slots.length}`],
    ["Your staff", String(recommendations.length - recruitCount)],
    ["Recruit", String(recruitCount)],
    ["Unfilled", String(vacancyCount)],
  ] as const;

  return (
    <div className="space-y-3">
      <dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {metrics.map(([label, value]) => (
          <div key={label}>
            <dt className="text-label-md text-on-surface-variant">{label}</dt>
            <dd className="text-headline-sm text-on-surface">{value}</dd>
          </div>
        ))}
      </dl>
      <p className="text-body-sm text-on-surface-variant">
        Picked from {plural(result.joinedCandidateCount, "person", "people")} on
        your shortlist and staff.
      </p>
      {vacancyCount > 0 ? (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-warning/40 bg-warning-container px-3 py-2 text-body-sm text-on-warning-container">
          <TriangleAlert aria-hidden="true" className="size-4 shrink-0" />
          <p className="min-w-0 flex-1">
            {plural(vacancyCount, "slot has", "slots have")} no suitable person.
            Lower your staffing needs, or shortlist more staff in FM.
          </p>
          <div className="flex gap-2">
            <Button variant="secondary" onClick={onRequestConfiguration}>
              Adjust staffing needs
            </Button>
            <Button variant="secondary" onClick={onReviewShortlist}>
              Review shortlist
            </Button>
          </div>
        </div>
      ) : null}
      {result.slots.length > 0 ? (
        // The dialog body is the only scroller, so the table never scrolls inside it.
        <div className="overflow-clip rounded-lg border border-outline-variant">
          <table className="w-full text-left text-body-sm text-on-surface">
            <caption className="sr-only">
              Staff assignment recommendations and vacancies
            </caption>
            <thead className="sticky top-0 z-10 bg-surface-container-lowest text-label-md text-on-surface-variant">
              <tr>
                <th scope="col" className="px-2 py-2">
                  Slot
                </th>
                <th scope="col" className="px-2 py-2">
                  Person
                </th>
                <th scope="col" className="px-2 py-2 text-right">
                  Score
                </th>
                <th scope="col" className="px-2 py-2">
                  Action
                </th>
                <th scope="col" className="px-2 py-2">
                  <span className="sr-only">Skip</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {groupAdjacentSlots(result.slots).map((group) => (
                <Fragment
                  key={`${group.displayName}:${group.slots[0].jobId}:${group.slots[0].slotNumber}`}
                >
                  <tr>
                    <th
                      scope="rowgroup"
                      colSpan={5}
                      className="bg-surface-container-high px-2 py-2 text-label-md text-on-surface"
                    >
                      {group.displayName}:{" "}
                      {
                        group.slots.filter(
                          (slot) => slot.kind === "recommendation",
                        ).length
                      }{" "}
                      of {group.slots.length} filled
                    </th>
                  </tr>
                  {group.slots.map((slot) => {
                    const label = slotLabel(slot, result.slots);
                    return (
                      <tr
                        key={`${slot.scope}:${slot.jobId}:${slot.slotNumber}`}
                        className="border-t border-outline-variant"
                      >
                        <td className="px-2 py-2">
                          <span className="block">{label}</span>
                          {slot.coachRequirement ? (
                            <span className="block text-on-surface-variant">
                              {coachRequirementLabels[slot.coachRequirement]}
                            </span>
                          ) : null}
                        </td>
                        {slot.kind === "recommendation" ? (
                          <>
                            <td className="max-w-48 px-2 py-2">
                              {slot.name ? (
                                <button
                                  type="button"
                                  title="Open Staff Profile"
                                  className="block max-w-full cursor-pointer truncate text-left underline decoration-outline underline-offset-2 hover:decoration-on-surface"
                                  onClick={() => onOpenStaff(slot.uid)}
                                >
                                  {slot.name}
                                </button>
                              ) : (
                                "—"
                              )}
                            </td>
                            <td className="px-2 py-2 text-right">
                              <ScoreBadge
                                score={slot.score}
                                roleName={slot.jobLabel}
                              />
                            </td>
                            <td className="px-2 py-2">
                              <span className="flex flex-wrap items-center gap-x-2 gap-y-1">
                                <StatusChip
                                  tone={
                                    actionPresentation[slot.action.kind].tone
                                  }
                                  icon={
                                    actionPresentation[slot.action.kind].icon
                                  }
                                >
                                  {actionPresentation[slot.action.kind].label}
                                </StatusChip>
                                {actionDetail(slot.action, teamName) ? (
                                  <span className="text-on-surface-variant">
                                    {actionDetail(slot.action, teamName)}
                                  </span>
                                ) : null}
                              </span>
                              {slot.action.kind === "keep" ? null : (
                                <span className="mt-1 block text-on-surface-variant">
                                  {replacementDetail(slot.replaces, teamName)}
                                </span>
                              )}
                            </td>
                            <td className="px-2 py-2 text-right">
                              <Button
                                variant="ghost"
                                icon={UserX}
                                aria-label={`Skip ${slot.name ?? `staff ${slot.uid}`} for ${slot.scopeDisplayName} ${label}`}
                                disabled={skipDisabled}
                                className="gap-1 px-2"
                                onClick={() => onSetSkip(slot.uid, true)}
                              >
                                Skip
                              </Button>
                            </td>
                          </>
                        ) : (
                          <>
                            <td colSpan={3} className="px-2 py-2">
                              <span className="flex flex-wrap items-center gap-x-2 gap-y-1">
                                <StatusChip tone="warning" icon={TriangleAlert}>
                                  Unfilled
                                </StatusChip>
                                <span className="text-on-surface-variant">
                                  {vacancyReason(slot)}
                                </span>
                              </span>
                            </td>
                            <td />
                          </>
                        )}
                      </tr>
                    );
                  })}
                </Fragment>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p role="status" className="text-body-md text-on-surface-variant">
          No staffing needs are set.
        </p>
      )}
      {result.surplus.length > 0 ? (
        <section
          aria-labelledby={surplusHeadingId}
          className="flex flex-col gap-2"
        >
          <Tooltip
            content="Your staff who lost every slot they could fill to someone better."
            focusable
          >
            <h3
              id={surplusHeadingId}
              className="w-fit text-label-md text-on-surface-variant"
            >
              Staff to sack ({result.surplus.length})
            </h3>
          </Tooltip>
          <ul className="grid gap-x-6 gap-y-1 text-body-sm text-on-surface sm:grid-cols-2">
            {result.surplus.map((staff) => (
              <li key={staff.uid} className="min-w-0">
                <span className="flex min-w-0 items-baseline gap-2">
                  {staff.name ? (
                    <button
                      type="button"
                      title="Open Staff Profile"
                      className="min-w-0 cursor-pointer truncate text-left underline decoration-outline underline-offset-2 hover:decoration-on-surface"
                      onClick={() => onOpenStaff(staff.uid)}
                    >
                      {staff.name}
                    </button>
                  ) : (
                    <span>Staff {staff.uid}</span>
                  )}
                  {staff.currentJob ? (
                    <span className="shrink-0 text-on-surface-variant">
                      {staff.currentTeam
                        ? `${teamName(staff.currentTeam)} ${staff.currentJob}`
                        : staff.currentJob}
                    </span>
                  ) : null}
                </span>
                {staff.replacedBy ? (
                  <span className="block text-on-surface-variant">
                    Replaced by{" "}
                    {staff.replacedBy.name ?? `Staff ${staff.replacedBy.uid}`}
                  </span>
                ) : null}
              </li>
            ))}
          </ul>
        </section>
      ) : null}
      {result.skipped.length > 0 ? (
        <section
          aria-labelledby={skippedHeadingId}
          className="flex flex-col gap-2"
        >
          <Tooltip
            content="Left out of every recommendation until you restore them."
            focusable
          >
            <h3
              id={skippedHeadingId}
              className="w-fit text-label-md text-on-surface-variant"
            >
              Skipped staff
            </h3>
          </Tooltip>
          <ul className="flex flex-wrap gap-2">
            {result.skipped.map((skip) => {
              const name = skip.name ?? `Staff ${skip.uid}`;
              return (
                <li
                  key={skip.uid}
                  className="flex items-center gap-1 rounded-md border border-outline-variant py-0.5 pr-0.5 pl-2 text-body-sm text-on-surface"
                >
                  <span className="max-w-48 truncate" title={name}>
                    {name}
                  </span>
                  <Button
                    variant="ghost"
                    icon={RotateCcw}
                    aria-label={`Restore ${name}`}
                    disabled={skipDisabled}
                    className="gap-1 px-2"
                    onClick={() => onSetSkip(skip.uid, false)}
                  >
                    Restore
                  </Button>
                </li>
              );
            })}
          </ul>
        </section>
      ) : null}
      {result.unsupportedPreferredJobCount > 0 ? (
        <p className="text-body-sm text-on-surface-variant">
          {plural(
            result.unsupportedPreferredJobCount,
            "person on your shortlist wants a job",
            "people on your shortlist want jobs",
          )}{" "}
          that no slot here covers, so they were left out.
        </p>
      ) : null}
    </div>
  );
}
