import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Maximize2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { plannerUnassignedPlayersQueryOptions } from "../api/planner-unassigned-players-query-options";
import type { PlannerTactic, TacticOptions } from "../types/tactic";
import { linkedPositionDescriptionForId } from "../utils/tactic-editor";

const headingClass =
  "sticky top-0 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

function Score({ label, score }: { label: string; score: number | null }) {
  return score === null ? (
    <span className="font-mono text-mono-sm text-on-surface-variant">—</span>
  ) : (
    <ScoreBadge roleName={label} score={score} />
  );
}

/**
 * A button that counts the managed club's players in no squad and opens their
 * list: who the plan leaves out.
 */
export function PlannerUnassignedPlayers({
  activeSaveId,
  tactic,
  options,
}: {
  activeSaveId: number;
  tactic: PlannerTactic;
  options: TacticOptions;
}) {
  const [open, setOpen] = useState(false);
  const query = useQuery(plannerUnassignedPlayersQueryOptions(activeSaveId));
  const players = query.data ?? [];
  return (
    <>
      <Button
        variant="secondary"
        icon={Maximize2}
        disabled={players.length === 0}
        onClick={() => setOpen(true)}
      >
        {query.data ? `Unassigned (${players.length})` : "Unassigned"}
      </Button>
      <Modal
        open={open}
        onClose={() => setOpen(false)}
        title="Unassigned players"
        variant="informational"
        className="max-w-[52rem]!"
      >
        <p className="pb-3 text-body-md text-on-surface-variant">
          Your players in no squad, best fit first. Each shows the slot of your
          tactic he fits best, with his score there now and the score he is
          expected to reach.
        </p>
        <div className="max-h-[28rem] overflow-y-auto rounded-md border border-outline-variant">
          <table className="w-full border-collapse text-left text-body-sm text-on-surface">
            <caption className="sr-only">
              Players in no squad, best current fit first
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
                  Best slot
                </th>
                <th scope="col" className={headingClass}>
                  Now
                </th>
                <th scope="col" className={headingClass}>
                  Expected
                </th>
              </tr>
            </thead>
            <tbody>
              {players.map((player) => (
                <tr
                  key={player.playerUid}
                  className="border-t border-outline-variant"
                >
                  <th scope="row" className="px-3 py-2 font-normal">
                    <Link
                      to="/players/$uid"
                      params={{ uid: String(player.playerUid) }}
                      search={{}}
                      className="underline-offset-2 hover:underline"
                    >
                      {player.name}
                    </Link>
                  </th>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {player.age ?? "—"}
                  </td>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {player.laneId === null
                      ? "Suits no slot"
                      : linkedPositionDescriptionForId(
                          player.laneId,
                          tactic.lanes,
                          options,
                        )}
                  </td>
                  <td className="px-3 py-2">
                    <Score
                      label={`${player.name} now`}
                      score={player.currentScore}
                    />
                  </td>
                  <td className="px-3 py-2">
                    <Score
                      label={`${player.name} expected`}
                      score={player.expectedScore}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Modal>
    </>
  );
}
