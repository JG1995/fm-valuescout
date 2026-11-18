import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixturePlayerMoneyball,
  fixturePlayerMoneyballWithoutPosition,
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
          comparisons: [
            {
              family: "winger",
              peers: 24,
              percentiles: { goals: 83, goals_per_90: 75 },
            },
            {
              family: "wide_midfielder",
              peers: 1,
              percentiles: { goals: 50, goals_per_90: 50 },
            },
          ],
        })}
      />,
    );

    expect(screen.getByText("18")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("1,500")).toBeInTheDocument();
    expect(screen.queryByText("Asking price")).not.toBeInTheDocument();
    expect(
      screen.getByRole("combobox", { name: "Percentiles vs" }),
    ).toHaveValue("winger");
    expect(screen.getByText("· 24 peers")).toBeInTheDocument();
    expect(screen.getByText("xGP")).toBeInTheDocument();
    expect(screen.getByText("xG Overperformance")).toBeInTheDocument();
    for (const hiddenLabel of [
      "Goals From Outside The Box",
      "Goals From Outside The Box / 90",
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
    const results = within(screen.getByRole("tabpanel", { name: "Results" }));
    expect(results.getByText("Average Rating")).toBeInTheDocument();
    expect(results.getByText("Games Won")).toBeInTheDocument();

    // The switch ranks the same statistics among the other family's peers.
    await user.click(screen.getByRole("tab", { name: "Shooting" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Percentiles vs" }),
      "Wide midfielders",
    );
    expect(
      screen.getByRole("img", { name: "Goals: 50, Average" }),
    ).toBeInTheDocument();
    expect(screen.getByText("· 1 peer")).toBeInTheDocument();

    rerender(
      <MoneyballProfilePanel
        {...undatedStats}
        profile={fixturePlayerMoneyball()}
      />,
    );
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
    expect(screen.getByText("Percentiles vs wingers")).toBeInTheDocument();
  });

  it("notes a short sample and a family without peers yet", () => {
    render(
      <MoneyballProfilePanel
        {...undatedStats}
        profile={fixturePlayerMoneyball({
          minutes: 300,
          comparisons: [{ family: "striker", peers: 0, percentiles: {} }],
        })}
      />,
    );

    expect(
      screen.getByText(
        "Short sample: under 450 minutes, so this player is ranked against the peers without counting among them.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Not enough data: no strikers have 450 minutes yet."),
    ).toBeInTheDocument();
  });

  it("keeps raw metrics without percentiles for a player rated at no position", () => {
    render(
      <MoneyballProfilePanel
        {...undatedStats}
        profile={fixturePlayerMoneyballWithoutPosition({
          statistics: { goals: 10 },
        })}
      />,
    );

    expect(screen.getByText("10")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Percentile scores unavailable: this player is rated at no position.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("img", { name: /^Goals:/ }),
    ).not.toBeInTheDocument();
  });

  it("dates stats borrowed from a kept snapshot in place of the current ones", () => {
    render(
      <MoneyballProfilePanel
        profile={fixturePlayerMoneyball({
          minutes: 2100,
          borrowedStats: { statsDate: "2026-05-20", currentMinutes: 120 },
        })}
        moneyballStatsGameDate="2026-08-14"
        snapshotGameDate="2026-08-14"
      />,
    );

    expect(
      screen.getByText(
        "Stats as of 20th May 2026, from a kept snapshot: 120 minutes this season is under 450.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("2,100")).toBeInTheDocument();
    expect(screen.queryByText(/Short sample/)).not.toBeInTheDocument();
    expect(
      screen.queryByText("Stats as of 14th August 2026"),
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
      "Moneyball stats come from the FM save file linked in Settings. Use Refresh stats in Player Search to read the latest save.",
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
      "Use Refresh stats in Player Search to read the linked FM save file again.",
    );
  });
});
