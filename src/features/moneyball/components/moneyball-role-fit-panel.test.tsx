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
      },
      {
        metricKey: "possession_lost_per_90",
        sourceLabel: "Possession Lost per 90",
        weight: 0.4,
        direction: "lower",
        percentile: partial.score,
        weightedContribution:
          partial.score === null ? null : partial.score * 0.4,
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
        positions={{ MC: 20, GK: null }}
        roleScores={roles}
        catalogVersion={1}
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
      within(highRow).getByText("Catalog v1 · full imported cohort."),
    ).toBeInTheDocument();
    expect(within(highRow).getByText(/Higher is better/)).toBeInTheDocument();
    expect(within(highRow).getByText("Weight 60%")).toBeInTheDocument();
    expect(within(highRow).getByText("Contribution 49.2")).toBeInTheDocument();
  });

  it("identifies the missing percentile in an unavailable role disclosure", async () => {
    const user = userEvent.setup();
    render(
      <MoneyballRoleFitPanel
        positions={{ MC: null }}
        catalogVersion={1}
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
});
