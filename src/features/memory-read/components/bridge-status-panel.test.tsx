import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveBusyLoadDataRequest,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";
import { setBridgeInstallIpcMockMode } from "../api/bridge-install-ipc-mock";
import {
  resolveBusyDumpRequest,
  setBridgeStatusIpcMockMode,
  setDumpRequestIpcMockMode,
} from "../api/bridge-status-ipc-mock";

describe("bridge status panel", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
    setDumpRequestIpcMockMode("success");
    setLoadDataIpcMockMode("success");
    setBridgeInstallIpcMockMode("absent");
  });

  afterEach(() => {
    resolveBusyDumpRequest();
    resolveBusyLoadDataRequest();
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
    expect(screen.getAllByText(/BepInEx\/plugins/i).length).toBeGreaterThan(0);
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

  it("Load Data trigger shows ingest success after load_data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows truncated ingest warning", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Partial ingest \(capped at 10000 players\)/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows scan failure from load_data", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Could not load data/i)).toBeInTheDocument();
    expect(
      screen.getByText(/scan produced zero player candidates/i),
    ).toBeInTheDocument();
  });

  it("Load Data trigger shows ingest failure from load_data", async () => {
    setLoadDataIpcMockMode("ingestFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Could not load data/i)).toBeInTheDocument();
    expect(screen.getByText(/dump validation failed/i)).toBeInTheDocument();
  });

  it("Load Data button shows busy label while request is pending", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge:/i);
    await user.click(screen.getByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByRole("button", { name: "Loading…" }),
    ).toBeDisabled();
    expect(
      await screen.findByText(/Scanning and ingesting FM data/i),
    ).toBeInTheDocument();
  });

  it("renders plugin install status from mock IPC", async () => {
    renderWithProviders();

    expect(
      await screen.findByText(/^Bridge plugin install/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/^Plugin DLL:/i)).toHaveTextContent(
      "not installed",
    );
    expect(screen.getByText(/^BepInEx:/i)).toHaveTextContent("found");
    expect(
      screen.getByRole("button", { name: "Install plugin" }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Remove plugin" }),
    ).toBeDisabled();
  });

  it("install plugin action updates install status", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge plugin install/i);
    await user.click(screen.getByRole("button", { name: "Install plugin" }));

    expect(
      await screen.findByText(/Plugin installed\. Restart Football Manager/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/^Plugin DLL:/i)).toHaveTextContent("installed");
    expect(screen.getByRole("button", { name: "Update plugin" })).toBeEnabled();
  });

  it("remove plugin action clears installed status", async () => {
    setBridgeInstallIpcMockMode("installed");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Plugin DLL:/i);
    expect(screen.getByText(/^Plugin DLL:/i)).toHaveTextContent("installed");

    await user.click(screen.getByRole("button", { name: "Remove plugin" }));

    expect(await screen.findByText(/Plugin removed from/i)).toBeInTheDocument();
    expect(screen.getByText(/^Plugin DLL:/i)).toHaveTextContent(
      "not installed",
    );
  });

  it("shows install failure when BepInEx is missing", async () => {
    setBridgeInstallIpcMockMode("bepinexMissing");
    renderWithProviders();

    await screen.findByText(/^Bridge plugin install/i);
    expect(
      screen.getByRole("button", { name: "Install plugin" }),
    ).toBeDisabled();
    expect(screen.getByText(/^BepInEx:/i)).toHaveTextContent("not found");
  });

  it("shows Windows-only install error without crashing the home page", async () => {
    setBridgeInstallIpcMockMode("unsupportedPlatform");
    renderWithProviders();

    expect(
      await screen.findByText(/Windows required for plugin install/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/only supported on Windows/i)).toBeInTheDocument();
    expect(await screen.findByText(/^Bridge:/i)).toHaveTextContent("ready");
  });

  it("shows only remove success after install then remove", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Bridge plugin install/i);
    await user.click(screen.getByRole("button", { name: "Install plugin" }));
    expect(
      await screen.findByText(/Plugin installed\. Restart Football Manager/i),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Remove plugin" }));

    expect(await screen.findByText(/Plugin removed from/i)).toBeInTheDocument();
    expect(
      screen.queryByText(/Plugin installed\. Restart Football Manager/i),
    ).not.toBeInTheDocument();
  });
});
