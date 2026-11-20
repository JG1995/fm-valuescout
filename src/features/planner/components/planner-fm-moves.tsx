import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowUpDown, TriangleAlert } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import {
  applyPlannerFmMoves,
  plannerFmMovesQueryOptions,
} from "../api/planner-fm-moves";
import { plannerKeys } from "../api/planner-keys";
import {
  FM_TEAM_LEVEL_NAMES,
  type PlannerFmMoveResult,
} from "../types/fm-moves";

const headingClass =
  "sticky top-0 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function players(count: number) {
  return count === 1 ? "1 player" : `${count} players`;
}

/**
 * A button that lists the moves that would put each placed player in the FM
 * team his Planner squad stands for, and makes the ticked ones in FM.
 */
export function PlannerFmMoves({
  activeSaveId,
  disabled,
}: {
  activeSaveId: number;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [unticked, setUnticked] = useState<ReadonlySet<number>>(new Set());
  const queryClient = useQueryClient();
  const plan = useQuery({
    ...plannerFmMovesQueryOptions(activeSaveId),
    enabled: open,
  });
  const apply = useMutation({
    mutationFn: applyPlannerFmMoves,
    // FM may have made some moves even when the request failed.
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: plannerKeys.all }),
  });

  const moves = plan.data?.moves ?? [];
  const chosen = moves.filter((move) => !unticked.has(move.playerUid));
  const unavailableReason = plan.data?.unavailableReason ?? null;
  const result = apply.data;
  const allTicked = moves.length > 0 && chosen.length === moves.length;

  const toggle = (playerUid: number) =>
    setUnticked((previous) => {
      const next = new Set(previous);
      if (!next.delete(playerUid)) {
        next.add(playerUid);
      }
      return next;
    });
  const close = () => {
    if (!apply.isPending) {
      setOpen(false);
    }
  };

  return (
    <>
      <Button
        variant="secondary"
        icon={ArrowUpDown}
        disabled={disabled}
        onClick={() => {
          setUnticked(new Set());
          apply.reset();
          setOpen(true);
        }}
      >
        Apply in FM
      </Button>
      <Modal
        open={open}
        onClose={close}
        title="Apply squads in FM"
        className="max-w-[56rem]!"
        footer={
          result ? (
            <Button onClick={close}>Close</Button>
          ) : (
            <>
              <Button
                variant="secondary"
                disabled={apply.isPending}
                onClick={close}
              >
                Cancel
              </Button>
              <Button
                disabled={
                  chosen.length === 0 ||
                  unavailableReason !== null ||
                  !plan.data
                }
                loading={apply.isPending}
                loadingLabel="Moving…"
                onClick={() =>
                  apply.mutate(chosen.map((move) => move.playerUid))
                }
              >
                {`Move ${players(chosen.length)}`}
              </Button>
            </>
          )
        }
      >
        <p className="pb-3 text-body-md text-on-surface-variant">
          Moves each player into the FM team his Planner squad stands for:
          Senior to the first team, Reserves to the reserve team, and Youth to
          the youth team. FM makes each move itself, as its Move to menu does.
          Players outside the Planner stay where they are.
        </p>
        {result ? (
          <MoveResult result={result} />
        ) : plan.error ? (
          <p className="text-body-sm text-error" role="alert">
            Could not plan the moves: {errorMessage(plan.error)}
          </p>
        ) : !plan.data ? (
          <p className="text-body-sm text-on-surface-variant">
            Checking FM squads…
          </p>
        ) : moves.length === 0 ? (
          <p className="text-body-md text-on-surface">
            Every placed player is already in the matching FM team.
          </p>
        ) : (
          <div className="max-h-[24rem] overflow-auto rounded-md border border-outline-variant">
            <table className="w-full border-collapse text-left text-body-sm text-on-surface">
              <caption className="sr-only">Planned moves into FM teams</caption>
              <thead>
                <tr>
                  <th scope="col" className={`${headingClass} w-10`}>
                    <input
                      type="checkbox"
                      aria-label="Move every player"
                      checked={allTicked}
                      disabled={apply.isPending}
                      onChange={() =>
                        setUnticked(
                          allTicked
                            ? new Set(moves.map((move) => move.playerUid))
                            : new Set(),
                        )
                      }
                    />
                  </th>
                  <th scope="col" className={headingClass}>
                    Player
                  </th>
                  <th scope="col" className={headingClass}>
                    Age
                  </th>
                  <th scope="col" className={headingClass}>
                    In FM now
                  </th>
                  <th scope="col" className={headingClass}>
                    Moves to
                  </th>
                </tr>
              </thead>
              <tbody>
                {moves.map((move) => (
                  <tr
                    key={move.playerUid}
                    className="border-t border-outline-variant"
                  >
                    <td className="px-3 py-2">
                      <input
                        type="checkbox"
                        aria-label={`Move ${move.name}`}
                        checked={!unticked.has(move.playerUid)}
                        disabled={apply.isPending}
                        onChange={() => toggle(move.playerUid)}
                      />
                    </td>
                    <th scope="row" className="px-3 py-2 font-normal">
                      {move.name}
                    </th>
                    <td className="px-3 py-2 text-on-surface-variant">
                      {move.age ?? "–"}
                    </td>
                    <td className="px-3 py-2 text-on-surface-variant">
                      {move.fromLevel
                        ? FM_TEAM_LEVEL_NAMES[move.fromLevel]
                        : "Unknown"}
                    </td>
                    <td className="px-3 py-2">
                      <span className="flex flex-wrap items-center gap-2">
                        {move.toTeamName}
                        {move.overAge ? (
                          <StatusChip tone="warning" icon={TriangleAlert}>
                            Over age
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
        {!result && unavailableReason ? (
          <p className="pt-3 text-body-sm text-error" role="alert">
            {unavailableReason}
          </p>
        ) : null}
        {apply.error ? (
          <p className="pt-3 text-body-sm text-error" role="alert">
            The moves failed: {errorMessage(apply.error)}. Load Data to see FM's
            squads.
          </p>
        ) : null}
      </Modal>
    </>
  );
}

function MoveResult({ result }: { result: PlannerFmMoveResult }) {
  return (
    <div className="space-y-2" role="status">
      <p className="text-body-md text-on-surface">
        {result.moved.length === 0
          ? "FM moved nobody."
          : `FM moved ${players(result.moved.length)}.`}
        {result.unchanged.length > 0
          ? ` ${players(result.unchanged.length)} already sat in the right FM team.`
          : ""}
      </p>
      {result.failed.length > 0 ? (
        <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface-variant">
          {result.failed.map((failure) => (
            <li key={failure.playerUid}>
              {failure.name}: {failure.reason}
            </li>
          ))}
        </ul>
      ) : null}
      {result.error ? (
        <p className="text-body-sm text-error">
          {result.error}. Load Data to see FM's squads.
        </p>
      ) : null}
    </div>
  );
}
