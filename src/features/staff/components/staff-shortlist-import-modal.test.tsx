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

  it("closes after importing a selected staff CSV", async () => {
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
        open
        replacesExisting={false}
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    await waitFor(() => {
      expect(onImported).toHaveBeenCalledOnce();
      expect(onClose).toHaveBeenCalledOnce();
    });
  });
});
