import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type { PlannerTeam } from "../types/club-family";

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

type PlannerClearTeamControlProps = {
  selectedTeam: PlannerTeam;
  target: PlannerTeam | null;
  open: boolean;
  pending: boolean;
  disabled: boolean;
  error: string | null;
  onRequest: () => void;
  onClose: () => void;
  onConfirm: (team: PlannerTeam) => void;
};

export function PlannerClearTeamControl({
  selectedTeam,
  target,
  open,
  pending,
  disabled,
  error,
  onRequest,
  onClose,
  onConfirm,
}: PlannerClearTeamControlProps) {
  return (
    <>
      <Button variant="destructive" disabled={disabled} onClick={onRequest}>
        Clear {TEAM_LABELS[selectedTeam]} squad
      </Button>
      {target ? (
        <Modal
          open={open}
          title={`Clear ${TEAM_LABELS[target]} squad?`}
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
                onClick={() => onConfirm(target)}
              >
                Clear {TEAM_LABELS[target]} squad
              </Button>
            </>
          }
        >
          <p className="text-body-md text-on-surface-variant">
            This clears every assignment from the {TEAM_LABELS[target]} squad.
          </p>
          {error ? (
            <p className="mt-3 text-body-sm text-error" role="alert">
              {error}
            </p>
          ) : null}
        </Modal>
      ) : null}
    </>
  );
}
