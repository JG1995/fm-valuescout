import { FileUp } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { useCsvImport } from "../utils/use-csv-import";
import { CsvImportOutcome } from "./csv-import-outcome";

type CsvImportPanelProps = {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
  onYouthImported: () => void;
};

export function CsvImportPanel({
  activeSaveId,
  snapshotId,
  onYouthImported,
}: CsvImportPanelProps) {
  const { chooseCsv, state } = useCsvImport({
    activeSaveId,
    snapshotId,
    onYouthImported,
  });

  return (
    <Panel
      title="CSV enrichment"
      actions={
        <Button
          variant="secondary"
          icon={FileUp}
          loading={state.status === "pending"}
          loadingLabel="Importing CSV…"
          disabled={snapshotId === undefined}
          onClick={() => {
            void chooseCsv();
          }}
        >
          Import CSV
        </Button>
      }
    >
      {snapshotId !== undefined ? (
        <CsvImportOutcome state={state} />
      ) : (
        <EmptyState icon={FileUp} title="No snapshot loaded">
          Load Data before importing a CSV export.
        </EmptyState>
      )}
    </Panel>
  );
}
