import { Building2, ChevronDown, ChevronUp, TriangleAlert } from "lucide-react";
import { useId, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import type {
  CoachRequirement,
  StaffAssignmentOptimization,
  StaffAssignmentSlot,
} from "../types/staff-assignment";

type StaffAssignmentResultsProps = {
  result: StaffAssignmentOptimization;
  onRequestConfiguration: () => void;
  onReviewShortlist: () => void;
};

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

function coachRequirementText(requirement: CoachRequirement | null) {
  return requirement
    ? `Coach requirement: ${coachRequirementLabels[requirement]}.`
    : null;
}

function evidenceText(slot: Extract<StaffAssignmentSlot, { kind: "vacancy" }>) {
  const { eligibleScoreCount, joinedCandidateCount, unavailableScoreCount } =
    slot.evidence;
  const evidence = `${eligibleScoreCount} eligible score${eligibleScoreCount === 1 ? "" : "s"}; ${unavailableScoreCount} unavailable score${unavailableScoreCount === 1 ? "" : "s"}; ${joinedCandidateCount} joined shortlisted candidate${joinedCandidateCount === 1 ? "" : "s"}.`;
  const requirement = coachRequirementText(slot.coachRequirement);
  return requirement ? `${requirement} ${evidence}` : evidence;
}

export function StaffAssignmentResults({
  result,
  onRequestConfiguration,
  onReviewShortlist,
}: StaffAssignmentResultsProps) {
  const [expanded, setExpanded] = useState(true);
  const bodyId = useId();
  const ToggleIcon = expanded ? ChevronUp : ChevronDown;
  const filledSlotCount = result.slots.filter(
    (slot) => slot.kind === "recommendation",
  ).length;
  const vacancyCount = result.slots.filter(
    (slot) => slot.kind === "vacancy",
  ).length;
  const currentStaffCount = result.slots.filter(
    (slot) =>
      slot.kind === "recommendation" && slot.classification === "current_staff",
  ).length;
  const recruitCount = result.slots.filter(
    (slot) =>
      slot.kind === "recommendation" && slot.classification === "recruitment",
  ).length;

  return (
    <Panel
      title="Assignment recommendations"
      actions={
        <Button
          size="icon"
          variant="ghost"
          icon={ToggleIcon}
          aria-label={`${expanded ? "Collapse" : "Expand"} assignment recommendations`}
          aria-controls={bodyId}
          aria-expanded={expanded}
          onClick={() => setExpanded((current) => !current)}
        />
      }
      className="w-full shrink-0 basis-full"
    >
      <div id={bodyId} hidden={!expanded} className="space-y-3">
        <dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {[
            ["Filled slots", filledSlotCount],
            ["Vacancies", vacancyCount],
            ["Current staff", currentStaffCount],
            ["Recruits", recruitCount],
          ].map(([label, count]) => (
            <div key={label}>
              <dt className="text-label-md text-on-surface-variant">{label}</dt>
              <dd className="text-headline-sm text-on-surface">{count}</dd>
            </div>
          ))}
        </dl>
        <p className="text-body-sm text-on-surface-variant">
          {result.joinedCandidateCount} joined shortlisted candidate
          {result.joinedCandidateCount === 1 ? "" : "s"};{" "}
          {result.configuredSlotCount} configured slot
          {result.configuredSlotCount === 1 ? "" : "s"}.
        </p>
        {result.slots.length > 0 ? (
          <div className="max-h-80 overflow-auto rounded-lg border border-outline-variant">
            <table className="w-full text-left text-body-sm text-on-surface">
              <caption className="sr-only">
                Staff assignment recommendations and vacancies
              </caption>
              <thead className="bg-surface-container-lowest text-label-md text-on-surface-variant">
                <tr>
                  <th scope="col" className="px-2 py-2">
                    Scope
                  </th>
                  <th scope="col" className="px-2 py-2">
                    Target
                  </th>
                  <th scope="col" className="px-2 py-2">
                    Person
                  </th>
                  <th scope="col" className="px-2 py-2 text-right">
                    Score
                  </th>
                  <th scope="col" className="px-2 py-2">
                    Evidence
                  </th>
                </tr>
              </thead>
              <tbody>
                {result.slots.map((slot) => (
                  <tr
                    key={`${slot.scope}:${slot.jobId}:${slot.slotNumber}`}
                    className="border-t border-outline-variant"
                  >
                    <td className="px-2 py-2">{slot.scopeDisplayName}</td>
                    <td className="px-2 py-2">
                      {slot.jobLabel} · Slot {slot.slotNumber}
                    </td>
                    {slot.kind === "recommendation" ? (
                      <>
                        <td
                          className="max-w-48 px-2 py-2"
                          title={slot.name ?? undefined}
                        >
                          <span className="flex min-w-0 items-center gap-1">
                            {slot.classification === "current_staff" ? (
                              <Building2
                                aria-label="Current staff"
                                className="size-3.5 shrink-0 text-info"
                                role="img"
                              />
                            ) : null}
                            <span
                              className={`truncate ${slot.classification === "current_staff" ? "font-medium text-info" : ""}`}
                            >
                              {slot.name ?? "—"}
                            </span>
                          </span>
                        </td>
                        <td className="px-2 py-2 text-right">
                          <ScoreBadge
                            score={slot.score}
                            roleName={slot.jobLabel}
                          />
                        </td>
                        <td className="px-2 py-2 text-on-surface-variant">
                          Preferred Job: {slot.preferredJob}. Eligible for this
                          target.
                          {slot.coachRequirement
                            ? ` ${coachRequirementText(slot.coachRequirement)}`
                            : null}
                        </td>
                      </>
                    ) : (
                      <>
                        <td className="px-2 py-2">
                          <StatusChip tone="warning" icon={TriangleAlert}>
                            Vacancy
                          </StatusChip>
                        </td>
                        <td className="px-2 py-2 text-right">—</td>
                        <td className="space-y-2 px-2 py-2 text-on-surface-variant">
                          <p>
                            No eligible shortlisted candidate filled this slot.
                          </p>
                          <p>{evidenceText(slot)}</p>
                          <div className="flex flex-wrap gap-2">
                            <Button
                              variant="secondary"
                              onClick={onRequestConfiguration}
                            >
                              Adjust staffing needs
                            </Button>
                            <Button
                              variant="secondary"
                              onClick={onReviewShortlist}
                            >
                              Review shortlist
                            </Button>
                          </div>
                        </td>
                      </>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p role="status" className="text-body-md text-on-surface-variant">
            No assignment slots are configured.
          </p>
        )}
        {result.unsupportedPreferredJobCount > 0 ? (
          <p className="text-body-sm text-on-surface-variant">
            {result.unsupportedPreferredJobCount} shortlisted person
            {result.unsupportedPreferredJobCount === 1 ? " has" : "s have"} an
            unsupported Preferred Job and cannot fill an assignment target.
          </p>
        ) : null}
      </div>
    </Panel>
  );
}
