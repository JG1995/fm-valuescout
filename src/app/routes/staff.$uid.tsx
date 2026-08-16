import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, UserX } from "lucide-react";
import { Suspense, useRef } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { setHiddenInformationRevealed } from "@/features/player-profile/api/set-hidden-information-revealed";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { boostStaffCurrentAbility } from "@/features/staff/api/boost-staff-current-ability";
import { getStaffQueryOptions } from "@/features/staff/api/get-staff-query-options";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { StaffAttributesPanel } from "@/features/staff/components/staff-attributes-panel";
import { StaffOverviewPanel } from "@/features/staff/components/staff-overview-panel";
import { StaffRoleFitPanel } from "@/features/staff/components/staff-role-fit-panel";
import { useLayoutStore } from "@/stores/use-layout-store";
import { cn } from "@/utils/cn";

function parseUid(raw: string): number | null {
  const uid = Number(raw);
  return Number.isInteger(uid) ? uid : null;
}

export const Route = createFileRoute("/staff/$uid")({
  loader: ({ context: { queryClient }, params }) => {
    const uid = parseUid(params.uid);
    if (uid === null)
      return queryClient.ensureQueryData(currentSnapshotQueryOptions);
    return Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(getStaffQueryOptions(uid)),
    ]);
  },
  component: StaffProfileRoute,
});

function profileWorkspaceClassName(railExpanded: boolean) {
  return cn(
    "grid h-0 min-h-0 flex-1 gap-gutter [&>*]:min-h-0",
    railExpanded
      ? "grid-cols-1 grid-rows-[minmax(0,1fr)_minmax(0,1fr)] 2xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)] 2xl:grid-rows-[minmax(0,1fr)]"
      : "grid-rows-[minmax(0,1fr)] lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]",
  );
}

function StaffProfileFallback() {
  return (
    <div
      className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant"
      aria-busy="true"
    >
      Loading staff profile…
    </div>
  );
}

function StaffNotFound() {
  return (
    <Panel>
      <EmptyState icon={UserX} title="Staff member not in this snapshot">
        This staff member is not in the active save’s current snapshot. Return
        to Staff or load a fresher snapshot.
      </EmptyState>
    </Panel>
  );
}

function StaffProfileContent({ uid }: { uid: number }) {
  const railExpanded = useLayoutStore((state) => state.railExpanded);
  const queryClient = useQueryClient();
  const outcomeRef = useRef<HTMLDivElement>(null);
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: staff } = useSuspenseQuery(getStaffQueryOptions(uid));
  const hiddenInformation = useMutation({
    mutationFn: ({ revealed }: { revealed: boolean }) =>
      setHiddenInformationRevealed(revealed),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: playerKeys.all }),
        queryClient.invalidateQueries({ queryKey: staffKeys.all }),
      ]);
    },
  });
  const boost = useMutation({
    mutationFn: () => boostStaffCurrentAbility(uid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: staffKeys.all }),
        queryClient.invalidateQueries({ queryKey: snapshotKeys.all }),
      ]);
    },
  });

  if (!snapshot) {
    return (
      <Panel>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest staff.
        </EmptyState>
      </Panel>
    );
  }
  if (!staff) return <StaffNotFound />;

  return (
    <div className="flex h-full min-h-0 flex-col gap-gutter overflow-hidden">
      <StaffOverviewPanel
        staff={staff}
        hiddenInformationPending={hiddenInformation.isPending}
        hiddenInformationError={hiddenInformation.error}
        onToggleHiddenInformation={() =>
          hiddenInformation.mutate({
            revealed: !staff.hiddenInformationRevealed,
          })
        }
        boostPending={boost.isPending}
        boostError={boost.error}
        onBoost={() => boost.mutateAsync()}
        onOpenBoostConfirmation={boost.reset}
        fallbackFocusTo={() => outcomeRef.current}
      />
      <div
        ref={outcomeRef}
        data-testid="staff-profile-boost-outcome"
        tabIndex={-1}
        className="rounded-sm [&:not(:empty)]:-mt-2 focus:outline-2 focus:outline-offset-2 focus:outline-primary"
        aria-live="polite"
      >
        {boost.data ? (
          <p className="text-body-sm text-success" role="status">
            Staff CA boosted from {boost.data.previousCurrentAbility} to{" "}
            {boost.data.currentAbility}.
          </p>
        ) : null}
        {boost.error && !boost.isPending ? (
          <p className="text-body-sm text-error" role="alert">
            Could not apply staff CA boost. {boost.error.message}
          </p>
        ) : null}
      </div>
      <div className={profileWorkspaceClassName(railExpanded)}>
        <StaffAttributesPanel staff={staff} />
        <StaffRoleFitPanel staff={staff} />
      </div>
    </div>
  );
}

function StaffProfileRoute() {
  const { uid: uidParam } = Route.useParams();
  const uid = parseUid(uidParam);
  if (uid === null) return <StaffNotFound />;

  return (
    <Suspense fallback={<StaffProfileFallback />}>
      <StaffProfileContent uid={uid} />
    </Suspense>
  );
}
