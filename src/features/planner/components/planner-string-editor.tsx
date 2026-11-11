import { useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button/button";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import type { PlannerStringSettingInput } from "../api/save-planner-teams";
import type { PlannerDepthTeam, PlannerString } from "../types/depth";
import { ordinal } from "../utils/string-label";

const MAX_NAME_LENGTH = 40;

function nameError(name: string, otherNames: string[]) {
  const trimmed = name.trim();
  if (trimmed === "") {
    return "Enter a string name";
  }
  if ([...trimmed].length > MAX_NAME_LENGTH) {
    return `Use ${MAX_NAME_LENGTH} characters or fewer`;
  }
  return otherNames.some(
    (other) => other.toLowerCase() === trimmed.toLowerCase(),
  )
    ? "String names must be unique"
    : undefined;
}

/**
 * Renames, reorders, or removes one string of a team. Removing a string that
 * holds players asks first.
 */
export function PlannerStringEditor({
  open,
  team,
  plannerString,
  pending,
  error,
  onClose,
  onSave,
}: {
  open: boolean;
  team: PlannerDepthTeam;
  plannerString: PlannerString;
  pending: boolean;
  error: string | null;
  onClose: () => void;
  /** Saves the team's complete string list in its new order. */
  onSave: (
    strings: PlannerStringSettingInput[],
    confirmPopulatedRemoval: boolean,
  ) => void;
}) {
  const positionId = useId();
  const strings = [...team.strings].sort(
    (left, right) => left.stringOrder - right.stringOrder,
  );
  const currentPosition = strings.findIndex(
    (candidate) => candidate.id === plannerString.id,
  );
  const [name, setName] = useState(plannerString.displayName);
  const [position, setPosition] = useState(currentPosition);
  const [confirmRemoval, setConfirmRemoval] = useState(false);

  useEffect(() => {
    if (open) {
      setName(plannerString.displayName);
      setPosition(currentPosition);
      setConfirmRemoval(false);
    }
  }, [open, plannerString.displayName, currentPosition]);

  const others = strings.filter(
    (candidate) => candidate.id !== plannerString.id,
  );
  const invalid = nameError(
    name,
    others.map((candidate) => candidate.displayName),
  );
  const assignmentCount = plannerString.assignments.length;
  const input = (candidate: PlannerString): PlannerStringSettingInput => ({
    id: candidate.id,
    displayName: candidate.displayName,
  });

  const save = () => {
    if (invalid || pending) {
      return;
    }
    const next = others.map(input);
    next.splice(position, 0, {
      id: plannerString.id,
      displayName: name.trim(),
    });
    onSave(next, false);
  };
  const remove = () => {
    if (!pending) {
      onSave(others.map(input), assignmentCount > 0);
    }
  };

  return (
    <Modal
      open={open}
      title={
        confirmRemoval
          ? `Remove ${plannerString.displayName}?`
          : `Edit ${team.displayName} string`
      }
      variant={confirmRemoval ? "destructive" : "form"}
      onClose={() => {
        if (!pending) onClose();
      }}
      footer={
        confirmRemoval ? (
          <>
            <Button
              variant="secondary"
              disabled={pending}
              onClick={() => setConfirmRemoval(false)}
            >
              Back
            </Button>
            <Button
              variant="destructive"
              loading={pending}
              loadingLabel="Removing…"
              onClick={remove}
            >
              Remove string
            </Button>
          </>
        ) : (
          <>
            <Button
              variant="secondary"
              disabled={pending || others.length === 0}
              title={
                others.length === 0
                  ? "A team keeps at least one string"
                  : undefined
              }
              onClick={() =>
                assignmentCount > 0 ? setConfirmRemoval(true) : remove()
              }
            >
              Remove string
            </Button>
            <Button variant="secondary" disabled={pending} onClick={onClose}>
              Cancel
            </Button>
            <Button
              loading={pending}
              loadingLabel="Saving…"
              disabled={invalid !== undefined}
              onClick={save}
            >
              Save string
            </Button>
          </>
        )
      }
    >
      {confirmRemoval ? (
        <p className="text-body-md text-on-surface-variant">
          Removing {plannerString.displayName} from {team.displayName}{" "}
          permanently deletes its {assignmentCount} assignment
          {assignmentCount === 1 ? "" : "s"}.
        </p>
      ) : (
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            save();
          }}
        >
          <TextField
            label="String name"
            value={name}
            error={invalid}
            disabled={pending}
            onChange={(event) => setName(event.target.value)}
          />
          <div className="space-y-1">
            <label className={fieldLabelClasses} htmlFor={positionId}>
              Position in {team.displayName}
            </label>
            <select
              id={positionId}
              className={`${fieldClasses} w-full`}
              value={position}
              disabled={pending || strings.length === 1}
              onChange={(event) => setPosition(Number(event.target.value))}
            >
              {strings.map((candidate, index) => (
                <option key={candidate.id} value={index}>
                  {ordinal(index)}
                </option>
              ))}
            </select>
          </div>
          {error ? (
            <p className="text-body-sm text-error" role="alert">
              {error}
            </p>
          ) : null}
        </form>
      )}
    </Modal>
  );
}
