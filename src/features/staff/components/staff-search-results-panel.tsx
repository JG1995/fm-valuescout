import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { DatabaseZap, SearchX, UsersRound } from "lucide-react";
import { useMemo, useRef } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
} from "@/components/player-table/player-table-header";
import { ConfigurableVirtualizedTable } from "@/components/player-table/virtualized-player-table";
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
import { boostMyStaffCurrentAbility } from "../api/boost-my-staff-current-ability";
import { staffKeys } from "../api/staff-keys";
import {
  STAFF_PAGE_SIZE,
  staffMyStaffQueryOptions,
  staffSearchQueryOptions,
  staffShortlistQueryOptions,
} from "../api/staff-query-options";
import type { StaffFilterRule } from "../types/staff-filter-rule";
import type { StaffSortDir, StaffSortField } from "../types/staff-sort";
import type { StaffPage, StaffSummary } from "../types/staff-summary";
import { completeStaffFilterRules } from "../utils/staff-filter-registry";
import {
  defaultDirForStaffSortField,
  getStaffMetric,
  getStaffShortlistMetric,
  STAFF_BASIC_METRIC_IDS,
  STAFF_METRICS,
  STAFF_SHORTLIST_METRICS,
} from "../utils/staff-metrics";
import { MyStaffBoostOutcome, MyStaffCaBoost } from "./my-staff-ca-boost";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const AGE_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

export type StaffWorkspaceScope = "search" | "my-staff" | "shortlist";
type StaffLayoutId = "staff-search" | "my-staff" | "staff-shortlist";

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
        text:
          staff.weeklyWageGbp === null ? "—" : formatMoney(staff.weeklyWageGbp),
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

function StaffSearchTable({
  total,
  sortBy,
  sortDir,
  columns,
  pageQueryOptions,
  caption,
  testId,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
  onRowActivate,
  scope,
  configurable,
}: {
  total: number;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  columns: ConfigurableTableColumn[];
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
  scope: StaffWorkspaceScope;
  configurable: boolean;
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
      getPageRows={(page) => page.staff}
      getRowKey={(staff) => staff.uid}
      onRowActivate={onRowActivate}
      header={
        <ConfigurableTableHeader
          columns={columns}
          configurable={configurable}
          sortable
          metrics={
            scope === "shortlist" ? STAFF_SHORTLIST_METRICS : STAFF_METRICS
          }
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
            if (
              column.id === "preferred_job" ||
              column.id === "club_job" ||
              column.id === "coaching_qualifications"
            ) {
              const value =
                column.id === "preferred_job"
                  ? staff?.shortlist?.preferredJob
                  : column.id === "club_job"
                    ? staff?.shortlist?.clubJob
                    : staff?.shortlist?.coachingQualifications;
              return (
                <td key={column.id} className={`${TEXT_CELL} text-on-surface`}>
                  {value || (staff === undefined ? "…" : "—")}
                </td>
              );
            }
            if (column.id.startsWith("role.")) {
              const score = staff?.dynamicValues?.[column.id];
              return (
                <td key={column.id} className={NUM_CELL}>
                  {staff === undefined ||
                  score === null ||
                  score === undefined ? (
                    <span className="text-on-surface-variant">
                      {staff === undefined ? "…" : "—"}
                    </span>
                  ) : (
                    <ScoreBadge
                      score={score}
                      roleName={`${column.label} role score`}
                    />
                  )}
                </td>
              );
            }
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
                  : `${column.id === "age" ? AGE_CELL : TEXT_CELL} ${column.id === "age" || column.id === "division" ? "text-on-surface-variant" : "text-on-surface"}`
              }
              title={cell.title}
            >
              {cell.text}
            </td>
          );
        })
      }
      testId={testId}
      total={total}
    />
  );
}

export function StaffSearchResultsPanel({
  activeSnapshotId,
  scope = "search",
  sortBy,
  sortDir,
  filters,
  filterCombine,
  preferredJob,
  unemployedOnly = false,
  onSortChange,
  onBoostSuccess,
  onRowActivate,
  visibleColumnIds,
}: {
  activeSnapshotId: number | null;
  scope?: StaffWorkspaceScope;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  filters: StaffFilterRule[];
  filterCombine: "and" | "or";
  preferredJob?: string;
  unemployedOnly?: boolean;
  onSortChange: (sort: StaffSortField, dir: StaffSortDir) => void;
  onBoostSuccess?: () => Promise<void>;
  onRowActivate?: (staff: StaffSummary) => void;
  visibleColumnIds?: string[];
}) {
  const layoutId: StaffLayoutId =
    scope === "my-staff"
      ? "my-staff"
      : scope === "shortlist"
        ? "staff-shortlist"
        : "staff-search";
  const layout = usePlayerTableStore((state) => state.layouts[layoutId]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeColumn = usePlayerTableStore((state) => state.removeColumn);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const queryClient = useQueryClient();
  const boost = useMutation({
    mutationFn: ({
      onProgress,
    }: {
      snapshotId: number;
      onProgress: Parameters<typeof boostMyStaffCurrentAbility>[0];
    }) => boostMyStaffCurrentAbility(onProgress),
    onSettled: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: staffKeys.all }),
        onBoostSuccess?.(),
      ]);
    },
  });
  const boostContextIsCurrent =
    boost.variables?.snapshotId === activeSnapshotId;
  const boostOutcomeRef = useRef<HTMLDivElement>(null);
  const columns = useMemo(
    () =>
      (visibleColumnIds ?? layout.columnIds).flatMap((id) => {
        const metric =
          scope === "shortlist"
            ? getStaffShortlistMetric(id)
            : getStaffMetric(id);
        const column = metric
          ? {
              id: metric.id,
              label: metric.label,
              align: metric.align,
              width: layout.widths[id] ?? metric.defaultWidth,
            }
          : undefined;
        return column ? [column] : [];
      }),
    [layout, scope, visibleColumnIds],
  );
  const requestedFields = useMemo(
    () =>
      columns
        .filter(
          (column) =>
            !STAFF_BASIC_METRIC_IDS.includes(column.id) &&
            !["preferred_job", "club_job", "coaching_qualifications"].includes(
              column.id,
            ),
        )
        .map((column) => column.id)
        .sort(),
    [columns],
  );
  const { data: page } = useSuspenseQuery(
    scope === "my-staff"
      ? staffMyStaffQueryOptions(
          0,
          STAFF_PAGE_SIZE,
          sortBy,
          sortDir,
          requestedFields,
        )
      : scope === "shortlist"
        ? staffShortlistQueryOptions(
            0,
            STAFF_PAGE_SIZE,
            sortBy,
            sortDir,
            preferredJob,
            unemployedOnly,
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
      <Panel title={scope === "my-staff" ? "Staff" : "Results"} flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          Use Load Data to scan Football Manager and ingest staff into the
          database.
        </EmptyState>
      </Panel>
    );
  }
  if (page.state === "no_managed_club") {
    return (
      <Panel title="Staff" flush>
        <EmptyState
          icon={UsersRound}
          title="Choose your managed club"
          action={
            <Link
              to="/my-club"
              hash="managed-club"
              className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
            >
              Open Managed Club
            </Link>
          }
        >
          Choose your managed club in My Club before reviewing your staff.
        </EmptyState>
      </Panel>
    );
  }
  if (page.state === "no_shortlist") {
    return (
      <Panel title="Staff Shortlist" flush>
        <EmptyState icon={UsersRound} title="No Staff Shortlist uploaded">
          Upload a staff CSV to view the people included in it.
        </EmptyState>
      </Panel>
    );
  }
  if (page.total === 0) {
    const isShortlist = scope === "shortlist";
    const hasShortlistFilter = preferredJob || unemployedOnly;
    return (
      <Panel
        title={
          scope === "my-staff"
            ? "Staff"
            : isShortlist
              ? "Staff Shortlist"
              : "Results"
        }
        flush
      >
        <EmptyState
          icon={scope === "my-staff" || isShortlist ? UsersRound : SearchX}
          title={
            scope === "my-staff"
              ? "No staff at your managed club"
              : isShortlist
                ? hasShortlistFilter
                  ? "No shortlist staff match these filters"
                  : "No shortlisted staff in this snapshot"
                : completeStaffFilterRules(filters).length > 0
                  ? "No staff match these filters"
                  : "No staff in snapshot"
          }
        >
          {scope === "my-staff"
            ? "No current-snapshot staff match your managed club."
            : isShortlist
              ? hasShortlistFilter
                ? "Choose All jobs or turn off Only unemployed to widen the results."
                : "Load Data to restore saved shortlist people who are absent from the current snapshot."
              : completeStaffFilterRules(filters).length > 0
                ? "Adjust or clear filters to widen the result set."
                : "The snapshot exists but contains no staff rows."}
        </EmptyState>
      </Panel>
    );
  }

  const sortMetric =
    scope === "shortlist"
      ? getStaffShortlistMetric(sortBy)
      : getStaffMetric(sortBy);
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
    if (scope === "shortlist" && visibleColumnIds) return;
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
      title={
        scope === "my-staff"
          ? "Staff"
          : scope === "shortlist"
            ? "Staff Shortlist"
            : "Results"
      }
      actions={
        scope === "my-staff" ? (
          <MyStaffCaBoost
            pending={boostContextIsCurrent && boost.isPending}
            disabled={
              boostContextIsCurrent && boost.data?.recoveryRequired === true
            }
            error={boostContextIsCurrent ? boost.error : null}
            onBoost={(onProgress) =>
              boost.mutateAsync({
                snapshotId: activeSnapshotId ?? 0,
                onProgress,
              })
            }
            onOpenConfirmation={boost.reset}
            fallbackFocusTo={() => boostOutcomeRef.current}
          />
        ) : undefined
      }
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
        {scope === "my-staff" && boostContextIsCurrent && !boost.isPending ? (
          <MyStaffBoostOutcome result={boost.data} error={boost.error} />
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
        pageQueryOptions={(offset, limit) =>
          scope === "my-staff"
            ? staffMyStaffQueryOptions(
                offset,
                limit,
                sortBy,
                sortDir,
                requestedFields,
              )
            : scope === "shortlist"
              ? staffShortlistQueryOptions(
                  offset,
                  limit,
                  sortBy,
                  sortDir,
                  preferredJob,
                  unemployedOnly,
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
          scope === "my-staff"
            ? "Staff overview"
            : scope === "shortlist"
              ? "Staff Shortlist"
              : "Staff search results"
        }
        testId={`${layoutId}-results-scroller`}
        onSortChange={onSortChange}
        onAddColumn={(id) => {
          if (!visibleColumnIds) addColumns(layoutId, [id]);
        }}
        onRemoveColumn={removeStoredColumn}
        onMoveColumn={(id, target) => {
          if (!visibleColumnIds) moveColumn(layoutId, id, target);
        }}
        onResizeColumn={(id, width) => {
          if (!visibleColumnIds) setColumnWidth(layoutId, id, width);
        }}
        onRowActivate={onRowActivate}
        scope={scope}
        configurable={!visibleColumnIds}
      />
    </Panel>
  );
}
