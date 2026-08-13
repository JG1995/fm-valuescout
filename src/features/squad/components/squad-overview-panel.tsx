import { useSuspenseQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { UsersRound } from "lucide-react";
import { type ReactNode, useMemo } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type PlayerTableColumn,
  PlayerTableHeader,
} from "@/components/player-table/player-table-header";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
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
  SQUAD_SORT_FIELDS,
} from "../types/squad-sort";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

type BasicSquadSortField = (typeof SQUAD_SORT_FIELDS)[number];
type TableColumn = PlayerTableColumn;

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

function tableColumnForMetric(
  metricId: string,
  width: number | undefined,
): TableColumn | undefined {
  const metric = getPlayerMetric(metricId);
  if (!metric) {
    return undefined;
  }
  return {
    id: metric.id,
    label: metric.id === "age" ? "Age / DOB" : metric.label,
    align: metric.align,
    width: width ?? metric.defaultWidth,
  };
}

function formatDynamicCell(
  player: SquadPlayer | undefined,
  fieldId: string,
): string {
  if (!player) {
    return "…";
  }
  const value = player.dynamicValues?.[fieldId];
  if (value === undefined || value === null) {
    return "—";
  }
  return String(value);
}

function SquadOverviewTable({
  total,
  sortBy,
  sortDir,
  columns,
  requestedFields,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
}: {
  total: number;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  columns: TableColumn[];
  requestedFields: string[];
  onSortChange: SquadOverviewPanelProps["onSortChange"];
  onAddColumn: (metricId: string) => void;
  onRemoveColumn: (metricId: string) => void;
  onMoveColumn: (metricId: string, targetIndex: number) => void;
  onResizeColumn: (metricId: string, width: number) => void;
}) {
  const navigate = useNavigate();

  return (
    <VirtualizedPlayerTable
      caption="Squad overview"
      columnCount={columns.length}
      columns={columns}
      header={
        <PlayerTableHeader
          columns={columns}
          sortBy={sortBy}
          sortDir={sortDir}
          onSortChange={(metricId) => {
            const next = nextSort(sortBy, sortDir, metricId);
            onSortChange(next.sortBy, next.sortDir);
          }}
          onAddColumn={onAddColumn}
          onRemoveColumn={onRemoveColumn}
          onMoveColumn={onMoveColumn}
          onResizeColumn={onResizeColumn}
        />
      }
      pageQueryOptions={(offset, limit) =>
        squadPlayersQueryOptions(
          offset,
          limit,
          sortBy,
          sortDir,
          requestedFields,
        )
      }
      pageSize={SQUAD_PAGE_SIZE}
      renderCells={(player) =>
        columns.map((column) => {
          if (!(SQUAD_SORT_FIELDS as readonly string[]).includes(column.id)) {
            const text = formatDynamicCell(player, column.id);
            return (
              <td
                key={column.id}
                className={
                  column.align === "right"
                    ? NUM_CELL
                    : `${TEXT_CELL} text-on-surface`
                }
                title={text !== "—" && text !== "…" ? text : undefined}
              >
                {text}
              </td>
            );
          }
          if (column.id === "nationality" && player) {
            return (
              <td
                key={column.id}
                className="h-table-row-height-two-line px-2 align-middle text-on-surface"
              >
                <NationalityCell nationalities={player.nationalities} />
              </td>
            );
          }
          const cell = basicCell(player, column.id as BasicSquadSortField);
          if (column.id === "name" && player) {
            return (
              <td
                key={column.id}
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
              key={column.id}
              className={
                cell.numeric
                  ? NUM_CELL
                  : `${TEXT_CELL} ${column.id === "age" || column.id === "division" ? "text-on-surface-variant" : "text-on-surface"}`
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
  const layout = usePlayerTableStore((state) => state.layouts.squad);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeStoredColumn = usePlayerTableStore((state) => state.removeColumn);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const columns = useMemo<TableColumn[]>(
    () =>
      layout.columnIds.flatMap((metricId) => {
        const column = tableColumnForMetric(metricId, layout.widths[metricId]);
        return column ? [column] : [];
      }),
    [layout],
  );
  const requestedFields = useMemo(
    () =>
      columns
        .filter(
          (column) =>
            !(SQUAD_SORT_FIELDS as readonly string[]).includes(column.id),
        )
        .map((column) => column.id)
        .sort(),
    [columns],
  );
  const { data: page } = useSuspenseQuery(
    squadPlayersQueryOptions(
      0,
      SQUAD_PAGE_SIZE,
      sortBy,
      sortDir,
      requestedFields,
    ),
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
  const sortMetric = getPlayerMetric(sortBy);
  const sortLabel = sortMetric
    ? sortMetric.id === "age"
      ? "Age / DOB"
      : sortMetric.label
    : sortBy;
  const removeColumn = (metricId: string) => {
    const remainingColumns = columns.filter((column) => column.id !== metricId);
    if (remainingColumns.length === columns.length) {
      return;
    }
    removeStoredColumn("squad", metricId);
    if (sortBy !== metricId) {
      return;
    }
    const nextColumn =
      remainingColumns.find((column) => column.id === "ca") ??
      remainingColumns[0];
    if (!nextColumn) {
      return;
    }
    onSortChange(nextColumn.id, defaultDirForSquadSortField(nextColumn.id));
  };

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
        columns={columns}
        requestedFields={requestedFields}
        onSortChange={onSortChange}
        onAddColumn={(metricId) => addColumns("squad", [metricId])}
        onRemoveColumn={removeColumn}
        onMoveColumn={(metricId, targetIndex) =>
          moveColumn("squad", metricId, targetIndex)
        }
        onResizeColumn={(metricId, width) =>
          setColumnWidth("squad", metricId, width)
        }
      />
    </Panel>
  );
}
