import { describe, expect, it } from "vitest";
import type { AcademyMember } from "../types/academy";
import { summarizeAcademyMembers } from "./academy-statistics";

function academyMember(
  outcome: AcademyMember["outcome"],
  career: {
    reportedCareerAppearances?: number | null;
    goals?: number | null;
    assists?: number | null;
    internationalCaps?: number | null;
  } = {},
): AcademyMember {
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
    reportedCareerAppearances: career.reportedCareerAppearances ?? null,
    goals: career.goals ?? null,
    assists: career.assists ?? null,
    internationalCaps: career.internationalCaps ?? null,
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

  it("uses complete reported career data for graduation and totals", () => {
    expect(
      summarizeAcademyMembers([
        academyMember(null, {
          reportedCareerAppearances: 2,
          goals: 3,
          assists: 4,
          internationalCaps: 5,
        }),
        academyMember(null, {
          reportedCareerAppearances: 0,
          goals: 6,
          assists: 7,
          internationalCaps: 8,
        }),
      ]),
    ).toMatchObject({
      graduates: 1,
      goals: 9,
      assists: 11,
      internationalCaps: 13,
    });

    expect(
      summarizeAcademyMembers([
        academyMember(null, {
          reportedCareerAppearances: 2,
          goals: 3,
          assists: 4,
          internationalCaps: 5,
        }),
        academyMember(null, {
          reportedCareerAppearances: null,
          goals: null,
          assists: null,
          internationalCaps: null,
        }),
      ]),
    ).toMatchObject({
      graduates: null,
      goals: null,
      assists: null,
      internationalCaps: null,
    });
  });
});
