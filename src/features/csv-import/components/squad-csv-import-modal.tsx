import { getCurrentWebview } from "@tauri-apps/api/webview";
import { FileUp } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import type {
  CsvImportFormat,
  CsvImportSummary,
} from "../types/csv-import-summary";
import { type CsvImportSelection, useCsvImport } from "../utils/use-csv-import";
import { CsvImportOutcome } from "./csv-import-outcome";

type SquadCsvImportModalProps = {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
  format: CsvImportFormat;
  open: boolean;
  onClose: () => void;
  onYouthImported: () => void;
  onMoneyballImported?: (summary: CsvImportSummary) => void;
};

function formatName(format: CsvImportFormat) {
  return format === "moneyball" ? "Moneyball" : "Youth Academy";
}

function useNativeCsvDrop(
  open: boolean,
  captureSelection: () => CsvImportSelection,
  onDrop: (paths: string[], selection: CsvImportSelection) => void,
) {
  const onDropRef = useRef(onDrop);
  const [isDragging, setIsDragging] = useState(false);
  onDropRef.current = onDrop;

  useEffect(() => {
    if (!open) {
      setIsDragging(false);
      return;
    }

    let listening = true;
    let unlisten: (() => void) | undefined;
    const selection = captureSelection();
    void Promise.resolve()
      .then(() =>
        getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "enter" || event.payload.type === "over") {
            setIsDragging(true);
            return;
          }
          if (event.payload.type === "leave") {
            setIsDragging(false);
            return;
          }
          setIsDragging(false);
          onDropRef.current(event.payload.paths, selection);
        }),
      )
      .then((dispose) => {
        if (listening) {
          unlisten = dispose;
        } else {
          dispose();
        }
      })
      .catch(() => {
        // Browser development and tests do not have a native Tauri webview.
      });

    return () => {
      listening = false;
      unlisten?.();
    };
  }, [captureSelection, open]);

  return isDragging;
}

export function SquadCsvImportModal({
  activeSaveId,
  snapshotId,
  format,
  open,
  onClose,
  onYouthImported,
  onMoneyballImported,
}: SquadCsvImportModalProps) {
  const {
    captureSelection,
    chooseCsv,
    contextKey,
    importPath,
    reportError,
    reset,
    state,
  } = useCsvImport({
    activeSaveId,
    snapshotId,
    expectedFormat: format,
    onYouthImported,
    onMoneyballImported,
  });
  const previousContextKey = useRef(contextKey);
  const wasOpen = useRef(false);
  const isDragging = useNativeCsvDrop(
    open,
    captureSelection,
    (paths, selection) => {
      if (paths.length !== 1) {
        reportError(new Error("Drop one CSV file at a time"), selection);
        return;
      }
      void importPath(paths[0], selection);
    },
  );

  useEffect(() => {
    if (open && !wasOpen.current) {
      reset();
    }
    wasOpen.current = open;
  }, [open, reset]);

  useEffect(() => {
    if (previousContextKey.current !== contextKey) {
      previousContextKey.current = contextKey;
      if (open) {
        onClose();
      }
    }
  }, [contextKey, onClose, open]);

  const label = formatName(format);
  const isPending = state.status === "pending";
  const close = () => {
    if (!isPending) {
      onClose();
    }
  };

  return (
    <Modal
      open={open}
      title={`Upload ${label} CSV`}
      onClose={close}
      footer={
        <>
          <Button variant="secondary" disabled={isPending} onClick={close}>
            Close
          </Button>
          <Button
            icon={FileUp}
            loading={isPending}
            loadingLabel="Importing CSV…"
            onClick={() => {
              void chooseCsv();
            }}
          >
            Browse files
          </Button>
        </>
      }
    >
      <div
        className={`rounded-lg border border-dashed p-6 text-center transition-colors duration-150 ${
          isDragging
            ? "border-primary bg-primary-container/40"
            : "border-outline-variant bg-surface-container-high"
        }`}
      >
        <FileUp
          aria-hidden="true"
          size={24}
          strokeWidth={1.5}
          className="mx-auto mb-3 text-primary"
        />
        <p className="text-body-md text-on-surface">
          Drop one CSV file here, or browse your files.
        </p>
        <p className="mt-1 text-body-sm text-on-surface-variant">
          Only a {label} export can be imported from this dialog.
        </p>
      </div>
      <div className="mt-4">
        <CsvImportOutcome
          state={state}
          expectedFormat={format}
          youthLabel="Youth Academy"
        />
      </div>
    </Modal>
  );
}
