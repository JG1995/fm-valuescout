import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import type { DashboardScoreMode } from "../types/dashboard";
import type { CardSize } from "../utils/card-capacity";
import { Portrait } from "./portrait";

/**
 * The smallest a player card may be, in pixels. The height holds a two-line
 * name over two detail lines, the figure, and the slot comparison.
 */
export const PLAYER_CARD_MIN: CardSize = { width: 280, height: 212 };

/** The least height a grid of player cards accepts: `PLAYER_CARD_MIN.height`. */
export const cardGridMinHeightClass = "min-h-53";

/**
 * One player in a card grid: the portrait beside the identity and the figure
 * the list is ranked by, over the slot comparison along the bottom. It fills
 * its grid cell, and the portrait grows with the cell.
 */
export function PlayerCard({
  uid,
  name,
  view,
  portrait,
  details,
  metricLabel,
  metric,
  children,
}: {
  uid: number;
  name: string;
  /** The profile's analysis view; absent lets the configured default apply. */
  view?: DashboardScoreMode;
  portrait: ReactNode;
  /** Lines of context under the name. */
  details: ReactNode;
  metricLabel: string;
  metric: ReactNode;
  /** The slot the card is about and how the player compares there. */
  children: ReactNode;
}) {
  return (
    <div className="flex min-w-0 flex-1 flex-col justify-between gap-2 p-4">
      <div className="flex min-w-0 gap-3">
        <Portrait className="size-[clamp(3.5rem,min(46cqh,38cqw),9.5rem)] rounded-md">
          {portrait}
        </Portrait>
        <div className="flex min-w-0 flex-1 flex-col justify-between gap-1">
          <div className="min-w-0">
            {/* Above the panel's expand hit area, so the name opens the profile. */}
            <Link
              to="/players/$uid"
              params={{ uid: String(uid) }}
              search={view ? { view } : {}}
              className="relative z-10 line-clamp-2 text-body-lg leading-6 font-semibold break-words text-on-surface underline-offset-2 hover:underline"
              title={name}
            >
              {name}
            </Link>
            {details}
          </div>
          <p className="flex items-baseline gap-2">
            <span className="text-label-sm text-on-surface-variant uppercase">
              {metricLabel}
            </span>
            {metric}
          </p>
        </div>
      </div>
      <div className="min-w-0 space-y-1 border-t border-outline-variant pt-2">
        {children}
      </div>
    </div>
  );
}

export const cardDetailClass =
  "flex min-w-0 items-center gap-1 text-body-sm text-on-surface-variant";

/** A card's slot line: the placement, its role context, and an optional aside. */
export function CardSlotLine({
  placement,
  context,
  title,
  aside,
}: {
  placement: string;
  context: string;
  /** The slot's whole definition. */
  title: string;
  aside?: string;
}) {
  return (
    <p className="flex min-w-0 items-baseline gap-2" title={title}>
      <span className="shrink-0 text-body-sm font-semibold text-on-surface">
        {placement}
      </span>
      <span className="min-w-0 flex-1 truncate text-[11px] leading-4 text-on-surface-variant">
        {context}
      </span>
      {aside ? (
        <span className="shrink-0 text-[11px] leading-4 text-on-surface-variant">
          {aside}
        </span>
      ) : null}
    </p>
  );
}
