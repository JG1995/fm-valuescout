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
  const [openFormat, setOpenFormat] = useState<
    "moneyball" | "youthTracker" | null
  >(null);

  return (
    <>
      <div className="flex flex-wrap justify-end gap-2">
        <Button
          variant="secondary"
          icon={FileUp}
          onClick={() => {
            setOpenFormat("moneyball");
          }}
        >
          Upload Moneyball CSV
        </Button>
        <Button
          variant="secondary"
          icon={FileUp}
          onClick={() => {
            setOpenFormat("youthTracker");
          }}
        >
          Upload Youth Academy CSV
        </Button>
      </div>
      <SquadCsvImportModal
        activeSaveId={activeSaveId}
        snapshotId={snapshotId}
        format="moneyball"
        open={openFormat === "moneyball"}
        onClose={() => {
          setOpenFormat(null);
        }}
        onYouthImported={onYouthImported}
      />
      <SquadCsvImportModal
        activeSaveId={activeSaveId}
        snapshotId={snapshotId}
        format="youthTracker"
        open={openFormat === "youthTracker"}
        onClose={() => {
          setOpenFormat(null);
        }}
        onYouthImported={onYouthImported}
      />
    </>
  );
}
