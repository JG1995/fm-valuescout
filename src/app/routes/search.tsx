import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { searchPlayersQueryOptions } from "@/features/search/api/search-players-query-options";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

export const Route = createFileRoute("/search")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(searchPlayersQueryOptions(0)),
    ]),
  component: SearchPage,
});

function PanelFallback() {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      Loading search results…
    </div>
  );
}

function SearchPageBody() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);

  if (!snapshot) {
    return (
      <Panel title="Results" flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest players into the database.
        </EmptyState>
      </Panel>
    );
  }

  return <SearchResultsPanel />;
}

function SearchPage() {
  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Search</h1>
      <Suspense fallback={<PanelFallback />}>
        <SearchPageBody />
      </Suspense>
    </div>
  );
}
