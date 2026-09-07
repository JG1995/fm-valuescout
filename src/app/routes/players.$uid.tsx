import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, UserX } from "lucide-react";
import { type ReactNode, Suspense, useEffect, useState } from "react";
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
import { PlayerIdentityRail } from "@/features/player-profile/components/player-identity";
import { PlayerOverviewPanel } from "@/features/player-profile/components/player-overview-panel";
import {
  type PlayerProfileView,
  PlayerSectionTabs,
  parsePlayerProfileView,
} from "@/features/player-profile/components/player-profile-navigation";
import { PlayerRolesPanel } from "@/features/player-profile/components/player-roles-panel";
import type { PlayerBoostResult } from "@/features/player-profile/types/player-boost";
import type { PlayerDetail } from "@/features/player-profile/types/player-detail";
import { isGoalkeeper } from "@/features/player-profile/utils/position-families";
import {
  isStandardProfileSection,
  type ProfileSection,
  parseProfileSection,
  resolveProfileSection,
} from "@/features/player-profile/utils/profile-section";
import {
  defaultProfileTab,
  type ProfileTab,
  parseProfileTab,
} from "@/features/player-profile/utils/profile-tab";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import { cn } from "@/utils/cn";

export type PlayerProfileSearch = {
  tab?: ProfileTab;
  view?: PlayerProfileView;
  section?: ProfileSection;
};

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

export const Route = createFileRoute("/players/$uid")({
  validateSearch: (search: Record<string, unknown>): PlayerProfileSearch => ({
    tab: parseProfileTab(search.tab),
    view: parsePlayerProfileView(search.view),
    section: parseProfileSection(search.section),
  }),
  loaderDeps: ({ search }) => ({
    section: resolveProfileSection({
      section: search.section,
      view: search.view,
      tab: search.tab,
      defaultAnalysisView:
        useMoneyballPreferences.getState().defaultAnalysisView,
    }),
  }),
  loader: ({ context: { queryClient }, params, deps: { section } }) => {
    const uid = parseUid(params.uid);
    if (uid === null) {
      return queryClient.ensureQueryData(currentSnapshotQueryOptions);
    }
    const queries: Promise<unknown>[] = [
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(getPlayerQueryOptions(uid)),
    ];
    if (section === "moneyball") {
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
  return (
    <div
      className="flex min-h-0 flex-col gap-gutter lg:h-full lg:overflow-hidden"
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
      <div className={profileWorkspaceClassName()}>
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

function profileWorkspaceClassName() {
  return cn(
    "grid min-h-0 gap-gutter [&>*]:min-h-0 lg:h-0 lg:flex-1",
    "grid-rows-[minmax(0,1fr)] lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]",
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

function PlayerProfileHeader({
  overview,
  section,
  onSectionChange,
  restoreFocus,
  onFocusRestored,
}: {
  overview: ReactNode;
  section: ProfileSection;
  onSectionChange: (section: ProfileSection, restoreFocus?: boolean) => void;
  restoreFocus: boolean;
  onFocusRestored: () => void;
}) {
  return (
    <header
      data-testid="player-profile-header"
      className="flex flex-col gap-gutter"
    >
      <PlayerSectionTabs
        section={section}
        onSectionChange={onSectionChange}
        restoreFocus={restoreFocus}
        onFocusRestored={onFocusRestored}
      />
      {overview}
    </header>
  );
}

function GeneralPlayerProfile({
  uid,
  snapshot,
  player,
  section,
  tab,
  onTabChange,
  onSectionChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
  boostPending,
  boostResult,
  boostError,
  onBoostCurrentAbility,
  onBoostWonderkidMentality,
  onOpenBoostConfirmation,
}: {
  uid: number;
  snapshot: SnapshotSummary;
  player: PlayerDetail;
  section: Exclude<ProfileSection, "moneyball">;
  tab?: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  onSectionChange: (section: ProfileSection, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
  hiddenInformationPending: boolean;
  hiddenInformationError: Error | null;
  onToggleHiddenInformation: () => void;
  boostPending: boolean;
  boostResult: PlayerBoostResult | undefined;
  boostError: Error | null;
  onBoostCurrentAbility: () => Promise<unknown>;
  onBoostWonderkidMentality: () => Promise<unknown>;
  onOpenBoostConfirmation: () => void;
}) {
  const activeTab = tab ?? defaultProfileTab(isGoalkeeper(player.positions));
  const showAttributes = section !== "role-fit";
  const showRoles = section !== "attributes";

  return (
    <div className="flex min-h-0 flex-col gap-gutter lg:h-full lg:overflow-hidden">
      <PlayerProfileHeader
        overview={
          <PlayerOverviewPanel
            player={player}
            hiddenInformationPending={hiddenInformationPending}
            hiddenInformationError={hiddenInformationError}
            onToggleHiddenInformation={onToggleHiddenInformation}
            showAbilitySummary={section === "overview"}
            showTacticalFitSummary={section === "overview"}
            actions={
              player.hiddenInformationRevealed ? (
                <PlayerDevelopmentActions
                  key={`${snapshot.id}:${uid}`}
                  player={player}
                  pending={boostPending}
                  result={boostResult}
                  error={boostError}
                  onBoostCurrentAbility={onBoostCurrentAbility}
                  onBoostWonderkidMentality={onBoostWonderkidMentality}
                  onOpenConfirmation={onOpenBoostConfirmation}
                />
              ) : null
            }
          />
        }
        section={section}
        onSectionChange={onSectionChange}
        restoreFocus={restoreAnalysisFocus}
        onFocusRestored={onAnalysisFocusRestored}
      />
      <div
        id="player-analysis-panel"
        role="tabpanel"
        aria-labelledby={`player-analysis-tab-${section}`}
        className={
          showAttributes && showRoles
            ? profileWorkspaceClassName()
            : section === "attributes"
              ? "grid min-h-0 min-w-0 w-full flex-1 grid-rows-[minmax(0,1fr)] lg:h-0 lg:grid-cols-1"
              : undefined
        }
      >
        {showAttributes ? (
          <PlayerAttributesPanel
            player={player}
            tab={activeTab}
            onTabChange={onTabChange}
            hiddenInformationRevealed={player.hiddenInformationRevealed}
          />
        ) : null}
        {showRoles ? (
          <PlayerRolesPanel
            key={player.uid}
            player={player}
            hiddenInformationRevealed={player.hiddenInformationRevealed}
          />
        ) : null}
      </div>
    </div>
  );
}

function MoneyballPlayerProfile({
  uid,
  player,
  onSectionChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
}: {
  uid: number;
  player: PlayerDetail;
  onSectionChange: (section: ProfileSection, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
}) {
  const { data: profile } = useSuspenseQuery(
    getPlayerMoneyballQueryOptions(uid),
  );

  if (!profile) return <PlayerNotFound />;
  const readyProfile = profile.state === "ready" ? profile : null;

  return (
    <div className="flex min-h-0 flex-col gap-gutter lg:h-full lg:overflow-hidden">
      <PlayerProfileHeader
        overview={
          <PlayerOverviewPanel
            player={player}
            mode="moneyball"
            roleScores={
              readyProfile?.comparisonBasis.kind === "available"
                ? (readyProfile.roleScores ?? [])
                : []
            }
          />
        }
        section="moneyball"
        onSectionChange={onSectionChange}
        restoreFocus={restoreAnalysisFocus}
        onFocusRestored={onAnalysisFocusRestored}
      />
      <div
        id="player-analysis-panel"
        role="tabpanel"
        aria-labelledby="player-analysis-tab-moneyball"
        className={profileWorkspaceClassName()}
      >
        <MoneyballProfilePanel profile={profile} />
        {readyProfile ? (
          <MoneyballRoleFitPanel
            key={player.uid}
            positions={player.positions}
            roleScores={readyProfile.roleScores}
            catalogVersion={readyProfile.roleCatalogVersion}
            comparisonBasis={readyProfile.comparisonBasis}
          />
        ) : null}
      </div>
    </div>
  );
}

function PlayerProfileContent({
  uid,
  section,
  tab,
  onTabChange,
  onSectionChange,
  restoreAnalysisFocus,
  onAnalysisFocusRestored,
}: {
  uid: number;
  section: ProfileSection;
  tab?: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  onSectionChange: (section: ProfileSection, restoreFocus?: boolean) => void;
  restoreAnalysisFocus: boolean;
  onAnalysisFocusRestored: () => void;
}) {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: player } = useSuspenseQuery(getPlayerQueryOptions(uid));
  const queryClient = useQueryClient();
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
  const boostReset = boost.reset;
  const hiddenInformationReset = hiddenInformation.reset;
  // Standard sections share one mutation owner: leaving the
  // Overview/Attributes/Role Fit family discards mutation feedback so a
  // standard → Moneyball → standard round-trip cannot resurface it.
  useEffect(() => {
    if (!isStandardProfileSection(section)) {
      boostReset();
      hiddenInformationReset();
    }
  }, [section, boostReset, hiddenInformationReset]);

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

  return (
    <div className="flex min-h-0 flex-col gap-gutter lg:h-full lg:flex-row lg:overflow-hidden">
      <PlayerIdentityRail player={player} />
      <section
        aria-label="Player analysis"
        data-testid="player-analysis-workspace"
        className="flex min-h-0 min-w-0 flex-1 flex-col lg:h-full lg:overflow-hidden"
      >
        {section === "moneyball" ? (
          <MoneyballPlayerProfile
            uid={uid}
            player={player}
            onSectionChange={onSectionChange}
            restoreAnalysisFocus={restoreAnalysisFocus}
            onAnalysisFocusRestored={onAnalysisFocusRestored}
          />
        ) : (
          <GeneralPlayerProfile
            uid={uid}
            snapshot={snapshot}
            player={player}
            section={section}
            tab={tab}
            onTabChange={onTabChange}
            onSectionChange={onSectionChange}
            restoreAnalysisFocus={restoreAnalysisFocus}
            onAnalysisFocusRestored={onAnalysisFocusRestored}
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
            boostPending={boostContextIsCurrent && boost.isPending}
            boostResult={boostContextIsCurrent ? boost.data : undefined}
            boostError={boostContextIsCurrent ? boost.error : null}
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
            onOpenBoostConfirmation={boost.reset}
          />
        )}
      </section>
    </div>
  );
}

function PlayerProfileRoute() {
  const { uid: uidParam } = Route.useParams();
  const { tab, view, section: sectionParam } = Route.useSearch();
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const navigate = Route.useNavigate();
  const uid = parseUid(uidParam);
  const section = resolveProfileSection({
    section: sectionParam,
    view,
    tab,
    defaultAnalysisView,
  });
  const [analysisFocusSection, setAnalysisFocusSection] =
    useState<ProfileSection | null>(null);

  const onTabChange = (next: ProfileTab) => {
    void navigate({
      search: (previous) => ({ ...previous, tab: next }),
      replace: true,
    });
  };
  const onSectionChange = (next: ProfileSection, restoreFocus = false) => {
    setAnalysisFocusSection(restoreFocus ? next : null);
    void navigate({
      search: (previous) => ({ ...previous, section: next }),
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
        section={section}
        tab={tab}
        onTabChange={onTabChange}
        onSectionChange={onSectionChange}
        restoreAnalysisFocus={analysisFocusSection === section}
        onAnalysisFocusRestored={() => setAnalysisFocusSection(null)}
      />
    </Suspense>
  );
}
