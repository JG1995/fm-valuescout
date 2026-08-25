import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import {
  type ClearPlayerResultContext,
  playerResultContextMutationKey,
} from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import { createSave } from "../api/create-save";
import { deleteSave, type SaveDeleteResult } from "../api/delete-save";
import { renameSave } from "../api/rename-save";
import { savesQueryOptions } from "../api/saves-query-options";
import { snapshotKeys } from "../api/snapshot-keys";
import type { SaveSummary } from "../types/save";

function readName(form: HTMLFormElement) {
  const name = new FormData(form).get("name");
  return typeof name === "string" ? name : "";
}

function saveTargetLabel(save: SaveSummary) {
  return `${save.name} (save ${save.id})`;
}

type SaveSwitcherProps = {
  /** Route-owned invalidation for products that only read the current snapshot. */
  onCurrentContextChanged?: () => void;
  onBeforeContextChange: ClearPlayerResultContext;
};

type SaveDeletionModalProps = {
  target: SaveSummary | null;
  targetIsActive: boolean;
  saveCount: number;
  onClose: () => void;
  onDeleted: (save: SaveSummary, result: SaveDeleteResult) => void;
  fallbackFocusTo: () => HTMLElement | null;
  onBeforeContextChange: ClearPlayerResultContext;
};

function SaveDeletionModal({
  target,
  targetIsActive,
  saveCount,
  onClose,
  onDeleted,
  fallbackFocusTo,
  onBeforeContextChange,
}: SaveDeletionModalProps) {
  const [visibleTarget, setVisibleTarget] = useState<SaveSummary | null>(
    target,
  );
  const [visibleSaveCount, setVisibleSaveCount] = useState(saveCount);
  const remove = useMutation({
    mutationKey: targetIsActive ? playerResultContextMutationKey : undefined,
    mutationFn: async () => {
      if (targetIsActive) {
        await onBeforeContextChange();
      }
      return deleteSave(
        visibleTarget?.id ?? 0,
        visibleTarget?.contextToken ?? "",
      );
    },
    onSuccess: (result) => {
      if (visibleTarget) {
        onDeleted(visibleTarget, result);
      }
      onClose();
    },
  });
  const { reset } = remove;

  useEffect(() => {
    if (target) {
      setVisibleTarget(target);
      setVisibleSaveCount(saveCount);
      reset();
    }
  }, [reset, saveCount, target]);

  if (!visibleTarget) {
    return null;
  }

  const replacingFinalSave = visibleSaveCount === 1;

  return (
    <Modal
      open={target !== null}
      title={`Delete save ${saveTargetLabel(visibleTarget)}?`}
      variant="destructive"
      onClose={() => {
        if (!remove.isPending) {
          onClose();
        }
      }}
      fallbackFocusTo={fallbackFocusTo}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={remove.isPending}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            variant="destructive"
            loading={remove.isPending}
            loadingLabel="Deleting…"
            onClick={() => remove.mutate()}
          >
            Delete save
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface-variant">
        This permanently removes every snapshot, player, staff, Moneyball
        import, Planner setting, Academy record, and Youth enrichment in this
        save.
      </p>
      <p className="mt-3 text-body-sm text-on-surface-variant">
        {replacingFinalSave
          ? "A blank Default save will replace it."
          : targetIsActive
            ? "Another save will become active."
            : "The active save stays unchanged."}
      </p>
      {remove.isError ? (
        <p className="mt-3 text-body-sm text-error" role="alert">
          {remove.error.message}
        </p>
      ) : null}
    </Modal>
  );
}

// Switching the active save lives in the top bar, where it stays reachable from
// every screen. This panel keeps the rarer management actions.
export function SaveSwitcher({
  onBeforeContextChange,
  onCurrentContextChanged,
}: SaveSwitcherProps) {
  const queryClient = useQueryClient();
  const panelRef = useRef<HTMLDivElement>(null);
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];
  const [deleteTarget, setDeleteTarget] = useState<SaveSummary | null>(null);
  const currentDeleteTarget = deleteTarget
    ? saves.find(
        (save) =>
          save.id === deleteTarget.id &&
          save.contextToken === deleteTarget.contextToken,
      )
    : null;

  const create = useMutation({
    mutationFn: createSave,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    },
  });

  const rename = useMutation({
    mutationFn: ({ saveId, name }: { saveId: number; name: string }) =>
      renameSave(saveId, name),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    },
  });

  return (
    <div ref={panelRef} tabIndex={-1} className="outline-none">
      <Panel title="Saves">
        <div className="grid gap-4 sm:grid-cols-2">
          <form
            className="space-y-2"
            onSubmit={(event) => {
              event.preventDefault();
              if (activeSave) {
                rename.mutate({
                  saveId: activeSave.id,
                  name: readName(event.currentTarget),
                });
              }
            }}
          >
            {/* Keyed to the save so a draft cannot survive a switch made from the
                top bar and then rename whichever save became active. */}
            <TextField
              key={activeSave?.id}
              label="Rename active save"
              name="name"
              defaultValue={activeSave?.name ?? ""}
              error={rename.isError ? rename.error.message : undefined}
            />
            <Button
              type="submit"
              variant="secondary"
              disabled={!activeSave}
              loading={rename.isPending}
              loadingLabel="Renaming…"
            >
              Rename save
            </Button>
          </form>
          <form
            className="space-y-2"
            onSubmit={(event) => {
              event.preventDefault();
              const form = event.currentTarget;
              create.mutate(readName(form), {
                onSuccess: () => form.reset(),
              });
            }}
          >
            <TextField
              label="New save"
              name="name"
              error={create.isError ? create.error.message : undefined}
            />
            <Button
              type="submit"
              variant="secondary"
              loading={create.isPending}
              loadingLabel="Creating…"
            >
              Create save
            </Button>
          </form>
        </div>
        <ul className="mt-4 divide-y divide-outline-variant border-t border-outline-variant">
          {saves.map((save) => (
            <li
              key={save.id}
              className="flex items-center justify-between gap-3 py-3"
            >
              <div className="min-w-0">
                <p className="truncate text-body-sm text-on-surface">
                  {save.name}
                </p>
                {save.isActive ? (
                  <p className="text-body-sm text-on-surface-variant">Active</p>
                ) : null}
              </div>
              <Button
                variant="ghost"
                aria-label={`Delete save ${saveTargetLabel(save)}`}
                onClick={() => setDeleteTarget(save)}
              >
                Delete
              </Button>
            </li>
          ))}
        </ul>
      </Panel>
      <SaveDeletionModal
        target={deleteTarget}
        targetIsActive={currentDeleteTarget?.isActive ?? false}
        saveCount={saves.length}
        onClose={() => setDeleteTarget(null)}
        fallbackFocusTo={() => panelRef.current}
        onBeforeContextChange={onBeforeContextChange}
        onDeleted={(save, result) => {
          if (result.deletedWasActive) {
            void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
            onCurrentContextChanged?.();
          } else {
            void queryClient.invalidateQueries({
              queryKey: snapshotKeys.saves(),
            });
            queryClient.removeQueries({
              queryKey: snapshotKeys.history(save.id),
            });
          }
        }}
      />
    </div>
  );
}
