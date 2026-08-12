import { useSuspenseQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ChevronDown, ChevronUp, SearchX } from "lucide-react";
import { useMemo } from "react";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import {
  SEARCH_PAGE_SIZE,
  searchPlayersQueryOptions,
} from "../api/search-players-query-options";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import { filterValueToIpc } from "../types/filter-rule";
import type { PlayerSummary } from "../types/player-summary";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  type BASIC_SEARCH_SORT_FIELDS,
  defaultDirForSortField,
  isBasicSearchSortField,
} from "../types/search-sort";
import {
  dynamicColumnFields,
  dynamicColumnLabel,
} from "../utils/dynamic-columns";
import { completeFilterRules, getFilterField } from "../utils/filter-registry";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

const BASIC_COLUMNS = [
  { key: "name", label: "Name", align: "left" as const },
  { key: "age", label: "Age / DOB", align: "left" as const },
  { key: "nationality", label: "Nationality", align: "left" as const },
  { key: "club", label: "Club", align: "left" as const },
  { key: "division", label: "Division", align: "left" as const },
  { key: "ca", label: "CA", align: "right" as const },
  { key: "pa", label: "PA", align: "right" as const },
  { key: "value", label: "Value", align: "right" as const },
] as const satisfies ReadonlyArray<{
  key: (typeof BASIC_SEARCH_SORT_FIELDS)[number];
  label: string;
  align: "left" | "right";
}>;

type TableColumn = {
  key: SearchSortField;
  label: string;
  align: "left" | "right";
  dynamic?: boolean;
};

const BASIC_SORT_LABELS: Record<
  (typeof BASIC_SEARCH_SORT_FIELDS)[number],
  string
> = {
  name: "Name",
  age: "Age / DOB",
  nationality: "Nationality",
  club: "Club",
  division: "Division",
  ca: "CA",
  pa: "PA",
  value: "Value",
};

type SearchResultsPanelProps = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
};

function nextSort(
  currentBy: SearchSortField,
  currentDir: SearchSortDir,
  clicked: SearchSortField,
): { sortBy: SearchSortField; sortDir: SearchSortDir } {
  if (clicked === currentBy) {
    return {
      sortBy: currentBy,
      sortDir: currentDir === "asc" ? "desc" : "asc",
    };
  }
  return { sortBy: clicked, sortDir: defaultDirForSortField(clicked) };
}

function formatDynamicCell(
  player: PlayerSummary | undefined,
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

function basicCell(
  player: PlayerSummary | undefined,
  key: (typeof BASIC_SEARCH_SORT_FIELDS)[number],
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

function SearchResultsVirtualTable({
  total,
  sortBy,
  sortDir,
  filters,
  filterCombine,
  columns,
  requestedFields,
  onSortChange,
}: {
  total: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  columns: TableColumn[];
  requestedFields: string[];
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
}) {
  const navigate = useNavigate();

  return (
    <VirtualizedPlayerTable
      caption="Player search results"
      columnCount={columns.length}
      header={
        <thead className="sticky top-0 z-10">
          <tr className="bg-surface-container-lowest">
            {columns.map((column) => {
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
        searchPlayersQueryOptions(
          offset,
          limit,
          sortBy,
          sortDir,
          filters,
          filterCombine,
          requestedFields,
        )
      }
      pageSize={SEARCH_PAGE_SIZE}
      renderCells={(player) =>
        columns.map((column) => {
          if (column.dynamic) {
            const text = formatDynamicCell(player, column.key);
            return (
              <td
                key={column.key}
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
          const cell = basicCell(
            player,
            column.key as (typeof BASIC_SEARCH_SORT_FIELDS)[number],
          );
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
      testId="search-results-scroller"
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

/** Assumes a current snapshot exists — the route handles the no-snapshot empty. */
export function SearchResultsPanel({
  sortBy,
  sortDir,
  filters,
  filterCombine,
  onSortChange,
}: SearchResultsPanelProps) {
  const dynamicFields = useMemo(() => dynamicColumnFields(filters), [filters]);
  const columns = useMemo<TableColumn[]>(
    () => [
      ...BASIC_COLUMNS.map((column) => ({ ...column })),
      ...dynamicFields.map((fieldId) => {
        const kind = getFilterField(fieldId)?.kind;
        const numeric = kind === "integer" || kind === "boolean";
        return {
          key: fieldId,
          label: dynamicColumnLabel(fieldId),
          align: numeric ? ("right" as const) : ("left" as const),
          dynamic: true,
        };
      }),
    ],
    [dynamicFields],
  );

  const { data: page } = useSuspenseQuery(
    searchPlayersQueryOptions(
      0,
      SEARCH_PAGE_SIZE,
      sortBy,
      sortDir,
      filters,
      filterCombine,
      dynamicFields,
    ),
  );
  const listKey = useMemo(
    () =>
      [
        filterCombine,
        ...filters.map(
          (rule) =>
            `${rule.field}:${rule.op}:${String(filterValueToIpc(rule.value))}`,
        ),
      ].join("|"),
    [filterCombine, filters],
  );

  if (page.total === 0) {
    const appliedFilters = completeFilterRules(filters);
    if (appliedFilters.length > 0) {
      return (
        <Panel title="Results" flush>
          <EmptyState icon={SearchX} title="No players match these filters">
            Adjust or clear filters in the strip above to widen the result set.
          </EmptyState>
        </Panel>
      );
    }

    return (
      <Panel title="Results" flush>
        <EmptyState icon={SearchX} title="No players in snapshot">
          The snapshot exists but holds no player rows. Run Load Data again with
          Football Manager in an active save.
        </EmptyState>
      </Panel>
    );
  }

  const dirLabel = sortDir === "asc" ? "ascending" : "descending";
  const sortLabel = isBasicSearchSortField(sortBy)
    ? BASIC_SORT_LABELS[sortBy]
    : dynamicColumnLabel(sortBy);

  return (
    <Panel
      title="Results"
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <p className="shrink-0 px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
        players · sorted by {sortLabel} ({dirLabel})
      </p>
      <SearchResultsVirtualTable
        key={listKey}
        total={page.total}
        sortBy={sortBy}
        sortDir={sortDir}
        filters={filters}
        filterCombine={filterCombine}
        columns={columns}
        requestedFields={dynamicFields}
        onSortChange={onSortChange}
      />
    </Panel>
  );
}
