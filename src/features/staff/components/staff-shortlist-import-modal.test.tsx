import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { StaffShortlistImportModal } from "./staff-shortlist-import-modal";

const { openFileDialog, invokeCommand } = vi.hoisted(() => ({
  openFileDialog: vi.fn(),
  invokeCommand: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openFileDialog }));
vi.mock("@/lib/tauri-client", () => ({ invokeCommand }));

describe("StaffShortlistImportModal", () => {
  beforeEach(() => {
    openFileDialog.mockReset();
    invokeCommand.mockReset();
  });

  it("reports the import summary before closing", async () => {
    openFileDialog.mockResolvedValue("C:\\exports\\staff.csv");
    invokeCommand.mockResolvedValue({
      totalStaff: 2,
      storedStaff: 2,
      skippedStaff: 0,
    });
    const onClose = vi.fn();
    const onImported = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(
      <StaffShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        replacesExisting={false}
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    await waitFor(() => {
      expect(onImported).toHaveBeenCalledWith({
        totalStaff: 2,
        storedStaff: 2,
        skippedStaff: 0,
      });
      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  it("reports pending context work until a deferred import completes", async () => {
    let resolveImport: (summary: {
      totalStaff: number;
      storedStaff: number;
      skippedStaff: number;
    }) => void = () => {
      throw new Error("Expected import resolver");
    };
    openFileDialog.mockResolvedValue("C:\\exports\\staff.csv");
    invokeCommand.mockReturnValue(
      new Promise((resolve) => {
        resolveImport = resolve;
      }),
    );
    const onPendingChange = vi.fn();
    const user = userEvent.setup();
    render(
      <StaffShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        replacesExisting={false}
        onClose={vi.fn()}
        onImported={vi.fn().mockResolvedValue(undefined)}
        onPendingChange={onPendingChange}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    await waitFor(() => expect(onPendingChange).toHaveBeenCalledWith(true));
    resolveImport({ totalStaff: 1, storedStaff: 1, skippedStaff: 0 });
    await waitFor(() =>
      expect(onPendingChange).toHaveBeenLastCalledWith(false),
    );
  });

  it("abandons a picked file when the active snapshot changes", async () => {
    let resolvePath: (path: string) => void = (_path) => {
      throw new Error("Expected file picker resolver");
    };
    openFileDialog.mockReturnValue(
      new Promise<string>((resolve) => {
        resolvePath = resolve;
      }),
    );
    const onClose = vi.fn();
    const onImported = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    const { rerender } = render(
      <StaffShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        replacesExisting={false}
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    rerender(
      <StaffShortlistImportModal
        activeSaveId={1}
        snapshotId={2}
        open
        replacesExisting={false}
        onClose={onClose}
        onImported={onImported}
      />,
    );
    resolvePath("C:\\exports\\staff.csv");

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(invokeCommand).not.toHaveBeenCalled();
    expect(onImported).not.toHaveBeenCalled();
  });
});
