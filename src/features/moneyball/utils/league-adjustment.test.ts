import { describe, expect, it } from "vitest";
import type { MoneyballLeagueBreakdown } from "../types/moneyball-profile";
import { leagueAdjustmentSummary, signedPoints } from "./league-adjustment";

const breakdown = (
  overrides: Partial<MoneyballLeagueBreakdown>,
): MoneyballLeagueBreakdown => ({
  division: "Sky Bet Championship",
  strength: 124.9,
  estimated: false,
  referenceDivision: "Premier League",
  referenceStrength: 149.4,
  shift: -12,
  ...overrides,
});

describe("leagueAdjustmentSummary", () => {
  it("names both leagues, their strengths, and the points a weaker or stronger league moves", () => {
    expect(leagueAdjustmentSummary(breakdown({}))).toBe(
      "League strength: role scores −12 (Sky Bet Championship 125 against Premier League 149)",
    );
    expect(
      leagueAdjustmentSummary(
        breakdown({
          referenceDivision: "Sky Bet League One",
          referenceStrength: 109,
          shift: 8,
        }),
      ),
    ).toBe(
      "League strength: role scores +8 (Sky Bet Championship 125 against Sky Bet League One 109)",
    );
  });

  it("marks an estimated strength and a player with no league", () => {
    expect(
      leagueAdjustmentSummary(
        breakdown({ division: "Esiliiga", strength: 80, estimated: true }),
      ),
    ).toBe(
      "League strength: role scores −12 (Esiliiga 80, estimated against Premier League 149)",
    );
    expect(
      leagueAdjustmentSummary(
        breakdown({ division: null, strength: 80, estimated: true }),
      ),
    ).toBe(
      "League strength: role scores −12 (no league recorded, estimated against Premier League 149)",
    );
  });

  it("says nothing moved in the reference league or a league as strong", () => {
    expect(
      leagueAdjustmentSummary(
        breakdown({ division: "Premier League", strength: 149.4, shift: 0 }),
      ),
    ).toBe(
      "League strength: role scores not moved (Premier League is the reference league)",
    );
    expect(
      leagueAdjustmentSummary(
        breakdown({ division: "LaLiga", strength: 149, shift: 0 }),
      ),
    ).toBe(
      "League strength: role scores not moved (LaLiga 149 against Premier League 149)",
    );
  });
});

describe("signedPoints", () => {
  it("shows the direction of a shift", () => {
    expect([signedPoints(12), signedPoints(-24), signedPoints(0)]).toEqual([
      "+12",
      "−24",
      "0",
    ]);
  });
});
