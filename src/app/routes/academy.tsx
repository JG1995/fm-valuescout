import { useQueries, useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { DatabaseZap, FolderOpen, Plus, UsersRound } from "lucide-react";
import { Suspense, useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyClassQueryOptions } from "@/features/academy/api/academy-class-query-options";
import { academyClassesQueryOptions } from "@/features/academy/api/academy-classes-query-options";
import { AcademyClassCreationModal } from "@/features/academy/components/academy-class-creation-modal";
import { AcademyClassDeletionModal } from "@/features/academy/components/academy-class-deletion-modal";
import { AcademyClassWorkspace } from "@/features/academy/components/academy-class-workspace";
import { AcademyGraduatesWorkspace } from "@/features/academy/components/academy-graduates-workspace";
import { AcademyOverview } from "@/features/academy/components/academy-overview";
import {
  AcademyWorkspaceTabs,
  academyWorkspacePanelProps,
} from "@/features/academy/components/academy-workspace-tabs";
import type {
  AcademyClass,
  AcademyView,
} from "@/features/academy/types/academy";
import { academyDetailsAreComplete } from "@/features/academy/utils/academy-statistics";
import {
  parseAcademyClassId,
  parseAcademyView,
  snapshotYear,
} from "@/features/academy/utils/academy-workspace";
import { plannerClubFamilyQueryOptions } from "@/features/planner/api/get-planner-club-family-query-options";
import { plannerClubsQueryOptions } from "@/features/planner/api/planner-clubs-query-options";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

export type AcademySearch = {
  view: AcademyView;
  classId?: number;
};

export const Route = createFileRoute("/academy")({
  validateSearch: (search: Record<string, unknown>): AcademySearch => {
    const view = parseAcademyView(search.view);
    const classId = parseAcademyClassId(search.classId);
    if (view === "class") {
      return classId !== null ? { view, classId } : { view };
    }
    return { view };
  },
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(plannerClubFamilyQueryOptions),
      queryClient.ensureQueryData(academyClassesQueryOptions),
    ]),
  component: AcademyRoute,
});

function AcademyFallback() {
  return (
    <div className="space-y-gutter" aria-busy="true" aria-live="polite">
      <h1 className="text-headline-lg text-on-surface">Youth Academy</h1>
      <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
        Loading academy…
      </div>
    </div>
  );
}

function AcademyNoSnapshot() {
  return (
    <Panel title="Youth Academy" flush>
      <EmptyState icon={DatabaseZap} title="No data loaded for this save">
        No snapshot loaded for the active save. Use Load Data to scan Football
        Manager and ingest players before creating academy classes.
      </EmptyState>
    </Panel>
  );
}

function AcademyNoClubFamily() {
  return (
    <Panel title="Set up your club family" flush>
      <EmptyState
        icon={UsersRound}
        title="Academy needs your club family"
        action={
          <Link
            to="/"
            hash="club-setup"
            className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
          >
            Open Club Setup
          </Link>
        }
      >
        Youth Academy uses the same club-family sources as Planner. Configure
        them once to define which players can be tracked.
      </EmptyState>
    </Panel>
  );
}

function AcademyNoClasses({ onCreate }: { onCreate: () => void }) {
  return (
    <Panel title="Class" flush>
      <EmptyState
        icon={FolderOpen}
        title="No academy classes available"
        action={
          <Button icon={Plus} onClick={onCreate}>
            Create class
          </Button>
        }
      >
        Create a class to start grouping players by the year they came through
        your club.
      </EmptyState>
    </Panel>
  );
}

function AcademyPageContent() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: clubFamily } = useSuspenseQuery(plannerClubFamilyQueryOptions);
  const { data: classes } = useSuspenseQuery(academyClassesQueryOptions);
  const clubOptions = useQuery({
    ...plannerClubsQueryOptions,
    enabled: Boolean(snapshot),
  });
  const classDetailQueries = useQueries({
    queries: classes.map((academyClass) => ({
      ...academyClassQueryOptions(academyClass.id),
      enabled: Boolean(snapshot),
    })),
  });
  const classDetails = classDetailQueries.flatMap((query) =>
    query.data ? [query.data] : [],
  );
  const classDetailsReady = academyDetailsAreComplete(classes, classDetails);
  const classDetailsError = classDetailQueries.find(
    (query) => query.isError,
  )?.error;
  const classDetailsPending = classDetailQueries.some(
    (query) => query.isPending,
  );
  const { view, classId } = Route.useSearch();
  const navigate = Route.useNavigate();
  const [createOpen, setCreateOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<AcademyClass | null>(null);
  const selectedClass = classId
    ? (classes.find((academyClass) => academyClass.id === classId) ?? null)
    : null;
  const activeView = view;

  useEffect(() => {
    if (view === "class" && classes.length > 0 && !selectedClass) {
      void navigate({ search: { view: "overview" }, replace: true });
    }
  }, [classes.length, navigate, selectedClass, view]);

  const onViewChange = (nextView: AcademyView) => {
    if (nextView === "class") {
      const targetClass = selectedClass ?? classes[0];
      if (targetClass) {
        void navigate({
          search: { view: "class", classId: targetClass.id },
          replace: true,
        });
      } else {
        void navigate({ search: { view: "class" }, replace: true });
      }
      return;
    }
    void navigate({ search: { view: nextView }, replace: true });
  };

  const onOpenClass = (academyClass: AcademyClass) => {
    void navigate({
      search: { view: "class", classId: academyClass.id },
      replace: true,
    });
  };

  if (!snapshot) {
    return (
      <div className="space-y-gutter">
        <h1 className="text-headline-lg text-on-surface">Youth Academy</h1>
        <AcademyNoSnapshot />
      </div>
    );
  }

  if (!clubFamily.primaryClub) {
    return (
      <div className="space-y-gutter">
        <h1 className="text-headline-lg text-on-surface">Youth Academy</h1>
        <AcademyNoClubFamily />
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <header className="flex flex-col items-start gap-2">
        <div>
          <h1 className="text-headline-lg text-on-surface">Youth Academy</h1>
          <p className="text-body-sm text-on-surface-variant">
            {clubFamily.primaryClub} · Class cohorts from your club family
          </p>
        </div>
        <AcademyWorkspaceTabs view={activeView} onViewChange={onViewChange} />
      </header>

      <div {...academyWorkspacePanelProps("overview", activeView)}>
        <AcademyOverview
          classes={classes}
          classDetails={classDetails}
          classDetailsReady={classDetailsReady}
          classDetailsPending={classDetailsPending}
          classDetailsError={classDetailsError}
          onCreate={() => setCreateOpen(true)}
          onOpenClass={onOpenClass}
        />
      </div>
      <div {...academyWorkspacePanelProps("graduates", activeView)}>
        <AcademyGraduatesWorkspace
          classDetails={classDetails}
          detailsPending={classDetailsPending}
          detailsReady={classDetailsReady}
          detailsError={classDetailsError}
        />
      </div>
      <div {...academyWorkspacePanelProps("class", activeView)}>
        {selectedClass ? (
          <AcademyClassWorkspace
            academyClass={selectedClass}
            academyClasses={classes}
            clubOptions={clubOptions.data ?? []}
            onSelectClass={onOpenClass}
            onDelete={() => setDeleteTarget(selectedClass)}
          />
        ) : (
          <AcademyNoClasses onCreate={() => setCreateOpen(true)} />
        )}
      </div>

      <AcademyClassCreationModal
        open={createOpen}
        prefillYear={snapshotYear(snapshot.gameDate)}
        onClose={() => setCreateOpen(false)}
        onCreated={() => setCreateOpen(false)}
      />
      <AcademyClassDeletionModal
        target={deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onDeleted={() => {
          setDeleteTarget(null);
          void navigate({ search: { view: "overview" }, replace: true });
        }}
      />
    </div>
  );
}

function AcademyRoute() {
  return (
    <Suspense fallback={<AcademyFallback />}>
      <AcademyPageContent />
    </Suspense>
  );
}
