import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import { TauriCommandError } from "@/lib/tauri-client";
import type { PlayerBoostResult } from "../types/player-boost";
import type { PlayerDetail } from "../types/player-detail";

type CurrentAbilityBoostPreview =
  | {
      target: number;
      increase: number;
      cappedByPotential: boolean;
    }
  | { reason: string };

type PlayerDevelopmentBoostsPanelProps = {
  player: PlayerDetail;
  pending: boolean;
  result: PlayerBoostResult | undefined;
  error: Error | null;
  onBoostCurrentAbility: () => Promise<unknown>;
};

function currentAbilityBoostPreview(
  player: PlayerDetail,
): CurrentAbilityBoostPreview {
  if (player.age === null || !Number.isInteger(player.age) || player.age < 0) {
    return {
      reason: "Age is unavailable. Load Data again to refresh this player.",
    };
  }
  const potentialAbility = player.pa;
  if (
    typeof potentialAbility !== "number" ||
    !Number.isInteger(potentialAbility) ||
    potentialAbility < 1 ||
    potentialAbility > 200
  ) {
    return {
      reason:
        "Potential ability is unavailable. Load Data again to refresh this player.",
    };
  }
  if (!Number.isInteger(player.ca) || player.ca < 1 || player.ca > 200) {
    return {
      reason:
        "Current ability is unavailable. Load Data again to refresh this player.",
    };
  }
  if (player.ca >= 200) {
    return { reason: "Current ability is already at the maximum of 200." };
  }
  if (player.ca >= potentialAbility) {
    return {
      reason: "Current ability is already at this player’s potential ability.",
    };
  }

  const increment = player.age <= 21 ? 5 : 10;
  const target = Math.min(player.ca + increment, potentialAbility, 200);
  return {
    target,
    increase: target - player.ca,
    cappedByPotential: target < player.ca + increment,
  };
}

function boostErrorTitle(error: Error) {
  if (error instanceof TauriCommandError) {
    switch (error.phase) {
      case "eligibility":
        return "Boost unavailable";
      case "liveValue":
        return "FM values changed";
      case "snapshotSync":
        return "Load Data required";
      case "bridge":
        return "Bridge rejected the boost";
      default:
        break;
    }
  }
  return "Could not boost CA";
}

function BoostOutcome({
  result,
  error,
}: Pick<PlayerDevelopmentBoostsPanelProps, "result" | "error">) {
  if (
    result?.operation === "boost-current-ability" &&
    result.previousCurrentAbility !== null &&
    result.currentAbility !== null
  ) {
    return (
      <p className="text-body-sm text-success" role="status">
        CA boosted from {result.previousCurrentAbility} to{" "}
        {result.currentAbility}.
      </p>
    );
  }

  if (error) {
    return (
      <p className="text-body-sm text-error" role="alert">
        {boostErrorTitle(error)}. {error.message}
      </p>
    );
  }

  return null;
}

export function PlayerDevelopmentBoostsPanel({
  player,
  pending,
  result,
  error,
  onBoostCurrentAbility,
}: PlayerDevelopmentBoostsPanelProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const preview = currentAbilityBoostPreview(player);
  const eligible = "target" in preview;

  const confirmBoost = () => {
    if (!eligible) {
      return;
    }
    void onBoostCurrentAbility().then(
      () => setConfirmationOpen(false),
      () => undefined,
    );
  };

  return (
    <>
      <Panel title="Development boosts">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div>
              <p className="text-body-md text-on-surface">
                Apply a fixed current-ability boost from this snapshot.
              </p>
              {eligible ? (
                <p className="mt-1 font-mono text-mono-sm text-on-surface-variant tabular-nums">
                  CA {player.ca} → {preview.target} (+{preview.increase})
                  {preview.cappedByPotential ? " · capped by PA" : ""}
                </p>
              ) : (
                <p className="mt-1 text-body-sm text-on-surface-variant">
                  {preview.reason}
                </p>
              )}
            </div>
            <Button
              disabled={!eligible || pending}
              loading={pending}
              loadingLabel="Boosting…"
              onClick={() => setConfirmationOpen(true)}
            >
              Boost CA
            </Button>
          </div>
          <p className="text-body-sm text-on-surface-variant">
            FM may redistribute attributes over the next few in-game days.
          </p>
          <div aria-live="polite">
            {!confirmationOpen ? (
              <BoostOutcome result={result} error={error} />
            ) : null}
          </div>
        </div>
      </Panel>
      {eligible ? (
        <Modal
          open={confirmationOpen}
          title="Boost CA?"
          onClose={() => {
            if (!pending) {
              setConfirmationOpen(false);
            }
          }}
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
                onClick={confirmBoost}
              >
                Boost CA
              </Button>
            </>
          }
        >
          <div className="space-y-3">
            <p className="text-body-md text-on-surface-variant">
              This raises current ability from {player.ca} to {preview.target}.
            </p>
            <p className="text-body-sm text-on-surface-variant">
              FM may redistribute attributes over the next few in-game days.
            </p>
            <div aria-live="polite">
              <BoostOutcome result={undefined} error={error} />
            </div>
          </div>
        </Modal>
      ) : null}
    </>
  );
}
