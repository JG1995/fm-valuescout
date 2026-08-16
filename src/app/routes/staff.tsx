import { createFileRoute } from "@tanstack/react-router";
import { UsersRound } from "lucide-react";
import { Suspense, useMemo } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { staffSearchQueryOptions } from "@/features/staff/api/staff-query-options";
import { StaffFilterBar } from "@/features/staff/components/staff-filter-bar";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
import {
  StaffWorkspaceTabs,
  staffWorkspacePanelProps,
} from "@/features/staff/components/staff-workspace-tabs";
import type { StaffFilterRule } from "@/features/staff/types/staff-filter-rule";
import type {
  StaffSortDir,
  StaffSortField,
} from "@/features/staff/types/staff-sort";
import {
  DEFAULT_STAFF_SORT_DIR,
  DEFAULT_STAFF_SORT_FIELD,
  defaultDirForStaffSortField,
  isStaffSortDir,
  isStaffSortField,
} from "@/features/staff/types/staff-sort";
import {
  parseStaffCombine,
  parseStaffFilters,
  parseStaffView,
  staffFiltersForUrl,
} from "@/features/staff/utils/staff-url-search";
import { usePlayerTableStore } from "@/stores/use-player-table-store";

export type StaffSearch = {
  view: "search" | "my-staff";
  sort: StaffSortField;
  dir: StaffSortDir;
  filters: ReturnType<typeof staffFiltersForUrl>;
  combine: "and" | "or";
};

export const Route = createFileRoute("/staff")({
  validateSearch: (search: Record<string, unknown>): StaffSearch => {
    const sort = isStaffSortField(search.sort)
      ? search.sort
      : DEFAULT_STAFF_SORT_FIELD;
    const dir = isStaffSortDir(search.dir)
      ? search.dir
      : isStaffSortField(search.sort)
        ? defaultDirForStaffSortField(sort)
        : DEFAULT_STAFF_SORT_DIR;
    const filters = parseStaffFilters(search.filters);
    return {
      view: parseStaffView(search.view),
      sort,
      dir,
      filters: staffFiltersForUrl(filters),
      combine: parseStaffCombine(search.combine),
    };
  },
  loaderDeps: ({ search: { sort, dir, filters, combine } }) => ({
    sort,
    dir,
    filters,
    combine,
  }),
  loader: ({
    context: { queryClient },
    deps: { sort, dir, filters, combine },
  }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(
        staffSearchQueryOptions(
          0,
          undefined,
          sort,
          dir,
          parseStaffFilters(filters),
          combine,
          [],
        ),
      ),
    ]),
  component: StaffPage,
});

function StaffFallback() {
  return (
    <div
      className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant"
      aria-busy="true"
    >
      Loading staff…
    </div>
  );
}

function MyStaffNextPanel() {
  return (
    <Panel title="My Staff" flush>
      <EmptyState icon={UsersRound} title="My Staff overview is coming next">
        The configured club-family staff overview will be available in the next
        Staff workspace update.
      </EmptyState>
    </Panel>
  );
}

function StaffPageContent() {
  const { view, sort, dir, filters: filterUrls, combine } = Route.useSearch();
  const navigate = Route.useNavigate();
  const filters = useMemo(() => parseStaffFilters(filterUrls), [filterUrls]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);

  const updateSearch = (
    patch: Partial<{
      view: "search" | "my-staff";
      sort: StaffSortField;
      dir: StaffSortDir;
      filters: StaffFilterRule[];
      combine: "and" | "or";
    }>,
  ) =>
    navigate({
      search: (previous) => ({
        view: patch.view ?? previous.view,
        sort: patch.sort ?? previous.sort,
        dir: patch.dir ?? previous.dir,
        filters:
          patch.filters !== undefined
            ? staffFiltersForUrl(patch.filters)
            : previous.filters,
        combine: patch.combine ?? previous.combine,
      }),
      replace: true,
    });

  return (
    <>
      <header className="flex flex-col items-start gap-2">
        <h1 className="text-headline-lg text-on-surface">Staff</h1>
        <StaffWorkspaceTabs
          view={view}
          onViewChange={(nextView) => updateSearch({ view: nextView })}
        />
      </header>
      <div {...staffWorkspacePanelProps("search", view)}>
        <div className="flex min-h-0 flex-1 flex-col gap-gutter">
          <StaffFilterBar
            rules={filters}
            combine={combine}
            onRulesChange={(rules) => updateSearch({ filters: rules })}
            onApply={(rules, nextCombine) => {
              void updateSearch({ filters: rules, combine: nextCombine }).then(
                () =>
                  addColumns(
                    "staff-search",
                    rules.map((rule) => rule.field),
                  ),
              );
            }}
          />
          <div className="flex min-h-0 flex-1 flex-col">
            <Suspense fallback={<StaffFallback />}>
              <StaffSearchResultsPanel
                sortBy={sort}
                sortDir={dir}
                filters={filters}
                filterCombine={combine}
                onSortChange={(nextSort, nextDir) =>
                  updateSearch({ sort: nextSort, dir: nextDir })
                }
              />
            </Suspense>
          </div>
        </div>
      </div>
      <div {...staffWorkspacePanelProps("my-staff", view)}>
        <MyStaffNextPanel />
      </div>
    </>
  );
}

function StaffPage() {
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <Suspense fallback={<StaffFallback />}>
        <StaffPageContent />
      </Suspense>
    </div>
  );
}
