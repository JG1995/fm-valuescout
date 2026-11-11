import { Link } from "@tanstack/react-router";
import { Modal } from "@/components/ui/modal/modal";
import {
  portraitCoordinateForPlacement,
  projectPosition,
} from "@/utils/tactic-pitch-geometry";
import type {
  DashboardGraphics,
  DashboardYouthXiSlot,
} from "../types/dashboard";
import { orderedSlots, slotLabel } from "../utils/slot-label";
import { Score } from "./dashboard-table";
import { Portrait } from "./portrait";

function YouthXiMarker({
  slot,
  graphics,
}: {
  slot: DashboardYouthXiSlot;
  graphics: DashboardGraphics;
}) {
  const label = slotLabel(slot);
  const player = slot.player;
  return (
    <div
      className="flex flex-col items-center rounded-md bg-surface-container-high px-2 py-1 text-center"
      title={label.full}
    >
      <Portrait className="size-8 rounded-sm bg-surface-container-highest">
        {player ? graphics.portrait(player.uid, player.name) : null}
      </Portrait>
      {player ? (
        <Link
          to="/players/$uid"
          params={{ uid: String(player.uid) }}
          search={{}}
          className="block max-w-full truncate text-body-sm text-on-surface underline-offset-2 hover:underline"
          title={player.name}
        >
          {player.name}
        </Link>
      ) : (
        <span className="text-body-sm text-on-surface-variant">No player</span>
      )}
      <span className="flex items-center gap-1 text-[11px] leading-4 text-on-surface-variant">
        {label.compact}
        {player ? (
          <>
            <Score
              score={player.currentScore}
              name={`${player.name} now at ${label.compact}`}
            />
            <span aria-hidden="true">→</span>
            <Score
              score={player.potentialScore}
              name={`${player.name} potential at ${label.compact}`}
            />
          </>
        ) : null}
      </span>
    </div>
  );
}

/**
 * The club's young players in the tactic's shape, each slot taking the player
 * the potential scores allocate to it. The team attacks toward the right.
 */
export function YouthXiModal({
  open,
  onClose,
  slots,
  maxAge,
  graphics,
}: {
  open: boolean;
  onClose: () => void;
  slots: readonly DashboardYouthXiSlot[];
  maxAge: number;
  graphics: DashboardGraphics;
}) {
  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Youth XI"
      variant="informational"
      className="max-w-[72rem]!"
    >
      <p className="pb-3 text-body-md text-on-surface-variant">
        Your players aged {maxAge} or younger in your tactic, placed by
        potential score. Each shows the score now and the potential.
      </p>
      <div className="relative h-[34rem] overflow-hidden rounded-lg bg-surface-container">
        {/* The pitch, attacking toward the right. Each line keeps its 1px
            width however the drawing stretches. */}
        <svg
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 h-full w-full text-outline-variant"
          fill="none"
          preserveAspectRatio="none"
          stroke="currentColor"
          viewBox="0 0 100 100"
        >
          <rect
            height="96"
            width="98"
            x="1"
            y="2"
            vectorEffect="non-scaling-stroke"
          />
          <line
            x1="50"
            x2="50"
            y1="2"
            y2="98"
            vectorEffect="non-scaling-stroke"
          />
          <rect
            height="30"
            width="12"
            x="1"
            y="35"
            vectorEffect="non-scaling-stroke"
          />
          <rect
            height="30"
            width="12"
            x="87"
            y="35"
            vectorEffect="non-scaling-stroke"
          />
        </svg>
        <ul aria-label="Youth XI by tactic slot">
          {orderedSlots(slots).map((slot) => {
            const placement = portraitCoordinateForPlacement(slot.ipPosition);
            if (!placement) return null;
            const { x, y } = projectPosition(placement, "landscape");
            return (
              <li
                key={slot.laneId}
                className="absolute w-34 -translate-x-1/2 -translate-y-1/2"
                style={{ left: `${x * 100}%`, top: `${y * 100}%` }}
              >
                <YouthXiMarker slot={slot} graphics={graphics} />
              </li>
            );
          })}
        </ul>
      </div>
    </Modal>
  );
}
