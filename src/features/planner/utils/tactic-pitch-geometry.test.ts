import { describe, expect, it } from "vitest";
import { TACTIC_POSITION_ORDER } from "@/utils/position-order";
import {
  comparePitchOrder,
  type PitchCoordinate,
  PORTRAIT_PLACEMENT_COORDINATES,
  portraitCoordinateForPlacement,
  projectPosition,
} from "./tactic-pitch-geometry";

function pitchCoord(placement: string): PitchCoordinate {
  const coordinate = portraitCoordinateForPlacement(placement);
  if (!coordinate) {
    throw new Error(`Missing portrait coordinate for ${placement}`);
  }
  return coordinate;
}

function ordered(coordinates: PitchCoordinate[]): PitchCoordinate[] {
  return [...coordinates].sort(comparePitchOrder);
}

describe("tactic-pitch-geometry", () => {
  it("keeps portrait coordinates unchanged", () => {
    expect(projectPosition({ x: 0.2, y: 0.7 }, "portrait")).toEqual({
      x: 0.2,
      y: 0.7,
    });
  });

  it("rotates landscape clockwise so portrait attack-up becomes attack-right", () => {
    expect(projectPosition({ x: 0, y: 0 }, "landscape")).toEqual({
      x: 1,
      y: 0,
    });
    expect(projectPosition({ x: 0.5, y: 0.25 }, "landscape")).toEqual({
      x: 0.75,
      y: 0.5,
    });
  });

  it("projects strikers to the right half and the goalkeeper to the left half in landscape", () => {
    const striker = projectPosition(pitchCoord("STC"), "landscape");
    const goalkeeper = projectPosition(pitchCoord("GK"), "landscape");
    expect(striker.x).toBeGreaterThan(0.5);
    expect(goalkeeper.x).toBeLessThan(0.5);
  });

  it("maps every supported placement plus legacy ST inside normalized bounds", () => {
    for (const placement of [...TACTIC_POSITION_ORDER, "ST"]) {
      const coordinate = portraitCoordinateForPlacement(placement);
      expect(coordinate, placement).toBeDefined();
      if (!coordinate) {
        continue;
      }
      expect(coordinate.x).toBeGreaterThanOrEqual(0);
      expect(coordinate.x).toBeLessThanOrEqual(1);
      expect(coordinate.y).toBeGreaterThanOrEqual(0);
      expect(coordinate.y).toBeLessThanOrEqual(1);
      const projected = projectPosition(coordinate, "landscape");
      expect(projected.x).toBeGreaterThanOrEqual(0);
      expect(projected.x).toBeLessThanOrEqual(1);
      expect(projected.y).toBeGreaterThanOrEqual(0);
      expect(projected.y).toBeLessThanOrEqual(1);
    }
  });

  it("treats legacy ST as canonical STC", () => {
    expect(portraitCoordinateForPlacement("ST")).toEqual(
      portraitCoordinateForPlacement("STC"),
    );
    expect(PORTRAIT_PLACEMENT_COORDINATES.ST).toBeUndefined();
  });

  it("orders portrait markers attack to goalkeeper with left-to-right ties", () => {
    const markers = ordered([
      pitchCoord("GK"),
      pitchCoord("DR"),
      pitchCoord("DC"),
      pitchCoord("DL"),
      pitchCoord("STC"),
    ]);
    expect(markers).toEqual([
      pitchCoord("STC"),
      pitchCoord("DL"),
      pitchCoord("DC"),
      pitchCoord("DR"),
      pitchCoord("GK"),
    ]);
  });

  it("orders projected landscape markers in the current visual reading order", () => {
    const markers = ordered(
      ["GK", "STC", "DL"].map((placement) =>
        projectPosition(pitchCoord(placement), "landscape"),
      ),
    );
    expect(markers).toEqual([
      projectPosition(pitchCoord("DL"), "landscape"),
      projectPosition(pitchCoord("GK"), "landscape"),
      projectPosition(pitchCoord("STC"), "landscape"),
    ]);
  });
});
