import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PlayerShortlistImportModal } from "./player-shortlist-import-modal";

const { openFileDialog, invokeCommand } = vi.hoisted(() => ({
  openFileDialog: vi.fn(),
  invokeCommand: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openFileDialog }));
vi.mock("@/lib/tauri-client", () => ({ invokeCommand }));

describe("PlayerShortlistImportModal", () => {
  beforeEach(() => {
    openFileDialog.mockReset();
    invokeCommand.mockReset();
  });

  it("reports the import summary before closing", async () => {
    openFileDialog.mockResolvedValue("C:\\exports\\shortlist.csv");
    invokeCommand.mockResolvedValue({
      totalPlayers: 3,
      storedPlayers: 2,
      skippedPlayers: 1,
    });
    const onClose = vi.fn();
    const onImported = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(
      <PlayerShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    await waitFor(() => {
      expect(invokeCommand).toHaveBeenCalledWith(
        "import_player_shortlist_csv",
        { path: "C:\\exports\\shortlist.csv" },
      );
      expect(onImported).toHaveBeenCalledWith({
        totalPlayers: 3,
        storedPlayers: 2,
        skippedPlayers: 1,
      });
      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  it("surfaces a zero-match error without closing", async () => {
    openFileDialog.mockResolvedValue("C:\\exports\\shortlist.csv");
    invokeCommand.mockRejectedValue(
      new Error("CSV does not contain players in the current snapshot"),
    );
    const onClose = vi.fn();
    const onImported = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(
      <PlayerShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    await waitFor(() => {
      expect(
        screen.getByText(
          "CSV does not contain players in the current snapshot",
        ),
      ).toBeInTheDocument();
    });
    expect(onClose).not.toHaveBeenCalled();
    expect(onImported).not.toHaveBeenCalled();
  });

  it("abandons a picked file when context tokens change for the same save and snapshot", async () => {
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
      <PlayerShortlistImportModal
        activeSaveId={1}
        activeSaveContextToken="save-token-1"
        snapshotId={1}
        snapshotContextToken="snapshot-token-1"
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    rerender(
      <PlayerShortlistImportModal
        activeSaveId={1}
        activeSaveContextToken="save-token-2"
        snapshotId={1}
        snapshotContextToken="snapshot-token-2"
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );
    resolvePath("C:\\exports\\shortlist.csv");

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(invokeCommand).not.toHaveBeenCalled();
    expect(onImported).not.toHaveBeenCalled();
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
      <PlayerShortlistImportModal
        activeSaveId={1}
        snapshotId={1}
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    rerender(
      <PlayerShortlistImportModal
        activeSaveId={1}
        snapshotId={2}
        open
        onClose={onClose}
        onImported={onImported}
      />,
    );
    resolvePath("C:\\exports\\shortlist.csv");

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(invokeCommand).not.toHaveBeenCalled();
    expect(onImported).not.toHaveBeenCalled();
  });
});
