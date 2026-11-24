import { useMutation, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Dumbbell, MapPinOff, TriangleAlert } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import { TableSkeleton } from "@/components/ui/skeleton/skeleton";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { applyFmTraining, fmTrainingQueryOptions } from "../api/fm-training";
import type { FmTrainingPlayer, FmTrainingResult } from "../types/fm-training";

const headingClass =
  "sticky top-0 z-10 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

const NO_SQUAD = "Not in a squad";

type TrainingFilter = {
  id: string;
  label: string;
  matches: (player: FmTrainingPlayer) => boolean;
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function players(count: number) {
  return count === 1 ? "1 player" : `${count} players`;
}

function groupLabel(player: FmTrainingPlayer) {
  return player.slot
    ? `${player.slot.teamName} · ${player.slot.stringName}`
    : NO_SQUAD;
}

/** Every player, each Planner squad, the players in no squad, and the players learning a position. */
function trainingFilters(planned: readonly FmTrainingPlayer[]) {
  const filters: TrainingFilter[] = [
    { id: "all", label: "All", matches: () => true },
  ];
  for (const { slot } of planned) {
    if (slot && !filters.some((filter) => filter.id === slot.team)) {
      filters.push({
        id: slot.team,
        label: slot.teamName,
        matches: (player) => player.slot?.team === slot.team,
      });
    }
  }
  filters.push(
    {
      id: "no-squad",
      label: "No squad",
      matches: (player) => player.slot === null,
    },
    {
      id: "learning",
      label: "Learning a position",
      matches: (player) => player.learning !== null,
    },
  );
  return filters
    .map((filter) => ({
      ...filter,
      count: planned.filter(filter.matches).length,
    }))
    .filter((filter) => filter.id === "all" || filter.count > 0);
}

/** Consecutive players of one squad string, as the plan orders them. */
function groupPlayers(listed: readonly FmTrainingPlayer[]) {
  const groups: { label: string; players: FmTrainingPlayer[] }[] = [];
  for (const player of listed) {
    const label = groupLabel(player);
    const last = groups.at(-1);
    if (last?.label === label) {
      last.players.push(player);
    } else {
      groups.push({ label, players: [player] });
    }
  }
  return groups;
}

/**
 * The Training workspace: the individual training each player at the managed
 * club would get in FM, his Planner slot's roles and his Suggested Training,
 * and the action that sets the ticked players' training in FM.
 */
export function TrainingWorkspace({
  activeSaveId,
  disabled,
}: {
  activeSaveId: number;
  /** Another change to the squad is running. */
  disabled: boolean;
}) {
  const [filterId, setFilterId] = useState("all");
  const [unticked, setUnticked] = useState<ReadonlySet<number>>(new Set());
  const [confirming, setConfirming] = useState(false);
  const unavailableId = useId();
  const selectAllRef = useRef<HTMLInputElement>(null);
  const plan = useQuery(fmTrainingQueryOptions(activeSaveId));
  // Nothing in the app changes: snapshots do not hold training.
  const apply = useMutation({
    mutationFn: applyFmTraining,
    onSuccess: () => setConfirming(false),
  });

  const planned = plan.data?.players ?? [];
  const filters = trainingFilters(planned);
  const filter = filters.find((item) => item.id === filterId) ?? filters[0];
  const listed = planned.filter(filter.matches);
  const chosen = listed.filter((player) => !unticked.has(player.playerUid));
  const unavailableReason = plan.data?.unavailableReason ?? null;
  const allTicked = listed.length > 0 && chosen.length === listed.length;
  const someTicked = chosen.length > 0 && !allTicked;
  const learningCount = planned.filter(
    (player) => player.learning !== null,
  ).length;

  useEffect(() => {
    if (selectAllRef.current) {
      selectAllRef.current.indeterminate = someTicked;
    }
  }, [someTicked]);

  const toggle = (playerUid: number) =>
    setUnticked((previous) => {
      const next = new Set(previous);
      if (!next.delete(playerUid)) {
        next.add(playerUid);
      }
      return next;
    });
  const toggleListed = () =>
    setUnticked((previous) => {
      const next = new Set(previous);
      for (const player of listed) {
        if (allTicked) {
          next.add(player.playerUid);
        } else {
          next.delete(player.playerUid);
        }
      }
      return next;
    });
  const close = () => {
    if (!apply.isPending) {
      setConfirming(false);
    }
  };

  return (
    <Panel
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2 px-4 pt-4">
        <p className="max-w-[100ch] min-w-0 flex-1 basis-80 text-body-sm text-on-surface-variant">
          The individual training each player would get in FM: his Planner
          slot's roles, and his Suggested Training as the focus. A player who is
          not fully familiar with a slot position trains there first.
        </p>
        <Button
          icon={Dumbbell}
          disabled={
            disabled ||
            chosen.length === 0 ||
            unavailableReason !== null ||
            // A revisit paints the last plan while the current one loads.
            plan.isFetching ||
            !plan.data
          }
          aria-describedby={unavailableReason ? unavailableId : undefined}
          onClick={() => {
            apply.reset();
            setConfirming(true);
          }}
        >
          {plan.data
            ? `Apply training in FM (${chosen.length})`
            : "Apply training in FM"}
        </Button>
      </div>
      <div className="mt-1 min-h-6 shrink-0 px-4 pb-2">
        {unavailableReason ? (
          <p
            id={unavailableId}
            className="flex items-start gap-2 text-body-sm text-on-surface"
          >
            <TriangleAlert
              aria-hidden="true"
              className="mt-0.5 size-4 shrink-0 text-warning"
            />
            {unavailableReason}
          </p>
        ) : null}
        {apply.data ? <TrainingResult result={apply.data} /> : null}
      </div>
      {plan.error ? (
        <div
          className="flex flex-wrap items-center justify-between gap-3 border-t border-outline-variant p-4"
          role="alert"
        >
          <p className="text-body-sm text-error">
            Could not plan the training: {errorMessage(plan.error)}
          </p>
          <Button variant="secondary" onClick={() => void plan.refetch()}>
            Retry
          </Button>
        </div>
      ) : !plan.data ? (
        <TableSkeleton label="Planning training…" />
      ) : planned.length === 0 ? (
        <p className="border-t border-outline-variant p-4 text-body-md text-on-surface">
          The managed club has no players to train.
        </p>
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-2 border-y border-outline-variant px-4 py-2">
            <p className="text-body-sm text-on-surface-variant">
              {players(planned.length)}
              {learningCount > 0
                ? ` · ${learningCount} learning a position`
                : null}
              {planned.every((player) => player.slot === null)
                ? " · no one is in a Planner slot, so only focuses are set"
                : null}
            </p>
            <fieldset className="inline-flex flex-wrap rounded-md border border-outline bg-surface-container-high p-0.5">
              <legend className="sr-only">Show players</legend>
              {filters.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  aria-pressed={item.id === filter.id}
                  className={
                    item.id === filter.id
                      ? "h-7 rounded-sm bg-surface-container-lowest px-3 text-label-md text-on-surface"
                      : "h-7 rounded-sm px-3 text-label-md font-normal text-on-surface-variant hover:text-on-surface"
                  }
                  onClick={() => setFilterId(item.id)}
                >
                  {item.label}{" "}
                  <span className="font-normal tabular-nums">{item.count}</span>
                </button>
              ))}
            </fieldset>
          </div>
          <div className="min-h-0 flex-1 overflow-auto">
            <table className="w-full border-collapse text-left text-body-sm text-on-surface">
              <caption className="sr-only">Planned training in FM</caption>
              <thead>
                <tr>
                  <th scope="col" className={`${headingClass} w-10`}>
                    <input
                      ref={selectAllRef}
                      type="checkbox"
                      aria-label="Set training for every listed player"
                      checked={allTicked}
                      disabled={apply.isPending}
                      onChange={toggleListed}
                    />
                  </th>
                  <th scope="col" className={headingClass}>
                    Player
                  </th>
                  <th scope="col" className={headingClass}>
                    Slot
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
              {groupPlayers(listed).map((group) => (
                <tbody key={group.label}>
                  <tr className="border-t border-outline-variant">
                    <th
                      scope="rowgroup"
                      colSpan={6}
                      className="bg-surface-container-high px-3 py-1.5 text-label-md uppercase text-on-surface-variant"
                    >
                      {group.label}
                    </th>
                  </tr>
                  {group.players.map((player) => (
                    <TrainingRow
                      key={player.playerUid}
                      player={player}
                      ticked={!unticked.has(player.playerUid)}
                      disabled={apply.isPending}
                      onToggle={() => toggle(player.playerUid)}
                    />
                  ))}
                </tbody>
              ))}
            </table>
          </div>
        </>
      )}
      <Modal
        open={confirming}
        onClose={close}
        title="Apply training in FM"
        footer={
          <>
            <Button
              variant="secondary"
              disabled={apply.isPending}
              onClick={close}
            >
              Cancel
            </Button>
            <Button
              loading={apply.isPending}
              loadingLabel="Setting training…"
              onClick={() =>
                apply.mutate(chosen.map((player) => player.playerUid))
              }
            >
              {`Set training for ${players(chosen.length)}`}
            </Button>
          </>
        }
      >
        <p className="text-body-md text-on-surface-variant">
          Sets the individual training of the {players(chosen.length)} ticked in
          the list. A player in a Planner slot gets the roles listed, and one in
          no slot keeps his roles. Every player's additional focus becomes the
          one listed, and a player without one has his focus cleared. Players
          out on loan train with their loan club and are left out.
        </p>
        {apply.error ? (
          <p className="pt-3 text-body-sm text-error" role="alert">
            Setting training failed: {errorMessage(apply.error)}. Check the
            training in FM.
          </p>
        ) : null}
      </Modal>
    </Panel>
  );
}

function Muted({ children }: { children: string }) {
  return <span className="text-on-surface-variant">{children}</span>;
}

function TrainingRow({
  player,
  ticked,
  disabled,
  onToggle,
}: {
  player: FmTrainingPlayer;
  ticked: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  const { slot, learning } = player;
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
        <Link
          to="/players/$uid"
          params={{ uid: String(player.playerUid) }}
          search={{}}
          className="underline-offset-2 hover:underline"
        >
          {player.name}
        </Link>
        {player.age === null ? null : (
          <span className="text-on-surface-variant"> · {player.age}</span>
        )}
      </th>
      <td className="px-3 py-2 font-mono text-mono-sm text-on-surface-variant">
        {slot ? (
          slot.ipPosition === slot.oopPosition ? (
            slot.ipPosition
          ) : (
            `${slot.ipPosition}/${slot.oopPosition}`
          )
        ) : (
          <span className="font-sans text-body-sm">No slot</span>
        )}
      </td>
      <td className="px-3 py-2">
        {player.inPossessionRole ? (
          <>
            <span className="flex flex-wrap items-center gap-x-2 gap-y-1">
              <span>
                {player.inPossessionRole} <Muted>{player.position ?? ""}</Muted>
              </span>
              {learning ? (
                <StatusChip tone="warning" icon={MapPinOff}>
                  {`Learning ${learning.position}`}
                </StatusChip>
              ) : null}
            </span>
            {learning ? (
              <span className="block pt-0.5 text-[11px] text-on-surface-variant">
                {learning.familiarity > 0
                  ? `Familiarity ${learning.familiarity} of 20`
                  : "No familiarity yet"}
                {slot && learning.outOfPossession
                  ? ` · then ${slot.inPossessionRole} at ${slot.ipPosition}`
                  : null}
              </span>
            ) : null}
          </>
        ) : (
          <Muted>Unchanged</Muted>
        )}
      </td>
      <td className="px-3 py-2">
        {player.outOfPossessionRole ?? <Muted>Unchanged</Muted>}
      </td>
      <td className="px-3 py-2">{player.focus ?? <Muted>Cleared</Muted>}</td>
    </tr>
  );
}

function TrainingResult({ result }: { result: FmTrainingResult }) {
  return (
    <div className="space-y-1" role="status">
      <p className="text-body-sm text-on-surface">
        {result.applied.length === 0
          ? "FM took no player's training."
          : `FM took the training for ${players(result.applied.length)}.`}
        {result.failed.length > 0
          ? ` It did not take ${players(result.failed.length)}:`
          : null}
      </p>
      {result.failed.length > 0 ? (
        <ul
          // biome-ignore lint/a11y/noNoninteractiveTabindex: keyboard users scroll the clipped list
          tabIndex={0}
          aria-label="Training FM did not take"
          className="max-h-24 list-disc space-y-1 overflow-y-auto pl-5 text-body-sm text-on-surface-variant"
        >
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
