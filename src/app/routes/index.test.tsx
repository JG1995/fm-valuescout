import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";

describe("Dashboard", () => {
  it("renders only the planned placeholder content", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Placeholder.")).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Club Setup" })).toBeNull();
    expect(
      screen.queryByRole("heading", { name: "CSV enrichment" }),
    ).toBeNull();
    expect(screen.queryByRole("heading", { name: "Bridge" })).toBeNull();
    expect(screen.queryByText(/^Snapshot$/)).toBeNull();
  });
});
