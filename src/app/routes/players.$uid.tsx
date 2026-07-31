import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, UserX } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { getPlayerQueryOptions } from "@/features/player-profile/api/get-player-query-options";
import { PlayerAttributesPanel } from "@/features/player-profile/components/player-attributes-panel";
import { PlayerOverviewPanel } from "@/features/player-profile/components/player-overview-panel";
import {
  PlayerProfileTabs,
  profileTabPanelProps,
} from "@/features/player-profile/components/player-profile-tabs";
import { PlayerRolesPanel } from "@/features/player-profile/components/player-roles-panel";
import {
  type ProfileTab,
  parseProfileTab,
} from "@/features/player-profile/utils/profile-tab";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { cn } from "@/utils/cn";

export type PlayerProfileSearch = {
  tab: ProfileTab;
};

function parseUid(raw: string): number | null {
  const uid = Number(raw);
  return Number.isInteger(uid) ? uid : null;
}

export const Route = createFileRoute("/players/$uid")({
  validateSearch: (search: Record<string, unknown>): PlayerProfileSearch => ({
    tab: parseProfileTab(search.tab),
  }),
  loader: ({ context: { queryClient }, params }) => {
    const uid = parseUid(params.uid);
    if (uid === null) {
      return queryClient.ensureQueryData(currentSnapshotQueryOptions);
    }
    return Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(getPlayerQueryOptions(uid)),
    ]);
  },
  component: PlayerProfileRoute,
});

function SkeletonBar({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "rounded-sm bg-surface-container-high motion-safe:animate-[pulse_1.5s_ease-in-out_infinite]",
        className,
      )}
    />
  );
}

const OVERVIEW_FIELD_SLOTS = [
  "f1",
  "f2",
  "f3",
  "f4",
  "f5",
  "f6",
  "f7",
  "f8",
] as const;
const ATTRIBUTE_SECTION_SLOTS = ["s1", "s2", "s3"] as const;
const ATTRIBUTE_ROW_SLOTS = ["r1", "r2", "r3", "r4", "r5", "r6"] as const;
const ROLE_SECTION_SLOTS = ["s1", "s2", "s3"] as const;
const ROLE_ROW_SLOTS = ["r1", "r2", "r3"] as const;

function OverviewSkeleton() {
  return (
    <Panel title="Overview">
      <div className="space-y-6">
        <div className="flex items-center gap-4">
          <SkeletonBar className="size-12 shrink-0 rounded-full" />
          <div className="min-w-0 flex-1 space-y-2">
            <SkeletonBar className="h-3 w-20" />
            <SkeletonBar className="h-4 w-40" />
          </div>
        </div>
        <div className="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-3 lg:grid-cols-4">
          {OVERVIEW_FIELD_SLOTS.map((slot) => (
            <div key={slot} className="space-y-2">
              <SkeletonBar className="h-3 w-16" />
              <SkeletonBar className="h-4 w-full" />
            </div>
          ))}
        </div>
      </div>
    </Panel>
  );
}

function AttributesSkeleton() {
  return (
    <Panel title="Attributes">
      <div className="space-y-6">
        {ATTRIBUTE_SECTION_SLOTS.map((section) => (
          <div key={section} className="space-y-3">
            <SkeletonBar className="h-4 w-28" />
            <div className="grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2 lg:grid-cols-3">
              {ATTRIBUTE_ROW_SLOTS.map((row) => (
                <div
                  key={`${section}-${row}`}
                  className="flex items-baseline justify-between gap-3"
                >
                  <SkeletonBar className="h-4 w-24" />
                  <SkeletonBar className="h-4 w-8" />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Panel>
  );
}

function RolesSkeleton() {
  return (
    <Panel title="Roles">
      <div className="space-y-6">
        {ROLE_SECTION_SLOTS.map((section) => (
          <div key={section} className="space-y-3">
            <SkeletonBar className="h-4 w-36" />
            <ul className="space-y-2">
              {ROLE_ROW_SLOTS.map((row) => (
                <li
                  key={`${section}-${row}`}
                  className="flex min-w-0 items-center justify-between gap-3"
                >
                  <div className="min-w-0 flex-1 space-y-2">
                    <SkeletonBar className="h-4 w-40" />
                    <SkeletonBar className="h-3 w-10" />
                  </div>
                  <SkeletonBar className="size-7 shrink-0 rounded-full" />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </Panel>
  );
}

function ProfileFallback({ tab }: { tab: ProfileTab }) {
  return (
    <div
      className="space-y-gutter"
      aria-busy="true"
      aria-live="polite"
      data-testid="profile-loading"
    >
      <SkeletonBar className="h-8 w-48" />
      <div
        className="inline-flex h-9 w-72 rounded-full bg-surface-container-high"
        aria-hidden
      />
      {tab === "overview" ? <OverviewSkeleton /> : null}
      {tab === "attributes" ? <AttributesSkeleton /> : null}
      {tab === "roles" ? <RolesSkeleton /> : null}
      <p className="sr-only">Loading player…</p>
      <p className="hidden text-body-md text-on-surface-variant motion-reduce:block">
        Loading…
      </p>
    </div>
  );
}

function PlayerNotFound() {
  return (
    <Panel>
      <EmptyState icon={UserX} title="Player not in this snapshot">
        This player is not in the active save’s current snapshot. Return to
        Search or load a fresher snapshot.
      </EmptyState>
    </Panel>
  );
}

function PlayerProfileContent({
  uid,
  tab,
  onTabChange,
}: {
  uid: number;
  tab: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
}) {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: player } = useSuspenseQuery(getPlayerQueryOptions(uid));

  if (!snapshot) {
    return (
      <Panel>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest players into the database.
        </EmptyState>
      </Panel>
    );
  }

  if (!player) {
    return <PlayerNotFound />;
  }

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">{player.name}</h1>
      <PlayerProfileTabs tab={tab} onTabChange={onTabChange} />
      <div {...profileTabPanelProps("overview", tab)}>
        {tab === "overview" ? <PlayerOverviewPanel player={player} /> : null}
      </div>
      <div {...profileTabPanelProps("attributes", tab)}>
        {tab === "attributes" ? (
          <PlayerAttributesPanel player={player} />
        ) : null}
      </div>
      <div {...profileTabPanelProps("roles", tab)}>
        {tab === "roles" ? <PlayerRolesPanel player={player} /> : null}
      </div>
    </div>
  );
}

function PlayerProfileRoute() {
  const { uid: uidParam } = Route.useParams();
  const { tab } = Route.useSearch();
  const navigate = Route.useNavigate();
  const uid = parseUid(uidParam);

  const onTabChange = (next: ProfileTab) => {
    void navigate({
      search: (previous) => ({ ...previous, tab: next }),
      replace: true,
    });
  };

  if (uid === null) {
    return <PlayerNotFound />;
  }

  return (
    <Suspense fallback={<ProfileFallback tab={tab} />}>
      <PlayerProfileContent uid={uid} tab={tab} onTabChange={onTabChange} />
    </Suspense>
  );
}
