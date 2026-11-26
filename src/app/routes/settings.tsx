import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import { clearPlayerResultContext } from "@/app/player-result-context";
import { SelectField } from "@/components/ui/field/select-field";
import { LoadingRegion, Skeleton } from "@/components/ui/skeleton/skeleton";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { graphicsStatusQueryOptions } from "@/features/graphics/api/graphics-query-options";
import { GraphicsSettingsSectionWithErrorBoundary } from "@/features/graphics/components/graphics-settings-section-with-error-boundary";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { ManagedClubBoost } from "@/features/managed-club-boost/components/managed-club-boost";
import { bridgeInstallQueryOptions } from "@/features/memory-read/api/bridge-install-query-options";
import { bridgeStatusQueryOptions } from "@/features/memory-read/api/bridge-status-query-options";
import { BridgeStatusPanelWithErrorBoundary } from "@/features/memory-read/components/bridge-status-panel-with-error-boundary";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { snapshotMetadataQueryOptions } from "@/features/snapshot/api/snapshot-metadata-query-options";
import { SnapshotPanelsWithErrorBoundary } from "@/features/snapshot/components/snapshot-panels-with-error-boundary";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { warmQuery } from "@/lib/warm-query";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import thirdPartyNotices from "../../../THIRD_PARTY_NOTICES.md?raw";

export const Route = createFileRoute("/settings")({
  beforeLoad: ({ location }) => {
    if (location.hash === "managed-club") {
      throw Route.redirect({
        to: "/my-club",
        hash: "managed-club",
        replace: true,
      });
    }
  },
  loader: async ({ context: { queryClient } }) => {
    // The bridge and graphics sections show placeholders while these load.
    warmQuery(queryClient, bridgeInstallQueryOptions);
    warmQuery(queryClient, bridgeStatusQueryOptions);
    warmQuery(queryClient, graphicsStatusQueryOptions);
    // Each section reports its own failure, so a failed read never fails the
    // page.
    const [saves] = await Promise.all([
      queryClient.ensureQueryData(savesQueryOptions).catch(() => undefined),
      queryClient
        .ensureQueryData(currentSnapshotQueryOptions)
        .catch(() => undefined),
    ]);
    const activeSave = saves?.find((save) => save.isActive);
    await queryClient
      .ensureQueryData(snapshotMetadataQueryOptions(activeSave?.id ?? 0))
      .catch(() => undefined);
  },
  component: SettingsPage,
});

/**
 * Stands in for a settings status panel: an optional title or header action,
 * status chips, two lines of text, and optional buttons.
 */
function StatusPanelSkeleton({
  header,
  buttons = false,
}: {
  header?: "title" | "action";
  buttons?: boolean;
}) {
  return (
    <div className="overflow-hidden rounded-lg border border-outline-variant bg-surface-container">
      {header === "title" ? (
        <div className="px-4 pt-4">
          <Skeleton className="h-5 w-40" />
        </div>
      ) : header === "action" ? (
        <div className="flex justify-end px-4 pt-4">
          <Skeleton className="size-8" />
        </div>
      ) : null}
      <div className="p-4">
        <div className="flex gap-2">
          <Skeleton className="h-5 w-28 rounded-full" />
          <Skeleton className="h-5 w-32 rounded-full" />
        </div>
        <Skeleton className="mt-3 h-4.5 w-full max-w-prose" />
        <Skeleton className="mt-2 h-4.5 w-64" />
        {buttons ? (
          <div className="mt-4 flex gap-2">
            <Skeleton className="h-8 w-44" />
            <Skeleton className="h-8 w-36" />
          </div>
        ) : null}
      </div>
    </div>
  );
}

function SectionFallback({ label }: { label: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      {label}
    </div>
  );
}

function SettingsPage() {
  const queryClient = useQueryClient();
  const snapshotQuery = useQuery(currentSnapshotQueryOptions);
  const savesQuery = useQuery(savesQueryOptions);
  const activeSave = savesQuery.data?.find((save) => save.isActive);
  const boostContextKey = `${activeSave?.contextToken ?? "no-save"}:${snapshotQuery.data?.contextToken ?? "no-snapshot"}`;
  const boostContextPending = snapshotQuery.isPending || savesQuery.isPending;
  const boostContextError = snapshotQuery.isError || savesQuery.isError;
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const setDefaultAnalysisView = useMoneyballPreferences(
    (state) => state.setDefaultAnalysisView,
  );
  const invalidateCurrentContext = () => {
    void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
    void queryClient.invalidateQueries({ queryKey: searchKeys.all });
    void queryClient.invalidateQueries({ queryKey: playerKeys.all });
    void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
    void queryClient.invalidateQueries({ queryKey: clubDnaKeys.all });
    void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
    void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    void queryClient.resetQueries({ queryKey: academyKeys.all });
    void queryClient.invalidateQueries({ queryKey: staffKeys.all });
  };

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-md text-on-surface">Settings</h1>

      <section
        aria-labelledby="preferences-heading"
        className="w-full max-w-2xl space-y-4"
      >
        <h2
          className="text-headline-sm text-on-surface"
          id="preferences-heading"
        >
          Preferences
        </h2>
        <SelectField
          className="max-w-md"
          label="Default player analysis view"
          value={defaultAnalysisView}
          onChange={(event) =>
            setDefaultAnalysisView(
              event.target.value === "moneyball" ? "moneyball" : "general",
            )
          }
        >
          <option value="general">General</option>
          <option value="moneyball">Moneyball</option>
        </SelectField>
      </section>

      <section
        aria-labelledby="all-boosts-heading"
        className="w-full max-w-2xl space-y-4"
      >
        <h2
          className="text-headline-sm text-on-surface"
          id="all-boosts-heading"
        >
          All boosts
        </h2>
        {boostContextPending ? (
          <SectionFallback label="Loading boost context…" />
        ) : boostContextError ? (
          <div
            className="flex min-h-24 items-center rounded-lg border border-outline-variant bg-surface-container p-4 text-body-md text-error"
            role="alert"
          >
            Could not load boost context
          </div>
        ) : (
          <ManagedClubBoost
            key={boostContextKey}
            contextKey={boostContextKey}
            onSettled={invalidateCurrentContext}
          />
        )}
      </section>

      <section
        aria-labelledby="graphics-heading"
        className="w-full max-w-2xl space-y-4"
      >
        <h2 className="text-headline-sm text-on-surface" id="graphics-heading">
          Graphics
        </h2>
        <Suspense
          fallback={
            <LoadingRegion label="Loading graphics status…">
              <StatusPanelSkeleton buttons />
            </LoadingRegion>
          }
        >
          <GraphicsSettingsSectionWithErrorBoundary />
        </Suspense>
      </section>

      <section aria-labelledby="save-data-heading" className="space-y-4">
        <h2 className="text-headline-sm text-on-surface" id="save-data-heading">
          Save data
        </h2>
        <Suspense fallback={<SectionFallback label="Loading save data…" />}>
          <SnapshotPanelsWithErrorBoundary
            onBeforeContextChange={() => clearPlayerResultContext(queryClient)}
            onCurrentContextChanged={invalidateCurrentContext}
          />
        </Suspense>
      </section>

      <section aria-labelledby="bridge-heading" className="space-y-4">
        <h2 className="text-headline-sm text-on-surface" id="bridge-heading">
          Bridge
        </h2>
        <Suspense
          fallback={
            <LoadingRegion
              label="Loading bridge status…"
              className="space-y-gutter"
            >
              <StatusPanelSkeleton header="title" buttons />
              <StatusPanelSkeleton header="action" />
            </LoadingRegion>
          }
        >
          <BridgeStatusPanelWithErrorBoundary />
        </Suspense>
      </section>

      <section
        aria-labelledby="open-source-notices-heading"
        className="w-full max-w-2xl space-y-4"
      >
        <h2
          className="text-headline-sm text-on-surface"
          id="open-source-notices-heading"
        >
          Open-source notices
        </h2>
        <pre className="whitespace-pre-wrap rounded-lg border border-outline-variant bg-surface-container p-4 text-body-sm text-on-surface-variant">
          {thirdPartyNotices}
        </pre>
      </section>
    </div>
  );
}
