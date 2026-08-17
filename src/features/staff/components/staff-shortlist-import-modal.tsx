import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { FileUp } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { invokeCommand } from "@/lib/tauri-client";

export type StaffShortlistImportSummary = {
  totalStaff: number;
  storedStaff: number;
  skippedStaff: number;
};

export function StaffShortlistImportModal({
  activeSaveId,
  snapshotId,
  open,
  replacesExisting,
  onClose,
  onImported,
}: {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
  open: boolean;
  replacesExisting: boolean;
  onClose: () => void;
  onImported: (summary: StaffShortlistImportSummary) => Promise<void>;
}) {
  const contextKey = `${activeSaveId ?? "none"}:${snapshotId ?? "none"}`;
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
  }, [contextKey]);
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
    setError(undefined);
    try {
      const summary = await invokeCommand<StaffShortlistImportSummary>(
        "import_staff_shortlist_csv",
        { path },
      );
      if (isCurrentSelection()) {
        await onImported(summary);
        onClose();
      }
    } catch (reason) {
      if (isCurrentSelection()) {
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    } finally {
      if (isCurrentSelection()) setPending(false);
    }
  };
  return (
    <Modal
      open={open}
      title="Upload Staff Shortlist CSV"
      onClose={() => !pending && onClose()}
      footer={
        <>
          <Button variant="secondary" disabled={pending} onClick={onClose}>
            Cancel
          </Button>
          <Button
            icon={FileUp}
            loading={pending}
            onClick={() => void chooseFile()}
          >
            {replacesExisting ? "Choose replacement CSV" : "Choose CSV"}
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface">
        {replacesExisting
          ? "Uploading a CSV replaces the active Staff Shortlist for this save."
          : "Choose a staff CSV to create the Staff Shortlist for this save."}
      </p>
      <p className="mt-2 text-body-sm text-on-surface-variant">
        The export must include Unique ID, Preferred Job, Club Job, and Coaching
        Qualifications.
      </p>
      {error ? (
        <p role="status" className="mt-4 text-body-sm text-on-surface-variant">
          {error}
        </p>
      ) : null}
    </Modal>
  );
}
