import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import {
  ManagedClubLogo,
  ManagedClubSetup,
} from "@/app/components/managed-club-setup";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  type DashboardSetupNote,
  DashboardSetupNotes,
} from "@/features/dashboard/components/dashboard-setup-notes";
import { DevelopmentProspectsPanel } from "@/features/dashboard/components/development-prospects-panel";
import { ExpiringContractsPanel } from "@/features/dashboard/components/expiring-contracts-panel";
import { SigningsWorkspace } from "@/features/dashboard/components/signings-workspace";
import type {
  DashboardContext,
  DashboardScoreMode,
} from "@/features/dashboard/types/dashboard";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "@/features/managed-club/api/managed-club-query-options";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import type { SaveSummary } from "@/features/snapshot/types/save";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { shortlistOptionLabels } from "@/features/snapshot/utils/shortlist-option-labels";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  formatAbsoluteUtc,
  formatLongGameDate,
  formatRelativeAge,
} from "@/utils/format";

type DashboardSearch = {
  /** The score mode; absent uses the configured default analysis view. */
  mode?: DashboardScoreMode;
};

export const Route = createFileRoute("/")({
  validateSearch: (search: Record<string, unknown>): DashboardSearch =>
    search.mode === "general" || search.mode === "moneyball"
      ? { mode: search.mode }
      : {},
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(managedClubQueryOptions),
      queryClient.prefetchQuery(managedClubOptionsQueryOptions),
    ]),
  component: DashboardPage,
});

const setupLinkClass =
  "inline-flex h-8 shrink-0 items-center rounded-md border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high active:bg-surface-container-highest";

/**
 * What the active save still needs before Moneyball mode has data. A missing
 * shortlist is not listed: Recommended signings says so where its rows would be.
 */
function setupNotes(
  save: SaveSummary,
  snapshot: SnapshotSummary,
): DashboardSetupNote[] {
  const notes: DashboardSetupNote[] = [];
  if (save.linkedSaveFileName === null) {
    notes.push({
      id: "save-file",
      text: "Link your FM save file to read season stats for Moneyball mode and the shortlists you keep in FM.",
      action: (
        <Link to="/settings" className={setupLinkClass}>
          Open Settings
        </Link>
      ),
    });
  } else if (snapshot.moneyballStatsGameDate === null) {
    notes.push({
      id: "season-stats",
      text: "This snapshot has no season stats yet. Use Load Data to read them from your linked FM save.",
    });
  }
  return notes;
}

function Fact({
  label,
  value,
  title,
}: {
  label: string;
  value: string;
  title?: string;
}) {
  return (
    <div>
      <dt className="text-label-sm text-on-surface-variant uppercase">
        {label}
      </dt>
      <dd className="text-body-sm text-on-surface" title={title}>
        {value}
      </dd>
    </div>
  );
}

function DashboardPage() {
  const search = Route.useSearch();
  const navigate = Route.useNavigate();
  const defaultMode = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const mode = search.mode ?? defaultMode;
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: managedClub } = useSuspenseQuery(managedClubQueryOptions);
  const { data: saves } = useQuery(savesQueryOptions);
  const activeSave = saves?.find((save) => save.isActive);
  // The snapshot and the save list refresh separately, so the panels wait
  // until both describe the same save.
  const save = activeSave?.id === snapshot?.saveId ? activeSave : undefined;
  const context: DashboardContext | null =
    snapshot && save && managedClub.clubName !== null
      ? {
          saveId: save.id,
          contextToken: save.contextToken,
          snapshotId: snapshot.id,
          clubName: managedClub.clubName,
        }
      : null;

  const header = (
    <header className="flex flex-wrap items-center justify-between gap-x-6 gap-y-2">
      <div className="flex min-w-0 flex-wrap items-center gap-4">
        <h1 className="text-headline-md text-on-surface">Dashboard</h1>
        {managedClub.clubName ? (
          <p className="flex min-w-0 items-center gap-2 text-body-sm text-on-surface-variant">
            <ManagedClubLogo
              clubUid={managedClub.clubUid}
              managedClubStatus={managedClub.status}
            />
            <span className="break-words">
              Managed club: {managedClub.clubName}
            </span>
          </p>
        ) : null}
      </div>
      {snapshot ? (
        <dl className="flex flex-wrap gap-x-6 gap-y-1">
          <Fact
            label="Snapshot loaded"
            value={formatRelativeAge(snapshot.loadedAtUtc)}
            title={formatAbsoluteUtc(snapshot.loadedAtUtc)}
          />
          <Fact
            label="Season stats"
            value={
              snapshot.moneyballStatsGameDate === null
                ? "Not loaded"
                : `As of ${formatLongGameDate(snapshot.moneyballStatsGameDate)}`
            }
          />
          {save ? (
            <Fact
              label="Shortlists"
              value={String(save.playerShortlists.length)}
            />
          ) : null}
        </dl>
      ) : null}
    </header>
  );

  if (!snapshot) {
    return (
      <div className="space-y-gutter">
        {header}
        <Panel flush>
          <EmptyState icon={DatabaseZap} title="No data loaded for this save">
            No snapshot loaded for the active save. Use Load Data to scan
            Football Manager and ingest players before reviewing your club.
          </EmptyState>
        </Panel>
      </div>
    );
  }

  if (managedClub.status !== "available") {
    return (
      <div className="space-y-gutter">
        {header}
        <Panel
          title={
            managedClub.status === "missing"
              ? "Your club is not in this snapshot"
              : "Choose your club"
          }
        >
          <p className="max-w-prose pb-4 text-body-md text-on-surface-variant">
            {managedClub.status === "missing"
              ? "The dashboard needs your club's players in the current snapshot. Load data from the save where you manage this club, or choose another club."
              : "The dashboard compares your squad with shortlisted players, so it needs to know which club you manage. Squad, Planner, Tactic, Youth, and My Staff use the same choice, and you can change it later in My Club."}
          </p>
          <ManagedClubSetup />
        </Panel>
      </div>
    );
  }

  if (!context || !save) {
    return (
      <div className="space-y-gutter">
        {header}
        <p role="status" className="text-body-md text-on-surface-variant">
          Loading dashboard…
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-gutter">
      {header}
      <DashboardSetupNotes notes={setupNotes(save, snapshot)} />
      <SigningsWorkspace
        key={`${context.saveId}:${context.contextToken}:${context.snapshotId}:${context.clubName}`}
        context={context}
        mode={mode}
        onModeChange={(nextMode) =>
          void navigate({ search: { mode: nextMode }, replace: true })
        }
        shortlists={shortlistOptionLabels(save.playerShortlists)}
      />
      <div className="grid grid-cols-2 gap-4">
        <ExpiringContractsPanel context={context} />
        <DevelopmentProspectsPanel context={context} />
      </div>
    </div>
  );
}
