import { useMutation } from "@tanstack/react-query";
import { Zap } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { boostManagedClub } from "../api/boost-managed-club";
import type {
  ManagedClubBoostPhase,
  ManagedClubBoostProgress,
  ManagedClubBoostResult,
} from "../types/managed-club-boost";

const phaseLabels: Record<ManagedClubBoostPhase, string> = {
  wonderkids: "Making Wonderkids",
  playerCurrentAbility: "Boosting player CA",
  staffCurrentAbility: "Boosting staff CA",
};

function resultSummary(result: ManagedClubBoostResult) {
  const batches = [
    result.wonderkids,
    result.playerCurrentAbility,
    result.staffCurrentAbility,
  ].filter((batch) => batch !== null);
  const updated = batches.reduce((total, batch) => total + batch.updated, 0);
  const skipped = batches.reduce((total, batch) => total + batch.skipped, 0);
  const failed = batches.reduce((total, batch) => total + batch.failed, 0);
  const recoveryRequired = batches.some((batch) => batch.recoveryRequired);

  return { updated, skipped, failed, recoveryRequired };
}

export function ManagedClubBoost({
  contextKey,
  onSettled,
}: {
  contextKey: string;
  onSettled: () => void;
}) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [progress, setProgress] = useState<ManagedClubBoostProgress | null>(
    null,
  );
  const boost = useMutation({
    mutationFn: (_variables: { contextKey: string }) =>
      boostManagedClub(setProgress),
    onSettled,
  });
  const boostContextIsCurrent = boost.variables?.contextKey === contextKey;
  const summary =
    boostContextIsCurrent && boost.data ? resultSummary(boost.data) : null;
  const currentProgress = boostContextIsCurrent ? progress : null;
  const currentError = boostContextIsCurrent ? boost.error : null;

  return (
    <div className="max-w-2xl space-y-3 rounded-lg border border-outline-variant bg-surface-container p-4">
      <p className="text-body-md text-on-surface-variant">
        Apply every managed-club player and staff boost from the current Load
        Data scan.
      </p>
      <Button
        id="apply-all-boosts"
        icon={Zap}
        disabled={summary?.recoveryRequired}
        loading={boost.isPending}
        loadingLabel="Applying…"
        onClick={() => {
          boost.reset();
          setProgress(null);
          setConfirmationOpen(true);
        }}
      >
        Apply all boosts
      </Button>
      <div aria-live="polite">
        {summary ? (
          <div
            className={
              summary.recoveryRequired
                ? "space-y-1 text-body-sm text-warning"
                : "text-body-sm text-success"
            }
            role="status"
          >
            <p>
              {summary.recoveryRequired ? "Stopped" : "Completed"} —{" "}
              {summary.updated} updated, {summary.skipped} skipped,{" "}
              {summary.failed} failed.
            </p>
            {summary.recoveryRequired ? (
              <p>Load Data is required before another boost.</p>
            ) : null}
          </div>
        ) : null}
      </div>
      <Modal
        open={confirmationOpen}
        title="Apply all boosts?"
        onClose={() => {
          if (!boost.isPending) setConfirmationOpen(false);
        }}
        fallbackFocusTo={() => document.getElementById("apply-all-boosts")}
        footer={
          <>
            <Button
              variant="secondary"
              disabled={boost.isPending}
              onClick={() => setConfirmationOpen(false)}
            >
              Cancel
            </Button>
            <Button
              loading={boost.isPending}
              loadingLabel="Applying…"
              onClick={() => {
                void boost.mutateAsync({ contextKey }).then(
                  () => setConfirmationOpen(false),
                  () => undefined,
                );
              }}
            >
              Apply all boosts
            </Button>
          </>
        }
      >
        <div className="space-y-3 text-body-md text-on-surface-variant">
          <p>This runs these actions in order:</p>
          <ol className="list-decimal space-y-1 pl-5">
            <li>Make all Wonderkids.</li>
            <li>Apply the age-restricted player CA boost.</li>
            <li>Apply the staff CA boost.</li>
          </ol>
          <p>Changes already applied cannot be undone.</p>
          {boost.isPending ? (
            <div className="space-y-2 text-body-sm" aria-live="polite">
              {currentProgress && currentProgress.total > 0 ? (
                <progress
                  aria-label="Managed club boost progress"
                  className="h-2 w-full accent-primary"
                  max={currentProgress.total}
                  value={currentProgress.processed}
                />
              ) : null}
              <p>
                {currentProgress
                  ? `${phaseLabels[currentProgress.phase]} — ${currentProgress.processed} of ${currentProgress.total} processed.`
                  : "Preparing managed club boosts…"}
              </p>
            </div>
          ) : null}
          {currentError ? (
            <div className="space-y-1 text-body-sm text-error" role="alert">
              <p>Could not apply all boosts. {currentError.message}</p>
              <p>Any changes already applied remain in FM.</p>
            </div>
          ) : null}
        </div>
      </Modal>
    </div>
  );
}
