import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getLastCsvPreviewIpcArgs,
  resolveBusyCsvPreviewRequest,
  setCsvPreviewIpcMockBusy,
  setCsvPreviewIpcMockError,
  setCsvPreviewIpcMockResult,
} from "@/testing/csv-import-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";

const open = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));

describe("Dashboard CSV reconciliation preview", () => {
  beforeEach(() => {
    open.mockReset();
    open.mockResolvedValue(null);
  });

  it("guides the user to load a snapshot before selecting a CSV", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("heading", { name: "CSV reconciliation" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Load Data before previewing a CSV export/i),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose CSV" })).toBeDisabled();
  });

  it("leaves the preview idle when the native picker is cancelled", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(open).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    expect(getLastCsvPreviewIpcArgs()).toBeUndefined();
    expect(
      screen.getByText(/Choose one Youth Tracker or Moneyball export/i),
    ).toBeInTheDocument();
  });

  it("uses safe error copy when the native picker fails", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\dialog-failure.csv";
    open.mockRejectedValue(new Error(`Could not open ${privatePath}`));
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not preview CSV.",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("shows a pending label while the preview command runs", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockBusy();
    open.mockResolvedValue("C:\\exports\\players.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(
      screen.getByRole("button", { name: "Checking CSV…" }),
    ).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Checking CSV");

    resolveBusyCsvPreviewRequest();
    expect(
      await screen.findByText(/Youth Tracker detected/i),
    ).toBeInTheDocument();
  });

  it("shows a complete Youth Tracker match without exposing the selected path", async () => {
    const user = userEvent.setup();
    const path = "C:\\Users\\Jonas\\Documents\\private-export.csv";
    setCsvPreviewIpcMockResult({
      format: "youthTracker",
      totalPlayers: 74,
      matchedPlayers: 74,
      unmatchedPlayers: 0,
    });
    open.mockResolvedValue(path);
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(
      await screen.findByText(/Youth Tracker detected/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText("74 of 74 player IDs match the current snapshot."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Every exported player ID matches/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(path)).not.toBeInTheDocument();
    expect(getLastCsvPreviewIpcArgs()).toEqual({ path });
  });

  it("shows the Moneyball format and unmatched-player warning", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockResult({
      format: "moneyball",
      totalPlayers: 75,
      matchedPlayers: 74,
      unmatchedPlayers: 1,
    });
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(await screen.findByText(/Moneyball detected/i)).toBeInTheDocument();
    expect(
      screen.getByText("74 of 75 player IDs match the current snapshot."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/1 exported player ID does not match/i),
    ).toBeInTheDocument();
  });

  it("uses safe error copy instead of a native error message", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\secret.csv";
    setCsvPreviewIpcMockError(new Error(`Could not read ${privatePath}`));
    open.mockResolvedValue(privatePath);
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not preview CSV.",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("shows safe record context for invalid CSV data", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\duplicate.csv";
    setCsvPreviewIpcMockError(
      new Error(
        "CSV file is invalid: CSV record 3 repeats the Unique ID from record 2",
      ),
    );
    open.mockResolvedValue(privatePath);
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "CSV record 3 repeats the Unique ID from record 2",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("explains when the snapshot changes while a CSV is being checked", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockError(
      new Error("The current save changed while the CSV was read"),
    );
    open.mockResolvedValue("C:\\exports\\players.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Snapshot changed.",
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Select the CSV again.",
    );
  });

  it("clears a completed preview when Load Data replaces the current snapshot", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockResult({
      format: "moneyball",
      totalPlayers: 75,
      matchedPlayers: 74,
      unmatchedPlayers: 1,
    });
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    expect(await screen.findByText(/Moneyball detected/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() => {
      expect(screen.queryByText(/Moneyball detected/i)).not.toBeInTheDocument();
    });
    expect(
      screen.getByText(/Choose one Youth Tracker or Moneyball export/i),
    ).toBeInTheDocument();
  });

  it("does not restore a late preview after Load Data replaces the snapshot", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockBusy();
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    expect(screen.getByRole("status")).toHaveTextContent("Checking CSV");

    await user.click(screen.getByRole("button", { name: "Load Data" }));
    expect(
      await screen.findByText(/Choose one Youth Tracker or Moneyball export/i),
    ).toBeInTheDocument();

    resolveBusyCsvPreviewRequest({
      format: "moneyball",
      totalPlayers: 75,
      matchedPlayers: 74,
      unmatchedPlayers: 1,
    });

    await waitFor(() => {
      expect(screen.queryByText(/Moneyball detected/i)).not.toBeInTheDocument();
    });
  });

  it("clears a completed preview when the active save changes", async () => {
    const user = userEvent.setup();
    setCsvPreviewIpcMockResult({
      format: "youthTracker",
      totalPlayers: 74,
      matchedPlayers: 74,
      unmatchedPlayers: 0,
    });
    open.mockResolvedValue("C:\\exports\\youth.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Choose CSV" }));
    expect(
      await screen.findByText(/Youth Tracker detected/i),
    ).toBeInTheDocument();

    await user.type(screen.getByLabelText("New save"), "Second save");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    expect(
      await screen.findByText(/Load Data before previewing a CSV export/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Youth Tracker detected/i),
    ).not.toBeInTheDocument();
  });
});
