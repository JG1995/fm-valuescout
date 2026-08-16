import { Zap } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type {
  MyStaffBoostProgress,
  MyStaffBoostResult,
} from "../types/my-staff-boost";

type MyStaffCaBoostProps = {
  pending: boolean;
  disabled: boolean;
  error: Error | null;
  onBoost: (
    onProgress: (progress: MyStaffBoostProgress) => void,
  ) => Promise<unknown>;
  onOpenConfirmation: () => void;
  fallbackFocusTo: () => HTMLElement | null;
};

export function MyStaffBoostOutcome({
  result,
  error,
}: {
  result: MyStaffBoostResult | undefined;
  error: Error | null;
}) {
  if (result) {
    const processed = result.updated + result.skipped + result.failed;
    return (
      <div
        className={
          result.recoveryRequired
            ? "space-y-1 text-body-sm text-warning"
            : "text-body-sm text-success"
        }
        role="status"
      >
        <p>
          {processed} processed — {result.updated} updated, {result.skipped}{" "}
          skipped, {result.failed} failed.
        </p>
        {result.recoveryRequired ? (
          <p>
            Stopped before all staff were processed. Load Data is required
            before another boost.
          </p>
        ) : null}
      </div>
    );
  }
  return error ? (
    <p className="text-body-sm text-error" role="alert">
      Could not boost My Staff. {error.message}
    </p>
  ) : null;
}

export function MyStaffCaBoost({
  pending,
  disabled,
  error,
  onBoost,
  onOpenConfirmation,
  fallbackFocusTo,
}: MyStaffCaBoostProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [progress, setProgress] = useState<MyStaffBoostProgress | null>(null);

  return (
    <>
      <Button
        icon={Zap}
        disabled={pending || disabled}
        loading={pending}
        loadingLabel="Boosting…"
        onClick={() => {
          onOpenConfirmation();
          setProgress(null);
          setConfirmationOpen(true);
        }}
      >
        Boost all CA
      </Button>
      <Modal
        open={confirmationOpen}
        title="Boost all CA?"
        onClose={() => {
          if (!pending) setConfirmationOpen(false);
        }}
        fallbackFocusTo={fallbackFocusTo}
        footer={
          <>
            <Button
              variant="secondary"
              disabled={pending}
              onClick={() => setConfirmationOpen(false)}
            >
              Cancel
            </Button>
            <Button
              loading={pending}
              loadingLabel="Boosting…"
              onClick={() => {
                void onBoost(setProgress).then(
                  () => setConfirmationOpen(false),
                  () => setProgress(null),
                );
              }}
            >
              Boost all CA
            </Button>
          </>
        }
      >
        <div className="space-y-3">
          <p className="text-body-md text-on-surface-variant">
            This boosts current ability by 10 for every eligible staff member in
            your configured club family, one at a time. Each boost stops at PA
            or 200.
          </p>
          <p className="text-body-sm text-on-surface-variant">
            Changes already applied cannot be undone.
          </p>
          {pending ? (
            <div
              className="space-y-2 text-body-sm text-on-surface-variant"
              aria-live="polite"
            >
              {progress && progress.total > 0 ? (
                <progress
                  aria-label="My Staff boost progress"
                  className="h-2 w-full accent-primary"
                  max={progress.total}
                  value={progress.processed}
                />
              ) : null}
              <p>
                {progress
                  ? `${progress.processed} of ${progress.total} staff processed.`
                  : "Preparing My Staff…"}
              </p>
            </div>
          ) : null}
          <div aria-live="polite">
            <MyStaffBoostOutcome result={undefined} error={error} />
          </div>
        </div>
      </Modal>
    </>
  );
}
