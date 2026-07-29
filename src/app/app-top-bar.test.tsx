import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { setBridgeStatusIpcMockMode } from "@/features/memory-read/api/bridge-status-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveBusyLoadDataRequest,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

// Load Data lives in the shell top bar, so its outcome banner is asserted here
// rather than against the bridge panel that used to own the button.
describe("app top bar", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
    setLoadDataIpcMockMode("success");
  });

  afterEach(() => {
    resolveBusyLoadDataRequest();
  });

  it("reports ingest success after load_data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
  });

  it("warns that a capped scan produced a partial ingest", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/the scan was capped at 500 players/i),
    ).toBeInTheDocument();
  });

  it("reports a scan failure from load_data", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();
    expect(
      screen.getByText(/scan produced zero player candidates/i),
    ).toBeInTheDocument();
  });

  it("reports an ingest failure from load_data", async () => {
    setLoadDataIpcMockMode("ingestFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Ingest failed/i)).toBeInTheDocument();
    expect(screen.getByText(/dump validation failed/i)).toBeInTheDocument();
  });

  it("drops a failure banner once the user switches save", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();

    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    // The failure described the previous save's scan, not this one's.
    expect(screen.queryByText(/Scan failed/i)).not.toBeInTheDocument();
  });

  it("swaps the button label for the scan phase while the request is pending", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();
  });
});
