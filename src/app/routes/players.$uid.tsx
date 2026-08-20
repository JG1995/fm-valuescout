import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, UserX } from "lucide-react";
import { type KeyboardEvent, Suspense, useEffect, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { getPlayerMoneyballQueryOptions } from "@/features/moneyball/api/get-player-moneyball-query-options";
import { MoneyballProfilePanel } from "@/features/moneyball/components/moneyball-profile-panel";
import { MoneyballRoleFitPanel } from "@/features/moneyball/components/moneyball-role-fit-panel";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { boostCurrentAbility } from "@/features/player-profile/api/boost-current-ability";
import { boostWonderkidMentality } from "@/features/player-profile/api/boost-wonderkid-mentality";
import { getPlayerQueryOptions } from "@/features/player-profile/api/get-player-query-options";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { setHiddenInformationRevealed } from "@/features/player-profile/api/set-hidden-information-revealed";
import { PlayerAttributesPanel } from "@/features/player-profile/components/player-attributes-panel";
import { PlayerDevelopmentActions } from "@/features/player-profile/components/player-development-boosts-panel";
import { PlayerOverviewPanel } from "@/features/player-profile/components/player-overview-panel";
import { PlayerRolesPanel } from "@/features/player-profile/components/player-roles-panel";
import type { PlayerDetail } from "@/features/player-profile/types/player-detail";
import { isGoalkeeper } from "@/features/player-profile/utils/position-families";
import {
  defaultProfileTab,
  type ProfileTab,
  parseProfileTab,
} from "@/features/player-profile/utils/profile-tab";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { useLayoutStore } from "@/stores/use-layout-store";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import { cn } from "@/utils/cn";

export type PlayerProfileSearch = {
  tab?: ProfileTab;
  view?: PlayerProfileView;
};

type PlayerProfileView = "general" | "moneyball";

const PLAYER_PROFILE_VIEWS = ["general", "moneyball"] as const;

type PlayerBoostAction = "currentAbility" | "wonderkidMentality";

type PlayerBoostMutation = {
  action: PlayerBoostAction;
  uid: number;
  snapshotId: number;
};

type PlayerHiddenInformationMutation = {
  saveId: number;
  uid: number;
  revealed: boolean;
};

function parseUid(raw: string): number | null {
  const uid = Number(raw);
  return Number.isInteger(uid) ? uid : null;
}

function parsePlayerProfileView(value: unknown): PlayerProfileView | undefined {
  if (value === "moneyball" || value === "general") return value;
  return undefined;
}

export const Route = createFileRoute("/players/$uid")({
  validateSearch: (search: Record<string, unknown>): PlayerProfileSearch => ({
    tab: parseProfileTab(search.tab),
    view: parsePlayerProfileView(search.view),
  }),
  loaderDeps: ({ search }) => ({
    view: search.view ?? useMoneyballPreferences.getState().defaultAnalysisView,
  }),
  loader: ({ context: { queryClient }, params, deps: { view } }) => {
    const uid = parseUid(params.uid);
    if (uid === null) {
      return queryClient.ensureQueryData(currentSnapshotQueryOptions);
    }
    const queries: Promise<unknown>[] = [
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(getPlayerQueryOptions(uid)),
    ];
    if (view === "moneyball") {
      queries.push(
        queryClient.ensureQueryData(getPlayerMoneyballQueryOptions(uid)),
      );
    }
    return Promise.all(queries);
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

const SKELETON_SLOTS = ["s1", "s2", "s3", "s4", "s5", "s6"] as const;

function ProfileFallback() {
  const railExpanded = useLayoutStore((state) => state.railExpanded);

  return (
    <div
      className="flex h-full min-h-0 flex-col gap-gutter overflow-hidden"
      aria-busy="true"
      aria-live="polite"
      data-testid="profile-loading"
    >
      <section className="grid gap-4 rounded-lg border border-outline-variant bg-surface-container p-4 lg:grid-cols-3">
        {SKELETON_SLOTS.slice(0, 3).map((slot) => (
          <div key={slot} className="space-y-3">
            <SkeletonBar className="h-6 w-36" />
            <SkeletonBar className="h-4 w-full" />
            <SkeletonBar className="h-4 w-4/5" />
          </div>
        ))}
      </section>
      <div className={profileWorkspaceClassName(railExpanded)}>
        <Panel title="Attributes">
          <SkeletonBar className="mb-4 h-8 w-full rounded-full" />
          <div className="grid grid-cols-1 gap-x-5 sm:grid-cols-3">
            {SKELETON_SLOTS.map((slot) => (
              <div
                key={slot}
                className="flex min-h-9 items-center justify-between"
              >
                <SkeletonBar className="h-4 w-24" />
                <SkeletonBar className="h-6 w-16" />
              </div>
            ))}
          </div>
        </Panel>
        <Panel title="Role fit">
          <div className="grid gap-4 sm:grid-cols-[minmax(180px,0.8fr)_minmax(240px,1.2fr)]">
            <SkeletonBar className="min-h-80 w-full rounded-lg" />
            <div className="space-y-3">
              {SKELETON_SLOTS.map((slot) => (
                <div
                  key={slot}
                  className="flex min-h-12 items-center justify-between gap-3"
                >
                  <SkeletonBar className="h-4 w-36" />
                  <SkeletonBar className="h-7 w-20 rounded-full" />
                </div>
              ))}
            </div>
          </div>
        </Panel>
      </div>
      <p className="sr-only">Loading player…</p>
      <p className="hidden text-body-md text-on-surface-variant motion-reduce:block">
        Loading…
      </p>
    </div>
  );
}

function profileWorkspaceClassName(railExpanded: boolean) {
  return cn(
    "grid h-0 min-h-0 flex-1 gap-gutter [&>*]:min-h-0",
    railExpanded
      ? "grid-cols-1 grid-rows-[minmax(0,1fr)_minmax(0,1fr)] 2xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)] 2xl:grid-rows-[minmax(0,1fr)]"
      : "grid-rows-[minmax(0,1fr)] lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]",
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

function PlayerAnalysisTabs({
  view,
  onViewChange,
  restoreFocus,
  onFocusRestored,
}: {
  view: PlayerProfileView;
  onViewChange: (view: PlayerProfileView, restoreFocus?: boolean) => void;
  restoreFocus: boolean;
  onFocusRestored: () => void;
}) {
  useEffect(() => {
    if (!restoreFocus) return;
    document.getElementById(`player-analysis-tab-${view}`)?.focus();
    onFocusRestored();
  }, [onFocusRestored, restoreFocus, view]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = PLAYER_PROFILE_VIEWS.indexOf(view);
    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % PLAYER_PROFILE_VIEWS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + PLAYER_PROFILE_VIEWS.length) % PLAYER_PROFILE_VIEWS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PLAYER_PROFILE_VIEWS.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = PLAYER_PROFILE_VIEWS[nextIndex];
    onViewChange(next, true);
  };

  return (
    <div
      role="tablist"
      aria-label="Player analysis view"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {PLAYER_PROFILE_VIEWS.map((candidate) => {
        const selected = candidate === view;
        return (
          <button
            key={candidate}
            id={`player-analysis-tab-${candidate}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls="player-analysis-panel"
            tabIndex={selected ? 0 : -1}
            className={
              selected
                ? "cursor-pointer rounded-full bg-primary px-3 py-1.5 text-label-md text-on-primary"
                : "cursor-pointer rounded-full px-3 py-1.5 text-label-md text-on-surface-variant hover:text-on-surface"
            }
            onClick={() => onViewChange(candidate)}
          >
            {candidate === "general" ? "General" : "Moneyball"}
          </button>
        );
      })}
    </div>
  );
}

function GeneralPlayerProfile({
  uid,
  tab,
  onTabChange,
  onViewChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
}: {
  uid: number;
  tab?: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  onViewChange: (view: PlayerProfileView, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
}) {
  const railExpanded = useLayoutStore((state) => state.railExpanded);
  const queryClient = useQueryClient();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: player } = useSuspenseQuery(getPlayerQueryOptions(uid));
  const hiddenInformation = useMutation({
    mutationFn: ({ revealed }: PlayerHiddenInformationMutation) =>
      setHiddenInformationRevealed(revealed),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: playerKeys.all });
      await queryClient.invalidateQueries({ queryKey: staffKeys.all });
    },
  });
  const boost = useMutation({
    mutationFn: ({ action, uid }: PlayerBoostMutation) =>
      action === "currentAbility"
        ? boostCurrentAbility(uid)
        : boostWonderkidMentality(uid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: snapshotKeys.all }),
        queryClient.invalidateQueries({ queryKey: searchKeys.all }),
        queryClient.invalidateQueries({ queryKey: playerKeys.all }),
        queryClient.invalidateQueries({ queryKey: plannerKeys.all }),
        queryClient.invalidateQueries({ queryKey: academyKeys.all }),
      ]);
    },
  });
  const boostContextIsCurrent =
    boost.variables?.uid === uid &&
    boost.variables.snapshotId === snapshot?.id &&
    (boost.data === undefined || boost.data.snapshotId === snapshot?.id);
  const hiddenInformationContextIsCurrent =
    hiddenInformation.variables?.uid === uid &&
    hiddenInformation.variables.saveId === snapshot?.saveId;

  if (!snapshot || !player) return null;

  const activeTab = tab ?? defaultProfileTab(isGoalkeeper(player.positions));

  return (
    <div className="flex h-full min-h-0 flex-col gap-gutter overflow-hidden">
      <PlayerOverviewPanel
        player={player}
        hiddenInformationPending={
          hiddenInformationContextIsCurrent && hiddenInformation.isPending
        }
        hiddenInformationError={
          hiddenInformationContextIsCurrent ? hiddenInformation.error : null
        }
        onToggleHiddenInformation={() =>
          hiddenInformation.mutate({
            saveId: snapshot.saveId,
            uid,
            revealed: !player.hiddenInformationRevealed,
          })
        }
        actions={
          player.hiddenInformationRevealed ? (
            <PlayerDevelopmentActions
              key={`${snapshot.id}:${uid}`}
              player={player}
              pending={boostContextIsCurrent && boost.isPending}
              result={boostContextIsCurrent ? boost.data : undefined}
              error={boostContextIsCurrent ? boost.error : null}
              onBoostCurrentAbility={() =>
                boost.mutateAsync({
                  action: "currentAbility",
                  uid,
                  snapshotId: snapshot.id,
                })
              }
              onBoostWonderkidMentality={() =>
                boost.mutateAsync({
                  action: "wonderkidMentality",
                  uid,
                  snapshotId: snapshot.id,
                })
              }
              onOpenConfirmation={boost.reset}
            />
          ) : null
        }
      />
      <PlayerAnalysisTabs
        view="general"
        onViewChange={onViewChange}
        restoreFocus={restoreAnalysisFocus}
        onFocusRestored={onAnalysisFocusRestored}
      />
      <div
        id="player-analysis-panel"
        role="tabpanel"
        aria-labelledby="player-analysis-tab-general"
        className={profileWorkspaceClassName(railExpanded)}
      >
        <PlayerAttributesPanel
          player={player}
          tab={activeTab}
          onTabChange={onTabChange}
          hiddenInformationRevealed={player.hiddenInformationRevealed}
        />
        <PlayerRolesPanel
          key={player.uid}
          player={player}
          hiddenInformationRevealed={player.hiddenInformationRevealed}
        />
      </div>
    </div>
  );
}

function MoneyballPlayerProfile({
  uid,
  player,
  onViewChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
}: {
  uid: number;
  player: PlayerDetail;
  onViewChange: (view: PlayerProfileView, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
}) {
  const railExpanded = useLayoutStore((state) => state.railExpanded);
  const { data: profile } = useSuspenseQuery(
    getPlayerMoneyballQueryOptions(uid),
  );

  if (!profile) return <PlayerNotFound />;
  const readyProfile = profile.state === "ready" ? profile : null;

  return (
    <div className="flex h-full min-h-0 flex-col gap-gutter overflow-hidden">
      <PlayerOverviewPanel
        player={player}
        mode="moneyball"
        roleScores={readyProfile?.roleScores ?? []}
      />
      <PlayerAnalysisTabs
        view="moneyball"
        onViewChange={onViewChange}
        restoreFocus={restoreAnalysisFocus}
        onFocusRestored={onAnalysisFocusRestored}
      />
      <div
        id="player-analysis-panel"
        role="tabpanel"
        aria-labelledby="player-analysis-tab-moneyball"
        className={
          readyProfile
            ? profileWorkspaceClassName(railExpanded)
            : "h-0 min-h-0 flex-1"
        }
      >
        <MoneyballProfilePanel profile={profile} />
        {readyProfile ? (
          <MoneyballRoleFitPanel
            key={player.uid}
            positions={player.positions}
            roleScores={readyProfile.roleScores}
            catalogVersion={readyProfile.roleCatalogVersion}
          />
        ) : null}
      </div>
    </div>
  );
}

function PlayerProfileContent({
  uid,
  view,
  tab,
  onTabChange,
  onViewChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
}: {
  uid: number;
  view: PlayerProfileView;
  tab?: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  onViewChange: (view: PlayerProfileView, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
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
  if (!player) return <PlayerNotFound />;

  if (view === "moneyball") {
    return (
      <MoneyballPlayerProfile
        uid={uid}
        player={player}
        onViewChange={onViewChange}
        restoreAnalysisFocus={restoreAnalysisFocus}
        onAnalysisFocusRestored={onAnalysisFocusRestored}
      />
    );
  }

  return (
    <GeneralPlayerProfile
      uid={uid}
      tab={tab}
      onTabChange={onTabChange}
      onViewChange={onViewChange}
      restoreAnalysisFocus={restoreAnalysisFocus}
      onAnalysisFocusRestored={onAnalysisFocusRestored}
    />
  );
}

function PlayerProfileRoute() {
  const { uid: uidParam } = Route.useParams();
  const { tab, view } = Route.useSearch();
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const navigate = Route.useNavigate();
  const uid = parseUid(uidParam);
  const [analysisFocusView, setAnalysisFocusView] =
    useState<PlayerProfileView | null>(null);

  const onTabChange = (next: ProfileTab) => {
    void navigate({
      search: (previous) => ({ ...previous, tab: next }),
      replace: true,
    });
  };
  const onViewChange = (next: PlayerProfileView, restoreFocus = false) => {
    setAnalysisFocusView(restoreFocus ? next : null);
    void navigate({
      search: (previous) => ({ ...previous, view: next }),
      replace: true,
    });
  };

  if (uid === null) {
    return <PlayerNotFound />;
  }

  return (
    <Suspense fallback={<ProfileFallback />}>
      <PlayerProfileContent
        uid={uid}
        view={view ?? defaultAnalysisView}
        tab={tab}
        onTabChange={onTabChange}
        onViewChange={onViewChange}
        restoreAnalysisFocus={
          analysisFocusView === (view ?? defaultAnalysisView)
        }
        onAnalysisFocusRestored={() => setAnalysisFocusView(null)}
      />
    </Suspense>
  );
}
