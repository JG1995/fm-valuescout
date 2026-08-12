import { useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronUp, UsersRound } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import {
  SQUAD_PAGE_SIZE,
  squadPlayersQueryOptions,
} from "../api/squad-players-query-options";
import type { SquadPlayer } from "../types/squad-player";
import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import { defaultDirForSquadSortField } from "../types/squad-sort";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

const COLUMNS = [
  { key: "name", label: "Name", align: "left" },
  { key: "age", label: "Age / DOB", align: "left" },
  { key: "nationality", label: "Nationality", align: "left" },
  { key: "club", label: "Club", align: "left" },
  { key: "division", label: "Division", align: "left" },
  { key: "ca", label: "CA", align: "right" },
  { key: "pa", label: "PA", align: "right" },
  { key: "value", label: "Value", align: "right" },
] as const satisfies ReadonlyArray<{
  key: SquadSortField;
  label: string;
  align: "left" | "right";
}>;

const SORT_LABELS: Record<SquadSortField, string> = {
  name: "Name",
  age: "Age / DOB",
  nationality: "Nationality",
  club: "Club",
  division: "Division",
  ca: "CA",
  pa: "PA",
  value: "Value",
};

type SquadOverviewPanelProps = {
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  onSortChange: (sortBy: SquadSortField, sortDir: SquadSortDir) => void;
};

function nextSort(
  currentBy: SquadSortField,
  currentDir: SquadSortDir,
  clicked: SquadSortField,
) {
  if (clicked === currentBy) {
    return {
      sortBy: currentBy,
      sortDir: currentDir === "asc" ? ("desc" as const) : ("asc" as const),
    };
  }
  return {
    sortBy: clicked,
    sortDir: defaultDirForSquadSortField(clicked),
  };
}

function basicCell(
  player: SquadPlayer,
  key: SquadSortField,
): { text: string; title?: string; numeric: boolean } {
  switch (key) {
    case "name":
      return { text: player.name, title: player.name, numeric: false };
    case "age": {
      const dob = formatPlayerDob(
        player.birthYear,
        player.birthDayOfYear,
        player.age,
      );
      return { text: dob, title: dob, numeric: false };
    }
    case "nationality": {
      const nationalities = String(
        formatMissable(player.nationalities.join(", ")),
      );
      return { text: nationalities, title: nationalities, numeric: false };
    }
    case "club": {
      const club = String(formatMissable(player.club));
      return {
        text: club,
        title: club !== "—" ? club : undefined,
        numeric: false,
      };
    }
    case "division": {
      const division = String(formatMissable(player.division));
      return {
        text: division,
        title: division !== "—" ? division : undefined,
        numeric: false,
      };
    }
    case "ca":
      return { text: String(player.ca), numeric: true };
    case "pa":
      return { text: String(player.pa), numeric: true };
    case "value":
      return {
        text:
          player.marketValueGbp === null
            ? "—"
            : formatMoney(player.marketValueGbp),
        numeric: true,
      };
  }
}

function SquadOverviewTable({
  players,
  sortBy,
  sortDir,
  onSortChange,
}: {
  players: SquadPlayer[];
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  onSortChange: SquadOverviewPanelProps["onSortChange"];
}) {
  const tableRef = useRef<HTMLTableElement>(null);
  const [keyboardFocusIndex, setKeyboardFocusIndex] = useState(0);
  const clampedFocusIndex = Math.min(
    keyboardFocusIndex,
    Math.max(0, players.length - 1),
  );

  const focusPlayer = (index: number) => {
    if (index < 0 || index >= players.length) {
      return;
    }
    setKeyboardFocusIndex(index);
    requestAnimationFrame(() => {
      tableRef.current
        ?.querySelector<HTMLAnchorElement>(
          `[data-squad-player-index="${index}"]`,
        )
        ?.focus();
    });
  };

  return (
    <div className="max-h-[min(70vh,720px)] overflow-auto">
      <table ref={tableRef} className="w-full border-collapse text-left">
        <caption className="sr-only">Squad overview</caption>
        <thead className="sticky top-0 z-10">
          <tr className="bg-surface-container-lowest">
            {COLUMNS.map((column) => {
              const active = column.key === sortBy;
              const ariaSort = active
                ? sortDir === "asc"
                  ? "ascending"
                  : "descending"
                : "none";
              const Caret = sortDir === "asc" ? ChevronUp : ChevronDown;
              return (
                <th
                  key={column.key}
                  scope="col"
                  aria-sort={ariaSort}
                  className={
                    column.align === "right"
                      ? "h-table-header-height px-2 text-right"
                      : "h-table-header-height px-2 text-left"
                  }
                >
                  <button
                    type="button"
                    className={
                      active
                        ? "inline-flex items-center gap-1 text-label-md text-primary uppercase"
                        : "inline-flex items-center gap-1 text-label-md text-on-surface-variant uppercase"
                    }
                    onClick={() => {
                      const next = nextSort(sortBy, sortDir, column.key);
                      onSortChange(next.sortBy, next.sortDir);
                    }}
                  >
                    {column.label}
                    {active ? (
                      <Caret
                        aria-hidden
                        className="size-3.5 shrink-0"
                        strokeWidth={2}
                      />
                    ) : null}
                  </button>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {players.map((player, index) => (
            <tr
              key={player.uid}
              className="border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-within:bg-surface-container-high"
            >
              {COLUMNS.map((column) => {
                const cell = basicCell(player, column.key);
                if (column.key === "name") {
                  return (
                    <td
                      key={column.key}
                      className={`${TEXT_CELL} text-on-surface`}
                      title={cell.title}
                    >
                      <Link
                        to="/players/$uid"
                        params={{ uid: String(player.uid) }}
                        search={{ tab: "technical" }}
                        data-squad-player-index={index}
                        tabIndex={index === clampedFocusIndex ? undefined : -1}
                        className="block scroll-mt-8 truncate text-on-surface underline decoration-outline-variant underline-offset-2 transition-colors duration-150 ease-out hover:text-primary focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
                        title={player.name}
                        onFocus={() => {
                          setKeyboardFocusIndex(index);
                        }}
                        onKeyDown={(event) => {
                          if (event.key === "ArrowDown") {
                            event.preventDefault();
                            focusPlayer(index + 1);
                          } else if (event.key === "ArrowUp") {
                            event.preventDefault();
                            focusPlayer(index - 1);
                          }
                        }}
                      >
                        {player.name}
                      </Link>
                    </td>
                  );
                }
                return (
                  <td
                    key={column.key}
                    className={
                      cell.numeric
                        ? NUM_CELL
                        : `${TEXT_CELL} ${column.key === "age" || column.key === "division" ? "text-on-surface-variant" : "text-on-surface"}`
                    }
                    title={cell.title}
                  >
                    {cell.text}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function SquadOverviewPanel({
  sortBy,
  sortDir,
  onSortChange,
}: SquadOverviewPanelProps) {
  const [offset, setOffset] = useState(0);
  const { data: page } = useSuspenseQuery(
    squadPlayersQueryOptions(offset, SQUAD_PAGE_SIZE, sortBy, sortDir),
  );
  const pageCount = Math.ceil(page.total / SQUAD_PAGE_SIZE);
  const maxOffset = Math.max(0, (pageCount - 1) * SQUAD_PAGE_SIZE);
  const needsPageClamp = offset > maxOffset;

  useEffect(() => {
    if (needsPageClamp) {
      setOffset(maxOffset);
    }
  }, [maxOffset, needsPageClamp]);

  if (needsPageClamp) {
    return (
      <Panel title="Squad overview" flush>
        <div
          aria-busy="true"
          aria-live="polite"
          className="flex min-h-40 items-center justify-center text-body-md text-on-surface-variant"
        >
          Refreshing squad overview…
        </div>
      </Panel>
    );
  }

  if (page.total === 0) {
    return (
      <Panel title="Squad overview" flush>
        <EmptyState icon={UsersRound} title="No players in your club family">
          No current-snapshot players match the clubs configured for this save.
        </EmptyState>
      </Panel>
    );
  }

  const pageNumber = Math.floor(offset / SQUAD_PAGE_SIZE) + 1;
  const dirLabel = sortDir === "asc" ? "ascending" : "descending";

  return (
    <Panel title="Squad overview" flush>
      <p className="px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
        {page.total === 1 ? "player" : "players"} · sorted by{" "}
        {SORT_LABELS[sortBy]} ({dirLabel})
      </p>
      <SquadOverviewTable
        players={page.players}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={onSortChange}
      />
      {pageCount > 1 ? (
        <nav
          aria-label="Squad overview pages"
          className="flex items-center justify-between gap-3 border-t border-outline-variant px-4 py-3"
        >
          <button
            type="button"
            className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high disabled:cursor-not-allowed disabled:opacity-50"
            disabled={offset === 0}
            onClick={() => {
              setOffset((current) => Math.max(0, current - SQUAD_PAGE_SIZE));
            }}
          >
            Previous page
          </button>
          <p className="text-body-sm text-on-surface-variant">
            Page {pageNumber} of {pageCount}
          </p>
          <button
            type="button"
            className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high disabled:cursor-not-allowed disabled:opacity-50"
            disabled={pageNumber === pageCount}
            onClick={() => {
              setOffset((current) => current + SQUAD_PAGE_SIZE);
            }}
          >
            Next page
          </button>
        </nav>
      ) : null}
    </Panel>
  );
}
