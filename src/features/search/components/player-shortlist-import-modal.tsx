import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { FileUp } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { importPlayerShortlistCsv } from "../api/import-player-shortlist";
import type { PlayerShortlistImportSummary } from "../types/player-shortlist-import-summary";

export function PlayerShortlistImportModal({
  activeSaveId,
  activeSaveContextToken,
  snapshotId,
  snapshotContextToken,
  open,
  onClose,
  onImported,
  onPendingChange,
  contextKey: providedContextKey,
}: {
  activeSaveId: number | undefined;
  activeSaveContextToken?: string;
  snapshotId: number | undefined;
  snapshotContextToken?: string;
  open: boolean;
  onClose: () => void;
  onImported: (summary: PlayerShortlistImportSummary) => Promise<void>;
  onPendingChange?: (pending: boolean) => void;
  contextKey?: string;
}) {
  const contextKey =
    providedContextKey ??
    `${activeSaveId ?? "none"}:${activeSaveContextToken ?? "none"}:${snapshotId ?? "none"}:${snapshotContextToken ?? "none"}`;
  const currentContext = useRef(contextKey);
  const contextGeneration = useRef(0);
  const previousContextKey = useRef(contextKey);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  useLayoutEffect(() => {
    currentContext.current = contextKey;
    contextGeneration.current += 1;
    setPending(false);
    setError(undefined);
    onPendingChange?.(false);
  }, [contextKey, onPendingChange]);
  useEffect(() => {
    if (previousContextKey.current !== contextKey && open) {
      onClose();
    }
    previousContextKey.current = contextKey;
  }, [contextKey, onClose, open]);
  const chooseFile = async () => {
    const selection = {
      contextKey,
      generation: contextGeneration.current,
    };
    const isCurrentSelection = () =>
      currentContext.current === selection.contextKey &&
      contextGeneration.current === selection.generation;
    const path = await openFileDialog({
      multiple: false,
      directory: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path || !isCurrentSelection()) return;
    setPending(true);
    onPendingChange?.(true);
    setError(undefined);
    try {
      const summary = await importPlayerShortlistCsv(path);
      if (isCurrentSelection()) {
        await onImported(summary);
        onClose();
      }
    } catch (reason) {
      if (isCurrentSelection()) {
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    } finally {
      if (isCurrentSelection()) {
        setPending(false);
        onPendingChange?.(false);
      }
    }
  };
  const close = () => {
    if (!pending) {
      onPendingChange?.(false);
      onClose();
    }
  };
  return (
    <Modal
      open={open}
      title="Upload Player Shortlist CSV"
      onClose={close}
      footer={
        <>
          <Button variant="secondary" disabled={pending} onClick={close}>
            Cancel
          </Button>
          <Button
            icon={FileUp}
            loading={pending}
            onClick={() => void chooseFile()}
          >
            Choose CSV
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface">
        Uploading a CSV replaces the saved player shortlist for this save.
      </p>
      <p className="mt-2 text-body-sm text-on-surface-variant">
        The export must include a Player UID column. All other columns are
        ignored.
      </p>
      {error ? (
        <p role="status" className="mt-4 text-body-sm text-on-surface-variant">
          {error}
        </p>
      ) : null}
    </Modal>
  );
}
