import { useQueryClient } from "@tanstack/react-query";
import { CircleAlert, Shield } from "lucide-react";
import { type ReactNode, Suspense } from "react";
import { clearPlayerResultContext } from "@/app/player-result-context";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { GraphicsImage } from "@/features/graphics/components/graphics-image";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { ManagedClubSelector } from "@/features/managed-club/components/managed-club-selector";
import type { ManagedClubStatus } from "@/features/managed-club/types/managed-club";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";

export function ManagedClubLogo({
  clubUid,
  managedClubStatus,
}: {
  clubUid: number | null;
  managedClubStatus: ManagedClubStatus["status"];
}) {
  return managedClubStatus === "available" ? (
    <GraphicsImage
      kind="clubLogo"
      uid={clubUid ?? 0}
      slot={{
        alt: "",
        ariaHidden: true,
        className: "size-6 shrink-0 object-contain",
        fallback: (
          <Shield
            aria-hidden="true"
            className="size-6 shrink-0 text-on-surface-variant"
            strokeWidth={1.5}
          />
        ),
      }}
    />
  ) : (
    <Shield
      aria-hidden="true"
      className="size-6 shrink-0 text-on-surface-variant"
      strokeWidth={1.5}
    />
  );
}

function ManagedClubError({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  const queryClient = useQueryClient();

  return (
    <Panel>
      <EmptyState
        icon={CircleAlert}
        title="Could not load managed club"
        action={
          <Button
            variant="secondary"
            onClick={() => {
              queryClient.resetQueries({ queryKey: managedClubKeys.all });
              reset();
            }}
          >
            Retry
          </Button>
        }
      >
        {error.message}
      </EmptyState>
    </Panel>
  );
}

/**
 * The managed-club selector with its loading and error states. A saved club
 * refreshes everything that reads club membership: Planner (with Squad and the
 * dashboard under its root), Academy, and Staff.
 */
export function ManagedClubSetup({ action }: { action?: ReactNode }) {
  const queryClient = useQueryClient();

  return (
    <ErrorBoundary
      fallback={({ error, reset }) => (
        <ManagedClubError error={error} reset={reset} />
      )}
    >
      <Suspense
        fallback={
          <div className="flex min-h-16 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-sm text-on-surface-variant">
            Loading managed club…
          </div>
        }
      >
        <ManagedClubSelector
          action={action}
          onBeforeContextChange={() => clearPlayerResultContext(queryClient)}
          onSaved={() => {
            void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
            void queryClient.resetQueries({ queryKey: academyKeys.all });
            void queryClient.invalidateQueries({ queryKey: staffKeys.all });
          }}
        />
      </Suspense>
    </ErrorBoundary>
  );
}
