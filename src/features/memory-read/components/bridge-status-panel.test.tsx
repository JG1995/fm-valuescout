import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import { setBridgeStatusIpcMockMode } from "../api/bridge-status-ipc-mock";

describe("bridge status panel", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
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
});
