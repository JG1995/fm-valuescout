import { useSuspenseQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { ChevronDown, ChevronUp, UsersRound } from "lucide-react";
import type { ReactNode } from "react";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import { getPlayerMetric } from "@/utils/player-metrics";
import {
  SQUAD_PAGE_SIZE,
  squadPlayersQueryOptions,
} from "../api/squad-players-query-options";
import type { SquadPlayer } from "../types/squad-player";
import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  defaultDirForSquadSortField,
  type SQUAD_SORT_FIELDS,
} from "../types/squad-sort";

type BasicSquadSortField = (typeof SQUAD_SORT_FIELDS)[number];

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
  key: BasicSquadSortField;
  label: string;
  align: "left" | "right";
}>;

const SORT_LABELS = {
  name: "Name",
  age: "Age / DOB",
  nationality: "Nationality",
  club: "Club",
  division: "Division",
  ca: "CA",
  pa: "PA",
  value: "Value",
} as const satisfies Record<BasicSquadSortField, string>;

type SquadOverviewPanelProps = {
  actions?: ReactNode;
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
  player: SquadPlayer | undefined,
  key: BasicSquadSortField,
): { text: string; title?: string; numeric: boolean } {
  if (!player) {
    return { text: "…", numeric: key !== "name" && key !== "age" };
  }
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
  total,
  sortBy,
  sortDir,
  onSortChange,
}: {
  total: number;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  onSortChange: SquadOverviewPanelProps["onSortChange"];
}) {
  const navigate = useNavigate();

  return (
    <VirtualizedPlayerTable
      caption="Squad overview"
      columnCount={COLUMNS.length}
      header={
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
      }
      pageQueryOptions={(offset, limit) =>
        squadPlayersQueryOptions(offset, limit, sortBy, sortDir)
      }
      pageSize={SQUAD_PAGE_SIZE}
      renderCells={(player) =>
        COLUMNS.map((column) => {
          const cell = basicCell(player, column.key);
          if (column.key === "name" && player) {
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
                  tabIndex={-1}
                  className="block truncate text-on-surface underline decoration-outline-variant underline-offset-2 transition-colors duration-150 ease-out hover:text-primary"
                  title={player.name}
                  onClick={(event) => {
                    event.stopPropagation();
                  }}
                >
                  {cell.text}
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
        })
      }
      testId="squad-overview-scroller"
      total={total}
      onPlayerActivate={(player) => {
        void navigate({
          to: "/players/$uid",
          params: { uid: String(player.uid) },
          search: { tab: "technical" },
        });
      }}
    />
  );
}

export function SquadOverviewPanel({
  actions,
  sortBy,
  sortDir,
  onSortChange,
}: SquadOverviewPanelProps) {
  const { data: page } = useSuspenseQuery(
    squadPlayersQueryOptions(0, SQUAD_PAGE_SIZE, sortBy, sortDir),
  );

  if (page.total === 0) {
    return (
      <Panel title="Squad overview" actions={actions} flush>
        <EmptyState icon={UsersRound} title="No players in your club family">
          No current-snapshot players match the clubs configured for this save.
        </EmptyState>
      </Panel>
    );
  }

  const dirLabel = sortDir === "asc" ? "ascending" : "descending";
  const sortLabel =
    (SORT_LABELS as Record<string, string>)[sortBy] ??
    getPlayerMetric(sortBy)?.label ??
    sortBy;

  return (
    <Panel
      title="Squad overview"
      actions={actions}
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <p className="shrink-0 px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
        {page.total === 1 ? "player" : "players"} · sorted by {sortLabel} (
        {dirLabel})
      </p>
      <SquadOverviewTable
        total={page.total}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={onSortChange}
      />
    </Panel>
  );
}
