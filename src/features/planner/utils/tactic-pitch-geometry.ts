// Normalized pitch geometry shared by the tactical pitch canvas.
//
// Portrait space is attack-up: x runs 0 (left) to 1 (right) from the
// viewer's perspective and y runs 0 (attack) to 1 (own goal), matching the
// current attack-to-goalkeeper board order. Landscape is a clockwise
// 90-degree rotation, so portrait attack-up becomes landscape attack-right.
// Pure coordinates only: no lane, component, store, or IPC knowledge.

export type PitchCoordinate = {
  readonly x: number;
  readonly y: number;
};

export type PitchOrientation = "portrait" | "landscape";

export function projectPosition(
  coordinate: PitchCoordinate,
  orientation: PitchOrientation,
): PitchCoordinate {
  if (orientation === "landscape") {
    return { x: 1 - coordinate.y, y: coordinate.x };
  }
  return { x: coordinate.x, y: coordinate.y };
}

// Portrait coordinate for every supported qualified placement, keyed by
// canonical placement (legacy "ST" resolves to "STC" via
// portraitCoordinateForPlacement and has no key of its own).
export const PORTRAIT_PLACEMENT_COORDINATES: Readonly<
  Record<string, PitchCoordinate>
> = {
  GK: { x: 0.5, y: 0.93 },
  DL: { x: 0.12, y: 0.8 },
  DCL: { x: 0.35, y: 0.8 },
  DC: { x: 0.5, y: 0.8 },
  DCR: { x: 0.65, y: 0.8 },
  DR: { x: 0.88, y: 0.8 },
  WBL: { x: 0.1, y: 0.64 },
  DMCL: { x: 0.35, y: 0.64 },
  DM: { x: 0.5, y: 0.64 },
  DMCR: { x: 0.65, y: 0.64 },
  WBR: { x: 0.9, y: 0.64 },
  ML: { x: 0.12, y: 0.46 },
  MCL: { x: 0.35, y: 0.46 },
  MC: { x: 0.5, y: 0.46 },
  MCR: { x: 0.65, y: 0.46 },
  MR: { x: 0.88, y: 0.46 },
  AML: { x: 0.13, y: 0.28 },
  AMCL: { x: 0.35, y: 0.28 },
  AMC: { x: 0.5, y: 0.28 },
  AMCR: { x: 0.65, y: 0.28 },
  AMR: { x: 0.87, y: 0.28 },
  STCL: { x: 0.35, y: 0.08 },
  STC: { x: 0.5, y: 0.08 },
  STCR: { x: 0.65, y: 0.08 },
};

export function portraitCoordinateForPlacement(
  placement: string,
): PitchCoordinate | undefined {
  const canonical = placement === "ST" ? "STC" : placement;
  return PORTRAIT_PLACEMENT_COORDINATES[canonical];
}

// Visual pitch reading order: top-to-bottom, then left-to-right. Apply to
// projected coordinates so tab/DOM order follows the active orientation.
// Full ties return 0; sort a deterministically ordered input (lane order)
// so repeated positions keep a deterministic place without lane-ID keys.
export function comparePitchOrder(
  left: PitchCoordinate,
  right: PitchCoordinate,
): number {
  if (left.y !== right.y) {
    return left.y - right.y;
  }
  return left.x - right.x;
}
