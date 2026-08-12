import { Zap } from "lucide-react";
import { useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type { SquadCurrentAbilityBoostResult } from "../types/squad-current-ability-boost";

type SquadCurrentAbilityBoostProps = {
  pending: boolean;
  result: SquadCurrentAbilityBoostResult | undefined;
  error: Error | null;
  onBoost: () => Promise<unknown>;
  onOpenConfirmation: () => void;
};

function resultSummary(result: SquadCurrentAbilityBoostResult) {
  return `Updated ${result.updated} ${result.updated === 1 ? "player" : "players"}. Skipped ${result.skipped}. Failed ${result.failed}.`;
}

function BoostOutcome({
  result,
  error,
}: Pick<SquadCurrentAbilityBoostProps, "result" | "error">) {
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
        Could not boost the squad. {error.message}
      </p>
    );
  }

  return null;
}

export function SquadCurrentAbilityBoost({
  pending,
  result,
  error,
  onBoost,
  onOpenConfirmation,
}: SquadCurrentAbilityBoostProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const outcomeRef = useRef<HTMLDivElement>(null);

  const confirm = () => {
    void onBoost().then(
      () => setConfirmationOpen(false),
      () => undefined,
    );
  };

  return (
    <>
      <div className="space-y-2">
        <Button
          icon={Zap}
          disabled={pending || result?.recoveryRequired}
          loading={pending}
          loadingLabel="Boosting…"
          onClick={() => {
            onOpenConfirmation();
            setConfirmationOpen(true);
          }}
        >
          Boost all CA
        </Button>
        <div
          ref={outcomeRef}
          tabIndex={-1}
          className="rounded-sm focus:outline-2 focus:outline-offset-2 focus:outline-primary"
          aria-live="polite"
        >
          {!confirmationOpen ? (
            <BoostOutcome result={result} error={error} />
          ) : null}
        </div>
      </div>
      <Modal
        open={confirmationOpen}
        title="Boost all CA?"
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
              loadingLabel="Boosting…"
              onClick={confirm}
            >
              Boost all CA
            </Button>
          </>
        }
      >
        <div className="space-y-3">
          <p className="text-body-md text-on-surface-variant">
            This boosts every eligible player in your configured club family,
            one player at a time.
          </p>
          <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface-variant">
            <li>Players aged 20 or younger receive +5 CA.</li>
            <li>Players aged 21 through 28 receive +10 CA.</li>
            <li>Players aged 29 or older are skipped.</li>
          </ul>
          <p className="text-body-sm text-on-surface-variant">
            Changes already applied cannot be undone.
          </p>
          <div aria-live="polite">
            <BoostOutcome result={undefined} error={error} />
          </div>
        </div>
      </Modal>
    </>
  );
}
