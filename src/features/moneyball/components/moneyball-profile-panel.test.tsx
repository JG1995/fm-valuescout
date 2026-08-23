import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixturePlayerMoneyball,
  fixturePlayerMoneyballWithoutNaturalPosition,
} from "@/testing/moneyball-ipc-mock";
import { MoneyballProfilePanel } from "./moneyball-profile-panel";

describe("MoneyballProfilePanel", () => {
  it("renders raw context separately from scored category metrics and supports keyboard tabs", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball({
          statistics: { goals: 10, goals_per_90: 0.6 },
          percentiles: { goals: 83, goals_per_90: 75 },
          comparisonBasis: {
            kind: "available",
            naturalPositions: ["AMR", "AMC"],
            comparisonPlayerCount: 24,
          },
        })}
      />,
    );

    expect(screen.getByText("18")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("1,500")).toBeInTheDocument();
    expect(
      screen.getByText("Natural positions: AMR, AMC · 24 comparison players"),
    ).toBeInTheDocument();
    const tabs = screen.getAllByRole("tab");
    expect(tabs.map((tab) => tab.textContent)).toEqual([
      "Shooting",
      "Creation",
      "Possession",
      "Defending",
      "Aerial",
      "Goalkeeping",
      "Discipline",
      "Results",
    ]);
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "Goals: 83, Excellent" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Shooting" }));
    await user.keyboard("{End}");
    expect(
      screen.getByRole("tab", { name: "Results", selected: true }),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole("tabpanel", { name: "Results" })).getByText(
        "Average Rating",
      ),
    ).toBeInTheDocument();

    rerender(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball({
          comparisonBasis: {
            kind: "available",
            naturalPositions: ["AMR"],
            comparisonPlayerCount: 1,
          },
        })}
      />,
    );
    expect(
      screen.getByText("Natural positions: AMR · 1 comparison player"),
    ).toBeInTheDocument();
  });

  it("keeps raw metrics while withholding stale percentiles without a natural position", () => {
    render(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyballWithoutNaturalPosition({
          statistics: { goals: 10 },
          percentiles: { goals: 50 },
        })}
      />,
    );

    expect(screen.getByText("10")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Percentile scores unavailable: this player has no natural position.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("img", { name: "Goals: 50, Average" }),
    ).not.toBeInTheDocument();
  });

  it("keeps missing and pre-score import states distinct", () => {
    const { rerender } = render(
      <MoneyballProfilePanel profile={{ state: "noData" }} />,
    );
    expect(
      screen.getByText(/not included in the current Moneyball import/i),
    ).toBeInTheDocument();

    rerender(<MoneyballProfilePanel profile={{ state: "needsReimport" }} />);
    expect(
      screen.getByText(/before percentile scores were available/i),
    ).toBeInTheDocument();
  });
});
