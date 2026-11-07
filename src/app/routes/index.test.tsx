import { screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";

describe("Dashboard", () => {
  it("says it is not built yet and links to the working destinations", async () => {
    renderWithProviders();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Dashboard is not built yet")).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Player Search" }),
    ).toHaveAttribute("href", "/search");
    expect(screen.getByRole("link", { name: "Open My Club" })).toHaveAttribute(
      "href",
      "/my-club",
    );
    expect(screen.queryByText("Placeholder.")).toBeNull();
    expect(screen.queryByRole("region", { name: "Club Setup" })).toBeNull();
    expect(
      screen.queryByRole("heading", { name: "CSV enrichment" }),
    ).toBeNull();
    expect(screen.queryByRole("heading", { name: "Bridge" })).toBeNull();
    expect(screen.queryByText(/^Snapshot$/)).toBeNull();
  });
});
