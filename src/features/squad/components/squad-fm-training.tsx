import { useMutation, useQuery } from "@tanstack/react-query";
import { Dumbbell } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import {
  applySquadFmTraining,
  squadFmTrainingQueryOptions,
} from "../api/squad-fm-training";
import type {
  SquadFmTrainingPlayer,
  SquadFmTrainingResult,
} from "../types/squad-fm-training";

const headingClass =
  "sticky top-0 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function players(count: number) {
  return count === 1 ? "1 player" : `${count} players`;
}

function Unchanged() {
  return (
    <span className="text-on-surface-variant">
      <span aria-hidden="true">—</span>
      <span className="sr-only">Unchanged</span>
    </span>
  );
}

/**
 * A button that lists the training each player at the managed club would get
 * in FM: his Planner slot's roles and his Suggested Training, and sets the
 * ticked players' training in FM.
 */
export function SquadFmTraining({
  activeSaveId,
  disabled,
}: {
  activeSaveId: number;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [unticked, setUnticked] = useState<ReadonlySet<number>>(new Set());
  const plan = useQuery({
    ...squadFmTrainingQueryOptions(activeSaveId),
    enabled: open,
  });
  // Nothing in the app changes: snapshots do not hold training.
  const apply = useMutation({ mutationFn: applySquadFmTraining });

  const planned = plan.data?.players ?? [];
  const chosen = planned.filter((player) => !unticked.has(player.playerUid));
  const unavailableReason = plan.data?.unavailableReason ?? null;
  const result = apply.data;
  const allTicked = planned.length > 0 && chosen.length === planned.length;

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
        icon={Dumbbell}
        disabled={disabled}
        onClick={() => {
          setUnticked(new Set());
          apply.reset();
          setOpen(true);
        }}
      >
        Apply training in FM
      </Button>
      <Modal
        open={open}
        onClose={close}
        title="Apply training in FM"
        className="max-w-[64rem]!"
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
                loadingLabel="Setting training…"
                onClick={() =>
                  apply.mutate(chosen.map((player) => player.playerUid))
                }
              >
                {`Set training for ${players(chosen.length)}`}
              </Button>
            </>
          )
        }
      >
        <p className="pb-3 text-body-md text-on-surface-variant">
          Sets each player's individual training in FM. A player in a Planner
          slot trains the slot's in-possession role at its position and its
          out-of-possession role. Every player's additional focus becomes his
          Suggested Training, and a player without one has his focus cleared.
          Players out on loan train with their loan club and are left out.
        </p>
        {result ? (
          <TrainingResult result={result} />
        ) : plan.error ? (
          <p className="text-body-sm text-error" role="alert">
            Could not plan the training: {errorMessage(plan.error)}
          </p>
        ) : !plan.data ? (
          <p className="text-body-sm text-on-surface-variant">
            Planning training…
          </p>
        ) : planned.length === 0 ? (
          <p className="text-body-md text-on-surface">
            The managed club has no players to train.
          </p>
        ) : (
          <div className="max-h-[24rem] overflow-auto rounded-md border border-outline-variant">
            <table className="w-full border-collapse text-left text-body-sm text-on-surface">
              <caption className="sr-only">Planned training in FM</caption>
              <thead>
                <tr>
                  <th scope="col" className={`${headingClass} w-10`}>
                    <input
                      type="checkbox"
                      aria-label="Set training for every player"
                      checked={allTicked}
                      disabled={apply.isPending}
                      onChange={() =>
                        setUnticked(
                          allTicked
                            ? new Set(planned.map((player) => player.playerUid))
                            : new Set(),
                        )
                      }
                    />
                  </th>
                  <th scope="col" className={headingClass}>
                    Player
                  </th>
                  <th scope="col" className={headingClass}>
                    In possession
                  </th>
                  <th scope="col" className={headingClass}>
                    Out of possession
                  </th>
                  <th scope="col" className={headingClass}>
                    Focus
                  </th>
                </tr>
              </thead>
              <tbody>
                {planned.map((player) => (
                  <TrainingRow
                    key={player.playerUid}
                    player={player}
                    ticked={!unticked.has(player.playerUid)}
                    disabled={apply.isPending}
                    onToggle={() => toggle(player.playerUid)}
                  />
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
            Setting training failed: {errorMessage(apply.error)}. Check the
            training in FM.
          </p>
        ) : null}
      </Modal>
    </>
  );
}

function TrainingRow({
  player,
  ticked,
  disabled,
  onToggle,
}: {
  player: SquadFmTrainingPlayer;
  ticked: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  return (
    <tr className="border-t border-outline-variant">
      <td className="px-3 py-2">
        <input
          type="checkbox"
          aria-label={`Set training for ${player.name}`}
          checked={ticked}
          disabled={disabled}
          onChange={onToggle}
        />
      </td>
      <th scope="row" className="px-3 py-2 font-normal">
        {player.name}
      </th>
      <td className="px-3 py-2">
        {player.inPossessionRole ? (
          <>
            {player.inPossessionRole}{" "}
            <span className="text-on-surface-variant">{player.position}</span>
          </>
        ) : (
          <Unchanged />
        )}
      </td>
      <td className="px-3 py-2">
        {player.outOfPossessionRole ?? <Unchanged />}
      </td>
      <td className="px-3 py-2">
        {player.focus ?? (
          <span className="text-on-surface-variant">Cleared</span>
        )}
      </td>
    </tr>
  );
}

function TrainingResult({ result }: { result: SquadFmTrainingResult }) {
  return (
    <div className="space-y-2" role="status">
      <p className="text-body-md text-on-surface">
        {result.applied.length === 0
          ? "FM took no player's training."
          : `FM took the training for ${players(result.applied.length)}.`}
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
          {result.error}. Check the training in FM.
        </p>
      ) : null}
    </div>
  );
}
