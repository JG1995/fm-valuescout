import { FileUp } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import type { CsvImportSummary } from "../types/csv-import-summary";
import { SquadCsvImportModal } from "./squad-csv-import-modal";

type SquadCsvImportActionsProps = {
  activeSaveId: number;
  snapshotId: number;
  onYouthImported: () => void;
  onMoneyballImported: (summary: CsvImportSummary) => void;
};

export function SquadCsvImportActions({
  activeSaveId,
  snapshotId,
  onYouthImported,
  onMoneyballImported,
}: SquadCsvImportActionsProps) {
  const [moneyballOpen, setMoneyballOpen] = useState(false);
  const [youthOpen, setYouthOpen] = useState(false);

  return (
    <>
      <div className="flex flex-wrap justify-end gap-2">
        <Button
          variant="secondary"
          icon={FileUp}
          onClick={() => {
            setMoneyballOpen(true);
          }}
        >
          Upload Squad CSV
        </Button>
        <Button
          variant="secondary"
          icon={FileUp}
          onClick={() => {
            setYouthOpen(true);
          }}
        >
          Upload Youth Academy CSV
        </Button>
      </div>
      <SquadCsvImportModal
        activeSaveId={activeSaveId}
        snapshotId={snapshotId}
        format="moneyball"
        open={moneyballOpen}
        onClose={() => {
          setMoneyballOpen(false);
        }}
        onYouthImported={() => undefined}
        onMoneyballImported={onMoneyballImported}
      />
      <SquadCsvImportModal
        activeSaveId={activeSaveId}
        snapshotId={snapshotId}
        format="youthTracker"
        open={youthOpen}
        onClose={() => {
          setYouthOpen(false);
        }}
        onYouthImported={onYouthImported}
      />
    </>
  );
}
