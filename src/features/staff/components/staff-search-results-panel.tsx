import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { DatabaseZap, SearchX, UsersRound } from "lucide-react";
import { type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  ConfigurableColumnsControl,
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
} from "@/components/player-table/player-table-header";
import {
  formatTableDynamicCell as dynamicCell,
  TABLE_NUMERIC_CELL_CLASS as NUM_CELL,
  TableScoreContent,
  TABLE_TEXT_CELL_CLASS as TEXT_CELL,
} from "@/components/player-table/table-cells";
import type { TableGroupInput } from "@/components/player-table/table-groups";
import { TableToolbar } from "@/components/player-table/table-toolbar";
import {
  type ConfigurableTableIdentity,
  ConfigurableVirtualizedTable,
} from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  isIdentityColumnId,
  usePlayerTableStore,
  withoutIdentityColumnIds,
} from "@/stores/use-player-table-store";
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
import { staffShortlistPresentation } from "../utils/staff-shortlist-presentation";
import { MyStaffBoostOutcome, MyStaffCaBoost } from "./my-staff-ca-boost";
import { StaffFilterBar } from "./staff-filter-bar";

const AGE_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-body-sm";

export type StaffWorkspaceScope = "search" | "my-staff";
type StaffLayoutId = "staff-search" | "my-staff" | "staff-shortlist";

function ShortlistSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange?: (next: boolean) => void;
}) {
  if (!onChange) return null;
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="inline-flex items-center gap-2 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
    >
      Shortlist: {checked ? "On" : "Off"}
    </button>
  );
}

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

export const STAFF_CONFIGURABLE_METRICS = STAFF_METRICS.filter(
  (metric) => !isIdentityColumnId(metric.id),
);

export const STAFF_SHORTLIST_CONFIGURABLE_METRICS =
  STAFF_SHORTLIST_METRICS.filter((metric) => !isIdentityColumnId(metric.id));

function StaffIdentityCell({
  name,
  club,
  division,
}: {
  name: string | undefined;
  club: string | null | undefined;
  division: string | null | undefined;
}) {
  const context =
    name === undefined
      ? null
      : [club, division]
          .filter((value): value is string => value !== null && value !== "")
          .join(" · ");
  return (
    <div className="flex h-table-row-height-two-line items-center gap-2 px-2">
      <span
        aria-hidden="true"
        className="h-7 w-7 shrink-0 rounded-sm bg-surface-container-high"
      />
      <span className="min-w-0 flex-1">
        <span
          className="block truncate text-body-sm text-on-surface"
          title={name}
        >
          {name ?? "…"}
        </span>
        {context ? (
          <span className="flex min-w-0 items-center gap-1 text-[11px] leading-4 text-on-surface-variant">
            <span
              aria-hidden="true"
              className="h-3 w-3 shrink-0 rounded-[2px] bg-surface-container-high"
            />
            <span className="block truncate">{context}</span>
          </span>
        ) : null}
      </span>
    </div>
  );
}

export const STAFF_TABLE_GROUPS: TableGroupInput = {
  groups: [
    { id: "profile", label: "Profile" },
    { id: "ability", label: "Ability" },
    { id: "attributes", label: "Attributes" },
    { id: "role-fit", label: "Role Fit" },
    { id: "contract", label: "Contract" },
  ],
  groupForColumn: (columnId) => {
    switch (getStaffMetric(columnId)?.category) {
      case "identity":
        return "profile";
      case "ability-reputation":
        return "ability";
      case "staff-attributes":
        return "attributes";
      case "current-role-scores":
        return "role-fit";
      case "club-contract":
        return "contract";
      default:
        return "other";
    }
  },
};

export const STAFF_SHORTLIST_TABLE_GROUPS: TableGroupInput = {
  groups: [
    ...STAFF_TABLE_GROUPS.groups,
    { id: "recruitment", label: "Recruitment" },
  ],
  groupForColumn: (columnId) => {
    switch (getStaffShortlistMetric(columnId)?.category) {
      case "identity":
        return "profile";
      case "ability-reputation":
        return "ability";
      case "staff-attributes":
        return "attributes";
      case "current-role-scores":
        return "role-fit";
      case "club-contract":
        return "contract";
      case "shortlist":
        return "recruitment";
      default:
        return "other";
    }
  },
};

type StaffResultsQueryParams = {
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  filters: StaffFilterRule[];
  filterCombine: "and" | "or";
  requestedFields: string[];
  shortlistOnly: boolean;
  preferredJob?: string;
  unemployedOnly: boolean;
};

function staffResultsPageOptions(
  scope: StaffWorkspaceScope,
  params: StaffResultsQueryParams,
  offset: number,
  limit: number,
) {
  // `shortlistOnly` doubles as the shortlist flag: it is only read on the
  // search scope, where it equals `isShortlist`.
  return scope === "my-staff"
    ? staffMyStaffQueryOptions(
        offset,
        limit,
        params.sortBy,
        params.sortDir,
        params.requestedFields,
      )
    : staffSearchQueryOptions(
        offset,
        limit,
        params.sortBy,
        params.sortDir,
        params.filters,
        params.filterCombine,
        params.requestedFields,
        params.shortlistOnly,
        params.preferredJob,
        params.unemployedOnly,
      );
}

function StaffSearchTable({
  total,
  sortBy,
  sortDir,
  columns,
  identity,
  pageQueryOptions,
  firstPageQueryOptions,
  isReplacementActive = false,
  unresolvedFieldIds,
  caption,
  testId,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
  onRowActivate,
  shortlist,
  configurable,
}: {
  total: number;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  columns: ConfigurableTableColumn[];
  identity: ConfigurableTableIdentity<StaffSummary>;
  pageQueryOptions: (
    offset: number,
    limit: number,
  ) => ReturnType<typeof staffSearchQueryOptions>;
  firstPageQueryOptions?: ReturnType<typeof staffSearchQueryOptions>;
  isReplacementActive?: boolean;
  unresolvedFieldIds?: ReadonlySet<string>;
  caption: string;
  testId: string;
  onSortChange: (sort: StaffSortField, dir: StaffSortDir) => void;
  onAddColumn: (id: string) => void;
  onRemoveColumn: (id: string) => void;
  onMoveColumn: (id: string, target: number) => void;
  onResizeColumn: (id: string, width: number) => void;
  onRowActivate?: (staff: StaffSummary) => void;
  shortlist: boolean;
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
      identity={identity}
      onRowActivate={onRowActivate}
      renderHeader={({ identity, columns: tableColumns, fixedColumns }) => (
        <ConfigurableTableHeader
          columns={tableColumns}
          configurable={configurable}
          fixedColumns={fixedColumns}
          groups={shortlist ? STAFF_SHORTLIST_TABLE_GROUPS : STAFF_TABLE_GROUPS}
          identity={identity}
          sortable
          metrics={
            shortlist
              ? STAFF_SHORTLIST_CONFIGURABLE_METRICS
              : STAFF_CONFIGURABLE_METRICS
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
      )}
      pageQueryOptions={pageQueryOptions}
      firstPageQueryOptions={firstPageQueryOptions}
      isReplacementActive={isReplacementActive}
      pageSize={STAFF_PAGE_SIZE}
      renderCells={(staff) =>
        columns.map((column) => {
          // Dynamic fields the replacement query has not delivered yet
          // render as loading, never as truthful-missing.
          const unresolvedDynamic =
            staff !== undefined &&
            (unresolvedFieldIds?.has(column.id) ?? false);
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
                  <TableScoreContent
                    score={score}
                    roleName={`${column.label} role score`}
                    isLoading={staff === undefined || unresolvedDynamic}
                  />
                </td>
              );
            }
            const text = unresolvedDynamic
              ? "…"
              : dynamicCell(staff, column.id);
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
  preferredJobOptions,
  unemployedOnly = false,
  shortlistOnly = false,
  onSortChange,
  onRulesChange,
  onApplyFilters,
  onShortlistOnlyChange,
  onPreferredJobChange,
  onUnemployedOnlyChange,
  onBoostSuccess,
  onRowActivate,
}: {
  activeSnapshotId: number | null;
  scope?: StaffWorkspaceScope;
  sortBy: StaffSortField;
  sortDir: StaffSortDir;
  filters: StaffFilterRule[];
  filterCombine: "and" | "or";
  preferredJob?: string;
  preferredJobOptions?: string[];
  unemployedOnly?: boolean;
  shortlistOnly?: boolean;
  onSortChange: (sort: StaffSortField, dir: StaffSortDir) => void;
  onRulesChange?: (rules: StaffFilterRule[]) => void;
  onApplyFilters?: (rules: StaffFilterRule[], combine: "and" | "or") => void;
  onShortlistOnlyChange?: (shortlistOnly: boolean) => void;
  onPreferredJobChange?: (preferredJob: string) => void;
  onUnemployedOnlyChange?: (unemployedOnly: boolean) => void;
  onBoostSuccess?: () => Promise<void>;
  onRowActivate?: (staff: StaffSummary) => void;
}) {
  const isShortlist = scope === "search" && shortlistOnly;
  const shortlistPresentation = isShortlist
    ? staffShortlistPresentation(preferredJob)
    : undefined;
  const fixedColumnIds = shortlistPresentation
    ? withoutIdentityColumnIds(shortlistPresentation.columnIds)
    : undefined;
  const layoutId: StaffLayoutId =
    scope === "my-staff"
      ? "my-staff"
      : isShortlist
        ? "staff-shortlist"
        : "staff-search";
  const layout = usePlayerTableStore((state) => state.layouts[layoutId]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeColumn = usePlayerTableStore((state) => state.removeColumn);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const identityWidth = usePlayerTableStore(
    (state) => state.layouts[layoutId].identityWidth,
  );
  const setIdentityWidth = usePlayerTableStore(
    (state) => state.setIdentityWidth,
  );
  const identity = useMemo<ConfigurableTableIdentity<StaffSummary>>(
    () => ({
      id: "identity",
      label: "Staff",
      width: identityWidth,
      onResize: (width) => setIdentityWidth(layoutId, width),
      renderCell: (staff) => (
        <StaffIdentityCell
          name={staff?.name ?? undefined}
          club={staff?.club}
          division={staff?.division}
        />
      ),
    }),
    [identityWidth, layoutId, setIdentityWidth],
  );
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
      (fixedColumnIds ?? layout.columnIds).flatMap((id) => {
        const metric = isShortlist
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
    [fixedColumnIds, isShortlist, layout],
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
  const requested: StaffResultsQueryParams = useMemo(
    () => ({
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
      shortlistOnly,
      preferredJob,
      unemployedOnly,
    }),
    [
      filterCombine,
      filters,
      preferredJob,
      requestedFields,
      shortlistOnly,
      sortBy,
      sortDir,
      unemployedOnly,
    ],
  );
  // Query replacement mirrors Search: the URL stays the source of truth
  // while only a fetched page commits, so sort replacement keeps the
  // committed rows and the toolbar mounted instead of suspending them.
  const [committed, setCommitted] = useState(requested);
  const committedOptions = staffResultsPageOptions(
    scope,
    committed,
    0,
    STAFF_PAGE_SIZE,
  );
  const requestedOptions = staffResultsPageOptions(
    scope,
    requested,
    0,
    STAFF_PAGE_SIZE,
  );
  const committedQuery = useQuery(committedOptions);
  const requestedQuery = useQuery(requestedOptions);
  const requestedKey = JSON.stringify(requestedOptions.queryKey);
  const requestedDataUpdateCount =
    queryClient.getQueryState(requestedOptions.queryKey)?.dataUpdateCount ?? 0;
  const requestedVersion = useRef({
    key: requestedKey,
    dataUpdateCount: requestedDataUpdateCount,
  });
  if (requestedVersion.current.key !== requestedKey) {
    requestedVersion.current = {
      key: requestedKey,
      dataUpdateCount: requestedDataUpdateCount,
    };
  }
  const requestMatchesCommitted =
    JSON.stringify(committedOptions.queryKey) ===
    JSON.stringify(requestedOptions.queryKey);
  const isSameListReplacement =
    !requestMatchesCommitted &&
    JSON.stringify({
      filters: committed.filters,
      filterCombine: committed.filterCombine,
      shortlistOnly: committed.shortlistOnly,
      preferredJob: committed.preferredJob,
      unemployedOnly: committed.unemployedOnly,
    }) ===
      JSON.stringify({
        filters: requested.filters,
        filterCombine: requested.filterCombine,
        shortlistOnly: requested.shortlistOnly,
        preferredJob: requested.preferredJob,
        unemployedOnly: requested.unemployedOnly,
      });
  // Staff keeps committed rows for sort and requested-field (column
  // add/remove) changes: the row set is unchanged, so the mounted table
  // applies column updates live instead of unmounting. Sort changes report
  // the pending/error replacement status; requested-field changes keep
  // their unresolved dynamic cells as loading placeholders with their own
  // error/retry state so unfetched values never read as truthful-missing.
  const isSortChanged =
    committed.sortBy !== requested.sortBy ||
    committed.sortDir !== requested.sortDir;
  const fieldsMatch =
    JSON.stringify(committed.requestedFields) ===
    JSON.stringify(requested.requestedFields);
  const isReplacementActive = !requestMatchesCommitted;
  const isReplacementPending =
    isSameListReplacement && isSortChanged && requestedQuery.isFetching;
  const replacementError =
    isSameListReplacement && isSortChanged && requestedQuery.isError
      ? requestedQuery.error
      : null;
  const isFieldReplacement = isSameListReplacement && !fieldsMatch;
  const unresolvedFieldIds = isFieldReplacement
    ? new Set(
        requested.requestedFields.filter(
          (field) => !committed.requestedFields.includes(field),
        ),
      )
    : new Set<string>();
  const fieldReplacementError =
    isFieldReplacement && requestedQuery.isError ? requestedQuery.error : null;

  useEffect(() => {
    if (
      requestedQuery.isSuccess &&
      !requestedQuery.isFetching &&
      (!requestedQuery.isStale ||
        requestedDataUpdateCount > requestedVersion.current.dataUpdateCount) &&
      !requestMatchesCommitted
    ) {
      setCommitted(requested);
    }
  }, [
    requestMatchesCommitted,
    requested,
    requestedDataUpdateCount,
    requestedQuery.isFetching,
    requestedQuery.isStale,
    requestedQuery.isSuccess,
  ]);

  const listKey = useMemo(
    () =>
      JSON.stringify([
        scope,
        committed.shortlistOnly,
        committed.preferredJob ?? "",
        committed.unemployedOnly,
        committed.filterCombine,
        committed.filters,
      ]),
    [committed, scope],
  );
  const page =
    requestMatchesCommitted || isSameListReplacement
      ? committedQuery.data
      : undefined;
  const removeStoredColumn = (metricId: string) => {
    if (isShortlist && fixedColumnIds) return;
    if (!columns.some((column) => column.id === metricId)) return;
    removeColumn(layoutId, metricId);
    if (sortBy === metricId) {
      const next = columns.find((column) => column.id !== metricId);
      if (next) {
        onSortChange(next.id, defaultDirForStaffSortField(next.id));
      }
    }
  };
  const columnsControl = (
    <ConfigurableColumnsControl
      groups={isShortlist ? STAFF_SHORTLIST_TABLE_GROUPS : STAFF_TABLE_GROUPS}
      metrics={
        isShortlist
          ? STAFF_SHORTLIST_CONFIGURABLE_METRICS
          : STAFF_CONFIGURABLE_METRICS
      }
      visibleColumnIds={fixedColumnIds ?? layout.columnIds}
      configurable={!fixedColumnIds}
      onAddColumn={(id) => {
        if (!fixedColumnIds) addColumns(layoutId, [id]);
      }}
      onRemoveColumn={removeStoredColumn}
    />
  );
  // Dataset toggles owned by the toolbar dataset slot: the Shortlist switch
  // is always offered on Staff Search, while Preferred Job and Only
  // unemployed appear only while shortlisting. Page actions (Upload,
  // Configure, Optimize) and the My Staff boost stay outside the toolbar.
  const datasetToggles =
    scope === "search" ? (
      <>
        <ShortlistSwitch
          checked={shortlistOnly === true}
          onChange={onShortlistOnlyChange}
        />
        {isShortlist ? (
          <>
            <label className="flex items-center gap-2 text-body-md text-on-surface">
              Preferred Job
              <select
                className="rounded-md border border-outline bg-surface px-2 py-1 text-on-surface"
                value={preferredJob ?? ""}
                onChange={(event) => onPreferredJobChange?.(event.target.value)}
              >
                <option value="">All jobs</option>
                {(preferredJobOptions ?? []).map((job) => (
                  <option key={job} value={job}>
                    {job}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex items-center gap-2 text-body-md text-on-surface">
              <input
                type="checkbox"
                checked={unemployedOnly === true}
                onChange={(event) =>
                  onUnemployedOnlyChange?.(event.target.checked)
                }
              />
              Only unemployed
            </label>
          </>
        ) : null}
      </>
    ) : null;
  // My Staff offers no filter surface and passes no filter callbacks:
  // summary plus grouped Columns only. Staff Search and both Staff
  // Shortlist paths share the filter toolbar.
  const renderToolbar = (summary?: ReactNode) =>
    scope !== "my-staff" && onRulesChange && onApplyFilters ? (
      <StaffFilterBar
        rules={filters}
        combine={filterCombine}
        onRulesChange={onRulesChange}
        onApply={onApplyFilters}
        summary={summary}
        columnsControl={columnsControl}
        datasetToggles={datasetToggles}
      />
    ) : (
      <TableToolbar
        toolbarLabel="Staff results toolbar"
        summary={summary}
        columnsControl={columnsControl}
        datasetToggles={datasetToggles}
      />
    );

  if (!page) {
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
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar()}
        <EmptyState
          icon={SearchX}
          title={
            requestedQuery.isError ? "Could not load staff" : "Loading staff"
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
            : "Loading staff results…"}
        </EmptyState>
      </Panel>
    );
  }

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
      <Panel
        title="Staff Shortlist"
        flush
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar()}
        <EmptyState icon={UsersRound} title="No Staff Shortlist uploaded">
          Upload a staff CSV to view the people included in it.
        </EmptyState>
      </Panel>
    );
  }
  const sortMetric = isShortlist
    ? getStaffShortlistMetric(committed.sortBy)
    : getStaffMetric(committed.sortBy);
  const sortLabel = sortMetric?.label ?? committed.sortBy;
  if (page.total === 0) {
    const hasShortlistFilter =
      Boolean(preferredJob) ||
      unemployedOnly ||
      completeStaffFilterRules(filters).length > 0;
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
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar(
          <p className="text-body-md text-on-surface-variant">
            <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
            staff · sorted by {sortLabel} (
            {committed.sortDir === "asc" ? "ascending" : "descending"})
          </p>,
        )}
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
                ? "Adjust or clear filters to widen the results."
                : "Load Data to restore saved shortlist people who are absent from the current snapshot."
              : completeStaffFilterRules(filters).length > 0
                ? "Adjust or clear filters to widen the result set."
                : "The snapshot exists but contains no staff rows."}
        </EmptyState>
      </Panel>
    );
  }

  const requestedRoleFields = committed.requestedFields.filter((field) =>
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
  return (
    <Panel
      title={
        scope === "my-staff"
          ? "Staff"
          : isShortlist
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
      {renderToolbar(
        <p className="text-body-md text-on-surface-variant">
          <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
          staff · sorted by {sortLabel} (
          {committed.sortDir === "asc" ? "ascending" : "descending"})
        </p>,
      )}
      {isReplacementPending ? (
        <p
          className="shrink-0 px-4 pb-3 text-body-sm text-on-surface-variant"
          role="status"
        >
          Sorting…
        </p>
      ) : null}
      {replacementError ? (
        <div
          className="flex shrink-0 items-center justify-between gap-3 px-4 pb-3 text-body-sm text-error"
          role="alert"
        >
          <span>Could not sort staff. {replacementError.message}</span>
          <button
            type="button"
            className="shrink-0 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => void requestedQuery.refetch()}
          >
            Retry
          </button>
        </div>
      ) : null}
      {fieldReplacementError ? (
        <div
          className="flex shrink-0 items-center justify-between gap-3 px-4 pb-3 text-body-sm text-error"
          role="alert"
        >
          <span>
            Could not load staff columns. {fieldReplacementError.message}
          </span>
          <button
            type="button"
            className="shrink-0 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => void requestedQuery.refetch()}
          >
            Retry
          </button>
        </div>
      ) : null}
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
        key={listKey}
        total={page.total}
        sortBy={committed.sortBy}
        sortDir={committed.sortDir}
        columns={columns}
        identity={identity}
        pageQueryOptions={(offset, limit) =>
          staffResultsPageOptions(scope, committed, offset, limit)
        }
        firstPageQueryOptions={committedOptions}
        isReplacementActive={isReplacementActive}
        unresolvedFieldIds={unresolvedFieldIds}
        caption={
          scope === "my-staff"
            ? "Staff overview"
            : isShortlist
              ? "Staff Shortlist"
              : "Staff search results"
        }
        testId={`${layoutId}-results-scroller`}
        onSortChange={onSortChange}
        onAddColumn={(id) => {
          if (!fixedColumnIds) addColumns(layoutId, [id]);
        }}
        onRemoveColumn={removeStoredColumn}
        onMoveColumn={(id, target) => {
          if (!fixedColumnIds) moveColumn(layoutId, id, target);
        }}
        onResizeColumn={(id, width) => {
          if (!fixedColumnIds) setColumnWidth(layoutId, id, width);
        }}
        onRowActivate={onRowActivate}
        shortlist={isShortlist}
        configurable={!fixedColumnIds}
      />
    </Panel>
  );
}
