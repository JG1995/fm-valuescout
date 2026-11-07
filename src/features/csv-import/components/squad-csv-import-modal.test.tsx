import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ComponentProps } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getLastCsvImportIpcArgs,
  setCsvImportIpcMockBusy,
  setCsvImportIpcMockError,
  setCsvImportIpcMockResult,
} from "@/testing/csv-import-ipc-mock";
import { SquadCsvImportModal } from "./squad-csv-import-modal";

const { open, onDragDropEvent } = vi.hoisted(() => ({
  open: vi.fn(),
  onDragDropEvent: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));
vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent }),
}));

function renderModal(
  overrides: Partial<ComponentProps<typeof SquadCsvImportModal>> = {},
) {
  const onClose = vi.fn();
  const onYouthImported = vi.fn();
  return {
    onClose,
    onYouthImported,
    ...render(
      <SquadCsvImportModal
        activeSaveId={1}
        snapshotId={1}
        open={true}
        onClose={onClose}
        onYouthImported={onYouthImported}
        {...overrides}
      />,
    ),
  };
}

describe("SquadCsvImportModal", () => {
  beforeEach(() => {
    open.mockReset();
    onDragDropEvent.mockReset();
    onDragDropEvent.mockResolvedValue(vi.fn());
  });

  it("browses one Youth Academy CSV and imports it", async () => {
    const user = userEvent.setup();
    open.mockResolvedValue("C:\\exports\\youth.csv");
    setCsvImportIpcMockResult({
      format: "youthTracker",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });
    const { onYouthImported } = renderModal();

    expect(
      screen.getByRole("dialog", { name: "Upload Youth Academy CSV" }),
    ).toHaveTextContent("Drop one CSV file here, or browse your files.");
    expect(
      screen.getByText(/Matching player rows add or update CSV enrichment/i),
    ).toHaveTextContent("Omitted players keep their earlier enrichment");
    await user.click(screen.getByRole("button", { name: "Browse files" }));

    expect(open).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    await waitFor(() => {
      expect(getLastCsvImportIpcArgs()).toEqual({
        path: "C:\\exports\\youth.csv",
      });
    });
    expect(
      await screen.findByText(/Youth Academy imported/i),
    ).toBeInTheDocument();
    expect(onYouthImported).toHaveBeenCalledOnce();
  });

  it("stays idle when the native picker is cancelled", async () => {
    const user = userEvent.setup();
    open.mockResolvedValue(null);
    renderModal();

    await user.click(screen.getByRole("button", { name: "Browse files" }));

    expect(getLastCsvImportIpcArgs()).toBeUndefined();
    expect(
      screen.getByText(/Choose a Youth Academy CSV export to import/i),
    ).toBeInTheDocument();
  });

  it("keeps the modal open while an import is pending", async () => {
    const user = userEvent.setup();
    open.mockResolvedValue("C:\\exports\\youth.csv");
    setCsvImportIpcMockBusy();
    const { onClose } = renderModal();

    await user.click(screen.getByRole("button", { name: "Browse files" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Importing CSV",
    );
    await user.keyboard("{Escape}");

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Close" })).toBeDisabled();
  });

  it("ignores a second native drop while the first import is pending", async () => {
    let onDrop:
      | ((event: { payload: { type: string; paths?: string[] } }) => void)
      | undefined;
    onDragDropEvent.mockImplementation(async (handler) => {
      onDrop = handler;
      return vi.fn();
    });
    setCsvImportIpcMockBusy();
    renderModal();

    await waitFor(() => expect(onDrop).toBeDefined());
    act(() => {
      onDrop?.({
        payload: { type: "drop", paths: ["C:\\exports\\first.csv"] },
      });
      onDrop?.({
        payload: { type: "drop", paths: ["C:\\exports\\second.csv"] },
      });
    });

    await waitFor(() => {
      expect(getLastCsvImportIpcArgs()).toEqual({
        path: "C:\\exports\\first.csv",
      });
    });
  });

  it("keeps the pending import active after a multi-file native drop", async () => {
    let onDrop:
      | ((event: { payload: { type: string; paths?: string[] } }) => void)
      | undefined;
    onDragDropEvent.mockImplementation(async (handler) => {
      onDrop = handler;
      return vi.fn();
    });
    setCsvImportIpcMockBusy();
    renderModal();

    await waitFor(() => expect(onDrop).toBeDefined());
    act(() => {
      onDrop?.({
        payload: { type: "drop", paths: ["C:\\exports\\first.csv"] },
      });
      onDrop?.({
        payload: {
          type: "drop",
          paths: ["C:\\exports\\second.csv", "C:\\exports\\third.csv"],
        },
      });
    });

    await waitFor(() => {
      expect(getLastCsvImportIpcArgs()).toEqual({
        path: "C:\\exports\\first.csv",
      });
    });
    expect(screen.getByRole("status")).toHaveTextContent("Importing CSV");
    expect(screen.getByRole("button", { name: "Close" })).toBeDisabled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("imports one dropped Youth Academy CSV and removes its native listener on close", async () => {
    const unlisten = vi.fn();
    let onDrop:
      | ((event: { payload: { type: string; paths?: string[] } }) => void)
      | undefined;
    onDragDropEvent.mockImplementation(async (handler) => {
      onDrop = handler;
      return unlisten;
    });
    setCsvImportIpcMockResult({
      format: "youthTracker",
      totalPlayers: 3,
      storedPlayers: 3,
      skippedPlayers: 0,
    });
    const { onYouthImported, rerender } = renderModal();

    await waitFor(() => expect(onDrop).toBeDefined());
    act(() => {
      onDrop?.({
        payload: { type: "drop", paths: ["C:\\exports\\youth.csv"] },
      });
    });

    await waitFor(() => {
      expect(getLastCsvImportIpcArgs()).toEqual({
        path: "C:\\exports\\youth.csv",
      });
    });
    expect(onYouthImported).toHaveBeenCalledOnce();

    rerender(
      <SquadCsvImportModal
        activeSaveId={1}
        snapshotId={1}
        open={false}
        onClose={vi.fn()}
        onYouthImported={onYouthImported}
      />,
    );
    await waitFor(() => expect(unlisten).toHaveBeenCalledOnce());
  });

  it("explains a rejected Moneyball export without exposing the local path", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\private-moneyball.csv";
    open.mockResolvedValue(privatePath);
    setCsvImportIpcMockError(
      new Error(
        "Moneyball stats now come from the linked FM save file, not a CSV export",
      ),
    );
    renderModal();

    await user.click(screen.getByRole("button", { name: "Browse files" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Moneyball stats now come from the FM save file linked in Settings.",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("clears import feedback and closes when its save context changes", async () => {
    const user = userEvent.setup();
    open.mockResolvedValue("C:\\exports\\youth.csv");
    setCsvImportIpcMockResult({
      format: "youthTracker",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });
    const { onClose, onYouthImported, rerender } = renderModal();

    await user.click(screen.getByRole("button", { name: "Browse files" }));
    expect(
      await screen.findByText(/Youth Academy imported/i),
    ).toBeInTheDocument();

    rerender(
      <SquadCsvImportModal
        activeSaveId={1}
        snapshotId={2}
        open={true}
        onClose={onClose}
        onYouthImported={onYouthImported}
      />,
    );

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(
      screen.getByText(/Choose a Youth Academy CSV export to import/i),
    ).toBeInTheDocument();
  });

  it("does not import a drop delivered from a replaced save context", async () => {
    let firstDrop:
      | ((event: { payload: { type: string; paths?: string[] } }) => void)
      | undefined;
    onDragDropEvent.mockImplementation(async (handler) => {
      firstDrop ??= handler;
      return vi.fn();
    });
    const { onClose, onYouthImported, rerender } = renderModal();

    await waitFor(() => expect(firstDrop).toBeDefined());
    rerender(
      <SquadCsvImportModal
        activeSaveId={1}
        snapshotId={2}
        open={true}
        onClose={onClose}
        onYouthImported={onYouthImported}
      />,
    );
    act(() => {
      firstDrop?.({
        payload: { type: "drop", paths: ["C:\\exports\\stale.csv"] },
      });
    });

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(getLastCsvImportIpcArgs()).toBeUndefined();
  });

  it("rejects a drop containing more than one path", async () => {
    let onDrop:
      | ((event: { payload: { type: string; paths?: string[] } }) => void)
      | undefined;
    onDragDropEvent.mockImplementation(async (handler) => {
      onDrop = handler;
      return vi.fn();
    });
    renderModal();

    await waitFor(() => expect(onDrop).toBeDefined());
    act(() => {
      onDrop?.({
        payload: {
          type: "drop",
          paths: ["C:\\exports\\one.csv", "C:\\exports\\two.csv"],
        },
      });
    });

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Drop one CSV file at a time.",
    );
    expect(getLastCsvImportIpcArgs()).toBeUndefined();
  });
});
