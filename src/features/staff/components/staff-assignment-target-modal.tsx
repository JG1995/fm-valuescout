import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Minus, Plus } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
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
  StaffAssignmentTargets,
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

export type StaffAssignmentModalStatus = {
  saved: boolean;
  error: string | null;
};

type StaffAssignmentTargetModalProps = {
  context: StaffAssignmentContext;
  contextKey: string;
  targets?: StaffAssignmentTargets;
  targetsError?: Error | null;
  targetsPending?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  onSaved?: () => void;
  onPendingChange?: (pending: boolean) => void;
  onStatusChange?: (status: StaffAssignmentModalStatus) => void;
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

function validDraftValue(target: StaffAssignmentTarget, draft: DraftTarget[]) {
  const value = draft.find(
    (candidate) => draftKey(candidate) === draftKey(target),
  )?.slotCount;
  return value !== undefined &&
    slotCountError(value, target.maxSlotCount) === undefined
    ? Number(value)
    : undefined;
}

export function StaffAssignmentTargetModal({
  context,
  contextKey,
  targets,
  targetsError = null,
  targetsPending = false,
  open,
  onOpenChange,
  onSaved,
  onPendingChange,
  onStatusChange,
}: StaffAssignmentTargetModalProps) {
  const queryClient = useQueryClient();
  const [internalOpen, setInternalOpen] = useState(false);
  const fallbackQuery = useQuery({
    ...staffAssignmentTargetsQueryOptions(context, contextKey),
    enabled: targets === undefined,
  });
  const resolvedTargets = targets ?? fallbackQuery.data;
  const resolvedError =
    targetsError ??
    (fallbackQuery.error instanceof Error ? fallbackQuery.error : null);
  const resolvedPending =
    targetsPending ?? (fallbackQuery.isPending || fallbackQuery.isFetching);
  const resolvedOpen = open ?? internalOpen;
  const changeOpen = onOpenChange ?? setInternalOpen;
  const [draft, setDraft] = useState<DraftTarget[]>([]);
  const [saved, setSaved] = useState(false);
  const previousOpen = useRef(resolvedOpen);
  const controlIdPrefix = useId();
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
      changeOpen(false);
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
    changeOpen(false);
    setDraft([]);
    setSaved(false);
    saveTargets.reset();
  }, [changeOpen, contextKey, onPendingChange, saveTargets.reset]);

  const pending = saveTargets.isPending;
  const formError = saveTargets.isError
    ? errorMessage(saveTargets.error)
    : resolvedError
      ? errorMessage(resolvedError)
      : null;

  useEffect(() => {
    onStatusChange?.({ saved, error: formError });
  }, [formError, onStatusChange, saved]);

  const errors = new Map(
    (resolvedTargets?.targets ?? []).map((target) => {
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
    resolvedTargets !== undefined &&
    draft.length === resolvedTargets.targets.length &&
    [...errors.values()].every((error) => error === undefined) &&
    !pending;

  const openModal = () => {
    if (!resolvedTargets || resolvedPending || resolvedError) {
      return;
    }
    setSaved(false);
    saveTargets.reset();
    setDraft(draftFromTargets(resolvedTargets.targets));
    changeOpen(true);
  };

  useEffect(() => {
    if (
      resolvedOpen &&
      !previousOpen.current &&
      resolvedTargets &&
      !resolvedPending &&
      !resolvedError
    ) {
      setSaved(false);
      setDraft(draftFromTargets(resolvedTargets.targets));
    }
    previousOpen.current = resolvedOpen;
  }, [resolvedError, resolvedOpen, resolvedPending, resolvedTargets]);

  const closeModal = () => {
    if (pending) {
      return;
    }
    changeOpen(false);
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

  const targetGroups = resolvedTargets
    ? [
        ...resolvedTargets.teams.map((team) => ({
          scope: team.team,
          title: team.displayName,
          targets: resolvedTargets.targets.filter(
            (target) =>
              target.scope === team.team ||
              (team.team === "senior" && target.scope === "club"),
          ),
        })),
        ...(!resolvedTargets.teams.some(({ team }) => team === "senior")
          ? [
              {
                scope: "club" as const,
                title: "Club",
                targets: resolvedTargets.targets.filter(
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
        disabled={resolvedPending || resolvedError !== null || pending}
        onClick={openModal}
      >
        Configure staffing needs
      </Button>
      {!onStatusChange && saved ? (
        <p role="status" className="text-body-sm text-success">
          Slot counts saved.
        </p>
      ) : null}
      {!onStatusChange && resolvedError && !resolvedOpen ? (
        <p role="alert" className="text-body-sm text-error">
          {formError}
        </p>
      ) : null}
      <Modal
        open={resolvedOpen}
        title="Configure staffing needs"
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
          <p className="text-body-sm text-on-surface-variant">
            Zero excludes a role from recommendations.
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
                {(() => {
                  const groupValues = group.targets.map((target) =>
                    validDraftValue(target, draft),
                  );
                  const groupTotal = groupValues.every(
                    (value) => value !== undefined,
                  )
                    ? groupValues.reduce(
                        (total, value) => total + (value ?? 0),
                        0,
                      )
                    : undefined;
                  return (
                    <>
                      <p className="text-body-sm text-on-surface-variant">
                        {groupTotal === undefined
                          ? `${group.title} total unavailable until corrected.`
                          : `${group.title} total: ${groupTotal}`}
                      </p>
                      {TARGET_SECTIONS.map((section) => {
                        const sectionTargets = group.targets.filter(
                          (target) => target.section === section.id,
                        );
                        if (sectionTargets.length === 0) {
                          return null;
                        }
                        const sectionValues = sectionTargets.map((target) =>
                          validDraftValue(target, draft),
                        );
                        const sectionTotal = sectionValues.every(
                          (value) => value !== undefined,
                        )
                          ? sectionValues.reduce(
                              (total, value) => total + (value ?? 0),
                              0,
                            )
                          : undefined;
                        return (
                          <fieldset key={section.id} className="space-y-2">
                            <legend className="text-label-md text-on-surface-variant">
                              {section.label}
                            </legend>
                            <p className="text-body-sm text-on-surface-variant">
                              {sectionTotal === undefined
                                ? `${section.label} total unavailable until corrected.`
                                : `${section.label} total: ${sectionTotal}`}
                            </p>
                            <div className="grid gap-3 sm:grid-cols-2">
                              {sectionTargets.map((target) => {
                                const key = draftKey(target);
                                const draftTarget = draft.find(
                                  (candidate) => draftKey(candidate) === key,
                                );
                                const error = errors.get(key);
                                const inputId = `${controlIdPrefix}-${target.scope}-${target.jobId}`;
                                const errorId = `${inputId}-error`;
                                const currentValue = validDraftValue(
                                  target,
                                  draft,
                                );
                                return (
                                  <div key={key} className="space-y-1">
                                    <label
                                      htmlFor={inputId}
                                      className="block text-label-md text-on-surface"
                                    >
                                      {target.jobLabel} slots
                                    </label>
                                    <div className="flex items-center gap-1">
                                      <Button
                                        variant="ghost"
                                        size="icon"
                                        icon={Minus}
                                        aria-label={`Decrease ${target.jobLabel} slots`}
                                        disabled={
                                          pending ||
                                          currentValue === undefined ||
                                          currentValue <= 0
                                        }
                                        className="size-7"
                                        onClick={() =>
                                          updateTarget(
                                            key,
                                            String((currentValue ?? 0) - 1),
                                          )
                                        }
                                      />
                                      <input
                                        id={inputId}
                                        type="number"
                                        min={0}
                                        max={target.maxSlotCount}
                                        step={1}
                                        value={draftTarget?.slotCount ?? ""}
                                        disabled={pending}
                                        aria-describedby={
                                          error ? errorId : undefined
                                        }
                                        aria-invalid={error ? true : undefined}
                                        className="w-full rounded-md border border-outline bg-surface px-2 py-1 text-right tabular-nums text-on-surface"
                                        onChange={(event) =>
                                          updateTarget(key, event.target.value)
                                        }
                                      />
                                      <Button
                                        variant="ghost"
                                        size="icon"
                                        icon={Plus}
                                        aria-label={`Increase ${target.jobLabel} slots`}
                                        disabled={
                                          pending ||
                                          currentValue === undefined ||
                                          currentValue >= target.maxSlotCount
                                        }
                                        className="size-7"
                                        onClick={() =>
                                          updateTarget(
                                            key,
                                            String((currentValue ?? 0) + 1),
                                          )
                                        }
                                      />
                                    </div>
                                    {error ? (
                                      <span
                                        id={errorId}
                                        className="block text-body-sm text-error"
                                      >
                                        {error}
                                      </span>
                                    ) : null}
                                  </div>
                                );
                              })}
                            </div>
                          </fieldset>
                        );
                      })}
                    </>
                  );
                })()}
              </fieldset>
            ))}
          </div>
        </form>
      </Modal>
    </>
  );
}
