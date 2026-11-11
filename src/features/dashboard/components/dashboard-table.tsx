import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { cn } from "@/utils/cn";
import type { DashboardScoreMode } from "../types/dashboard";
import type { SlotLabel } from "../utils/slot-label";
import { Portrait } from "./portrait";

const headerCellClass =
  "h-table-header-height px-2 text-label-md whitespace-nowrap text-on-surface-variant uppercase";
export const rowClass =
  "h-table-row-height-two-line border-t border-outline-variant";
export const numericCellClass =
  "px-2 text-right font-sans text-body-sm font-medium tabular-nums text-on-surface";
const secondaryLineClass =
  "block truncate text-[11px] leading-4 text-on-surface-variant";

export type DashboardTableHeading = {
  label: string;
  /** Right-aligns the heading over a numeric column. */
  numeric?: boolean;
  /** The column's width class; a column without one shares the remaining width. */
  width?: string;
};

/**
 * A dashboard list: one sticky recessed header over fixed two-line rows. The
 * layout is fixed, so the headings set the column widths.
 */
export function DashboardTable({
  caption,
  headings,
  children,
}: {
  caption: string;
  headings: readonly DashboardTableHeading[];
  /** The body rows. */
  children: ReactNode;
}) {
  return (
    <table className="w-full table-fixed border-collapse">
      <caption className="sr-only">{caption}</caption>
      <thead className="sticky top-0 z-10 bg-surface-container-lowest">
        <tr>
          {headings.map(({ label, numeric, width }) => (
            <th
              key={label}
              scope="col"
              className={cn(
                headerCellClass,
                numeric ? "text-right" : "text-left",
                width,
              )}
            >
              {label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>{children}</tbody>
    </table>
  );
}

/** A player's portrait beside the name as a profile link over one line of context. */
export function PlayerIdentity({
  uid,
  name,
  detail,
  view,
  portrait,
}: {
  uid: number;
  name: string;
  detail: string;
  /** The profile's analysis view; absent lets the configured default apply. */
  view?: DashboardScoreMode;
  portrait: ReactNode;
}) {
  return (
    <span className="flex min-w-0 items-center gap-2">
      <Portrait className="size-7 rounded-sm">{portrait}</Portrait>
      <span className="min-w-0 flex-1">
        <Link
          to="/players/$uid"
          params={{ uid: String(uid) }}
          search={view ? { view } : {}}
          className="block truncate text-body-sm text-on-surface underline-offset-2 hover:underline"
          title={name}
        >
          {name}
        </Link>
        <span className={secondaryLineClass} title={detail}>
          {detail}
        </span>
      </span>
    </span>
  );
}

/** A tactic slot: its placement over its role context. */
export function SlotCell({
  label,
  context = label.context,
}: {
  label: SlotLabel;
  context?: string;
}) {
  return (
    <td className="min-w-0 px-2" title={label.full}>
      <span className="block truncate text-body-sm text-on-surface">
        {label.compact}
      </span>
      <span className={secondaryLineClass}>{context}</span>
    </td>
  );
}

/** A 0–100 score in its tier colour, or a neutral dash when there is none. */
export function Score({
  score,
  name,
}: {
  score: number | null;
  /** What the score measures, for its accessible name. */
  name: string;
}) {
  return score === null ? (
    <span
      role="img"
      aria-label={`${name}: unavailable`}
      className="font-mono text-mono-sm text-on-surface-variant tabular-nums"
    >
      —
    </span>
  ) : (
    <ScoreBadge score={score} roleName={name} />
  );
}

/** Points gained over the compared score. The sign carries the direction. */
export function Gain({
  points,
  size = "sm",
}: {
  points: number;
  /** `lg` is a card's headline figure. */
  size?: "sm" | "lg";
}) {
  return (
    <span
      className={
        size === "lg"
          ? "font-mono text-mono-lg text-on-surface tabular-nums"
          : "font-mono text-mono-sm text-on-surface tabular-nums"
      }
    >
      +{points}
    </span>
  );
}
