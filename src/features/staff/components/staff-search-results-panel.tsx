import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { DatabaseZap, SearchX, UsersRound } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
} from "@/components/player-table/player-table-header";
import { ConfigurableVirtualizedTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { formatCount, formatMissable, formatPlayerDob } from "@/utils/format";
import { boostStaffCurrentAbility } from "../api/boost-staff-current-ability";
import { staffKeys } from "../api/staff-keys";
import {
  STAFF_PAGE_SIZE,
  staffMyStaffQueryOptions,
  staffSearchQueryOptions,
} from "../api/staff-query-options";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";
import type { StaffPage, StaffSummary } from "../types/staff-summary";
import { completeStaffFilterRules } from "../utils/staff-filter-registry";
import {
  defaultDirForStaffSortField,
  getStaffMetric,
  STAFF_BASIC_METRIC_IDS,
  STAFF_METRICS,
} from "../utils/staff-metrics";
import { StaffCaBoost } from "./staff-ca-boost";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

const STAFF_ACTION_COLUMN = {
  id: "actions",
  label: "Actions",
  align: "left" as const,
  width: 128,
};

export type StaffWorkspaceScope = "search" | "my-staff";
type StaffLayoutId = "staff-search" | "my-staff";

function nextSort(
  currentBy: StaffSortField,
  currentDir: StaffSortDir,
  clicked: StaffSortField,
) {
  if (clicked === currentBy) {
    return {
      sortBy: currentBy,
      sortDir: currentDir === "asc" ? "desc" : "asc",
    } as const;
  }
  return {
    sortBy: clicked,
    sortDir: defaultDirForStaffSortField(clicked),
  } as const;
}

function dynamicCell(staff: StaffSummary | undefined, fieldId: string) {
  if (!staff) {
    return "…";
  }
  const value = staff.dynamicValues?.[fieldId];
  return value === null || value === undefined ? "—" : String(value);
}

function basicCell(
  staff: StaffSummary | undefined,
  fieldId: string,
): { text: string; title?: string; numeric: boolean } {
  if (!staff) {
    return {
      text: "…",
      numeric: fieldId !== "name" && fieldId !== "nationality",
    };
  }
  switch (fieldId) {
    case "name": {
      const text = String(formatMissable(staff.name));
      return { text, title: text !== "—" ? text : undefined, numeric: false };
    }
    case "age": {
      const text =
        staff.birthYear !== null && staff.birthDayOfYear !== null
          ? formatPlayerDob(staff.birthYear, staff.birthDayOfYear, staff.age)
          : String(formatMissable(staff.age));
      return { text, title: text !== "—" ? text : undefined, numeric: false };
    }
    case "birth_year":
      return { text: String(formatMissable(staff.birthYear)), numeric: true };
    case "birth_day_of_year":
      return {
        text: String(formatMissable(staff.birthDayOfYear)),
        numeric: true,
      };
    case "nationality": {
      const text = String(formatMissable(staff.nationalities.join(", ")));
      return { text, title: text !== "—" ? text : undefined, numeric: false };
    }
    case "club":
    case "division": {
      const value = fieldId === "club" ? staff.club : staff.division;
      const text = String(formatMissable(value));
      return { text, title: text !== "—" ? text : undefined, numeric: false };
    }
    case "ca":
      return { text: String(staff.ca), numeric: true };
    case "pa":
      return { text: String(staff.pa), numeric: true };
    case "wage":
      return {
        text: String(formatMissable(staff.weeklyWageGbp)),
        numeric: true,
      };
    case "contract_year":
      return {
        text: String(formatMissable(staff.contractExpiryYear)),
        numeric: true,
      };
    case "contract_day":
      return {
        text: String(formatMissable(staff.contractExpiryDayOfYear)),
        numeric: true,
      };
    case "nation_uid":
      return { text: String(formatMissable(staff.nationUid)), numeric: true };
    case "gender":
      return { text: staff.gender || "—", numeric: false };
    case "job_id":
      return { text: String(formatMissable(staff.jobId)), numeric: true };
    default:
      return { text: "—", numeric: false };
  }
}

function columnForMetric(
  metricId: string,
  width: number | undefined,
): ConfigurableTableColumn | undefined {
  const metric = getStaffMetric(metricId);
  return metric
    ? {
        id: metric.id,
        label: metric.label,
        align: metric.align,
        width: width ?? metric.defaultWidth,
      }
    : undefined;
}

function StaffSearchTable({
  total,
  sortBy,
  sortDir,
  columns,
  scope,
  boostPending,
  boostUid,
  boostError,
  onBoost,
  onOpenConfirmation,
  fallbackFocusTo,
  pageQueryOptions,
  caption,
  testId,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
  onRowActivate,
}: {
  total: number;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  columns: ConfigurableTableColumn[];
  scope: StaffWorkspaceScope;
  boostPending: boolean;
  boostUid: number | undefined;
  boostError: Error | null;
  onBoost: (uid: number) => Promise<unknown>;
  onOpenConfirmation: () => void;
  fallbackFocusTo: () => HTMLElement | null;
  pageQueryOptions: (
    offset: number,
    limit: number,
  ) => ReturnType<typeof staffSearchQueryOptions>;
  caption: string;
  testId: string;
  onSortChange: (sort: StaffSortField, dir: StaffSortDir) => void;
  onAddColumn: (id: string) => void;
  onRemoveColumn: (id: string) => void;
  onMoveColumn: (id: string, target: number) => void;
  onResizeColumn: (id: string, width: number) => void;
  onRowActivate?: (staff: StaffSummary) => void;
}) {
  return (
    <ConfigurableVirtualizedTable<
      StaffPage,
      StaffSummary,
      ReturnType<typeof staffKeys.list>
    >
      caption={caption}
      columnCount={columns.length}
      columns={columns}
      fixedColumns={scope === "my-staff" ? [STAFF_ACTION_COLUMN] : []}
      getPageRows={(page) => page.staff}
      getRowKey={(staff) => staff.uid}
      onRowActivate={onRowActivate}
      header={
        <ConfigurableTableHeader
          columns={columns}
          fixedColumns={scope === "my-staff" ? [STAFF_ACTION_COLUMN] : []}
          metrics={STAFF_METRICS}
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
      pageQueryOptions={pageQueryOptions}
      pageSize={STAFF_PAGE_SIZE}
      renderCells={(staff) =>
        columns.map((column) => {
          if (!STAFF_BASIC_METRIC_IDS.includes(column.id)) {
            const text = dynamicCell(staff, column.id);
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
          if (column.id === "nationality" && staff) {
            return (
              <td
                key={column.id}
                className="h-table-row-height-two-line px-2 align-middle text-on-surface"
              >
                <NationalityCell nationalities={staff.nationalities} />
              </td>
            );
          }
          const cell = basicCell(staff, column.id);
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
      renderFixedCells={
        scope === "my-staff"
          ? (staff) => (
              <td className="h-table-row-height-two-line align-middle">
                <StaffCaBoost
                  staff={staff}
                  pending={boostPending && boostUid === staff?.uid}
                  error={boostUid === staff?.uid ? boostError : null}
                  onBoost={() =>
                    staff ? onBoost(staff.uid) : Promise.resolve()
                  }
                  onOpenConfirmation={onOpenConfirmation}
                  fallbackFocusTo={fallbackFocusTo}
                />
              </td>
            )
          : undefined
      }
      testId={testId}
      total={total}
    />
  );
}

export function StaffSearchResultsPanel({
  scope = "search",
  sortBy,
  sortDir,
  filters,
  filterCombine,
  onSortChange,
  onBoostSuccess,
  onRowActivate,
}: {
  scope?: StaffWorkspaceScope;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  filters: StaffFilterRule[];
  filterCombine: "and" | "or";
  onSortChange: (sort: StaffSortField, dir: StaffSortDir) => void;
  onBoostSuccess?: () => Promise<void>;
  onRowActivate?: (staff: StaffSummary) => void;
}) {
  const layoutId: StaffLayoutId =
    scope === "my-staff" ? "my-staff" : "staff-search";
  const layout = usePlayerTableStore((state) => state.layouts[layoutId]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeColumn = usePlayerTableStore((state) => state.removeColumn);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const queryClient = useQueryClient();
  const boost = useMutation({
    mutationFn: ({ uid }: { uid: number }) => boostStaffCurrentAbility(uid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: staffKeys.all }),
        onBoostSuccess?.(),
      ]);
    },
  });
  const boostOutcomeRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!boost.data || boost.isPending) return;
    const frame = requestAnimationFrame(() => {
      if (!document.querySelector('[role="dialog"]')) {
        boostOutcomeRef.current?.focus();
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [boost.data, boost.isPending]);
  const columns = useMemo(
    () =>
      layout.columnIds.flatMap((id) => {
        const column = columnForMetric(id, layout.widths[id]);
        return column ? [column] : [];
      }),
    [layout],
  );
  const requestedFields = useMemo(
    () =>
      columns
        .filter((column) => !STAFF_BASIC_METRIC_IDS.includes(column.id))
        .map((column) => column.id)
        .sort(),
    [columns],
  );
  const boostContextIsCurrent =
    scope === "my-staff" && boost.variables?.uid !== undefined;
  const { data: page } = useSuspenseQuery(
    scope === "my-staff"
      ? staffMyStaffQueryOptions(
          0,
          STAFF_PAGE_SIZE,
          sortBy,
          sortDir,
          requestedFields,
        )
      : staffSearchQueryOptions(
          0,
          STAFF_PAGE_SIZE,
          sortBy,
          sortDir,
          filters,
          filterCombine,
          requestedFields,
        ),
  );

  if (page.state === "no_current_snapshot") {
    return (
      <Panel title={scope === "my-staff" ? "My Staff" : "Results"} flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          Use Load Data to scan Football Manager and ingest staff into the
          database.
        </EmptyState>
      </Panel>
    );
  }
  if (page.state === "no_club_family") {
    return (
      <Panel title="My Staff" flush>
        <EmptyState
          icon={UsersRound}
          title="Set up your club family"
          action={
            <Link
              to="/"
              hash="club-setup"
              className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
            >
              Open Club Setup
            </Link>
          }
        >
          Configure your club family in Dashboard before reviewing your staff.
        </EmptyState>
      </Panel>
    );
  }
  if (page.total === 0) {
    return (
      <Panel title={scope === "my-staff" ? "My Staff" : "Results"} flush>
        <EmptyState
          icon={scope === "my-staff" ? UsersRound : SearchX}
          title={
            scope === "my-staff"
              ? "No staff in your club family"
              : completeStaffFilterRules(filters).length > 0
                ? "No staff match these filters"
                : "No staff in snapshot"
          }
        >
          {scope === "my-staff"
            ? "No current-snapshot staff match the clubs configured for this save."
            : completeStaffFilterRules(filters).length > 0
              ? "Adjust or clear filters to widen the result set."
              : "The snapshot exists but contains no staff rows."}
        </EmptyState>
      </Panel>
    );
  }

  const sortMetric = getStaffMetric(sortBy);
  const sortLabel = sortMetric?.label ?? sortBy;
  const requestedRoleFields = requestedFields.filter((field) =>
    field.startsWith("role."),
  );
  const allScoresUnavailable =
    requestedRoleFields.length > 0 &&
    page.staff.length > 0 &&
    page.staff.every((staff) =>
      requestedRoleFields.every(
        (field) =>
          staff.dynamicValues?.[field] === null ||
          staff.dynamicValues?.[field] === undefined,
      ),
    );
  const removeStoredColumn = (metricId: string) => {
    if (columns.length <= 1) return;
    removeColumn(layoutId, metricId);
    if (sortBy === metricId) {
      const next =
        columns.find((column) => column.id !== metricId) ?? columns[0];
      onSortChange(next.id, defaultDirForStaffSortField(next.id));
    }
  };

  return (
    <Panel
      title={scope === "my-staff" ? "My Staff" : "Results"}
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <p className="shrink-0 px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span> staff
        · sorted by {sortLabel} (
        {sortDir === "asc" ? "ascending" : "descending"})
      </p>
      <div
        ref={boostOutcomeRef}
        data-testid="staff-boost-outcome"
        tabIndex={-1}
        className="rounded-sm px-4 [&:not(:empty)]:pb-3 focus:outline-2 focus:outline-offset-2 focus:outline-primary"
        aria-live="polite"
      >
        {boost.data ? (
          <p className="text-body-sm text-success" role="status">
            Staff CA boosted from {boost.data.previousCurrentAbility} to{" "}
            {boost.data.currentAbility}.
          </p>
        ) : null}
        {boost.error && !boost.isPending ? (
          <p className="text-body-sm text-error" role="alert">
            Could not apply staff CA boost. {boost.error.message}
          </p>
        ) : null}
      </div>
      {allScoresUnavailable ? (
        <p
          role="status"
          className="shrink-0 px-4 pb-3 text-body-sm text-warning"
        >
          Staff role scores are unavailable for this snapshot. Update the Bridge
          and run Load Data to calculate them.
        </p>
      ) : null}
      <StaffSearchTable
        total={page.total}
        sortBy={sortBy}
        sortDir={sortDir}
        columns={columns}
        scope={scope}
        boostPending={boostContextIsCurrent && boost.isPending}
        boostUid={boostContextIsCurrent ? boost.variables?.uid : undefined}
        boostError={boostContextIsCurrent ? boost.error : null}
        onBoost={(uid) => boost.mutateAsync({ uid })}
        onOpenConfirmation={boost.reset}
        fallbackFocusTo={() => boostOutcomeRef.current}
        pageQueryOptions={(offset, limit) =>
          scope === "my-staff"
            ? staffMyStaffQueryOptions(
                offset,
                limit,
                sortBy,
                sortDir,
                requestedFields,
              )
            : staffSearchQueryOptions(
                offset,
                limit,
                sortBy,
                sortDir,
                filters,
                filterCombine,
                requestedFields,
              )
        }
        caption={
          scope === "my-staff" ? "My Staff overview" : "Staff search results"
        }
        testId={`${layoutId}-results-scroller`}
        onSortChange={onSortChange}
        onAddColumn={(id) => addColumns(layoutId, [id])}
        onRemoveColumn={removeStoredColumn}
        onMoveColumn={(id, target) => moveColumn(layoutId, id, target)}
        onResizeColumn={(id, width) => setColumnWidth(layoutId, id, width)}
        onRowActivate={onRowActivate}
      />
    </Panel>
  );
}
