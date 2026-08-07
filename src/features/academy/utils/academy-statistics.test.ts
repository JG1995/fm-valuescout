import { describe, expect, it } from "vitest";
import type { AcademyMember } from "../types/academy";
import { summarizeAcademyMembers } from "./academy-statistics";

function academyMember(outcome: AcademyMember["outcome"]): AcademyMember {
  return {
    playerUid: 77,
    lastKnownName: "Academy Player",
    currentName: "Academy Player",
    state: "resolved",
    age: 18,
    nationalities: [],
    positions: { ST: 20 },
    currentClub: "Metro FC",
    parentClub: null,
    teamLevel: "youth",
    pa: 150,
    determination: 15,
    heightCm: 180,
    preferredFoot: "right",
    seniorLeagueAppearances: null,
    goals: null,
    assists: null,
    internationalCaps: null,
    outcome,
    isGraduate: null,
  };
}

describe("summarizeAcademyMembers", () => {
  it("sums only sales and keeps manual outcome totals known at zero", () => {
    expect(summarizeAcademyMembers([])).toMatchObject({
      saleFeeEur: 0,
      releasedPlayers: 0,
    });

    expect(
      summarizeAcademyMembers([
        academyMember({
          status: "sold",
          buyingClub: "Rovers FC",
          saleFeeEur: 1_250_000,
        }),
        academyMember({
          status: "released",
          buyingClub: null,
          saleFeeEur: null,
        }),
        academyMember(null),
      ]),
    ).toMatchObject({
      saleFeeEur: 1_250_000,
      releasedPlayers: 1,
    });
  });
});
