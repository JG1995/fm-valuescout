import { Sparkles, Zap } from "lucide-react";
import { useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type {
  SquadPlayerBoostProgress,
  SquadPlayerBoostResult,
} from "../types/squad-player-boost";

type SquadPlayerBoostAction = "currentAbility" | "wonderkidMentality";

type SquadPlayerBoostProps = {
  action: SquadPlayerBoostAction;
  pending: boolean;
  disabled: boolean;
  result: SquadPlayerBoostResult | undefined;
  error: Error | null;
  onBoost: (
    onProgress: (progress: SquadPlayerBoostProgress) => void,
  ) => Promise<unknown>;
  onOpenConfirmation: () => void;
};

type SquadCurrentAbilityBoostProps = Omit<SquadPlayerBoostProps, "action">;
type SquadWonderkidMentalityBoostProps = Omit<SquadPlayerBoostProps, "action">;

function resultSummary(result: SquadPlayerBoostResult) {
  return `Updated ${result.updated} ${result.updated === 1 ? "player" : "players"}. Skipped ${result.skipped}. Failed ${result.failed}.`;
}

function BoostOutcome({
  result,
  error,
  action,
}: Pick<SquadPlayerBoostProps, "result" | "error" | "action">) {
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
            Load Data is required before another boost.
            {result.recoveryMessage ? ` ${result.recoveryMessage}` : ""}
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
          className="h-2 w-full accent-primary"
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
  result,
  error,
  onBoost,
  onOpenConfirmation,
}: SquadPlayerBoostProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [progress, setProgress] = useState<SquadPlayerBoostProgress | null>(
    null,
  );
  const outcomeRef = useRef<HTMLDivElement>(null);
  const isCurrentAbility = action === "currentAbility";
  const actionLabel = isCurrentAbility ? "Boost all CA" : "Make all Wonderkids";
  const loadingLabel = isCurrentAbility ? "Boosting…" : "Applying…";
  const confirmationTitle = isCurrentAbility
    ? "Boost all CA?"
    : "Make all Wonderkids?";

  const confirm = () => {
    void onBoost(setProgress).then(
      () => setConfirmationOpen(false),
      () => setProgress(null),
    );
  };

  return (
    <>
      <div className="space-y-2">
        <Button
          icon={isCurrentAbility ? Zap : Sparkles}
          variant={isCurrentAbility ? "primary" : "secondary"}
          disabled={pending || disabled}
          loading={pending}
          loadingLabel={loadingLabel}
          onClick={() => {
            onOpenConfirmation();
            setProgress(null);
            setConfirmationOpen(true);
          }}
        >
          {actionLabel}
        </Button>
        <div
          ref={outcomeRef}
          tabIndex={-1}
          className="rounded-sm focus:outline-2 focus:outline-offset-2 focus:outline-primary"
          aria-live="polite"
        >
          {!confirmationOpen ? (
            <BoostOutcome result={result} error={error} action={action} />
          ) : null}
        </div>
      </div>
      <Modal
        open={confirmationOpen}
        title={confirmationTitle}
        onClose={() => {
          if (!pending) {
            setConfirmationOpen(false);
          }
        }}
        fallbackFocusTo={() => outcomeRef.current}
        footer={
          <>
            <Button
              disabled={pending}
              variant="secondary"
              onClick={() => setConfirmationOpen(false)}
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
            This{" "}
            {isCurrentAbility
              ? "boosts current ability for"
              : "applies Wonderkid Mentality to"}{" "}
            every eligible player in your configured club family, one player at
            a time.
          </p>
          {isCurrentAbility ? (
            <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface-variant">
              <li>Players aged 20 or younger receive +5 CA.</li>
              <li>Players aged 21 through 28 receive +10 CA.</li>
              <li>Players aged 29 or older are skipped.</li>
            </ul>
          ) : (
            <div className="space-y-1 text-body-sm text-on-surface-variant">
              <p>
                Known Ambition, Professionalism, and Determination values at 10
                or below can change.
              </p>
              <p>Each eligible value receives a random value from 11 to 20.</p>
              <p>Unknown and higher values are unchanged.</p>
            </div>
          )}
          <p className="text-body-sm text-on-surface-variant">
            Changes already applied cannot be undone.
          </p>
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
            <BoostOutcome result={undefined} error={error} action={action} />
          </div>
        </div>
      </Modal>
    </>
  );
}
