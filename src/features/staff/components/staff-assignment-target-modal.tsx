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
  StaffAssignmentScope,
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
  onSaved?: (targets: StaffAssignmentTargets) => void;
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

type NeedsColumn = { scope: StaffAssignmentScope; label: string };

/**
 * Each section's jobs among `targets`. Rust orders each team's jobs; a job one team lacks (the
 * First Team has no Manager) goes before the next job that team shares.
 */
function jobRowsBySection(targets: StaffAssignmentTarget[]) {
  return TARGET_SECTIONS.map((section) => {
    const sectionTargets = targets.filter(
      (target) => target.section === section.id,
    );
    const rows: { jobId: string; jobLabel: string }[] = [];
    const rowIndex = (jobId: string) =>
      rows.findIndex((row) => row.jobId === jobId);
    for (const [index, target] of sectionTargets.entries()) {
      if (rowIndex(target.jobId) !== -1) {
        continue;
      }
      const next = sectionTargets
        .slice(index + 1)
        .find(
          (later) =>
            later.scope === target.scope && rowIndex(later.jobId) !== -1,
        );
      rows.splice(next ? rowIndex(next.jobId) : rows.length, 0, {
        jobId: target.jobId,
        jobLabel: target.jobLabel,
      });
    }
    return { ...section, rows };
  }).filter((section) => section.rows.length > 0);
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
  const configureButtonId = useId();
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
      onSaved?.(result);
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

  const teamColumns: NeedsColumn[] = (resolvedTargets?.teams ?? []).map(
    (team) => ({ scope: team.team, label: team.displayName }),
  );
  const allTargets = resolvedTargets?.targets ?? [];
  const teamTargets = allTargets.filter((target) => target.scope !== "club");
  const clubTargets = allTargets.filter((target) => target.scope === "club");
  const invalidTargets = allTargets.filter(
    (target) => errors.get(draftKey(target)) !== undefined,
  );

  const cellName = (target: StaffAssignmentTarget) =>
    target.scope === "club"
      ? target.jobLabel
      : `${teamColumns.find((column) => column.scope === target.scope)?.label ?? ""} ${target.jobLabel}`;
  const errorId = (target: StaffAssignmentTarget) =>
    `${controlIdPrefix}-${target.scope}-${target.jobId}-error`;

  const countCell = (target: StaffAssignmentTarget, labelledBy: string) => {
    const key = draftKey(target);
    const value = validDraftValue(target, draft);
    const error = errors.get(key);
    return (
      <div className="inline-flex items-center gap-0.5">
        {/* Arrow keys step the number input, so the buttons stay out of the tab order. */}
        <Button
          variant="ghost"
          size="icon"
          icon={Minus}
          aria-label={`Decrease ${cellName(target)}`}
          tabIndex={-1}
          disabled={pending || value === undefined || value <= 0}
          className="size-6"
          onClick={() => updateTarget(key, String((value ?? 0) - 1))}
        />
        <input
          type="number"
          min={0}
          max={target.maxSlotCount}
          step={1}
          value={
            draft.find((candidate) => draftKey(candidate) === key)?.slotCount ??
            ""
          }
          disabled={pending}
          aria-labelledby={labelledBy}
          aria-describedby={error ? errorId(target) : undefined}
          aria-invalid={error ? true : undefined}
          className="h-7 w-12 rounded-md border border-outline bg-surface px-1 text-center tabular-nums text-on-surface aria-[invalid=true]:border-error"
          onChange={(event) => updateTarget(key, event.target.value)}
        />
        <Button
          variant="ghost"
          size="icon"
          icon={Plus}
          aria-label={`Increase ${cellName(target)}`}
          tabIndex={-1}
          disabled={
            pending || value === undefined || value >= target.maxSlotCount
          }
          className="size-6"
          onClick={() => updateTarget(key, String((value ?? 0) + 1))}
        />
      </div>
    );
  };

  const needsTable = (
    tableId: string,
    caption: string,
    columns: NeedsColumn[],
    targets: StaffAssignmentTarget[],
  ) => {
    const columnTotal = (scope: StaffAssignmentScope) => {
      const values = targets
        .filter((target) => target.scope === scope)
        .map((target) => validDraftValue(target, draft));
      return values.every((value) => value !== undefined)
        ? values.reduce((total, value) => total + (value ?? 0), 0)
        : undefined;
    };
    const columnHeaderId = (scope: StaffAssignmentScope) =>
      `${controlIdPrefix}-${tableId}-column-${scope}`;
    return (
      <table className="w-full text-left text-body-sm text-on-surface">
        <caption className="pb-1 text-left text-label-lg text-on-surface">
          {caption}
        </caption>
        {/* One Club column needs no visible header. */}
        <thead
          className={
            columns.length === 1
              ? "sr-only"
              : "text-label-md text-on-surface-variant"
          }
        >
          <tr>
            <th scope="col" className="py-1 pr-2">
              Job
            </th>
            {columns.map((column) => (
              <th
                key={column.scope}
                id={columnHeaderId(column.scope)}
                scope="col"
                className="px-1 py-1 text-center"
              >
                {column.label}
              </th>
            ))}
          </tr>
        </thead>
        {jobRowsBySection(targets).map((section) => (
          <tbody key={section.id}>
            <tr>
              <th
                scope="rowgroup"
                colSpan={columns.length + 1}
                className="pt-2 pb-0.5 text-label-md text-on-surface-variant"
              >
                {section.label}
              </th>
            </tr>
            {section.rows.map((row) => {
              const rowHeaderId = `${controlIdPrefix}-${tableId}-row-${row.jobId}`;
              return (
                <tr key={row.jobId} className="border-t border-outline-variant">
                  <th
                    id={rowHeaderId}
                    scope="row"
                    className="py-0.5 pr-2 font-normal"
                  >
                    {row.jobLabel}
                  </th>
                  {columns.map((column) => {
                    const target = targets.find(
                      (candidate) =>
                        candidate.scope === column.scope &&
                        candidate.jobId === row.jobId,
                    );
                    return (
                      <td
                        key={column.scope}
                        className="px-1 py-0.5 text-center"
                      >
                        {target ? (
                          countCell(
                            target,
                            column.scope === "club"
                              ? rowHeaderId
                              : `${columnHeaderId(column.scope)} ${rowHeaderId}`,
                          )
                        ) : column.scope === "senior" &&
                          row.jobId === "manager" ? (
                          <span
                            className="text-on-surface-variant"
                            title={`You manage the ${column.label}.`}
                          >
                            You
                          </span>
                        ) : (
                          <span className="text-on-surface-variant">—</span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        ))}
        <tfoot>
          <tr className="border-t border-outline">
            <th scope="row" className="py-1 pr-2 text-label-md">
              Total
            </th>
            {columns.map((column) => (
              <td
                key={column.scope}
                className="px-1 py-1 text-center text-label-md tabular-nums"
              >
                {columnTotal(column.scope) ?? "—"}
              </td>
            ))}
          </tr>
        </tfoot>
      </table>
    );
  };

  return (
    <>
      <Button
        id={configureButtonId}
        variant="secondary"
        disabled={resolvedPending || resolvedError !== null || pending}
        onClick={openModal}
      >
        Configure staffing needs
      </Button>
      {!onStatusChange && saved ? (
        <p role="status" className="text-body-sm text-success">
          Staffing needs saved.
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
        className="max-w-[64rem]!"
        // Adjust staffing needs opens this dialog from the results dialog,
        // which is gone by the time this one closes.
        fallbackFocusTo={() => document.getElementById(configureButtonId)}
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
              Save
            </Button>
          </>
        }
      >
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            save();
          }}
        >
          <p className="text-body-md text-on-surface-variant">
            Set how many people you want in each job. Jobs left at 0 get no
            recommendation.
          </p>
          {formError ? (
            <p role="alert" className="text-body-sm text-error">
              {formError}
            </p>
          ) : null}
          <div className="grid items-start gap-x-8 gap-y-4 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
            {teamTargets.length > 0
              ? needsTable("team", "Team jobs", teamColumns, teamTargets)
              : null}
            {clubTargets.length > 0
              ? needsTable(
                  "club",
                  "Club-wide jobs",
                  [{ scope: "club", label: "Club" }],
                  clubTargets,
                )
              : null}
          </div>
          {invalidTargets.length > 0 ? (
            <ul className="space-y-1 text-body-sm text-error">
              {invalidTargets.map((target) => (
                <li key={draftKey(target)} id={errorId(target)}>
                  {cellName(target)}: {errors.get(draftKey(target))}
                </li>
              ))}
            </ul>
          ) : null}
        </form>
      </Modal>
    </>
  );
}
