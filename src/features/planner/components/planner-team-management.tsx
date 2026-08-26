import { useMutation } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { fetchPlannerTeamRemovalImpacts } from "../api/fetch-planner-team-removal-impacts";
import {
  type PlannerTeamSettingInput,
  savePlannerTeams,
} from "../api/save-planner-teams";
import type { PlannerDepth } from "../types/depth";
import {
  PLANNER_TEAM_NAMES,
  PLANNER_TEAMS,
  type PlannerTeam,
} from "../types/team";
import type { PlannerTeamRemovalImpact } from "../types/team-removal-impact";

type PlannerTeamDraft = PlannerTeamSettingInput & {
  included: boolean;
};

type PlannerTeamManagementProps = {
  depth: PlannerDepth;
  disabled?: boolean;
  onPendingChange: (pending: boolean) => void;
  onSaved: (nextDepth: PlannerDepth, removedTeams: PlannerTeam[]) => void;
};

function draftFromDepth(depth: PlannerDepth): PlannerTeamDraft[] {
  return PLANNER_TEAMS.map((team) => {
    const current = depth.teams.find((candidate) => candidate.team === team);
    return {
      team,
      included: current !== undefined,
      displayName: current?.displayName ?? PLANNER_TEAM_NAMES[team],
    };
  });
}

function validateDraft(draft: PlannerTeamDraft[]) {
  const included = draft.filter((team) => team.included);
  const fieldErrors: Partial<Record<PlannerTeam, string>> = {};
  const names = new Map<string, PlannerTeam[]>();

  for (const team of included) {
    const name = team.displayName.trim();
    if (!name) {
      fieldErrors[team.team] = "Enter a team name";
    } else if ([...name].length > 40) {
      fieldErrors[team.team] = "Use 40 characters or fewer";
    }
    const key = name.toLowerCase();
    const matchingTeams = names.get(key) ?? [];
    matchingTeams.push(team.team);
    names.set(key, matchingTeams);
  }

  for (const matchingTeams of names.values()) {
    if (matchingTeams.length < 2 || !matchingTeams[0]) {
      continue;
    }
    for (const team of matchingTeams) {
      if (!fieldErrors[team]) {
        fieldErrors[team] = "Team names must be unique";
      }
    }
  }

  return {
    fieldErrors,
    includedCount: included.length,
    valid: included.length > 0 && Object.keys(fieldErrors).length === 0,
  };
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function PlannerTeamManagement({
  depth,
  disabled = false,
  onPendingChange,
  onSaved,
}: PlannerTeamManagementProps) {
  const [open, setOpen] = useState(false);
  const [confirmRemoval, setConfirmRemoval] = useState(false);
  const [returnFocusToSave, setReturnFocusToSave] = useState(false);
  const [draft, setDraft] = useState(() => draftFromDepth(depth));
  const [removalImpacts, setRemovalImpacts] = useState<
    PlannerTeamRemovalImpact[]
  >([]);
  const [removalImpactError, setRemovalImpactError] = useState<string | null>(
    null,
  );
  const [checkingRemovalImpact, setCheckingRemovalImpact] = useState(false);
  const [previewedTeams, setPreviewedTeams] = useState<
    PlannerTeamSettingInput[] | null
  >(null);
  const removalImpactRequest = useRef(0);
  const save = useMutation({
    mutationKey: playerResultContextMutationKey,
    mutationFn: ({
      teams,
      confirmPopulatedRemoval,
    }: {
      teams: PlannerTeamSettingInput[];
      confirmPopulatedRemoval: boolean;
    }) => savePlannerTeams(teams, confirmPopulatedRemoval),
    onMutate: () => {
      onPendingChange(true);
    },
    onSuccess: (nextDepth) => {
      const removed = depth.teams
        .filter(
          (team) =>
            !draft.find((candidate) => candidate.team === team.team)?.included,
        )
        .map((team) => team.team);
      removalImpactRequest.current += 1;
      setConfirmRemoval(false);
      setPreviewedTeams(null);
      setOpen(false);
      onSaved(nextDepth, removed);
    },
    onSettled: () => {
      onPendingChange(false);
    },
  });

  const validation = useMemo(() => validateDraft(draft), [draft]);
  const serverError = save.isError
    ? errorMessage(save.error)
    : removalImpactError;

  useEffect(() => {
    removalImpactRequest.current += 1;
    if (open) {
      setDraft(draftFromDepth(depth));
      setConfirmRemoval(false);
      setPreviewedTeams(null);
      setRemovalImpacts([]);
      setRemovalImpactError(null);
      setCheckingRemovalImpact(false);
      save.reset();
    }
  }, [depth, open, save.reset]);

  useEffect(
    () => () => {
      removalImpactRequest.current += 1;
    },
    [],
  );

  useEffect(() => {
    if (!open || !confirmRemoval) {
      return;
    }
    document
      .querySelector<HTMLButtonElement>("[data-planner-team-confirm-cancel]")
      ?.focus();
  }, [confirmRemoval, open]);

  useEffect(() => {
    if (!open || confirmRemoval || !returnFocusToSave) {
      return;
    }
    document
      .querySelector<HTMLButtonElement>("[data-planner-team-save]")
      ?.focus();
    setReturnFocusToSave(false);
  }, [confirmRemoval, open, returnFocusToSave]);

  const updateDraft = (
    team: PlannerTeam,
    update: Partial<PlannerTeamDraft>,
  ) => {
    removalImpactRequest.current += 1;
    save.reset();
    setCheckingRemovalImpact(false);
    setPreviewedTeams(null);
    setRemovalImpacts([]);
    setRemovalImpactError(null);
    setDraft((current) =>
      current.map((candidate) =>
        candidate.team === team ? { ...candidate, ...update } : candidate,
      ),
    );
  };

  const inputs = () =>
    draft
      .filter((team) => team.included)
      .map(({ team, displayName }) => ({
        team,
        displayName: displayName.trim(),
      }));

  const submit = (
    teams: PlannerTeamSettingInput[],
    confirmPopulatedRemoval: boolean,
  ) => {
    save.mutate({ teams, confirmPopulatedRemoval });
  };

  const requestSave = async () => {
    if (!validation.valid || checkingRemovalImpact) {
      return;
    }
    const teams = inputs();
    const request = ++removalImpactRequest.current;
    setRemovalImpactError(null);
    setCheckingRemovalImpact(true);
    try {
      const impacts = await fetchPlannerTeamRemovalImpacts(teams);
      if (request !== removalImpactRequest.current) {
        return;
      }
      setRemovalImpacts(impacts);
      setPreviewedTeams(teams);
      if (
        impacts.some(
          (impact) =>
            impact.assignmentCount > 0 || impact.staffingTargets.length > 0,
        )
      ) {
        setConfirmRemoval(true);
        return;
      }
      submit(teams, false);
    } catch (error) {
      if (request === removalImpactRequest.current) {
        setRemovalImpactError(errorMessage(error));
      }
    } finally {
      if (request === removalImpactRequest.current) {
        setCheckingRemovalImpact(false);
      }
    }
  };

  const close = () => {
    if (!save.isPending && !checkingRemovalImpact) {
      removalImpactRequest.current += 1;
      setConfirmRemoval(false);
      setPreviewedTeams(null);
      setReturnFocusToSave(false);
      setOpen(false);
    }
  };

  const leaveConfirmation = () => {
    if (!save.isPending && !checkingRemovalImpact) {
      removalImpactRequest.current += 1;
      setConfirmRemoval(false);
      setPreviewedTeams(null);
      setRemovalImpacts([]);
      setReturnFocusToSave(true);
    }
  };

  return (
    <>
      <Button
        variant="secondary"
        disabled={disabled || save.isPending || checkingRemovalImpact}
        onClick={() => setOpen(true)}
        data-planner-manage-teams
        className="!h-7 !px-3 !text-label-md focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        Manage teams
      </Button>
      <Modal
        open={open}
        title={confirmRemoval ? "Remove planner teams?" : "Manage squad teams"}
        variant={confirmRemoval ? "destructive" : "form"}
        onClose={confirmRemoval ? leaveConfirmation : close}
        footer={
          confirmRemoval ? (
            <>
              <Button
                variant="secondary"
                disabled={save.isPending || checkingRemovalImpact}
                onClick={leaveConfirmation}
                data-planner-team-confirm-cancel
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                loading={save.isPending}
                loadingLabel="Removing…"
                disabled={checkingRemovalImpact || !previewedTeams}
                onClick={() => {
                  if (previewedTeams) {
                    submit(previewedTeams, true);
                  }
                }}
              >
                Remove teams
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="secondary"
                disabled={save.isPending || checkingRemovalImpact}
                onClick={close}
              >
                Cancel
              </Button>
              <Button
                loading={save.isPending || checkingRemovalImpact}
                loadingLabel={checkingRemovalImpact ? "Checking…" : "Saving…"}
                disabled={!validation.valid || checkingRemovalImpact}
                onClick={requestSave}
                data-planner-team-save
              >
                Save teams
              </Button>
            </>
          )
        }
      >
        {confirmRemoval ? (
          <div className="space-y-3">
            <p className="text-body-md text-on-surface-variant">
              Removing these teams permanently deletes their assignments and
              staffing targets.
            </p>
            <ul className="list-disc space-y-1 pl-5 text-body-md text-on-surface">
              {removalImpacts.map((team) => (
                <li key={team.team}>
                  {team.displayName}:{" "}
                  {team.assignmentCount > 0 ? (
                    <>
                      {team.assignmentCount} assignment
                      {team.assignmentCount === 1 ? "" : "s"}
                    </>
                  ) : null}
                  {team.assignmentCount > 0 && team.staffingTargets.length > 0
                    ? "; "
                    : null}
                  {team.staffingTargets.map((target, index) => (
                    <span key={target.jobId}>
                      {index > 0 ? ", " : null}
                      {target.jobLabel}: {target.slotCount} slot
                      {target.slotCount === 1 ? "" : "s"}
                    </span>
                  ))}
                </li>
              ))}
            </ul>
            {serverError ? (
              <p className="text-body-sm text-error" role="alert">
                {serverError}
              </p>
            ) : null}
          </div>
        ) : (
          <form
            id="planner-team-management-form"
            className="space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              requestSave();
            }}
          >
            <p className="text-body-md text-on-surface-variant">
              Choose one to three teams and set the names shown in Planner.
            </p>
            <fieldset className="space-y-3">
              <legend className="text-label-lg text-on-surface">
                Available teams
              </legend>
              {draft.map((team) => {
                const selectedCount = validation.includedCount;
                const soleTeam = team.included && selectedCount === 1;
                const maxTeams =
                  !team.included && selectedCount === PLANNER_TEAMS.length;
                return (
                  <div
                    key={team.team}
                    className="space-y-2 rounded-lg border border-outline-variant p-3"
                  >
                    <label className="flex items-center gap-2 text-body-md text-on-surface">
                      <input
                        type="checkbox"
                        checked={team.included}
                        disabled={
                          soleTeam ||
                          maxTeams ||
                          save.isPending ||
                          checkingRemovalImpact
                        }
                        onChange={(event) =>
                          updateDraft(team.team, {
                            included: event.target.checked,
                          })
                        }
                      />
                      <span>{PLANNER_TEAM_NAMES[team.team]}</span>
                    </label>
                    {team.included ? (
                      <TextField
                        label={`${PLANNER_TEAM_NAMES[team.team]} display name`}
                        value={team.displayName}
                        error={validation.fieldErrors[team.team]}
                        disabled={save.isPending || checkingRemovalImpact}
                        onChange={(event) =>
                          updateDraft(team.team, {
                            displayName: event.target.value,
                          })
                        }
                      />
                    ) : null}
                  </div>
                );
              })}
            </fieldset>
            {validation.includedCount === 0 ? (
              <p className="text-body-sm text-error" role="alert">
                Keep at least one team selected.
              </p>
            ) : null}
            {serverError ? (
              <p className="text-body-sm text-error" role="alert">
                {serverError}
              </p>
            ) : null}
          </form>
        )}
      </Modal>
    </>
  );
}
