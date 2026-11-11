import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { MoneyballRoleScore } from "../types/moneyball-profile";
import { MoneyballRoleFitPanel } from "./moneyball-role-fit-panel";

function role(
  partial: Pick<MoneyballRoleScore, "roleId" | "displayName" | "score"> &
    Partial<
      Pick<
        MoneyballRoleScore,
        "phase" | "positionFamily" | "positionTags" | "contributions"
      >
    >,
): MoneyballRoleScore {
  return {
    roleId: partial.roleId,
    displayName: partial.displayName,
    phase: partial.phase ?? "in_possession",
    positionFamily: partial.positionFamily ?? "central_midfielder",
    positionTags: partial.positionTags ?? ["MC"],
    score: partial.score,
    contributions: partial.contributions ?? [
      {
        metricKey: "goals_per_90",
        sourceLabel: "Goals per 90",
        weight: 0.6,
        direction: "higher",
        percentile: partial.score,
        weightedContribution:
          partial.score === null ? null : partial.score * 0.6,
        availableFromSource: true,
      },
      {
        metricKey: "possession_lost_per_90",
        sourceLabel: "Possession Lost per 90",
        weight: 0.4,
        direction: "lower",
        percentile: partial.score,
        weightedContribution:
          partial.score === null ? null : partial.score * 0.4,
        availableFromSource: true,
      },
    ],
  };
}

describe("MoneyballRoleFitPanel", () => {
  it("filters by position, sorts one Moneyball score, and discloses metric details", async () => {
    const user = userEvent.setup();
    const roles = [
      role({ roleId: "high", displayName: "High Role", score: 82 }),
      role({ roleId: "low", displayName: "Low Role", score: 41 }),
      role({
        roleId: "unavailable",
        displayName: "Unavailable Role",
        score: null,
      }),
      role({
        roleId: "goalkeeper",
        displayName: "Goalkeeper Role",
        positionFamily: "goalkeeper",
        positionTags: ["GK"],
        score: 99,
      }),
    ];

    render(
      <MoneyballRoleFitPanel
        leagueAdjustment={null}
        positions={{ MC: 20, GK: null }}
        roleScores={roles}
        catalogVersion={1}
        comparisonBasis={{
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        }}
      />,
    );

    const panel = await screen.findByRole("region", {
      name: "Moneyball role fit for MC",
    });
    expect(
      within(panel).getByRole("columnheader", { name: "Moneyball score" }),
    ).toBeInTheDocument();
    expect(
      within(panel).getByLabelText("High Role Moneyball score: 82, Excellent"),
    ).toBeInTheDocument();
    expect(
      within(panel).getByLabelText(
        "Unavailable Role Moneyball score: unavailable",
      ),
    ).toBeInTheDocument();
    expect(
      within(panel).queryByText("Goalkeeper Role"),
    ).not.toBeInTheDocument();

    const scoreHeader = within(panel).getByRole("columnheader", {
      name: "Moneyball score",
    });
    await user.click(within(scoreHeader).getByRole("button"));
    expect(
      within(panel)
        .getAllByRole("row")
        .slice(1)
        .map((row) => row.querySelector("summary")?.textContent),
    ).toEqual(["Low RoleIP", "High RoleIP", "Unavailable RoleIP"]);

    const highRow = within(panel)
      .getAllByRole("row")
      .find(
        (row) => row.querySelector("summary")?.textContent === "High RoleIP",
      );
    if (!highRow) throw new Error("High role row not found");
    await user.click(within(highRow).getByText("High Role"));
    expect(within(highRow).getByText("Goals per 90")).toBeInTheDocument();
    expect(
      within(highRow).getByText(
        "Catalog v1 · natural-position comparison cohort.",
      ),
    ).toBeInTheDocument();
    expect(
      within(highRow).getByText(
        "Weighted percentile score from the natural-position comparison cohort.",
      ),
    ).toBeInTheDocument();
    expect(within(highRow).getByText(/Higher is better/)).toBeInTheDocument();
    expect(within(highRow).getByText("Weight 60%")).toBeInTheDocument();
    expect(within(highRow).getByText("Contribution 49.2")).toBeInTheDocument();
  });

  it("identifies the missing percentile in an unavailable role disclosure", async () => {
    const user = userEvent.setup();
    render(
      <MoneyballRoleFitPanel
        leagueAdjustment={null}
        positions={{ MC: null }}
        catalogVersion={1}
        comparisonBasis={{
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        }}
        roleScores={[
          role({
            roleId: "unavailable",
            displayName: "Unavailable Role",
            score: null,
          }),
        ]}
      />,
    );

    const panel = await screen.findByRole("region", {
      name: "Moneyball role fit for MC",
    });
    await user.click(within(panel).getByText("Unavailable Role"));
    expect(
      within(panel).getByText(
        "Score unavailable: one or more metrics are missing.",
      ),
    ).toBeInTheDocument();
    expect(within(panel).getAllByText("Percentile unavailable")).toHaveLength(
      2,
    );
    expect(within(panel).getAllByText("Contribution unavailable")).toHaveLength(
      2,
    );
  });

  it("says how far the player's league moved a role score", async () => {
    const user = userEvent.setup();
    const { rerender } = render(
      <MoneyballRoleFitPanel
        leagueAdjustment={{
          division: "Sky Bet Championship",
          strength: 125,
          estimated: false,
          referenceDivision: "Premier League",
          referenceStrength: 149,
          shift: -12,
        }}
        positions={{ MC: 20 }}
        catalogVersion={1}
        comparisonBasis={{
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        }}
        roleScores={[
          role({ roleId: "scored", displayName: "Scored Role", score: 60 }),
        ]}
      />,
    );

    const panel = await screen.findByRole("region", {
      name: "Moneyball role fit for MC",
    });
    expect(
      within(panel).getByText(
        "League strength: role scores −12 (Sky Bet Championship 125 against Premier League 149)",
      ),
    ).toBeInTheDocument();
    await user.click(within(panel).getByText("Scored Role"));
    expect(
      within(panel).getByText(
        "Weighted percentile score from the natural-position comparison cohort, moved −12 for league strength.",
      ),
    ).toBeInTheDocument();

    rerender(
      <MoneyballRoleFitPanel
        leagueAdjustment={null}
        positions={{ MC: 20 }}
        catalogVersion={1}
        comparisonBasis={{
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        }}
        roleScores={[
          role({ roleId: "scored", displayName: "Scored Role", score: 60 }),
        ]}
      />,
    );
    expect(
      within(panel).getByText(
        "Weighted percentile score from the natural-position comparison cohort.",
      ),
    ).toBeInTheDocument();
    expect(within(panel).queryByText(/League strength/)).toBeNull();
  });

  it("marks a metric the save cannot provide and keeps its listed weight", async () => {
    const user = userEvent.setup();
    render(
      <MoneyballRoleFitPanel
        leagueAdjustment={null}
        positions={{ MC: 20 }}
        catalogVersion={1}
        comparisonBasis={{
          kind: "available",
          naturalPositions: ["MC"],
          comparisonPlayerCount: 24,
        }}
        roleScores={[
          role({
            roleId: "pressing",
            displayName: "Pressing Role",
            score: 60,
            contributions: [
              {
                metricKey: "pressures_completed_per_90",
                sourceLabel: "Pressures Completed per 90",
                weight: 0.35,
                direction: "higher",
                percentile: null,
                weightedContribution: null,
                availableFromSource: false,
              },
              {
                metricKey: "tackles_completed_per_90",
                sourceLabel: "Tackles Completed per 90",
                weight: 0.65,
                direction: "higher",
                percentile: 60,
                weightedContribution: 60,
                availableFromSource: true,
              },
            ],
          }),
        ]}
      />,
    );

    const panel = await screen.findByRole("region", {
      name: "Moneyball role fit for MC",
    });
    await user.click(within(panel).getByText("Pressing Role"));
    expect(
      within(panel).getByText("Not available from the save"),
    ).toBeInTheDocument();
    expect(within(panel).getByText("Weight 35%")).toBeInTheDocument();
    expect(
      within(panel).queryByText("Percentile unavailable"),
    ).not.toBeInTheDocument();
    expect(within(panel).getByText("Contribution 60")).toBeInTheDocument();
  });

  it("withholds stale role scores when the player has no natural position", () => {
    render(
      <MoneyballRoleFitPanel
        leagueAdjustment={null}
        positions={{ MC: 19 }}
        catalogVersion={1}
        comparisonBasis={{ kind: "unavailableNoNaturalPosition" }}
        roleScores={[
          role({ roleId: "stale", displayName: "Stale Role", score: 50 }),
        ]}
      />,
    );

    expect(
      screen.getByText(
        "Role scores unavailable: this player has no natural position.",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText("Stale Role")).not.toBeInTheDocument();
    expect(
      screen.queryByLabelText("Stale Role Moneyball score: 50, Average"),
    ).not.toBeInTheDocument();
  });
});
