import { Zap } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { TauriCommandError } from "@/lib/tauri-client";
import type { StaffSummary } from "../types/staff-summary";

type StaffBoostPreview =
  | {
      target: number;
      increase: number;
      capped: boolean;
    }
  | { reason: string };

type StaffCaBoostProps = {
  staff: StaffSummary | undefined;
  pending: boolean;
  error: Error | null;
  onBoost: () => Promise<unknown>;
  onOpenConfirmation: () => void;
  fallbackFocusTo: () => HTMLElement | null;
};

function boostPreview(staff: StaffSummary | undefined): StaffBoostPreview {
  if (!staff) {
    return { reason: "Staff data is still loading." };
  }
  if (!Number.isInteger(staff.ca) || staff.ca < 1 || staff.ca > 200) {
    return {
      reason:
        "Current ability is unavailable. Load Data again to refresh this staff member.",
    };
  }
  if (!Number.isInteger(staff.pa) || staff.pa < 1 || staff.pa > 200) {
    return {
      reason:
        "Potential ability is unavailable. Load Data again to refresh this staff member.",
    };
  }
  if (staff.ca >= staff.pa) {
    return {
      reason:
        "Current ability is already at this staff member’s potential ability.",
    };
  }
  if (staff.ca >= 200) {
    return { reason: "Current ability is already at the maximum of 200." };
  }
  const target = Math.min(staff.ca + 10, staff.pa, 200);
  return {
    target,
    increase: target - staff.ca,
    capped: target < staff.ca + 10,
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
  return "Could not apply boost";
}

export function StaffCaBoost({
  staff,
  pending,
  error,
  onBoost,
  onOpenConfirmation,
  fallbackFocusTo,
}: StaffCaBoostProps) {
  const [confirmationOpen, setConfirmationOpen] = useState(false);
  const preview = boostPreview(staff);
  const eligible = "target" in preview;

  const confirm = () => {
    if (!eligible) return;
    void onBoost().then(
      () => setConfirmationOpen(false),
      () => undefined,
    );
  };

  return (
    <div className="flex min-w-28 items-center gap-2 px-2">
      <Button
        icon={Zap}
        disabled={!eligible || pending}
        loading={pending}
        loadingLabel="Boosting…"
        title={
          eligible
            ? `CA ${staff?.ca} → ${preview.target} (+${preview.increase})${preview.capped ? " · capped by PA" : ""}`
            : preview.reason
        }
        onClick={(event) => {
          event.stopPropagation();
          onOpenConfirmation();
          setConfirmationOpen(true);
        }}
        onKeyDown={(event) => event.stopPropagation()}
      >
        Boost CA
      </Button>
      <Modal
        open={confirmationOpen}
        title="Boost CA?"
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
              disabled={!eligible}
              onClick={confirm}
            >
              Boost CA
            </Button>
          </>
        }
      >
        <div className="space-y-3">
          {eligible ? (
            <p className="font-mono text-mono-sm text-on-surface tabular-nums">
              CA {staff?.ca} → {preview.target} (+{preview.increase})
              {preview.capped ? " · capped by PA" : ""}
            </p>
          ) : (
            <p className="text-body-md text-on-surface-variant">
              {preview.reason}
            </p>
          )}
          <p className="text-body-sm text-on-surface-variant">
            Staff CA boosts always add 10 and stop at PA or 200. Changes already
            applied cannot be undone.
          </p>
          {pending ? (
            <p
              className="text-body-sm text-on-surface-variant"
              aria-live="polite"
            >
              Updating Football Manager…
            </p>
          ) : null}
          {error ? (
            <p className="text-body-sm text-error" role="alert">
              {boostErrorTitle(error)}. {error.message}
            </p>
          ) : null}
        </div>
      </Modal>
    </div>
  );
}

export { boostPreview as staffCurrentAbilityBoostPreview };
