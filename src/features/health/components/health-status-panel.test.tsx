import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import { setHealthSimulateError } from "../api/health-simulate-error";

describe("health status panel", () => {
  beforeEach(() => {
    setHealthSimulateError(false);
  });

  it("renders status from mock IPC", async () => {
    renderWithProviders();

    const statusLine = await screen.findByText(/^Status:/i);
    expect(statusLine).toHaveTextContent("ok");
  });

  it("renders demo value from mock IPC and saves updates", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const input = await screen.findByLabelText(/Demo value \(SQLite\):/i);
    expect(input).toHaveValue("");

    await user.type(input, "persisted");
    await user.click(screen.getByRole("button", { name: "Save demo value" }));

    expect(await screen.findByText(/Stored value:/i)).toHaveTextContent(
      "persisted",
    );
  });

  it("shows localized error UI when simulate error is triggered", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/^Status:/i);

    await user.click(screen.getByRole("button", { name: "Simulate error" }));

    expect(
      await screen.findByText(/Could not load health data/i),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Retry" }));

    const statusLine = await screen.findByText(/^Status:/i);
    expect(statusLine).toHaveTextContent("ok");
  });
});
