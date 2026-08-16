import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { StaffDetail, StaffRoleScore } from "../types/staff-detail";

function orderedScores(scores: StaffRoleScore[]) {
  return scores
    .map((score, index) => ({ score, index }))
    .sort((left, right) => {
      if (left.score.score === right.score.score)
        return left.index - right.index;
      if (left.score.score === null) return 1;
      if (right.score.score === null) return -1;
      return right.score.score - left.score.score;
    })
    .map(({ score }) => score);
}

export function StaffRoleFitPanel({ staff }: { staff: StaffDetail }) {
  return (
    <Panel
      title="Role fit"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          Current score
        </span>
      }
      className="flex min-h-0 flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        <table className="w-full table-fixed border-collapse">
          <caption className="sr-only">Current staff role scores</caption>
          <colgroup>
            <col />
            <col className="w-[84px]" />
          </colgroup>
          <thead className="sticky top-0 z-10 bg-surface-container">
            <tr className="border-b border-outline-variant">
              <th
                scope="col"
                className="pb-2 text-left text-label-md text-on-surface-variant"
              >
                Job title
              </th>
              <th
                scope="col"
                className="pb-2 text-center text-label-md text-on-surface-variant"
              >
                Score
              </th>
            </tr>
          </thead>
          <tbody>
            {orderedScores(staff.roleScores).map((role) => (
              <tr
                key={role.roleId}
                className="h-12 border-b border-outline-variant/70"
              >
                <td className="min-w-0 pr-2 text-body-md text-on-surface">
                  {role.displayName}
                </td>
                <td className="text-center">
                  {role.score === null ? (
                    <span className="font-mono text-mono-sm text-on-surface-variant tabular-nums">
                      {formatMissable(null)}
                    </span>
                  ) : (
                    <ScoreBadge
                      score={role.score}
                      roleName={`${role.displayName} current score`}
                      variant="card"
                    />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Panel>
  );
}
