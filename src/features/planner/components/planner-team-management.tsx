import { useMutation } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import {
  type PlannerTeamSettingInput,
  savePlannerTeams,
} from "../api/save-planner-teams";
import {
  PLANNER_TEAM_NAMES,
  PLANNER_TEAMS,
  type PlannerTeam,
} from "../types/club-family";
import type { PlannerDepth } from "../types/depth";

type PlannerTeamDraft = PlannerTeamSettingInput & {
  included: boolean;
};

type RemovedTeam = {
  team: PlannerTeam;
  displayName: string;
  assignmentCount: number;
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

function assignmentCount(depth: PlannerDepth, team: PlannerTeam) {
  return (
    depth.teams
      .find((candidate) => candidate.team === team)
      ?.strings.reduce(
        (count, plannerString) => count + plannerString.assignments.length,
        0,
      ) ?? 0
  );
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

function removedTeams(
  depth: PlannerDepth,
  draft: PlannerTeamDraft[],
): RemovedTeam[] {
  return depth.teams
    .filter(
      (team) =>
        !draft.find((candidate) => candidate.team === team.team)?.included,
    )
    .map((team) => ({
      team: team.team,
      displayName: team.displayName,
      assignmentCount: assignmentCount(depth, team.team),
    }));
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
  const save = useMutation({
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
      const removed = removedTeams(depth, draft).map((team) => team.team);
      setConfirmRemoval(false);
      setOpen(false);
      onSaved(nextDepth, removed);
    },
    onSettled: () => {
      onPendingChange(false);
    },
  });

  const validation = useMemo(() => validateDraft(draft), [draft]);
  const pendingRemovalTeams = useMemo(
    () => removedTeams(depth, draft).filter((team) => team.assignmentCount > 0),
    [depth, draft],
  );
  const serverError = save.isError ? errorMessage(save.error) : null;

  useEffect(() => {
    if (open) {
      setDraft(draftFromDepth(depth));
      setConfirmRemoval(false);
      save.reset();
    }
  }, [depth, open, save.reset]);

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
    save.reset();
    setDraft((current) =>
      current.map((candidate) =>
        candidate.team === team ? { ...candidate, ...update } : candidate,
      ),
    );
  };

  const submit = (confirmPopulatedRemoval: boolean) => {
    if (!validation.valid) {
      return;
    }
    save.mutate({
      teams: draft
        .filter((team) => team.included)
        .map(({ team, displayName }) => ({
          team,
          displayName: displayName.trim(),
        })),
      confirmPopulatedRemoval,
    });
  };

  const requestSave = () => {
    if (!validation.valid) {
      return;
    }
    if (pendingRemovalTeams.length > 0) {
      setConfirmRemoval(true);
      return;
    }
    submit(false);
  };

  const close = () => {
    if (!save.isPending) {
      setConfirmRemoval(false);
      setReturnFocusToSave(false);
      setOpen(false);
    }
  };

  const leaveConfirmation = () => {
    if (!save.isPending) {
      setConfirmRemoval(false);
      setReturnFocusToSave(true);
    }
  };

  return (
    <>
      <Button
        variant="secondary"
        disabled={disabled || save.isPending}
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
                disabled={save.isPending}
                onClick={leaveConfirmation}
                data-planner-team-confirm-cancel
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                loading={save.isPending}
                loadingLabel="Removing…"
                onClick={() => submit(true)}
              >
                Remove teams
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="secondary"
                disabled={save.isPending}
                onClick={close}
              >
                Cancel
              </Button>
              <Button
                loading={save.isPending}
                loadingLabel="Saving…"
                disabled={!validation.valid}
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
              Removing these teams permanently deletes their assignments.
            </p>
            <ul className="list-disc space-y-1 pl-5 text-body-md text-on-surface">
              {pendingRemovalTeams.map((team) => (
                <li key={team.team}>
                  {team.displayName}: {team.assignmentCount} assignment
                  {team.assignmentCount === 1 ? "" : "s"}
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
                        disabled={soleTeam || maxTeams || save.isPending}
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
                        disabled={save.isPending}
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
