import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveBusyDumpRequest,
  setBridgeStatusIpcMockMode,
  setDumpRequestIpcMockMode,
} from "../api/bridge-status-ipc-mock";

describe("bridge status panel", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
    setDumpRequestIpcMockMode("success");
  });

  afterEach(() => {
    resolveBusyDumpRequest();
  });

  it("renders ready state from mock IPC", async () => {
    renderWithProviders();

    expect(await screen.findByText(/^Bridge:/i)).toHaveTextContent("ready");
    expect(screen.getByText(/^Plugin version:/i)).toHaveTextContent("0.1.0");
    expect(screen.getByText(/^FM modules:/i)).toHaveTextContent("detected");
  });

  it("shows missing bridge guidance when status file is absent", async () => {
    setBridgeStatusIpcMockMode("missing");
    renderWithProviders();

    expect(await screen.findByText(/Bridge not detected/i)).toBeInTheDocument();
    expect(screen.getByText(/BepInEx\/plugins/i)).toBeInTheDocument();
    expect(screen.getByText(/bridge\/README\.md/i)).toBeInTheDocument();
  });

  it("shows unsupported platform message on non-Windows hosts", async () => {
    setBridgeStatusIpcMockMode("unsupportedPlatform");
    renderWithProviders();

    expect(await screen.findByText(/Windows required/i)).toBeInTheDocument();
    expect(screen.getByText(/only supported on Windows/i)).toBeInTheDocument();
  });

  it("shows version mismatch message for unsupported protocol", async () => {
    setBridgeStatusIpcMockMode("unsupportedVersion");
    renderWithProviders();

    expect(
      await screen.findByText(/Bridge version mismatch/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/unsupported protocol version/i),
    ).toBeInTheDocument();
  });

  it("shows corrupt status error and retries to ready", async () => {
    setBridgeStatusIpcMockMode("corrupt");
    const user = userEvent.setup();
    renderWithProviders();

    expect(
      await screen.findByText(/Could not read bridge status/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/not valid JSON/i)).toBeInTheDocument();

    setBridgeStatusIpcMockMode("ready");
    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByText(/^Bridge:/i)).toHaveTextContent("ready");
  });

  it("Load Data trigger shows success after dump request", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Dump ready \(12 players\)/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows truncated dump warning", async () => {
    setDumpRequestIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Partial dump \(capped at 10000 players\)/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows scan failure from bridge", async () => {
    setDumpRequestIpcMockMode("failed");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Scan failed: scan produced zero player/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows IPC timeout error", async () => {
    setDumpRequestIpcMockMode("timeout");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Could not request dump/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/timed out waiting/i)).toBeInTheDocument();
  });

  it("Load Data button shows busy label while request is pending", async () => {
    setDumpRequestIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();
    expect(
      await screen.findByText(/Waiting for the FM bridge dump/i),
    ).toBeInTheDocument();
  });
});
