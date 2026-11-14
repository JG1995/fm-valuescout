import { Link } from "@tanstack/react-router";
import { ArrowRightLeft } from "lucide-react";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import type { PlannerLoanCandidate } from "../types/depth";
import type { PlannerTactic, TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import { linkedPositionDescriptionForId } from "../utils/tactic-editor";

const headingClass =
  "sticky top-0 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

/**
 * The managed club's players aged 18 to 23 outside the senior squad, ranked by
 * how much a loan would help them.
 */
export function PlannerLoanCandidates({
  candidates,
  error,
  tactic,
  options,
  teamLabels,
}: {
  /** Undefined while the candidates load. */
  candidates: PlannerLoanCandidate[] | undefined;
  error: unknown;
  tactic: PlannerTactic;
  options: TacticOptions;
  teamLabels: Partial<Record<PlannerTeam, string>>;
}) {
  return (
    <Panel title="Loan candidates">
      <p className="max-w-prose pb-3 text-body-md text-on-surface-variant">
        Your players aged 18 to 23 outside the senior squad, most to gain from a
        loan first. The benefit weighs how much each is expected to grow, how
        close he is to the senior squad in his slot, and how little he would
        play here. Recommended players are also marked on the board. A player
        already out on loan may be listed, because loans cannot be read from
        FM27.
      </p>
      {error ? (
        <p className="text-body-sm text-error" role="alert">
          Could not load loan candidates: {errorMessage(error)}
        </p>
      ) : candidates === undefined ? (
        <p className="text-body-md text-on-surface-variant">
          Loading loan candidates…
        </p>
      ) : candidates.length === 0 ? (
        <p className="text-body-md text-on-surface-variant">
          No player aged 18 to 23 is outside the senior squad.
        </p>
      ) : (
        <div className="max-h-[28rem] w-fit max-w-full overflow-auto rounded-md border border-outline-variant">
          <table className="border-collapse text-left text-body-sm text-on-surface">
            <caption className="sr-only">
              Loan candidates, most to gain from a loan first
            </caption>
            <thead>
              <tr>
                <th scope="col" className={headingClass}>
                  Player
                </th>
                <th scope="col" className={headingClass}>
                  Age
                </th>
                <th scope="col" className={headingClass}>
                  Squad
                </th>
                <th scope="col" className={headingClass}>
                  Slot
                </th>
                <th scope="col" className={headingClass}>
                  Now
                </th>
                <th scope="col" className={headingClass}>
                  Expected
                </th>
                <th scope="col" className={headingClass}>
                  Loan benefit
                </th>
              </tr>
            </thead>
            <tbody>
              {candidates.map((candidate) => (
                <tr
                  key={candidate.playerUid}
                  className="border-t border-outline-variant"
                >
                  <th scope="row" className="px-3 py-2 font-normal">
                    <Link
                      to="/players/$uid"
                      params={{ uid: String(candidate.playerUid) }}
                      search={{}}
                      className="underline-offset-2 hover:underline"
                    >
                      {candidate.name}
                    </Link>
                  </th>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {candidate.age}
                  </td>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {candidate.team === null
                      ? "No squad"
                      : `${teamLabels[candidate.team] ?? candidate.team} · ${candidate.stringName}`}
                  </td>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {linkedPositionDescriptionForId(
                      candidate.laneId,
                      tactic.lanes,
                      options,
                    )}
                  </td>
                  <td className="px-3 py-2">
                    <ScoreBadge
                      roleName={`${candidate.name} now`}
                      score={candidate.currentScore}
                    />
                  </td>
                  <td className="px-3 py-2">
                    <ScoreBadge
                      roleName={`${candidate.name} expected`}
                      score={candidate.expectedScore}
                    />
                  </td>
                  <td className="px-3 py-2">
                    <span className="flex items-center gap-2">
                      <span className="w-7 text-right font-mono text-mono-sm text-on-surface tabular-nums">
                        {candidate.benefit}
                      </span>
                      {candidate.recommended ? (
                        <StatusChip tone="info" icon={ArrowRightLeft}>
                          Recommended
                        </StatusChip>
                      ) : null}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </Panel>
  );
}
