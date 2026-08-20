import { FileUp } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SquadCsvImportModal } from "./squad-csv-import-modal";

type SquadCsvImportActionsProps = {
  activeSaveId: number;
  snapshotId: number;
  onYouthImported: () => void;
};

export function SquadCsvImportActions({
  activeSaveId,
  snapshotId,
  onYouthImported,
}: SquadCsvImportActionsProps) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <div className="flex flex-wrap justify-end gap-2">
        <Button
          variant="secondary"
          icon={FileUp}
          onClick={() => {
            setOpen(true);
          }}
        >
          Upload Youth Academy CSV
        </Button>
      </div>
      <SquadCsvImportModal
        activeSaveId={activeSaveId}
        snapshotId={snapshotId}
        format="youthTracker"
        open={open}
        onClose={() => {
          setOpen(false);
        }}
        onYouthImported={onYouthImported}
      />
    </>
  );
}
