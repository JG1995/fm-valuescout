import { ChevronsUp, Sprout } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type {
  SquadPlayerBoostProgress,
  SquadPlayerBoostResult,
} from "../types/squad-player-boost";

export type SquadPlayerBoostAction = "currentAbility" | "wonderkidMentality";

type SquadPlayerBoostProps = {
  action: SquadPlayerBoostAction;
  pending: boolean;
  disabled: boolean;
  error: Error | null;
  onBoost: (
    onProgress: (progress: SquadPlayerBoostProgress) => void,
  ) => Promise<unknown>;
  onOpenConfirmation: () => void;
  onConfirmationChange: (open: boolean) => void;
  fallbackFocusTo: () => HTMLElement | null;
};

type SquadCurrentAbilityBoostProps = Omit<SquadPlayerBoostProps, "action">;
type SquadWonderkidMentalityBoostProps = Omit<SquadPlayerBoostProps, "action">;

function resultSummary(result: SquadPlayerBoostResult) {
  const processed = result.updated + result.skipped + result.failed;
  return `${processed} processed: ${result.updated} updated, ${result.skipped} skipped, ${result.failed} failed.`;
}

export function SquadBoostOutcome({
  result,
  error,
  action,
}: Pick<SquadPlayerBoostProps, "error" | "action"> & {
  result: SquadPlayerBoostResult | undefined;
}) {
  if (result) {
    return (
      <div
        className={
          result.recoveryRequired
            ? "space-y-1 text-body-sm text-warning"
            : "text-body-sm text-success"
        }
        role="status"
      >
        <p>{resultSummary(result)}</p>
        {result.recoveryRequired ? (
          <p>
            Stopped before all players were processed. Load Data is required
            before another boost.
          </p>
        ) : null}
      </div>
    );
  }

  if (error) {
    return (
      <p className="text-body-sm text-error" role="alert">
        Could not {action === "currentAbility" ? "boost" : "update"} the squad.{" "}
        {error.message}
      </p>
    );
  }

  return null;
}

function BoostProgress({ progress }: { progress: SquadPlayerBoostProgress }) {
  return (
    <div className="space-y-2 text-body-sm text-on-surface-variant">
      {progress.total > 0 ? (
        <progress
          aria-label="Squad boost progress"
          className="w-full"
          max={progress.total}
          value={progress.processed}
        />
      ) : null}
      <p>
        {progress.processed} of {progress.total} players processed.
      </p>
    </div>
  );
}

export function SquadCurrentAbilityBoost(props: SquadCurrentAbilityBoostProps) {
  return <SquadPlayerBoost action="currentAbility" {...props} />;
}

export function SquadWonderkidMentalityBoost(
  props: SquadWonderkidMentalityBoostProps,
) {
  return <SquadPlayerBoost action="wonderkidMentality" {...props} />;
}

function SquadPlayerBoost({
  action,
  pending,
  disabled,
  error,
  onBoost,
  onOpenConfirmation,
  onConfirmationChange,
  fallbackFocusTo,
}: SquadPlayerBoostProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [progress, setProgress] = useState<SquadPlayerBoostProgress | null>(
    null,
  );
  const isCurrentAbility = action === "currentAbility";
  const actionLabel = isCurrentAbility ? "Boost all CA" : "Make all Wonderkids";
  const loadingLabel = isCurrentAbility ? "Boosting…" : "Applying…";
  const confirmationTitle = isCurrentAbility
    ? "Boost all CA?"
    : "Make all Wonderkids?";

  const confirm = () => {
    void onBoost(setProgress).then(
      () => {
        setConfirmationOpen(false);
        onConfirmationChange(false);
      },
      () => setProgress(null),
    );
  };

  return (
    <>
      <div className="space-y-2">
        <Button
          icon={isCurrentAbility ? ChevronsUp : Sprout}
          variant="secondary"
          disabled={pending || disabled}
          loading={pending}
          loadingLabel={loadingLabel}
          onClick={() => {
            onOpenConfirmation();
            onConfirmationChange(true);
            setProgress(null);
            setConfirmationOpen(true);
          }}
        >
          {actionLabel}
        </Button>
      </div>
      <Modal
        open={confirmationOpen}
        title={confirmationTitle}
        onClose={() => {
          if (!pending) {
            setConfirmationOpen(false);
            onConfirmationChange(false);
          }
        }}
        fallbackFocusTo={fallbackFocusTo}
        footer={
          <>
            <Button
              disabled={pending}
              variant="secondary"
              onClick={() => {
                setConfirmationOpen(false);
                onConfirmationChange(false);
              }}
            >
              Cancel
            </Button>
            <Button
              loading={pending}
              loadingLabel={loadingLabel}
              onClick={confirm}
            >
              {actionLabel}
            </Button>
          </>
        }
      >
        <div className="space-y-3">
          <p className="text-body-md text-on-surface-variant">
            {isCurrentAbility
              ? "Boosts the current ability of every eligible player at your club."
              : "Applies Wonderkid Mentality to every eligible player at your club."}{" "}
            Cannot be undone.
          </p>
          <details className="text-body-sm text-on-surface-variant">
            <summary className="cursor-pointer text-on-surface">
              How the boost is worked out
            </summary>
            <ul className="list-disc space-y-1 pt-2 pl-5">
              {isCurrentAbility ? (
                <>
                  <li>
                    Each player gains the gap between CA and PA, divided by the
                    seasons left until age 28 and rounded up.
                  </li>
                  <li>Players aged 27 or 28 count as one season left.</li>
                  <li>No player gains more than +20 CA.</li>
                  <li>Players aged 29 or older are skipped.</li>
                  <li>Run it once a season.</li>
                </>
              ) : (
                <>
                  <li>
                    Known Ambition, Professionalism, and Determination values at
                    10 or below can change.
                  </li>
                  <li>
                    Each eligible value receives a random value from 11 to 20.
                  </li>
                  <li>Unknown and higher values are unchanged.</li>
                </>
              )}
            </ul>
          </details>
          {pending ? (
            <div aria-live="polite">
              {progress ? (
                <BoostProgress progress={progress} />
              ) : (
                <p className="text-body-sm text-on-surface-variant">
                  Preparing squad…
                </p>
              )}
            </div>
          ) : null}
          <div aria-live="polite">
            <SquadBoostOutcome
              result={undefined}
              error={error}
              action={action}
            />
          </div>
        </div>
      </Modal>
    </>
  );
}
