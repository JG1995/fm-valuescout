import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { DatabaseZap } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import {
  type ClearPlayerResultContext,
  playerResultContextMutationKey,
} from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatAbsoluteUtc,
  formatCount,
  formatRelativeAge,
} from "@/utils/format";
import { deleteSnapshot } from "../api/delete-snapshot";
import { renameSnapshot } from "../api/rename-snapshot";
import { savesQueryOptions } from "../api/saves-query-options";
import { snapshotKeys } from "../api/snapshot-keys";
import { snapshotMetadataQueryOptions } from "../api/snapshot-metadata-query-options";
import {
  type SnapshotGameDateUpdateResult,
  updateSnapshotDate,
} from "../api/update-snapshot-date";
import type {
  SnapshotDeleteResult,
  SnapshotMetadata,
  SnapshotSummary,
} from "../types/snapshot";

type SnapshotHistoryPanelProps = {
  /** Route-owned invalidation for products that only read the current snapshot. */
  onCurrentContextChanged?: () => void;
  onBeforeContextChange: ClearPlayerResultContext;
};

type SnapshotModalProps = {
  target: SnapshotMetadata | null;
  onClose: () => void;
  fallbackFocusTo: () => HTMLElement | null;
};

function snapshotLabel(snapshot: SnapshotMetadata) {
  return snapshot.customName ?? snapshot.gameDate ?? "Unknown in-game date";
}

function snapshotDate(snapshot: SnapshotMetadata) {
  return snapshot.gameDate ?? "Unknown in-game date";
}

function snapshotTargetLabel(snapshot: SnapshotMetadata) {
  const label = snapshot.customName
    ? `${snapshot.customName} (${snapshotDate(snapshot)})`
    : snapshotDate(snapshot);
  return `${label} (loaded ${formatAbsoluteUtc(snapshot.loadedAtUtc)}; snapshot #${snapshot.id})`;
}

function SnapshotRenameModal({
  target,
  onClose,
  fallbackFocusTo,
}: SnapshotModalProps) {
  const queryClient = useQueryClient();
  const formId = useId();
  const [visibleTarget, setVisibleTarget] = useState<SnapshotMetadata | null>(
    target,
  );
  const [name, setName] = useState(target?.customName ?? "");
  const rename = useMutation({
    mutationFn: () =>
      renameSnapshot(
        visibleTarget?.id ?? 0,
        visibleTarget?.contextToken ?? "",
        name.trim() || null,
      ),
    onSuccess: (snapshot) => {
      void queryClient.invalidateQueries({
        queryKey: snapshotKeys.history(snapshot.saveId),
      });
      onClose();
    },
  });
  const { reset } = rename;

  useEffect(() => {
    if (target) {
      setVisibleTarget(target);
      setName(target.customName ?? "");
      reset();
    }
  }, [reset, target]);

  if (!visibleTarget) {
    return null;
  }

  return (
    <Modal
      open={target !== null}
      title={`Rename snapshot ${snapshotDate(visibleTarget)}`}
      variant="form"
      onClose={() => {
        if (!rename.isPending) {
          onClose();
        }
      }}
      fallbackFocusTo={fallbackFocusTo}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={rename.isPending}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            form={formId}
            type="submit"
            loading={rename.isPending}
            loadingLabel="Saving…"
          >
            Save name
          </Button>
        </>
      }
    >
      <form
        id={formId}
        className="space-y-3"
        onSubmit={(event) => {
          event.preventDefault();
          rename.mutate();
        }}
      >
        <TextField
          label="Snapshot name"
          value={name}
          onChange={(event) => setName(event.target.value)}
          error={rename.isError ? rename.error.message : undefined}
        />
        <p className="text-body-sm text-on-surface-variant">
          Leave this blank to use the in-game date as the label.
        </p>
      </form>
    </Modal>
  );
}

type SnapshotDateEditModalProps = SnapshotModalProps & {
  onBeforeContextChange: ClearPlayerResultContext;
  onDateUpdated: (
    snapshot: SnapshotMetadata,
    result: SnapshotGameDateUpdateResult,
  ) => void;
};

function SnapshotDateEditModal({
  target,
  onClose,
  fallbackFocusTo,
  onBeforeContextChange,
  onDateUpdated,
}: SnapshotDateEditModalProps) {
  const [visibleTarget, setVisibleTarget] = useState<SnapshotMetadata | null>(
    target,
  );
  const [gameDate, setGameDate] = useState(target?.gameDate ?? "");
  const [localError, setLocalError] = useState<string | null>(null);
  const formId = useId();
  const editDate = useMutation({
    mutationFn: async () => {
      await onBeforeContextChange();
      return updateSnapshotDate(
        visibleTarget?.id ?? 0,
        visibleTarget?.contextToken ?? "",
        gameDate,
      );
    },
    onSuccess: (result) => {
      if (visibleTarget) {
        onDateUpdated(visibleTarget, result);
      }
      onClose();
    },
  });
  const { reset } = editDate;

  useEffect(() => {
    if (target) {
      setVisibleTarget(target);
      setGameDate(target.gameDate ?? "");
      setLocalError(null);
      reset();
    }
  }, [reset, target]);

  if (!visibleTarget) {
    return null;
  }

  return (
    <Modal
      open={target !== null}
      title={`Edit date for snapshot ${snapshotTargetLabel(visibleTarget)}`}
      variant="form"
      onClose={() => {
        if (!editDate.isPending) {
          onClose();
        }
      }}
      fallbackFocusTo={fallbackFocusTo}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={editDate.isPending}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            form={formId}
            type="submit"
            loading={editDate.isPending}
            loadingLabel="Saving…"
          >
            Save date
          </Button>
        </>
      }
    >
      <form
        id={formId}
        className="space-y-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!/^\d{4}-\d{2}-\d{2}$/.test(gameDate)) {
            setLocalError("Enter a valid date in YYYY-MM-DD format.");
            return;
          }
          setLocalError(null);
          editDate.mutate();
        }}
      >
        <TextField
          label="In-game date"
          value={gameDate}
          placeholder="YYYY-MM-DD"
          onChange={(event) => setGameDate(event.target.value)}
          error={
            localError ??
            (editDate.isError ? editDate.error.message : undefined)
          }
        />
      </form>
    </Modal>
  );
}

type SnapshotDeletionModalProps = SnapshotModalProps & {
  targetIsCurrent: boolean;
  onBeforeContextChange: ClearPlayerResultContext;
  onDeleted: (snapshot: SnapshotMetadata, result: SnapshotDeleteResult) => void;
};

function SnapshotDeletionModal({
  target,
  onClose,
  onDeleted,
  fallbackFocusTo,
  targetIsCurrent,
  onBeforeContextChange,
}: SnapshotDeletionModalProps) {
  const [visibleTarget, setVisibleTarget] = useState<SnapshotMetadata | null>(
    target,
  );
  const remove = useMutation({
    mutationKey: targetIsCurrent ? playerResultContextMutationKey : undefined,
    mutationFn: async () => {
      if (targetIsCurrent) {
        await onBeforeContextChange();
      }
      return deleteSnapshot(
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
      reset();
    }
  }, [reset, target]);

  if (!visibleTarget) {
    return null;
  }

  return (
    <Modal
      open={target !== null}
      title={`Delete snapshot ${snapshotTargetLabel(visibleTarget)}?`}
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
            Delete snapshot
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface-variant">
        This permanently removes this snapshot’s players, staff, role scores,
        bridge provenance, and Moneyball import data. Planner, Academy, and
        Youth data stay in this save.
      </p>
      {remove.isError ? (
        <p className="mt-3 text-body-sm text-error" role="alert">
          {remove.error.message}
        </p>
      ) : null}
    </Modal>
  );
}

export function SnapshotHistoryPanel({
  onBeforeContextChange,
  onCurrentContextChanged,
}: SnapshotHistoryPanelProps) {
  const queryClient = useQueryClient();
  const panelRef = useRef<HTMLDivElement>(null);
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];
  const { data: snapshots } = useSuspenseQuery(
    snapshotMetadataQueryOptions(activeSave?.id ?? 0),
  );
  const [renameTarget, setRenameTarget] = useState<SnapshotMetadata | null>(
    null,
  );
  const [deleteTarget, setDeleteTarget] = useState<SnapshotMetadata | null>(
    null,
  );
  const [dateEditTarget, setDateEditTarget] = useState<SnapshotMetadata | null>(
    null,
  );

  if (!activeSave) {
    return null;
  }

  return (
    <div ref={panelRef} tabIndex={-1} className="outline-none">
      <Panel title="Snapshot history" flush>
        {snapshots.length === 0 ? (
          <EmptyState icon={DatabaseZap} title="No snapshots stored">
            Load data to store the first dated snapshot for this save.
          </EmptyState>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse text-left">
              <caption className="sr-only">Snapshot history</caption>
              <thead>
                <tr className="bg-surface-container-lowest">
                  <th
                    scope="col"
                    className="h-table-header-height px-4 text-label-md text-on-surface-variant uppercase"
                  >
                    Snapshot
                  </th>
                  <th
                    scope="col"
                    className="h-table-header-height px-2 text-right text-label-md text-on-surface-variant uppercase"
                  >
                    Players
                  </th>
                  <th
                    scope="col"
                    className="h-table-header-height px-2 text-label-md text-on-surface-variant uppercase"
                  >
                    Loaded
                  </th>
                  <th
                    scope="col"
                    className="h-table-header-height px-4 text-right text-label-md text-on-surface-variant uppercase"
                  >
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody>
                {snapshots.map((snapshot) => (
                  <tr
                    key={snapshot.id}
                    className="border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high"
                  >
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                        <span className="text-body-sm text-on-surface">
                          {snapshotLabel(snapshot)}
                        </span>
                        {snapshot.isCurrent ? (
                          <span className="rounded-full bg-primary-container px-2 py-0.5 text-label-sm text-primary">
                            Current
                          </span>
                        ) : null}
                      </div>
                      {snapshot.customName ? (
                        <p className="mt-0.5 text-body-sm text-on-surface-variant">
                          {snapshotDate(snapshot)}
                        </p>
                      ) : null}
                    </td>
                    <td className="px-2 text-right font-mono text-mono-sm text-on-surface tabular-nums">
                      {formatCount(snapshot.playerCount)}
                    </td>
                    <td className="px-2 text-body-sm text-on-surface-variant">
                      <span title={formatAbsoluteUtc(snapshot.loadedAtUtc)}>
                        {formatRelativeAge(snapshot.loadedAtUtc)}
                      </span>
                    </td>
                    <td className="px-4">
                      <div className="flex justify-end gap-1">
                        <Button
                          variant="ghost"
                          onClick={() => setRenameTarget(snapshot)}
                          aria-label={`Rename snapshot ${snapshotDate(snapshot)}`}
                        >
                          Rename
                        </Button>
                        <Button
                          variant="ghost"
                          onClick={() => setDateEditTarget(snapshot)}
                          aria-label={`Edit date for snapshot ${snapshotTargetLabel(snapshot)}`}
                        >
                          Edit date
                        </Button>
                        <Button
                          variant="ghost"
                          onClick={() => setDeleteTarget(snapshot)}
                          aria-label={`Delete snapshot ${snapshotTargetLabel(snapshot)}`}
                        >
                          Delete
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Panel>
      <SnapshotRenameModal
        target={renameTarget}
        onClose={() => setRenameTarget(null)}
        fallbackFocusTo={() => panelRef.current}
      />
      <SnapshotDateEditModal
        target={dateEditTarget}
        onClose={() => setDateEditTarget(null)}
        fallbackFocusTo={() => panelRef.current}
        onBeforeContextChange={onBeforeContextChange}
        onDateUpdated={(snapshot, result) => {
          void queryClient.invalidateQueries({
            queryKey: snapshotKeys.history(snapshot.saveId),
          });
          if (result.previousCurrentSnapshotId !== result.currentSnapshotId) {
            void queryClient.invalidateQueries({
              queryKey: snapshotKeys.current(),
            });
            onCurrentContextChanged?.();
          } else if (snapshot.id === result.currentSnapshotId) {
            queryClient.setQueryData<SnapshotSummary | null>(
              snapshotKeys.current(),
              (cached) =>
                cached && cached.id === result.currentSnapshotId
                  ? { ...cached, gameDate: result.snapshot.gameDate }
                  : cached,
            );
          }
        }}
      />
      <SnapshotDeletionModal
        target={deleteTarget}
        onClose={() => setDeleteTarget(null)}
        fallbackFocusTo={() => panelRef.current}
        targetIsCurrent={deleteTarget?.isCurrent ?? false}
        onBeforeContextChange={onBeforeContextChange}
        onDeleted={(snapshot) => {
          void queryClient.invalidateQueries({
            queryKey: snapshotKeys.history(snapshot.saveId),
          });
          if (snapshot.isCurrent) {
            void queryClient.invalidateQueries({
              queryKey: snapshotKeys.current(),
            });
            onCurrentContextChanged?.();
          }
        }}
      />
    </div>
  );
}
