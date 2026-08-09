import { useRef, useState } from "react";
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

type MentalityAttribute = {
  label: "Ambition" | "Professionalism" | "Determination";
  value: number | null;
};

type WonderkidMentalityPreview = {
  attributes: MentalityAttribute[];
  eligible: MentalityAttribute[];
};

type BoostAction = "currentAbility" | "wonderkidMentality";

type PlayerDevelopmentBoostsPanelProps = {
  player: PlayerDetail;
  pending: boolean;
  result: PlayerBoostResult | undefined;
  error: Error | null;
  onBoostCurrentAbility: () => Promise<unknown>;
  onBoostWonderkidMentality: () => Promise<unknown>;
  onOpenConfirmation: () => void;
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

function knownMentalityValue(value: number | null | undefined) {
  return typeof value === "number" &&
    Number.isInteger(value) &&
    value >= 1 &&
    value <= 20
    ? value
    : null;
}

function wonderkidMentalityPreview(
  player: PlayerDetail,
): WonderkidMentalityPreview {
  const attributes: MentalityAttribute[] = [
    {
      label: "Ambition",
      value: knownMentalityValue(player.personality.Ambition),
    },
    {
      label: "Professionalism",
      value: knownMentalityValue(player.personality.Professionalism),
    },
    {
      label: "Determination",
      value: knownMentalityValue(player.attributes.Determination),
    },
  ];

  return {
    attributes,
    eligible: attributes.filter(
      (attribute) => attribute.value !== null && attribute.value <= 10,
    ),
  };
}

function mentalityPreviewLabel(attribute: MentalityAttribute) {
  if (attribute.value === null) {
    return `${attribute.label} unavailable → unchanged`;
  }
  if (attribute.value <= 10) {
    return `${attribute.label} ${attribute.value} → random 11–20`;
  }
  return `${attribute.label} ${attribute.value} → unchanged`;
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
  return "Could not apply boost";
}

function wonderkidResultSummary(result: PlayerBoostResult) {
  const changes = [
    ["Ambition", result.previousAmbition, result.ambition],
    ["Professionalism", result.previousProfessionalism, result.professionalism],
    ["Determination", result.previousDetermination, result.determination],
  ].flatMap(([label, previous, current]) =>
    typeof previous === "number" &&
    typeof current === "number" &&
    previous !== current
      ? [`${label} from ${previous} to ${current}`]
      : [],
  );

  return changes.length > 0
    ? `Wonderkid Mentality updated ${changes.join(", ")}.`
    : null;
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

  const wonderkidSummary =
    result?.operation === "wonderkid-mentality"
      ? wonderkidResultSummary(result)
      : null;
  if (wonderkidSummary) {
    return (
      <p className="text-body-sm text-success" role="status">
        {wonderkidSummary}
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
  onBoostWonderkidMentality,
  onOpenConfirmation,
}: PlayerDevelopmentBoostsPanelProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const [confirmationAction, setConfirmationAction] =
    useState<BoostAction>("currentAbility");
  const outcomeRef = useRef<HTMLDivElement>(null);
  const currentAbilityPreview = currentAbilityBoostPreview(player);
  const mentalityPreview = wonderkidMentalityPreview(player);
  const currentAbilityEligible = "target" in currentAbilityPreview;
  const mentalityEligible = mentalityPreview.eligible.length > 0;

  const openConfirmation = (action: BoostAction) => {
    onOpenConfirmation();
    setConfirmationAction(action);
    setConfirmationOpen(true);
  };

  const confirmBoost = () => {
    let request: Promise<unknown> | null = null;
    if (confirmationAction === "currentAbility" && currentAbilityEligible) {
      request = onBoostCurrentAbility();
    } else if (
      confirmationAction === "wonderkidMentality" &&
      mentalityEligible
    ) {
      request = onBoostWonderkidMentality();
    }
    if (request) {
      void request.then(
        () => setConfirmationOpen(false),
        () => undefined,
      );
    }
  };

  const currentAbilityConfirmation = confirmationAction === "currentAbility";

  return (
    <>
      <Panel title="Development boosts">
        <div className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div>
              <p className="text-body-md text-on-surface">
                Apply a fixed current-ability boost from this snapshot.
              </p>
              {currentAbilityEligible ? (
                <p className="mt-1 font-mono text-mono-sm text-on-surface-variant tabular-nums">
                  CA {player.ca} → {currentAbilityPreview.target} (+
                  {currentAbilityPreview.increase})
                  {currentAbilityPreview.cappedByPotential
                    ? " · capped by PA"
                    : ""}
                </p>
              ) : (
                <p className="mt-1 text-body-sm text-on-surface-variant">
                  {currentAbilityPreview.reason}
                </p>
              )}
            </div>
            <Button
              disabled={!currentAbilityEligible || pending}
              loading={pending && confirmationAction === "currentAbility"}
              loadingLabel="Boosting…"
              onClick={() => openConfirmation("currentAbility")}
            >
              Boost CA
            </Button>
          </div>
          <p className="text-body-sm text-on-surface-variant">
            FM may redistribute attributes over the next few in-game days.
          </p>
          <div className="border-t border-outline-variant pt-4">
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p className="text-body-md text-on-surface">
                  Let FM set each eligible mentality value to a random 11–20.
                </p>
                <ul className="mt-1 space-y-1 text-body-sm text-on-surface-variant">
                  {mentalityPreview.attributes.map((attribute) => (
                    <li key={attribute.label}>
                      {mentalityPreviewLabel(attribute)}
                    </li>
                  ))}
                </ul>
                {!mentalityEligible ? (
                  <p className="mt-1 text-body-sm text-on-surface-variant">
                    No known mentality attribute is 10 or lower.
                  </p>
                ) : null}
              </div>
              <Button
                disabled={!mentalityEligible || pending}
                loading={pending && confirmationAction === "wonderkidMentality"}
                loadingLabel="Applying…"
                variant="secondary"
                onClick={() => openConfirmation("wonderkidMentality")}
              >
                Wonderkid Mentality
              </Button>
            </div>
          </div>
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
      </Panel>
      <Modal
        open={confirmationOpen}
        title={
          currentAbilityConfirmation
            ? "Boost CA?"
            : "Apply Wonderkid Mentality?"
        }
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
              loadingLabel={
                currentAbilityConfirmation ? "Boosting…" : "Applying…"
              }
              onClick={confirmBoost}
            >
              {currentAbilityConfirmation
                ? "Boost CA"
                : "Apply Wonderkid Mentality"}
            </Button>
          </>
        }
      >
        {currentAbilityConfirmation ? (
          <div className="space-y-3">
            <p className="text-body-md text-on-surface-variant">
              This raises current ability from {player.ca} to{" "}
              {"target" in currentAbilityPreview
                ? currentAbilityPreview.target
                : player.ca}
              .
            </p>
            <p className="text-body-sm text-on-surface-variant">
              FM may redistribute attributes over the next few in-game days.
            </p>
            <div aria-live="polite">
              <BoostOutcome result={undefined} error={error} />
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            <p className="text-body-md text-on-surface-variant">
              FM assigns each eligible value a random number from 11 to 20.
            </p>
            <ul className="space-y-1 text-body-sm text-on-surface-variant">
              {mentalityPreview.attributes.map((attribute) => (
                <li key={attribute.label}>
                  {mentalityPreviewLabel(attribute)}
                </li>
              ))}
            </ul>
            <div aria-live="polite">
              <BoostOutcome result={undefined} error={error} />
            </div>
          </div>
        )}
      </Modal>
    </>
  );
}
