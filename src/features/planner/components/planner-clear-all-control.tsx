import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { joinPlannerTeamNames } from "../utils/team-display";

type PlannerClearAllControlProps = {
  open: boolean;
  pending: boolean;
  disabled: boolean;
  error: string | null;
  teamNames: string[];
  onRequest: () => void;
  onFocus: () => void;
  onClose: () => void;
  onConfirm: () => void;
};

export function PlannerClearAllControl({
  open,
  pending,
  disabled,
  error,
  teamNames,
  onRequest,
  onFocus,
  onClose,
  onConfirm,
}: PlannerClearAllControlProps) {
  return (
    <>
      <Button
        variant="destructive"
        disabled={disabled}
        onClick={onRequest}
        onFocus={onFocus}
        data-planner-clear-all
        className="!h-7 !px-3 !text-label-md focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        Clear all
      </Button>
      <Modal
        open={open}
        title="Clear all squads?"
        variant="destructive"
        onClose={onClose}
        footer={
          <>
            <Button variant="secondary" disabled={pending} onClick={onClose}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              loading={pending}
              loadingLabel="Clearing…"
              onClick={onConfirm}
            >
              Clear all
            </Button>
          </>
        }
      >
        <p className="text-body-md text-on-surface-variant">
          This clears every assignment from {joinPlannerTeamNames(teamNames)}.
        </p>
        {error ? (
          <p className="mt-3 text-body-sm text-error" role="alert">
            {error}
          </p>
        ) : null}
      </Modal>
    </>
  );
}
