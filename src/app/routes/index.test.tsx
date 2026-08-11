import { act, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { academyKeys } from "@/features/academy/api/academy-keys";
import {
  getLastCsvImportIpcArgs,
  resolveBusyCsvImportRequest,
  setCsvImportIpcMockBusy,
  setCsvImportIpcMockError,
  setCsvImportIpcMockResult,
} from "@/testing/csv-import-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";

const open = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/plugin-dialog", () => ({ open }));

describe("Dashboard CSV enrichment import", () => {
  beforeEach(() => {
    open.mockReset();
    open.mockResolvedValue(null);
  });

  it("guides the user to load a snapshot before importing a CSV", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("heading", { name: "CSV enrichment" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Load Data before importing a CSV export/i),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import CSV" })).toBeDisabled();
  });

  it("leaves the import idle when the native picker is cancelled", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(open).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    expect(getLastCsvImportIpcArgs()).toBeUndefined();
    expect(
      screen.getByText(/Choose a Youth Tracker or Moneyball export to import/i),
    ).toBeInTheDocument();
  });

  it("uses safe error copy when the native picker fails", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\dialog-failure.csv";
    open.mockRejectedValue(new Error(`Could not open ${privatePath}`));
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not import CSV.",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("shows a pending label while the import command runs", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockBusy();
    open.mockResolvedValue("C:\\exports\\players.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(
      screen.getByRole("button", { name: "Importing CSV…" }),
    ).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Importing CSV");

    resolveBusyCsvImportRequest();
    expect(
      await screen.findByText(/Youth Tracker imported/i),
    ).toBeInTheDocument();
  });

  it("reports a Youth Tracker import without exposing its path and refreshes Academy", async () => {
    const user = userEvent.setup();
    const path = "C:\\Users\\Jonas\\Documents\\private-export.csv";
    setCsvImportIpcMockResult({
      format: "youthTracker",
      totalPlayers: 74,
      storedPlayers: 74,
      skippedPlayers: 0,
    });
    open.mockResolvedValue(path);
    const { queryClient } = renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    queryClient.setQueryData(academyKeys.classes(), []);
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(
      await screen.findByText(/Youth Tracker imported/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText("74 of 74 player IDs were stored."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Every exported player ID was stored/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(path)).not.toBeInTheDocument();
    expect(getLastCsvImportIpcArgs()).toEqual({ path });
    await waitFor(() => {
      expect(
        queryClient.getQueryState(academyKeys.classes())?.isInvalidated,
      ).toBe(true);
    });
  });

  it("reports skipped Moneyball player IDs without invalidating Academy", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockResult({
      format: "moneyball",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    const { queryClient } = renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    queryClient.setQueryData(academyKeys.classes(), []);
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(await screen.findByText(/Moneyball imported/i)).toBeInTheDocument();
    expect(
      screen.getByText("74 of 75 player IDs were stored."),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /1 player ID was skipped because it does not match the current snapshot/i,
      ),
    ).toBeInTheDocument();
    expect(
      queryClient.getQueryState(academyKeys.classes())?.isInvalidated,
    ).toBe(false);
  });

  it("uses safe error copy instead of a native error message", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\secret.csv";
    setCsvImportIpcMockError(new Error(`Could not read ${privatePath}`));
    open.mockResolvedValue(privatePath);
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not import CSV.",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("shows safe record context for invalid CSV data", async () => {
    const user = userEvent.setup();
    const privatePath = "C:\\Users\\Jonas\\duplicate.csv";
    setCsvImportIpcMockError(
      new Error(
        "CSV file is invalid: CSV record 3 repeats the Unique ID from record 2",
      ),
    );
    open.mockResolvedValue(privatePath);
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "CSV record 3 repeats the Unique ID from record 2",
    );
    expect(screen.queryByText(privatePath)).not.toBeInTheDocument();
  });

  it("explains when the snapshot changes while a CSV is being imported", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockError(
      new Error(
        "The current save or snapshot changed while the CSV was imported",
      ),
    );
    open.mockResolvedValue("C:\\exports\\players.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Snapshot changed.",
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Select the CSV again.",
    );
  });

  it("clears a completed import when Load Data replaces the current snapshot", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockResult({
      format: "moneyball",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));
    expect(await screen.findByText(/Moneyball imported/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() => {
      expect(screen.queryByText(/Moneyball imported/i)).not.toBeInTheDocument();
    });
    expect(
      screen.getByText(/Choose a Youth Tracker or Moneyball export to import/i),
    ).toBeInTheDocument();
  });

  it("does not restore a late import after Load Data replaces the snapshot", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockBusy();
    open.mockResolvedValue("C:\\exports\\moneyball.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));
    expect(screen.getByRole("status")).toHaveTextContent("Importing CSV");

    await user.click(screen.getByRole("button", { name: "Load Data" }));
    expect(
      await screen.findByText(
        /Choose a Youth Tracker or Moneyball export to import/i,
      ),
    ).toBeInTheDocument();

    resolveBusyCsvImportRequest({
      format: "moneyball",
      totalPlayers: 75,
      storedPlayers: 74,
      skippedPlayers: 1,
    });

    await waitFor(() => {
      expect(screen.queryByText(/Moneyball imported/i)).not.toBeInTheDocument();
    });
  });

  it("does not restore a late import after returning to the same save", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockBusy();
    open.mockResolvedValue("C:\\exports\\youth.csv");
    const { queryClient } = renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    queryClient.setQueryData(academyKeys.classes(), []);
    await user.click(screen.getByRole("button", { name: "Import CSV" }));
    expect(screen.getByRole("status")).toHaveTextContent("Importing CSV");

    await user.type(screen.getByLabelText("New save"), "Second save");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );
    await screen.findByText(/Load Data before importing a CSV export/i);

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "1",
    );
    await screen.findByText(
      /Choose a Youth Tracker or Moneyball export to import/i,
    );

    await act(async () => {
      resolveBusyCsvImportRequest();
    });

    expect(
      screen.queryByText(/Youth Tracker imported/i),
    ).not.toBeInTheDocument();
    expect(
      queryClient.getQueryState(academyKeys.classes())?.isInvalidated,
    ).toBe(false);
  });

  it("clears a completed import when the active save changes", async () => {
    const user = userEvent.setup();
    setCsvImportIpcMockResult({
      format: "youthTracker",
      totalPlayers: 74,
      storedPlayers: 74,
      skippedPlayers: 0,
    });
    open.mockResolvedValue("C:\\exports\\youth.csv");
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await user.click(screen.getByRole("button", { name: "Import CSV" }));
    expect(
      await screen.findByText(/Youth Tracker imported/i),
    ).toBeInTheDocument();

    await user.type(screen.getByLabelText("New save"), "Second save");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    expect(
      await screen.findByText(/Load Data before importing a CSV export/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Youth Tracker imported/i),
    ).not.toBeInTheDocument();
  });
});
