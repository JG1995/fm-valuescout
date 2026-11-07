import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixturePlayerMoneyball,
  fixturePlayerMoneyballWithoutNaturalPosition,
} from "@/testing/moneyball-ipc-mock";
import { MoneyballProfilePanel } from "./moneyball-profile-panel";

const undatedStats = { moneyballStatsGameDate: null, snapshotGameDate: null };

describe("MoneyballProfilePanel", () => {
  it("renders raw context separately from scored category metrics and supports keyboard tabs", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <MoneyballProfilePanel
        {...undatedStats}
        profile={fixturePlayerMoneyball({
          statistics: { goals: 10, goals_per_90: 0.6 },
          percentiles: { goals: 83, goals_per_90: 75 },
          comparisonBasis: {
            kind: "available",
            naturalPositions: ["AMC", "AMR", "DL", "DR"],
            comparisonPlayerCount: 24,
          },
        })}
      />,
    );

    expect(screen.getByText("18")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("1,500")).toBeInTheDocument();
    expect(screen.queryByText("Asking price")).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "Natural positions: DR, DL, AMR, AMC · 24 comparison players",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("xGP")).toBeInTheDocument();
    for (const hiddenLabel of [
      "Expected Save %",
      "Pressures Completed / 90",
      "Games Won",
      "Team Goals",
    ]) {
      expect(screen.queryByText(hiddenLabel)).not.toBeInTheDocument();
    }
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
        {...undatedStats}
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
        {...undatedStats}
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

  it("labels the stats as-of date and notes a different snapshot date", () => {
    const { rerender } = render(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball()}
        moneyballStatsGameDate="2027-01-01"
        snapshotGameDate="2027-01-01"
      />,
    );
    expect(
      screen.getByText("Stats as of 1st January 2027"),
    ).toBeInTheDocument();

    rerender(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball()}
        moneyballStatsGameDate="2027-01-01"
        snapshotGameDate="2027-01-03"
      />,
    );
    expect(
      screen.getByText(
        "Stats as of 1st January 2027 · Snapshot dated 3rd January 2027",
      ),
    ).toBeInTheDocument();

    rerender(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball()}
        moneyballStatsGameDate={null}
        snapshotGameDate="2027-01-03"
      />,
    );
    expect(screen.queryByText(/Stats as of/)).not.toBeInTheDocument();
  });

  it("points missing and pre-score stats at the linked save and Refresh stats", () => {
    const { rerender } = render(
      <MoneyballProfilePanel {...undatedStats} profile={{ state: "noData" }} />,
    );
    expect(
      screen.getByText(/has no Moneyball stats in the current snapshot/i),
    ).toHaveTextContent(
      "Moneyball stats come from the FM save file linked in Settings. Use Refresh stats in Moneyball Search to read the latest save.",
    );

    rerender(
      <MoneyballProfilePanel
        {...undatedStats}
        profile={{ state: "needsReimport" }}
      />,
    );
    expect(
      screen.getByText(/before percentile scores were available/i),
    ).toHaveTextContent(
      "Use Refresh stats in Moneyball Search to read the linked FM save file again.",
    );
  });
});
