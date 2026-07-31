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

function ProfileFallback() {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      Loading player…
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
    <Suspense fallback={<ProfileFallback />}>
      <PlayerProfileContent uid={uid} tab={tab} onTabChange={onTabChange} />
    </Suspense>
  );
}
