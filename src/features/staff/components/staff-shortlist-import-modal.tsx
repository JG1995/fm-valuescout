import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { FileUp } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { invokeCommand } from "@/lib/tauri-client";

export function StaffShortlistImportModal({
  open,
  replacesExisting,
  onClose,
  onImported,
}: {
  open: boolean;
  replacesExisting: boolean;
  onClose: () => void;
  onImported: () => Promise<void>;
}) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  const chooseFile = async () => {
    const path = await openFileDialog({
      multiple: false,
      directory: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;
    setPending(true);
    setError(undefined);
    try {
      const summary = await invokeCommand<{
        totalStaff: number;
        storedStaff: number;
        skippedStaff: number;
      }>("import_staff_shortlist_csv", { path });
      await onImported();
      setError(
        `Imported ${summary.storedStaff} of ${summary.totalStaff} staff (${summary.skippedStaff} skipped).`,
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setPending(false);
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
