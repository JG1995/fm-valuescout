import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { UsersRound } from "lucide-react";
import {
  type ReactNode,
  type RefObject,
  useEffect,
  useMemo,
  useState,
} from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type PlayerTableColumn,
  PlayerTableHeader,
} from "@/components/player-table/player-table-header";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import { getPlayerMetric } from "@/utils/player-metrics";
import type { SquadPlayerPageContext } from "../api/squad-keys";
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
  feedback?: ReactNode;
  feedbackRef?: RefObject<HTMLDivElement | null>;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  onSortChange: (sortBy: SquadSortField, sortDir: SquadSortDir) => void;
  pageContext: SquadPlayerPageContext;
};

function SquadFeedbackSlot({
  feedback,
  feedbackRef,
}: Pick<SquadOverviewPanelProps, "feedback" | "feedbackRef">) {
  return (
    <div
      ref={feedbackRef}
      data-testid="squad-boost-feedback"
      tabIndex={-1}
      className="flex min-h-16 items-center px-4 pb-3 text-body-sm focus:outline-2 focus:outline-offset-2 focus:outline-primary"
    >
      {feedback}
    </div>
  );
}

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
  pageContext,
  firstPageQueryOptions,
  isReplacementActive,
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
  pageContext: SquadPlayerPageContext;
  firstPageQueryOptions: ReturnType<typeof squadPlayersQueryOptions>;
  isReplacementActive: boolean;
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
      firstPageQueryOptions={firstPageQueryOptions}
      isReplacementActive={isReplacementActive}
      pageQueryOptions={(offset, limit) =>
        squadPlayersQueryOptions(
          offset,
          limit,
          sortBy,
          sortDir,
          requestedFields,
          pageContext,
        )
      }
      pageSize={SQUAD_PAGE_SIZE}
      renderCells={(player) =>
        columns.map((column) => {
          if (!(SQUAD_SORT_FIELDS as readonly string[]).includes(column.id)) {
            if (column.id === "club_dna") {
              const score = player?.dynamicValues?.[column.id];
              return (
                <td key={column.id} className={NUM_CELL}>
                  {typeof score === "number" ? (
                    <ScoreBadge score={score} roleName={column.label} />
                  ) : (
                    <span className="text-on-surface-variant">
                      {player === undefined ? "…" : "—"}
                    </span>
                  )}
                </td>
              );
            }
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
            const identityContext = [player.club, player.division]
              .filter(
                (value): value is string => value !== null && value !== "",
              )
              .join(" · ");
            return (
              <td
                key={column.id}
                className={`${TEXT_CELL} text-on-surface`}
                title={cell.title}
              >
                {isReplacementActive ? (
                  <span className="block text-on-surface">{cell.text}</span>
                ) : (
                  <Link
                    to="/players/$uid"
                    params={{ uid: String(player.uid) }}
                    search={{}}
                    tabIndex={-1}
                    className="block text-on-surface underline decoration-outline-variant underline-offset-2 transition-colors duration-150 ease-out hover:text-primary"
                    title={player.name}
                    onClick={(event) => {
                      event.stopPropagation();
                    }}
                  >
                    <span className="block truncate">{cell.text}</span>
                    {identityContext ? (
                      <span className="block truncate text-[11px] leading-4 text-on-surface-variant">
                        {identityContext}
                      </span>
                    ) : null}
                  </Link>
                )}
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
          search: {},
        });
      }}
    />
  );
}

export function SquadOverviewPanel({
  actions,
  feedback,
  feedbackRef,
  sortBy,
  sortDir,
  onSortChange,
  pageContext,
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
  const requested = useMemo(
    () => ({ sortBy, sortDir, requestedFields, pageContext }),
    [pageContext, requestedFields, sortBy, sortDir],
  );
  const [committed, setCommitted] = useState(requested);
  const committedOptions = squadPlayersQueryOptions(
    0,
    SQUAD_PAGE_SIZE,
    committed.sortBy,
    committed.sortDir,
    committed.requestedFields,
    committed.pageContext,
  );
  const requestedOptions = squadPlayersQueryOptions(
    0,
    SQUAD_PAGE_SIZE,
    requested.sortBy,
    requested.sortDir,
    requested.requestedFields,
    requested.pageContext,
  );
  const committedQuery = useQuery(committedOptions);
  const requestedQuery = useQuery(requestedOptions);
  const requestMatchesCommitted =
    JSON.stringify(committedOptions.queryKey) ===
    JSON.stringify(requestedOptions.queryKey);
  const isSortReplacement =
    !requestMatchesCommitted &&
    JSON.stringify({
      requestedFields: committed.requestedFields,
      pageContext: committed.pageContext,
    }) ===
      JSON.stringify({
        requestedFields: requested.requestedFields,
        pageContext: requested.pageContext,
      });
  const isReplacementActive = !requestMatchesCommitted;
  const isReplacementPending = isSortReplacement && requestedQuery.isPending;
  const replacementError =
    isSortReplacement && requestedQuery.isError ? requestedQuery.error : null;
  const replacementLabel = requested.sortBy.startsWith("potential_role.")
    ? "Calculating and sorting…"
    : "Sorting…";

  useEffect(() => {
    if (requestedQuery.isSuccess && !requestMatchesCommitted) {
      setCommitted(requested);
    }
  }, [requestMatchesCommitted, requested, requestedQuery.isSuccess]);

  const page =
    requestMatchesCommitted || isSortReplacement
      ? committedQuery.data
      : undefined;
  if (!page) {
    return (
      <Panel title="Squad overview" actions={actions} flush>
        <SquadFeedbackSlot feedback={feedback} feedbackRef={feedbackRef} />
        <EmptyState
          icon={UsersRound}
          title={
            requestedQuery.isError ? "Could not load squad" : "Loading squad"
          }
          action={
            requestedQuery.isError ? (
              <button
                type="button"
                onClick={() => void requestedQuery.refetch()}
              >
                Retry
              </button>
            ) : undefined
          }
        >
          {requestedQuery.isError
            ? requestedQuery.error.message
            : "Loading squad overview…"}
        </EmptyState>
      </Panel>
    );
  }

  if (page.total === 0) {
    return (
      <Panel title="Squad overview" actions={actions} flush>
        <SquadFeedbackSlot feedback={feedback} feedbackRef={feedbackRef} />
        <EmptyState icon={UsersRound} title="No players at your managed club">
          No current-snapshot players match your managed club.
        </EmptyState>
      </Panel>
    );
  }

  const dirLabel = committed.sortDir === "asc" ? "ascending" : "descending";
  const sortMetric = getPlayerMetric(committed.sortBy);
  const sortLabel = sortMetric
    ? sortMetric.id === "age"
      ? "Age / DOB"
      : sortMetric.label
    : committed.sortBy;
  const removeColumn = (metricId: string) => {
    const remainingColumns = columns.filter((column) => column.id !== metricId);
    if (remainingColumns.length === columns.length) {
      return;
    }
    removeStoredColumn("squad", metricId);
    if (requested.sortBy !== metricId) {
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
      {isReplacementPending ? (
        <p
          className="shrink-0 px-4 pb-3 text-body-sm text-on-surface-variant"
          role="status"
        >
          {replacementLabel}
        </p>
      ) : null}
      {replacementError ? (
        <div
          className="flex shrink-0 items-center justify-between gap-3 px-4 pb-3 text-body-sm text-error"
          role="alert"
        >
          <span>Could not sort squad. {replacementError.message}</span>
          <button
            type="button"
            className="shrink-0 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => void requestedQuery.refetch()}
          >
            Retry
          </button>
        </div>
      ) : null}
      <SquadFeedbackSlot feedback={feedback} feedbackRef={feedbackRef} />
      <SquadOverviewTable
        total={page.total}
        sortBy={committed.sortBy}
        sortDir={committed.sortDir}
        columns={columns}
        requestedFields={committed.requestedFields}
        pageContext={committed.pageContext}
        firstPageQueryOptions={committedOptions}
        isReplacementActive={isReplacementActive}
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
