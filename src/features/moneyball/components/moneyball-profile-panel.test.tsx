import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { fixturePlayerMoneyball } from "@/testing/moneyball-ipc-mock";
import { MoneyballProfilePanel } from "./moneyball-profile-panel";

describe("MoneyballProfilePanel", () => {
  it("renders raw context separately from scored category metrics and supports keyboard tabs", async () => {
    const user = userEvent.setup();
    render(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball({
          statistics: { goals: 10, goals_per_90: 0.6 },
          percentiles: { goals: 83, goals_per_90: 75 },
        })}
      />,
    );

    expect(screen.getByText("18")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("1,500")).toBeInTheDocument();
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
