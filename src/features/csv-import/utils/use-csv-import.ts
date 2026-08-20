import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { importCsv } from "../api/import-csv";
import type {
  CsvImportFormat,
  CsvImportSummary,
} from "../types/csv-import-summary";
import { type CsvImportState, importStateForContext } from "./import-state";

type UseCsvImportOptions = {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
  expectedFormat?: CsvImportFormat;
  onYouthImported: () => void;
  onMoneyballImported?: (summary: CsvImportSummary) => void;
};

export type CsvImportSelection = {
  contextKey: string;
  contextGeneration: number;
};

export function useCsvImport({
  activeSaveId,
  snapshotId,
  expectedFormat,
  onYouthImported,
  onMoneyballImported,
}: UseCsvImportOptions) {
  const contextKey = `${activeSaveId ?? "none"}:${snapshotId ?? "none"}`;
  const currentContext = useRef(contextKey);
  const contextGeneration = useRef(0);
  const activeImport = useRef<symbol | null>(null);
  const [state, setState] = useState<CsvImportState>({ status: "idle" });
  const visibleState = importStateForContext(state, contextKey);

  useLayoutEffect(() => {
    currentContext.current = contextKey;
    contextGeneration.current += 1;
    activeImport.current = null;
    setState({ status: "idle" });
  }, [contextKey]);

  const captureSelection = useCallback(
    (): CsvImportSelection => ({
      contextKey,
      contextGeneration: contextGeneration.current,
    }),
    [contextKey],
  );

  const isCurrentSelection = (selection: CsvImportSelection) =>
    currentContext.current === selection.contextKey &&
    contextGeneration.current === selection.contextGeneration;

  const reportError = (
    error: Error,
    selection: CsvImportSelection = captureSelection(),
  ) => {
    if (!isCurrentSelection(selection) || activeImport.current) {
      return;
    }
    setState({
      status: "error",
      contextKey: selection.contextKey,
      error,
    });
  };

  const importPath = async (
    path: string,
    selection: CsvImportSelection = captureSelection(),
  ) => {
    if (!isCurrentSelection(selection) || activeImport.current) {
      return;
    }

    const importId = Symbol("csv-import");
    activeImport.current = importId;
    setState({ status: "pending", contextKey: selection.contextKey });
    try {
      const summary = await importCsv(path, expectedFormat);
      if (isCurrentSelection(selection)) {
        setState({
          status: "success",
          contextKey: selection.contextKey,
          summary,
        });
        if (summary.format === "youthTracker") {
          onYouthImported();
        } else {
          onMoneyballImported?.(summary);
        }
      }
    } catch (error) {
      if (isCurrentSelection(selection)) {
        setState({
          status: "error",
          contextKey: selection.contextKey,
          error: error instanceof Error ? error : new Error(String(error)),
        });
      }
    } finally {
      if (activeImport.current === importId) {
        activeImport.current = null;
      }
    }
  };

  const chooseCsv = async () => {
    const selection = captureSelection();
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!isCurrentSelection(selection) || !path) {
        return;
      }
      await importPath(path, selection);
    } catch (error) {
      reportError(
        error instanceof Error ? error : new Error(String(error)),
        selection,
      );
    }
  };

  return {
    contextKey,
    state: visibleState,
    captureSelection,
    chooseCsv,
    importPath,
    reportError,
    reset: () => {
      setState({ status: "idle" });
    },
  };
}
