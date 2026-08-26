import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import type {
  StaffAssignmentOptimization,
  StaffAssignmentSlot,
} from "../types/staff-assignment";

type StaffAssignmentResultsProps = {
  result: StaffAssignmentOptimization;
};

function evidenceText(slot: Extract<StaffAssignmentSlot, { kind: "vacancy" }>) {
  const { eligibleScoreCount, joinedCandidateCount, unavailableScoreCount } =
    slot.evidence;
  return `${eligibleScoreCount} eligible score${eligibleScoreCount === 1 ? "" : "s"}; ${unavailableScoreCount} unavailable score${unavailableScoreCount === 1 ? "" : "s"}; ${joinedCandidateCount} joined shortlisted candidate${joinedCandidateCount === 1 ? "" : "s"}.`;
}

export function StaffAssignmentResults({
  result,
}: StaffAssignmentResultsProps) {
  return (
    <Panel
      title="Assignment recommendations"
      className="w-full shrink-0 basis-full"
      contentClassName="space-y-3"
    >
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
                <th scope="col" className="px-2 py-2">
                  Classification
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
                        className="max-w-48 truncate px-2 py-2"
                        title={slot.name}
                      >
                        {slot.name}
                      </td>
                      <td className="px-2 py-2">
                        {slot.classification === "current_staff"
                          ? "Current staff"
                          : "Recruitment"}
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
                        {slot.coachDiscipline
                          ? ` Coach discipline: ${slot.coachDiscipline}.`
                          : null}
                      </td>
                    </>
                  ) : (
                    <>
                      <td className="px-2 py-2">—</td>
                      <td className="px-2 py-2">Vacancy</td>
                      <td className="px-2 py-2 text-right">—</td>
                      <td className="px-2 py-2 text-on-surface-variant">
                        {evidenceText(slot)}
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
    </Panel>
  );
}
