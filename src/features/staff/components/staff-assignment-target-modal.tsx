import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { saveStaffAssignmentTargets } from "../api/save-staff-assignment-targets";
import { staffAssignmentTargetsQueryOptions } from "../api/staff-assignment-targets-query-options";
import { staffKeys } from "../api/staff-keys";
import type {
  StaffAssignmentContext,
  StaffAssignmentSection,
  StaffAssignmentTarget,
  StaffAssignmentTargetInput,
} from "../types/staff-assignment";

type DraftTarget = Pick<StaffAssignmentTarget, "scope" | "jobId"> & {
  slotCount: string;
};

type SaveTargetRequest = {
  contextKey: string;
  generation: number;
  saveContextToken: string;
  targets: StaffAssignmentTargetInput[];
};

type StaffAssignmentTargetModalProps = {
  context: StaffAssignmentContext;
  contextKey: string;
  onSaved?: () => void;
  onPendingChange?: (pending: boolean) => void;
};

function draftKey(target: Pick<DraftTarget, "scope" | "jobId">) {
  return `${target.scope}:${target.jobId}`;
}

function draftFromTargets(targets: StaffAssignmentTarget[]): DraftTarget[] {
  return targets.map(({ scope, jobId, slotCount }) => ({
    scope,
    jobId,
    slotCount: String(slotCount),
  }));
}

const TARGET_SECTIONS: { id: StaffAssignmentSection; label: string }[] = [
  { id: "coaching", label: "Coaching" },
  { id: "recruitment", label: "Recruitment" },
  { id: "medical", label: "Medical" },
];

function slotCountError(value: string, maxSlotCount: number) {
  const message = `Enter a whole number from 0 to ${maxSlotCount}.`;
  if (!/^\d+$/.test(value)) {
    return message;
  }
  const count = Number(value);
  return count >= 0 && count <= maxSlotCount ? undefined : message;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function StaffAssignmentTargetModal({
  context,
  contextKey,
  onSaved,
  onPendingChange,
}: StaffAssignmentTargetModalProps) {
  const queryClient = useQueryClient();
  const targetsQuery = useQuery(
    staffAssignmentTargetsQueryOptions(context, contextKey),
  );
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<DraftTarget[]>([]);
  const [saved, setSaved] = useState(false);
  const currentContextKey = useRef(contextKey);
  const previousContextKey = useRef(contextKey);
  const requestGeneration = useRef(0);
  currentContextKey.current = contextKey;

  const isCurrentRequest = (request: SaveTargetRequest) =>
    request.contextKey === currentContextKey.current &&
    request.generation === requestGeneration.current;

  const saveTargets = useMutation({
    onMutate: (request: SaveTargetRequest) => {
      if (isCurrentRequest(request)) {
        onPendingChange?.(true);
      }
    },
    mutationFn: ({ saveContextToken, targets }: SaveTargetRequest) =>
      saveStaffAssignmentTargets(saveContextToken, targets),
    onSuccess: async (result, request) => {
      if (!isCurrentRequest(request)) {
        return;
      }
      queryClient.setQueryData(
        staffKeys.assignmentTargets(request.contextKey),
        result,
      );
      await queryClient.invalidateQueries({
        queryKey: staffKeys.assignmentTargets(request.contextKey),
      });
      if (!isCurrentRequest(request)) {
        return;
      }
      setSaved(true);
      setOpen(false);
      setDraft([]);
      onSaved?.();
    },
    onSettled: (_result, _error, request) => {
      if (isCurrentRequest(request)) {
        onPendingChange?.(false);
      }
    },
  });

  useEffect(() => {
    if (previousContextKey.current === contextKey) {
      return;
    }
    previousContextKey.current = contextKey;
    requestGeneration.current += 1;
    onPendingChange?.(false);
    setOpen(false);
    setDraft([]);
    setSaved(false);
    saveTargets.reset();
  }, [contextKey, onPendingChange, saveTargets.reset]);

  const pending = saveTargets.isPending;
  const errors = new Map(
    (targetsQuery.data?.targets ?? []).map((target) => {
      const key = draftKey(target);
      const draftTarget = draft.find(
        (candidate) => draftKey(candidate) === key,
      );
      return [
        key,
        slotCountError(draftTarget?.slotCount ?? "", target.maxSlotCount),
      ];
    }),
  );
  const canSave =
    targetsQuery.isSuccess &&
    draft.length === targetsQuery.data.targets.length &&
    [...errors.values()].every((error) => error === undefined) &&
    !pending;

  const openModal = () => {
    if (!targetsQuery.data || targetsQuery.isFetching || targetsQuery.isError) {
      return;
    }
    setSaved(false);
    saveTargets.reset();
    setDraft(draftFromTargets(targetsQuery.data.targets));
    setOpen(true);
  };

  const closeModal = () => {
    if (pending) {
      return;
    }
    setOpen(false);
    setDraft([]);
    saveTargets.reset();
  };

  const updateTarget = (key: string, slotCount: string) => {
    if (pending) {
      return;
    }
    saveTargets.reset();
    setSaved(false);
    setDraft((current) =>
      current.map((target) =>
        draftKey(target) === key ? { ...target, slotCount } : target,
      ),
    );
  };

  const save = () => {
    if (!canSave) {
      return;
    }
    saveTargets.mutate({
      contextKey,
      generation: requestGeneration.current,
      saveContextToken: context.saveContextToken,
      targets: draft.map(({ scope, jobId, slotCount }) => ({
        scope,
        jobId,
        slotCount: Number(slotCount),
      })),
    });
  };

  const formError = saveTargets.isError
    ? errorMessage(saveTargets.error)
    : targetsQuery.isError
      ? errorMessage(targetsQuery.error)
      : null;
  const targetGroups = targetsQuery.data
    ? [
        ...targetsQuery.data.teams.map((team) => ({
          scope: team.team,
          title: team.displayName,
          targets: targetsQuery.data.targets.filter(
            (target) =>
              target.scope === team.team ||
              (team.team === "senior" && target.scope === "club"),
          ),
        })),
        ...(!targetsQuery.data.teams.some(({ team }) => team === "senior")
          ? [
              {
                scope: "club" as const,
                title: "Club",
                targets: targetsQuery.data.targets.filter(
                  (target) => target.scope === "club",
                ),
              },
            ]
          : []),
      ]
    : [];

  return (
    <>
      <Button
        variant="secondary"
        disabled={targetsQuery.isFetching || targetsQuery.isError || pending}
        onClick={openModal}
      >
        Configure Club Staff
      </Button>
      {saved ? (
        <p role="status" className="text-body-sm text-success">
          Slot counts saved.
        </p>
      ) : null}
      {targetsQuery.isError && !open ? (
        <p role="alert" className="text-body-sm text-error">
          {formError}
        </p>
      ) : null}
      <Modal
        open={open}
        title="Configure assignment slots"
        onClose={closeModal}
        footer={
          <>
            <Button variant="secondary" disabled={pending} onClick={closeModal}>
              Cancel
            </Button>
            <Button
              loading={pending}
              loadingLabel="Saving…"
              disabled={!canSave}
              onClick={save}
            >
              Save slots
            </Button>
          </>
        }
      >
        <form
          className="space-y-5"
          onSubmit={(event) => {
            event.preventDefault();
            save();
          }}
        >
          <p className="text-body-md text-on-surface-variant">
            Set the required slots for each available staff role.
          </p>
          {formError ? (
            <p role="alert" className="text-body-sm text-error">
              {formError}
            </p>
          ) : null}
          <div className="space-y-5">
            {targetGroups.map((group) => (
              <fieldset key={group.scope} className="space-y-4">
                <legend className="text-label-lg text-on-surface">
                  {group.title}
                </legend>
                {TARGET_SECTIONS.map((section) => {
                  const sectionTargets = group.targets.filter(
                    (target) => target.section === section.id,
                  );
                  if (sectionTargets.length === 0) {
                    return null;
                  }
                  return (
                    <fieldset key={section.id} className="space-y-2">
                      <legend className="text-label-md text-on-surface-variant">
                        {section.label}
                      </legend>
                      <div className="grid gap-3 sm:grid-cols-2">
                        {sectionTargets.map((target) => {
                          const key = draftKey(target);
                          const draftTarget = draft.find(
                            (candidate) => draftKey(candidate) === key,
                          );
                          const error = errors.get(key);
                          const errorId = `${target.scope}-${target.jobId}-error`;
                          return (
                            <label key={key} className="space-y-1">
                              <span className="block text-label-md text-on-surface">
                                {target.jobLabel} slots
                              </span>
                              <input
                                type="number"
                                min={0}
                                max={target.maxSlotCount}
                                step={1}
                                value={draftTarget?.slotCount ?? ""}
                                disabled={pending}
                                aria-describedby={error ? errorId : undefined}
                                aria-invalid={error ? true : undefined}
                                className="w-full rounded-md border border-outline bg-surface px-2 py-1 text-right tabular-nums text-on-surface"
                                onChange={(event) =>
                                  updateTarget(key, event.target.value)
                                }
                              />
                              {error ? (
                                <span
                                  id={errorId}
                                  className="block text-body-sm text-error"
                                >
                                  {error}
                                </span>
                              ) : null}
                            </label>
                          );
                        })}
                      </div>
                    </fieldset>
                  );
                })}
              </fieldset>
            ))}
          </div>
        </form>
      </Modal>
    </>
  );
}
