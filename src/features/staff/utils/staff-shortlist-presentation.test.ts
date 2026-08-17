import { describe, expect, it } from "vitest";
import { staffShortlistPresentation } from "./staff-shortlist-presentation";

describe("staff shortlist presentation", () => {
  it("projects one matching score and ranks it descending", () => {
    expect(staffShortlistPresentation("Technical Director")).toMatchObject({
      sort: "role.technical_director",
      dir: "desc",
    });
  });

  it("maps Head Performance Analyst to its role score", () => {
    expect(
      staffShortlistPresentation("Head Performance Analyst"),
    ).toMatchObject({
      sort: "role.head_performance_analyst",
      dir: "desc",
    });
  });

  it("shows the six outfield coaching scores without choosing a sort", () => {
    const presentation = staffShortlistPresentation("Coach");
    expect(presentation?.sort).toBeUndefined();
    expect(presentation?.columnIds).toEqual(
      expect.arrayContaining([
        "role.coach_attacking_technical",
        "role.coach_possession_tactical",
      ]),
    );
    expect(presentation?.columnIds).not.toEqual(
      expect.arrayContaining(["role.coach_fitness", "role.coach_goalkeeping"]),
    );
  });
});
